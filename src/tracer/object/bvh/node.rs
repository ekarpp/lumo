use super::*;
use crate::Axis;

const COST_INTERSECT: Float = 15.0;
const COST_TRAVERSE: Float = 20.0;
const EMPTY_BONUS: Float = 0.2;

pub struct BVHNode {
    // left = self + 1
    pub right: usize,
    pub objects: Vec<usize>,
    pub codes: Vec<u64>,
    pub bounds: AaBoundingBox,
}

enum BVHSplitSide {
    Left, Right, Null,
}

impl BVHNode {
    pub fn new(
        objects: Vec<usize>,
        codes: Vec<u64>,
    ) -> Self {
        Self {
            objects, codes,
            right: IDX_NAN,
            bounds: AaBoundingBox::default(),
        }
    }

    pub fn split<T: ?Sized + Object>(
        &self,
        objects: &[Box<T>],
        depth: usize
    ) -> Option<(Self, Self)> {
        if self.objects.len() <= 1 {
            None
        } else if depth > SAH_MAX_DEPTH {
            self.morton_split(depth)
        } else {
            self.sah_split(objects)
        }
    }

    fn morton_split(&self, depth: usize) -> Option<(Self, Self)> {
        let rss = MORTON_BITS - depth;
        let first = (self.codes[0] >> rss) & 1;
        let last = (self.codes.last().unwrap() >> rss) & 1;

        let split = if first == last {
            // leaf
            if self.codes.len() > MAX_LEAF_SIZE {
                self.codes.len() / 2
            } else {
                return None;
            }
        } else {
            self.codes.partition_point(|c| ((c >> rss) & 1) == first)
        };

        let left = Self::new(
            self.objects[..split].to_vec(),
            self.codes[..split].to_vec(),
        );
        let right = Self::new(
            self.objects[split..].to_vec(),
            self.codes[split..].to_vec(),
        );

        Some((left, right))
    }

    fn sah_split<T: ?Sized + Object>(&self, objects: &[Box<T>]) -> Option<(Self, Self)> {
        let mut best_cost = crate::INF;
        let mut best_axis = Axis::X;
        let mut best_center = crate::INF;
        let mut best_side = BVHSplitSide::Null;

        for axis in [Axis::X, Axis::Y, Axis::Z] {
            let mut indices = self.objects.clone();
            indices.sort_by(|i, j| {
                let pi = objects[*i].bounding_box().center().axis(axis);
                let pj = objects[*j].bounding_box().center().axis(axis);
                pi.total_cmp(&pj)
            });

            let mut area_left = Vec::with_capacity(indices.len() + 1);
            area_left.push(crate::INF);
            let mut bounds = AaBoundingBox::default();
            for i in &indices {
                bounds = bounds.merge(&objects[*i].bounding_box());
                area_left.push(bounds.area());
            }
            let mut area_right = Vec::with_capacity(indices.len() + 1);
            area_right.push(crate::INF);
            let mut bounds = AaBoundingBox::default();
            for i in indices.iter().rev() {
                bounds = bounds.merge(&objects[*i].bounding_box());
                area_right.push(bounds.area());
            }
            let total_area = area_right[indices.len()];

            let mut i = 0;
            while i < indices.len() {
                let get_center = |i| {
                    if i == indices.len() {
                        crate::INF
                    } else {
                        #[allow(clippy::borrowed_box)]
                        let obj: &Box<T> = &objects[indices[i]];
                        obj.bounding_box().center().axis(axis)
                    }
                };
                let center = get_center(i);
                let mut num_middle = 1;
                // TODO: consider each middle separately, how to do partition?
                while num_middle + i <= indices.len()
                    && center == get_center(i + num_middle) { num_middle += 1 }

                let num_left = i;
                let num_right = indices.len() - i - num_middle;

                let (cost, side) = Self::sah_cost(
                    &area_left,
                    num_left,
                    num_middle,
                    &area_right,
                    num_right,
                    total_area,
                );
                if cost < best_cost {
                    best_cost = cost;
                    best_axis = axis;
                    best_center = center;
                    best_side = side;
                }
                i += num_middle;
            }
        }

        self.sah_partition(best_axis, best_center, best_side, objects)
    }

    fn sah_partition<T: ?Sized + Object>(
        &self,
        axis: Axis,
        center: Float,
        side: BVHSplitSide,
        objects: &[Box<T>]
    ) -> Option<(Self, Self)> {
        let mut left_codes = Vec::with_capacity(self.codes.len() / 2);
        let mut left_objects = Vec::with_capacity(self.codes.len() / 2);
        let mut right_codes = Vec::with_capacity(self.codes.len() / 2);
        let mut right_objects = Vec::with_capacity(self.codes.len() / 2);

        for i in 0..self.codes.len() {
            let object = &objects[self.objects[i]];
            let c = object.bounding_box().center().axis(axis);
            if c < center || (c == center && matches!(side, BVHSplitSide::Left)) {
                left_codes.push(self.codes[i]);
                left_objects.push(self.objects[i]);
            } else if c > center || (c == center && matches!(side, BVHSplitSide::Right)) {
                right_codes.push(self.codes[i]);
                right_objects.push(self.objects[i]);
            } else {
                unreachable!();
            }
        }

        let left = Self::new(left_objects, left_codes);
        let right = Self::new(right_objects, right_codes);

        if left.codes.is_empty() {
            Some((right, left))
        } else {
            Some((left, right))
        }
    }

    fn sah_cost(
        area_left: &[Float],
        num_left: usize,
        num_middle: usize,
        area_right: &[Float],
        num_right: usize,
        total_area: Float
    ) -> (Float, BVHSplitSide) {
        let get_cost = |num_left, num_right| {
            let al = area_left[num_left];
            let ar = area_right[num_right];
            let cost = COST_TRAVERSE + COST_INTERSECT
                * (num_left as Float * al + num_right as Float * ar)
                / total_area;
            if num_left == 0 || num_right == 0 {
                cost * (1.0 - EMPTY_BONUS)
            } else {
                cost
            }
        };

        let cost_left = get_cost(num_left + num_middle, num_right);
        let cost_right = get_cost(num_left, num_middle + num_right);

        if cost_left < cost_right {
            (cost_left, BVHSplitSide::Left)
        } else {
            (cost_right, BVHSplitSide::Right)
        }
    }
}
