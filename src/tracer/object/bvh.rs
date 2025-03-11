use super::*;
use std::time::Instant;
use std::collections::VecDeque;
use node::BVHNode;
use crate::formatting;
use crate::tracer::ColorWavelength;

mod node;

const MAX_LEAF_SIZE: usize = 4;
const IDX_NAN: usize = usize::MAX;
const MORTON_ORDER: usize = 10;
const MORTON_MAX: u64 = 1 << MORTON_ORDER;
const MORTON_BITS: usize = MORTON_ORDER * 3;
const SAH_MAX_DEPTH: usize = MORTON_BITS / 2;

const SAMPLE_POWER: bool = true;

/// Bounding volume hierarchy to accelerate scenes with multiple objects
pub struct BVH<T> {
    objects: Vec<T>,
    nodes: Vec<BVHNode>,
    boundary: AaBoundingBox,
    num_primitives: usize,
    alias_table: Vec<(Float, usize)>,
    alias_pdf: Vec<Float>,
}

impl<T> Default for BVH<T> {
    fn default() -> Self {
        BVH {
            objects: vec!(),
            nodes: vec!(),
            num_primitives: 0,
            boundary: AaBoundingBox::default(),
            alias_table: vec!(),
            alias_pdf: vec!(),
        }
    }
}

impl BVH<Box<dyn Object>> {
    /// Build the BVH
    pub fn build(&mut self) {
        self._build();
    }
}

impl BVH<Box<dyn Sampleable>> {
    /// Sample a object and return its pdf
    pub fn sample_light(&self, rand_u: Float) -> usize {
        if SAMPLE_POWER {
            self._sample_power(rand_u)
        } else {
            self._sample_uniform(rand_u)
        }
    }

    fn sample_pdf(&self, idx: usize) -> Float {
        if SAMPLE_POWER {
            self._pdf_power(idx)
        } else {
            self._pdf_uniform()
        }
    }

    fn _sample_power(&self, rand_u: Float) -> usize {
        let rand_u = rand_u * self.objects.len() as Float;
        let idx = rand_u.floor() as usize;
        let rand_u = rand_u.fract();

        if rand_u < self.alias_table[idx].0 {
            idx
        } else {
            self.alias_table[idx].1
        }
    }

    fn _sample_uniform(&self, rand_u: Float) -> usize {
        (rand_u * self.objects.len() as Float).floor() as usize
    }

    /// Get light with `idx` and probability it got sampled
    pub fn get_light(&self, idx: usize) -> (&dyn Sampleable, Float) {
        (self.objects[idx].as_ref(), self.sample_pdf(idx))
    }

    fn _pdf_power(&self, idx: usize) -> Float {
        self.alias_pdf[idx]
    }

    fn _pdf_uniform(&self) -> Float {
        1.0 / self.objects.len() as Float
    }

    /// Get option wrapped index to light at hit `h`
    pub fn get_light_at(&self, h: &Hit) -> Option<usize> {
        let xo = h.ray_origin(true);
        let ri = Ray::new(xo, -h.ng);
        // check distance?
        self._hit::<true>(&ri, 0.0, crate::INF)
    }

    /// Build the BVH and light power sampling data structures
    pub fn build(&mut self) {
        self._build();

        if !SAMPLE_POWER { return; }

        let start = Instant::now();

        let lambda = ColorWavelength::default();
        let mut sum = 0.0;
        let n = self.objects.len();
        for i in 0..n {
            let object = &self.objects[i];
            let power = object.area() * object.material().power(&lambda);
            let power = (power / lambda.pdf()).mean();
            sum += power;
            self.alias_pdf.push(power);
            self.alias_table.push((1.0, i));
        }

        let mut large = Vec::with_capacity(n / 2);
        let mut small = Vec::with_capacity(n / 2);
        let mut pdf = Vec::with_capacity(n);
        let pdf_uniform = 1.0 / n as Float;

        for i in 0..n {
            self.alias_pdf[i] /= sum;
            pdf.push(self.alias_pdf[i]);
            if self.alias_pdf[i] > pdf_uniform {
                large.push(i);
            } else {
                small.push(i);
            }
        }

        let mut idx_s = small.len();
        let mut idx_l = large.len();
        while idx_s > 0 && idx_l > 0 {
            idx_s -= 1;
            idx_l -= 1;
            let (s, l) = (small[idx_s], large[idx_l]);
            self.alias_table[s] = (pdf[s] * n as Float, l);
            pdf[l] += pdf[s] - pdf_uniform;
            if pdf[l] > pdf_uniform {
                large[idx_l] = l;
                idx_l += 1;
            } else {
                small[idx_s] = l;
                idx_s += 1;
            }
        }

        while idx_s > 0 {
            idx_s -= 1;
            let s = small[idx_s];
            self.alias_table[s].0 = 1.0;
        }

        while idx_l > 0 {
            idx_l -= 1;
            let l = large[idx_l];
            self.alias_table[l].0 = 1.0;
        }

        #[cfg(debug_assertions)]
        {
            for i in 0..n {
                let pdf = self.alias_pdf[i];

                let mut sum = self.alias_table[i].0;
                for j in 0..n {
                    if self.alias_table[j].1 == i {
                        sum += 1.0 - self.alias_table[j].0;
                    }
                }

                sum *= pdf_uniform;

                assert!((sum - pdf).abs() < crate::EPSILON);
            }
        }

        let elapsed = start.elapsed();
        println!(
            "Created power sampling data structures in {}",
            formatting::fmt_elapsed(elapsed),
        );
    }
}

impl<T: ?Sized + Object> BVH<Box<T>> {
    /// Add a object to the BVH
    pub fn add(&mut self, object: Box<T>) {
        self.boundary = self.boundary.merge(&object.bounding_box());
        self.num_primitives += object.num_primitives();
        self.objects.push(object)
    }

    /// Return the size of the BVH tree
    pub fn num_primitives(&self) -> usize { self.num_primitives }

    /// Return the number of objects in the BVH tree
    pub fn num_objects(&self) -> usize { self.objects.len() }

    fn morton_code(&self, center: Point) -> u64 {
        let diff = center - self.boundary.ax_min;
        let dim = self.boundary.ax_max - self.boundary.ax_min;
        let idx = (MORTON_MAX as Float * diff / dim).floor();
        let x = idx.x as u64;
        let y = idx.y as u64;
        let z = idx.z as u64;

        let interleave = |mut i: u64| -> u64 {
            if i >= MORTON_MAX { i = MORTON_MAX - 1; }

            i = (i | (i << 16)) & 0b00011000000000000000011111111;
            i = (i | (i <<  8)) & 0b00011000000001111000000001111;
            i = (i | (i <<  4)) & 0b00011000011000011000011000011;
            i = (i | (i <<  2)) & 0b01001001001001001001001001001;

            i
        };

        (interleave(z) << 2) | (interleave(y) << 1) | (interleave(x) << 0)
    }

    /// Build the BVH
    // Garanzha et al. 2011
    fn _build(&mut self) {
        assert!(!self.objects.is_empty());
        println!("Building BVH with {} objects", self.objects.len());
        let start = Instant::now();

        let mut codes = Vec::with_capacity(self.objects.len());
        for i in 0..self.objects.len() {
            let obj = &self.objects[i];
            let center = obj.bounding_box().center();
            let code = self.morton_code(center);
            codes.push((code, i));
        }

        codes.sort();

        let (codes, indices) = codes.drain(..).unzip();
        let root = BVHNode::new(
            indices,
            codes,
        );

        let mut que = VecDeque::new();
        que.push_back((root, IDX_NAN, true, 1));

        while let Some((node, idx, is_left, depth)) = que.pop_front() {
            // insert left nodes right after parent and right nodes to the end for cache
            self.nodes.push(node);
            let pos = self.nodes.len() - 1;

            // update parent pointers
            if idx != IDX_NAN && !is_left {
                self.nodes[idx].right = pos;
            }

            // create splits
            let Some((left, right)) = self.nodes[pos].split(&self.objects, depth) else {
                // "mark" it as a leaf
                self.nodes[pos].codes.clear();
                continue
            };

            // push left nodes to front and right nodes to back
            que.push_front((left, pos, true, depth + 1));

            if !right.objects.is_empty() {
                que.push_back((right, pos, false, depth + 1));
            }
        }

        // update bounding boxes and remove objects from non-leaf nodes
        for i in (0..self.nodes.len()).rev() {
            let is_leaf = {
                let node = &self.nodes[i];
                node.codes.len() != node.objects.len()
            };
            if is_leaf {
                self.nodes[i].bounds = {
                    let node = &self.nodes[i];
                    node.objects.iter()
                        .fold(AaBoundingBox::default(), |b1, i| {
                            b1.merge(&self.objects[*i].bounding_box())
                        })
                };
            } else {
                self.nodes[i].objects.clear();
                self.nodes[i].codes.clear();
                self.nodes[i].bounds = {
                    let node = &self.nodes[i];
                    let right = node.right;
                    if right == IDX_NAN {
                        self.nodes[i + 1].bounds
                    } else {
                        self.nodes[i + 1].bounds
                            .merge(&self.nodes[node.right].bounds)
                    }
                };
            }
        }

        let elapsed = start.elapsed();
        println!("Built BVH in {}", formatting::fmt_elapsed(elapsed));
    }

    fn _hit<const GEO: bool>(
        &self,
        r: &Ray,
        t_min: Float,
        t_max: Float,
    ) -> Option<usize> {
        let origin = r.origin;
        let inv_dir = 1.0 / r.dir;

        let mut stack = [0; 64];
        let mut stack_ptr = 0;
        let mut curr = 0;

        let mut idx = None;
        let mut tt = t_max;
        loop {
            let node = &self.nodes[curr];
            let (t_start, t_end) = node.bounds.intersect(origin, inv_dir);
            let (t_start, t_end) = (t_start.max(t_min), t_end.min(tt));

            if t_start <= t_end {
                if node.objects.is_empty() {
                    // move to left child
                    curr += 1;
                    if node.right != IDX_NAN {
                        stack[stack_ptr] = node.right;
                        stack_ptr += 1;
                    }
                    continue;
                } else {
                    for i in &node.objects {
                        let t = self.objects[*i].hit_t(r, t_min, tt);
                        if GEO {
                            if t < tt { tt = t; idx = Some(*i); }
                        } else {
                            if t < tt { return Some(*i); }
                        }
                    }
                }
            }

            if stack_ptr == 0 { break; }
            stack_ptr -= 1;
            curr = stack[stack_ptr];
        }

        idx
    }
}

impl<T: ?Sized + Object> Object for BVH<Box<T>> {
    fn hit(&self, r: &Ray, t_min: Float, t_max: Float) -> Option<Hit> {
        self._hit::<true>(r, t_min, t_max)
            .and_then(|idx| self.objects[idx].hit(r, t_min, t_max))
    }

    fn hit_t(&self, r: &Ray, t_min: Float, t_max: Float) -> Float {
        self._hit::<false>(r, t_min, t_max)
            .map_or(crate::INF, |idx| self.objects[idx].hit_t(r, t_min, t_max))
    }

    fn bounding_box(&self) -> AaBoundingBox {
        self.boundary
    }
}
