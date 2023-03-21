use super::*;

pub struct KdTree<T> {
    objects: Vec<T>,
    boundary: AaBoundingBox,
    root: KdNode,
}

impl<T: Bounded> KdTree<T> {
    pub fn new(objects: Vec<T>) -> Self {
        let indices = (0..objects.len()).collect();
        let boundary = objects.iter()
            .map(|obj| obj.bounding_box())
            .fold(AaBoundingBox::default(), |b1, b2| b1.merge(&b2));

        Self {
            root: KdNode::construct(&objects, indices),
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
    pub fn construct<T: Bounded>(objects: &[T], indices: Vec<usize>) -> Self {
        Self::Leaf(indices)
    }
}
