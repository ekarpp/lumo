use crate::{ Axis, Direction, Point, Float, Transform };

/// Ray abstraction
pub struct Ray {
    /// Point of origin of the ray
    pub origin: Point,
    /// Direction of the ray. Normalized.
    pub dir: Direction,
}

impl Ray {
    /// Constructs a ray. Normalize direction?
    pub fn new(origin: Point, dir: Direction) -> Self {
        Self {
            origin,
            dir: dir.normalize(),
        }
    }

    /// Applies `transformation` to `self`. Direction unnormalized to guarantee
    /// correct ray distances in Instance.
    pub fn transform(&self, transformation: Transform) -> Self {
        Self {
            origin: transformation.transform_point3(self.origin),
            dir: transformation.transform_vector3(self.dir),
        }
    }

    /// Position of the ray at time `t`
    pub fn at(&self, t: Float) -> Point {
        self.origin + t * self.dir
    }

    /// Coordinate of origin along axis
    pub fn o(&self, axis: Axis) -> Float {
        match axis {
            Axis::X => self.origin.x,
            Axis::Y => self.origin.y,
            Axis::Z => self.origin.z,
        }
    }

    /// Coordinate of direction along axis
    pub fn d(&self, axis: Axis) -> Float {
        match axis {
            Axis::X => self.dir.x,
            Axis::Y => self.dir.y,
            Axis::Z => self.dir.z,
        }
    }
}
