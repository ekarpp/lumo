use super::*;
use crate::formatting;
use std::{ cmp::Ordering, sync::mpsc, thread, time::Instant };
use rustc_hash::{FxHashMap, FxHashSet};
use node::{KdNode, KdNodeBuilder};
use event::{ KdEvent, KdEventType };

/// Triangle mesh constructed as a kD-tree
pub type Mesh = KdTree<Triangle>;

const IDX_NAN: usize = usize::MAX;

mod node;
mod event;
#[cfg(test)]
mod kdtree_tests;

/// A k dimensional tree used to accelerate ray intersection calculations.
/// Implements a binary tree that splits a large mesh of objects to smaller
/// subobjects.
pub struct KdTree<T> {
    objects: Vec<T>,
    nodes: Vec<KdNode>,
    boundary: AaBoundingBox,
}

impl KdTree<Triangle> {
    /// Return the material of the first triangle of the tree
    pub fn material(&self) -> &Material {
        self.objects[0].material()
    }
}

/// Implementation of a SAH based kd-tree
/// References:
/// [ekzhang/rpt](https://github.com/ekzhang/rpt/blob/master/src/kdtree.rs),
/// [fogleman/pt](https://github.com/fogleman/pt/blob/master/pt/tree.go),
/// [Article by Amsallem](https://www.flomonster.fr/articles/kdtree.html)
/// [Wald & Havran](https://www.irisa.fr/prive/kadi/Sujets_CTR/kadi/Kadi_sujet2_article_Kdtree.pdf)
impl<T: Object> KdTree<T> {
    /// Constructs a kD-tree of the given objects with the given material.
    /// Should each object have their own material instead?
    pub fn new(objects: Vec<T>) -> Self {
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
        KdNodeBuilder::construct(events, objects.len(), &boundary, 0, Some(tx));
        let root = rx.recv().unwrap();

        let mut nodes = Vec::with_capacity(root.num_nodes());
        root.build(&mut nodes, IDX_NAN);

        if objects.len() > 10_000 {
            println!("Created kd-tree in {}", formatting::fmt_elapsed(start.elapsed()));
        };

        Self {
            nodes,
            objects,
            boundary,
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

    fn _hit<const GEO: bool>(
        &self,
        r: &Ray,
        t_min: Float,
        t_max: Float
    ) -> Option<Hit> {
        let origin = [r.origin.x, r.origin.y, r.origin.z];
        let inv_dir = [1.0 / r.dir.x, 1.0 / r.dir.y, 1.0 / r.dir.z];

        let mut stack = [(0, t_min, t_max); 64];
        let mut stack_ptr = 0;
        let mut t_hit = crate::INF;
        let mut curr = 0;
        let mut idx = None;
        let (t_start, t_end) = self.boundary.intersect(r.origin, 1.0 / r.dir);
        let (mut t_start, mut t_end) = (t_start.max(t_min), t_end.min(t_max));

        loop {
            if t_hit < t_start { break; }
            let node = &self.nodes[curr];

            if node.leaf {
                for i in &node.indices {
                    let t = self.objects[*i].hit_t(r, t_min, t_end);
                    if GEO {
                        if t < t_end { t_end = t; t_hit = t; idx = Some(i); }
                    } else {
                        if t < t_end { return Hit::from_t(t); }
                    }
                }
                if stack_ptr == 0 { break; }
                stack_ptr -= 1;
                (curr, t_start, t_end) = stack[stack_ptr];
            } else {
                let point = node.point;
                let axis = node.axis;
                let t_split = (point - origin[axis as usize]) * inv_dir[axis as usize];
                let left_first = origin[axis as usize] < point
                    || (origin[axis as usize] == point && inv_dir[axis as usize] <= 0.0);

                let (first, second) = if left_first {
                    (curr + 1, node.right)
                } else {
                    (node.right, curr + 1)
                };

                if t_split > t_end || t_split <= 0.0 {
                    // PBR Figure 4.19 (a). we hit only the first aabb.
                    curr = first;
                } else if t_split < t_start {
                    // PBR Figure 4.19 (b). we hit only the second aabb.
                    curr = second;
                } else {
                    curr = first;
                    stack[stack_ptr] = (second, t_split, t_end);
                    t_end = t_split;
                    stack_ptr += 1;
                }
            }
        }

        let idx = idx?;

        if GEO {
            self.objects[*idx].hit(r, t_min, t_max)
        } else {
            Hit::from_t(crate::INF)
        }
    }
}

impl<T: Object> Object for KdTree<T> {
    fn hit(&self, r: &Ray, t_min: Float, t_max: Float) -> Option<Hit> {
        self._hit::<true>(r, t_min, t_max)
    }

    fn hit_t(&self, r: &Ray, t_min: Float, t_max: Float) -> Float {
        self._hit::<false>(r, t_min, t_max)
            .map_or(crate::INF, |h| h.t)
    }

    fn bounding_box(&self) -> AaBoundingBox {
        self.boundary
    }

    fn num_primitives(&self) -> usize { self.objects.len() }
}
