use super::*;
use std::{ cmp::Ordering, sync::mpsc, thread, time::Instant };
use rustc_hash::{FxHashMap, FxHashSet};
use node::KdNode;
use event::{ KdEvent, KdEventType };

/// Triangle mesh constructed as a kD-tree
pub type Mesh = KdTree<Triangle>;

mod node;
mod event;
#[cfg(test)]
mod kdtree_tests;

/// A k dimensional tree used to accelerate ray intersection calculations.
/// Implements a binary tree that splits a large mesh of objects to smaller
/// subobjects.
pub struct KdTree<T> {
    objects: Vec<T>,
    boundary: AaBoundingBox,
    root: Box<KdNode>,
    material: Material,
}

/// Implementation of a SAH based kd-tree
/// References:
/// [ekzhang/rpt](https://github.com/ekzhang/rpt/blob/master/src/kdtree.rs),
/// [fogleman/pt](https://github.com/fogleman/pt/blob/master/pt/tree.go),
/// [Article by Amsallem](https://www.flomonster.fr/articles/kdtree.html)
/// [Wald & Havran](https://www.irisa.fr/prive/kadi/Sujets_CTR/kadi/Kadi_sujet2_article_Kdtree.pdf)
impl<T: Bounded> KdTree<T> {
    /// Constructs a kD-tree of the given objects with the given material.
    /// Should each object have their own material instead?
    pub fn new(objects: Vec<T>, material: Material) -> Self {
        let start = Instant::now();
        if objects.len() > 10_000 {
            println!("Creating kd-tree of {} triangles", objects.len());
        }

        let bounds: Vec<AaBoundingBox> = objects
            .iter()
            .map(|obj| obj.bounding_box()).collect();
        let boundary = bounds
            .iter()
            .fold(AaBoundingBox::default(), |b1, b2| b1.merge(b2));

        let mut events: Vec<KdEvent> = Vec::with_capacity(3 * objects.len());
        for i in 0..objects.len() {
            let aabb = &bounds[i];
            for ax in [Axis::X, Axis::Y, Axis::Z] {
                let mi = aabb.min(ax);
                let mx = aabb.max(ax);
                if mi == mx {
                    events.push(KdEvent::new(mi, ax, KdEventType::Planar, i));
                } else {
                    events.push(KdEvent::new(mi, ax, KdEventType::Start, i));
                    events.push(KdEvent::new(mx, ax, KdEventType::End, i));
                }
            }
        }
        // could be parallelized, ~10% of construction
        events.sort_by(|a, b| a.cmp(b));

        let (tx, rx) = mpsc::channel();
        KdNode::construct(events, objects.len(), &boundary, 0, Some(tx));
        let root = rx.recv().unwrap();

        if objects.len() > 10_000 {
            let dt = start.elapsed().as_millis() as Float / 1e3;
            println!("Created kd-tree in {:.3} seconds", dt);
        }

        Self {
            root,
            objects,
            boundary,
            material,
        }
    }

    /// Returns self uniformly scaled as an instance with largest dimension
    /// of bounding box scaled to 1.0
    pub fn to_unit_size(self) -> Box<Instance<Self>> {
        let AaBoundingBox { ax_min, ax_max } = self.bounding_box();

        let bb_dim = ax_max - ax_min;
        let s = 1.0 / bb_dim.max_element();
        self.scale_uniform(s)
    }

    fn _hit<const GEO: bool>(&self, r: &Ray, t_min: Float, t_max: Float) -> Option<Hit> {
        let (t_start, t_end) = self.boundary.intersect(r);
        let (t_start, t_end) = (t_start.max(t_min), t_end.min(t_max));
        // box missed / is behind
        if t_start > t_end {
            None
        } else {
            self.hit_subtree::<GEO>(&self.root, r, t_min, t_end, &self.boundary)
        }
    }

    fn hit_subtree<const GEO: bool>(
        &self,
        node: &KdNode,
        r: &Ray,
        t_min: Float,
        t_max: Float,
        aabb: &AaBoundingBox,
    ) -> Option<Hit> {
        // extract split info or check for hit at leaf node
        let (axis, point, mut node_first, mut node_second) = match node {
            KdNode::Split(axis, point, left, right) => (*axis, *point, left, right),
            KdNode::Leaf(indices) => {
                let mut tt = t_max;
                let mut idx = None;

                for i in indices {
                    let t = self.objects[*i].hit_t(r, t_min, tt);
                    if t < tt { tt = t; idx = Some(i); }
                }

                let idx = idx?;

                return if GEO {
                    self.objects[*idx].hit(r, t_min, t_max)
                        .map(|mut h| { h.material = &self.material; h })
                } else {
                    Hit::from_t(tt)
                };
            }
        };

        let t_split = (point - r.o(axis)) / r.d(axis);

        let (mut aabb_first, mut aabb_second) = aabb.split(axis, point);

        let left_first = r.o(axis) < point || (r.o(axis) == point && r.d(axis) <= 0.0);
        // intersect first the AABB that we reach first
        if !left_first {
            std::mem::swap(&mut aabb_first, &mut aabb_second);
            std::mem::swap(&mut node_first, &mut node_second);
        }

        let (t_start, t_end) = aabb.intersect(r);
        let (t_start, t_end) = (t_start.max(t_min), t_end.min(t_max));

        // PBR Figure 4.19 (a). we hit only the first aabb.
        if t_split > t_end || t_split <= 0.0 {
            self.hit_subtree::<GEO>(node_first, r, t_min, t_end, &aabb_first)
        // PBR Figure 4.19 (b). we hit only the second aabb.
        } else if t_split < t_start {
            self.hit_subtree::<GEO>(node_second, r, t_min, t_end, &aabb_second)
        } else {
            match self.hit_subtree::<GEO>(node_first, r, t_start, t_end, &aabb_first) {
                None => self.hit_subtree::<GEO>(node_second, r, t_min, t_end, &aabb_second),
                Some(h1) => {
                    /* if we hit something in the first AABB before the split,
                     * there is no need to process the other subtree. */
                    if h1.t < t_split {
                        Some(h1)
                    } else {
                        self.hit_subtree::<GEO>(node_second, r, t_min, h1.t, &aabb_second)
                            .or(Some(h1))
                    }
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
    fn hit(&self, r: &Ray, t_min: Float, t_max: Float) -> Option<Hit> {
        self._hit::<true>(r, t_min, t_max)
    }

    fn hit_t(&self, r: &Ray, t_min: Float, t_max: Float) -> Float {
        self._hit::<false>(r, t_min, t_max).map_or(crate::INF, |h| h.t)
    }
}

impl<T: Sampleable + Bounded> Sampleable for KdTree<T> {
    fn area(&self) -> Float {
        // maybe sloooow for big ones
        self.objects.iter().fold(0.0, |sum, obj| sum + obj.area())
    }

    fn sample_on(&self, rand_sq: Vec2) -> Hit {
        let n = rand_utils::rand_float() * self.objects.len() as Float;
        let mut ho = self.objects[n.floor() as usize].sample_on(rand_sq);
        ho.material = &self.material;
        ho
    }
}
