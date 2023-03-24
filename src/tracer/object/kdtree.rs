use super::*;
use std::time::Instant;

/// Triangle mesh constructed as a kD-tree
pub type Mesh = KdTree<Triangle>;

/// A k dimensional tree used to accelerate ray intersection calculations.
/// Implements a binary tree that splits a large mesh of objects to smaller
/// subobjects.
pub struct KdTree<T> {
    objects: Vec<T>,
    boundary: AaBoundingBox,
    root: Box<KdNode>,
    material: Material,
}

/// https://github.com/ekzhang/rpt/blob/master/src/kdtree.rs
/// https://github.com/fogleman/pt/blob/master/pt/tree.go
impl<T: Bounded> KdTree<T> {
    /// Constructs a kD-tree of the given objects with the given material.
    /// Should each object have their own material instead?
    pub fn new(objects: Vec<T>, material: Material) -> Self {
        let start = Instant::now();

        let indices = (0..objects.len()).collect();
        let bounds: Vec<AaBoundingBox> = objects.iter()
            .map(|obj| obj.bounding_box())
            .collect();
        let boundary = bounds.iter()
            .fold(AaBoundingBox::default(), |b1, b2| b1.merge(b2));
        let root = KdNode::construct(&objects, &bounds, indices);

        println!("Constructed kd-tree of {} triangles in {:#?}",
                 objects.len(),
                 start.elapsed());

        Self {
            root,
            objects,
            boundary,
            material,
        }
    }

    fn hit_subtree(
        &self,
        node: &KdNode,
        r: &Ray,
        t_min: f64,
        t_max: f64,
        aabb: &AaBoundingBox
    ) -> Option<Hit> {
        // extract split info or check for hit at leaf node
        let (axis, median, mut node_first, mut node_second) = match node {
            KdNode::Split(axis, median, left, right) => {
                (*axis, *median, left, right)
            }
            KdNode::Leaf(indices) => {
                let mut tt = t_max;
                let mut h = None;
                for idx in indices {
                    h = self.objects[*idx].hit(r, t_min, tt)
                        .or(h);
                    tt = h.as_ref().map_or(tt, |hit| hit.t);
                }
                return h.map(|mut h| {
                    h.object = self;
                    h
                });
            }
        };

        let t_split = (median - r.o(axis)) / r.d(axis);

        let (mut aabb_first, mut aabb_second) = aabb.split(axis, median);

        let left_first = r.o(axis) < median ||
            (r.o(axis) == median && r.d(axis) <= 0.0);
        // intersect first the AABB that we reach first
        if !left_first {
            std::mem::swap(&mut aabb_first, &mut aabb_second);
            std::mem::swap(&mut node_first, &mut node_second);
        }

        let (t_start, t_end) = aabb.intersect(r);
        let (t_start, t_end) = (t_start.max(t_min), t_end.min(t_max));

        // PBR Figure 4.19 (a). we hit only the first aabb.
        if t_split > t_end || t_split <= 0.0 {
            self.hit_subtree(node_first, r, t_start, t_end, &aabb_first)
        // PBR Figure 4.19 (b). we hit only the second aabb.
        } else if t_split < t_start {
            self.hit_subtree(node_second, r, t_start, t_end, &aabb_second)
        } else {
            let h1 = self.hit_subtree(node_first, r, t_start, t_end, &aabb_first);

            /* if we hit something in the first AABB before the split, there
             * is no need to process the other subtree. */
            if h1.as_ref().filter(|h| h.t < t_split).is_some() {
                h1
            } else {
                let h2 =
                    self.hit_subtree(node_second, r, t_start, t_end, &aabb_second);
                if h1.is_some() && h2.is_some() {
                    if h1 < h2 { h1 } else { h2 }
                } else {
                    h1.or(h2)
                }
            }
        }
    }
}

impl<T: Bounded> Bounded for KdTree<T> {
    fn bounding_box(&self) -> AaBoundingBox {
        self.boundary
    }
}

impl<T: Bounded> Object for KdTree<T> {
    fn material(&self) -> &Material { &self.material }

    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        let (t_start, t_end) = self.boundary.intersect(r);
        let (t_start, t_end) = (t_start.max(t_min), t_end.min(t_max));
        // box missed / is behind
        if t_start > t_end {
            None
        } else {
            self.hit_subtree(&self.root, r, t_start, t_end, &self.boundary)
        }
    }

    fn sample_on(&self, _rand_sq: DVec2) -> DVec3 {
        // pick at random a triangle and do it
        todo!()
    }

    fn sample_towards(&self, _xo: DVec3, _rand_sq: DVec2) -> Ray {
        // pick at random a triangle and do it
        todo!()
    }
    fn sample_towards_pdf(&self, _ri: &Ray) -> f64 {
        // pick at random a triangle and do it
        todo!()
    }
}

/// A node in the kD-tree. Can be either a plane split or a leaf node.
pub enum KdNode {
    /// X-split, axis (x = 0, y = 1, z = 2), split point and child nodes
    Split(usize, f64, Box<KdNode>, Box<KdNode>),
    /// Stores indices to the object vector in the kD-tree
    Leaf(Vec<usize>),
}

impl KdNode {
    /// Constructs nodes of the kD-tree recursively. Median splits among the
    /// axes.
    pub fn construct<T: Bounded>(
        objects: &[T],
        bounds: &[AaBoundingBox],
        indices: Vec<usize>
    ) -> Box<Self> {
        if indices.len() < 16 {
            return Box::new(Self::Leaf(indices));
        }

        // filter relevant AABBs
        let aabbs: Vec<&AaBoundingBox> = indices.iter()
            .map(|idx| &bounds[*idx])
            .collect();

        // get AABB min/max values for each axis
        let mut bb_vals: [Vec<f64>; 3] = Default::default();
        aabbs.iter().for_each(|aabb| {
            let mx = aabb.ax_max.to_array();
            let mn = aabb.ax_min.to_array();
            for ax in 0..3 {
                bb_vals[ax].push(mn[ax]);
                bb_vals[ax].push(mx[ax]);
            }
        });

        // sort AABB min/max values
        (0..3).for_each(|ax| {
            bb_vals[ax].sort_by(|a: &f64, b: &f64| a.partial_cmp(b).unwrap());
        });

        let median = |svec: &[f64]| -> f64 {
            if svec.len() % 2 == 1 {
                svec[svec.len() / 2]
            } else {
                let mid = svec.len() / 2;
                (svec[mid] + svec[mid - 1]) / 2.0
            }
        };

        // find median among each axis
        let med: Vec<f64> = (0..3).map(|ax| median(&bb_vals[ax])).collect();

        let score = |axis: usize, median: f64| -> usize {
            let mut left = 0;
            let mut right = 0;
            aabbs.iter().for_each(|aabb| {
                if aabb.ax_min.to_array()[axis] <= median { left += 1; }
                if aabb.ax_max.to_array()[axis] >= median { right += 1; }
            });
            left.max(right)
        };

        // score each axis by how well objects are split between the medians
        let scores: Vec<usize> = (0..3).map(|ax| score(ax, med[ax])).collect();
        let threshold = (indices.len() as f64 * 0.85) as usize;

        // no good splits, make it a leaf
        if scores[0].min(scores[1].min(scores[2])) >= threshold {
            return Box::new(Self::Leaf(indices));
        }

        let mut split_axis = 3;
        let curr_bounds = aabbs.iter()
            .fold(AaBoundingBox::default(), |b1, b2| b1.merge(&b2));

        let max_width = curr_bounds.ax_max - curr_bounds.ax_min;

        // check if axis with maximum width has better score than threshold
        if max_width.x > max_width.y && max_width.x > max_width.z {
            if scores[0] < threshold {
                split_axis = 0;
            }
        } else if max_width.y > max_width.z {
            if scores[1] < threshold {
                split_axis = 1;
            }
        } else if scores[2] < threshold {
            split_axis = 2;
        }


        // otherwise take the axis with the best score
        if split_axis == 3 {
            if scores[0] < scores[1] && scores[0] < scores[2] {
                split_axis = 0;
            } else if scores[1] < scores[2] {
                split_axis = 1;
            } else {
                split_axis = 2;
            }
        }

        let median = med[split_axis];
        let partition = |axis: usize, median: f64| {
            let mut left = Vec::new();
            let mut right = Vec::new();
            aabbs.iter()
                .zip(indices)
                .for_each(|(aabb, idx)| {
                    if aabb.ax_min.to_array()[axis] <= median {
                        left.push(idx);
                    }
                    if aabb.ax_max.to_array()[axis] >= median {
                        right.push(idx);
                    }
                });
            (left, right)
        };

        // split among the chosen axis
        let (left, right) = partition(split_axis, median);

        Box::new(
            Self::Split(
                split_axis,
                median,
                Self::construct(objects, bounds, left),
                Self::construct(objects, bounds, right),
            )
        )
    }
}
