use super::*;

pub type Mesh = KdTree<Triangle>;

pub struct KdTree<T> {
    objects: Vec<T>,
    boundary: AaBoundingBox,
    root: Box<KdNode>,
}

/// https://github.com/ekzhang/rpt/blob/master/src/kdtree.rs
/// https://github.com/fogleman/pt/blob/master/pt/tree.go
impl<T: Bounded> KdTree<T> {
    pub fn new(objects: Vec<T>) -> Self {
        let indices = (0..objects.len()).collect();
        let bounds: Vec<AaBoundingBox> = objects.iter()
            .map(|obj| obj.bounding_box()).collect();
        let boundary = bounds.iter()
            .fold(AaBoundingBox::default(), |b1, b2| b1.merge(&b2));


        Self {
            root: KdNode::construct(&objects, &bounds, indices),
            objects,
            boundary,
        }
    }
}

/// A node in the kD-tree. Can be either a plane split or a leaf node.
pub enum KdNode {
    /// X-split, split point and child nodes
    SplitX(f64, Box<KdNode>, Box<KdNode>),
    /// Y-split, split point and child nodes
    SplitY(f64, Box<KdNode>, Box<KdNode>),
    /// Z-split, split point and child nodes
    SplitZ(f64, Box<KdNode>, Box<KdNode>),
    /// Stores indices to the object vector in the kD-tree
    Leaf(Vec<usize>),
}

impl KdNode {
    pub fn construct<T: Bounded>(
        objects: &[T],
        bounds: &[AaBoundingBox],
        indices: Vec<usize>
    ) -> Box<Self> {
        if indices.len() < 16 {
            return Box::new(Self::Leaf(indices));
        }

        let aabbs: Vec<&AaBoundingBox> = indices.iter()
            .map(|idx| &bounds[*idx]).collect();

        // smarter way?
        let mut xs: Vec<f64> = aabbs.iter().flat_map(|aabb| {
            [aabb.ax_min.x, aabb.ax_max.x]
        }).collect();
        let mut ys: Vec<f64> = aabbs.iter().flat_map(|aabb| {
            [aabb.ax_min.y, aabb.ax_max.y]
        }).collect();
        let mut zs: Vec<f64> = aabbs.iter().flat_map(|aabb| {
            [aabb.ax_min.z, aabb.ax_max.z]
        }).collect();


        let f64_cmp = |a: &f64, b: &f64| a.partial_cmp(b).unwrap();
        xs.sort_by(f64_cmp);
        ys.sort_by(f64_cmp);
        zs.sort_by(f64_cmp);

        let median = |svec: &[f64]| -> f64 {
            if svec.len() % 2 == 1 {
                svec[svec.len() / 2]
            } else {
                let mid = svec.len() / 2;
                (svec[mid] + svec[mid - 1]) / 2.0
            }
        };

        let (mx, my, mz) = (median(&xs), median(&ys), median(&zs));

        let score = |axis: usize, median: f64| -> usize {
            let mut left = 0;
            let mut right = 0;
            aabbs.iter().for_each(|aabb| {
                if aabb.get_min_axis(axis) <= median { left += 1; }
                if aabb.get_max_axis(axis) >= median { right += 1; }
            });
            left.max(right)
        };

        let (sx, sy, sz) = (score(0, mx), score(1, my), score(2, mz));

        let threshold = (indices.len() as f64 * 0.85) as usize;
        let best_score = sx.min(sy.min(sz));
        if best_score >= threshold {
            return Box::new(Self::Leaf(indices));
        }

        let (axis, median) = match best_score {
            _ if best_score == sx => (0, mx),
            _ if best_score == sy => (1, my),
            _ if best_score == sz => (2, mz),
            _ => panic!(),
        };

        let partition = |axis: usize, median: f64| {
            let mut left = Vec::new();
            let mut right = Vec::new();
            aabbs.iter().zip(indices).for_each(|(aabb, idx)| {
                if aabb.get_min_axis(axis) <= median {
                    left.push(idx);
                }
                if aabb.get_max_axis(axis) >= median {
                    right.push(idx);
                }
            });
            (left, right)
        };

        let (left, right) = partition(axis, median);

        match axis {
            0 => {
                Box::new(
                    Self::SplitX(
                        median,
                        Self::construct(objects, bounds, left),
                        Self::construct(objects, bounds, right),
                    )
                )
            }
            1 => {
                Box::new(
                    Self::SplitY(
                        median,
                        Self::construct(objects, bounds, left),
                        Self::construct(objects, bounds, right),
                    )
                )
            }
            2 => {
                Box::new(
                    Self::SplitY(
                        median,
                        Self::construct(objects, bounds, left),
                        Self::construct(objects, bounds, right),
                    )
                )
            }
            _ => panic!(),
        }
    }
}
