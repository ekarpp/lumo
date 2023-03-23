use super::*;

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
        let indices = (0..objects.len()).collect();

        let bounds: Vec<AaBoundingBox> = objects.iter()
            .map(|obj| obj.bounding_box())
            .collect();
        let boundary = bounds.iter()
            .fold(AaBoundingBox::default(), |b1, b2| b1.merge(b2));

        Self {
            root: KdNode::construct(&objects, &bounds, indices),
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
        let (axis, median, node_left, node_right) = match node {
            KdNode::Split(axis, median, left, right) => {
                (*axis, *median, left, right)
            }
            KdNode::Leaf(indices) => {
                return indices.iter()
                    .map(|idx| self.objects[*idx].hit(r, t_min, t_max))
                    .fold(None, |closest, hit| {
                        if closest.is_none() || (hit.is_some() && hit < closest) {
                            hit
                        } else {
                            closest
                        }
                    })
                    .map(|mut h| {
                        h.object = self;
                        h
                    })
            }
        };

        /* split to get the child AABBs. if we intersect with child AABB
         * then call hit_subtree recursively on the child. */
        let (aabb_left, aabb_right) = aabb.split(axis, median);
        let (left_start, left_end) = aabb_left.intersect(r);
        let (right_start, right_end) = aabb_right.intersect(r);

        let h_left = if left_end < t_min || left_start > left_end {
            None
        } else {
            self.hit_subtree(node_left, r, t_min, t_max, &aabb_left)
        };

        let h_right = if right_end < t_min || right_start > right_end {
            None
        } else {
            self.hit_subtree(node_right, r, t_min, t_max, &aabb_right)
        };

        if h_left.is_some() && h_right.is_some() {
            if h_left < h_right { h_left } else { h_right }
        } else {
            h_left.or(h_right)
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

        // box missed / is behind
        if t_start.max(t_min) > t_end.min(t_max) {
            None
        } else {
            self.hit_subtree(&self.root, r, t_min, t_max, &self.boundary)
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

        let (best_score, axis, median) = (1..3)
            .fold((scores[0], 0, med[0]), |best, ax| {
                if scores[ax] > best.0 {
                    (scores[ax], ax, med[ax])
                } else {
                    best
                }
            });

        let threshold = (indices.len() as f64 * 0.85) as usize;

        // no good splits, make it a leaf
        if best_score >= threshold {
            return Box::new(Self::Leaf(indices));
        }

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

        // split among the best axis
        let (left, right) = partition(axis, median);

        Box::new(
            Self::Split(
                axis,
                median,
                Self::construct(objects, bounds, left),
                Self::construct(objects, bounds, right),
            )
        )
    }
}
