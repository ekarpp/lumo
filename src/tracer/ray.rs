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
    #[inline]
    pub fn new(origin: Point, dir: Direction) -> Self {
        Self {
            origin,
            dir: dir.normalize(),
        }
    }

    /// Applies `transformation` to `self`. Direction unnormalized to guarantee
    /// correct ray distances in Instance.
    #[inline]
    pub fn transform<const NORMALIZE: bool>(&self, transformation: &Transform) -> Self {
        let origin = transformation.transform_pt_inv(self.origin);
        let dir = transformation.transform_dir_inv(self.dir);
        let dir = if NORMALIZE { dir.normalize() } else { dir };

        Self { origin, dir }
    }

    /// Position of the ray at time `t`
    #[inline]
    pub fn at(&self, t: Float) -> Point {
        self.origin + t * self.dir
    }

    /// Coordinate of origin along axis
    #[inline]
    pub fn o(&self, axis: Axis) -> Float {
        match axis {
            Axis::X => self.origin.x,
            Axis::Y => self.origin.y,
            Axis::Z => self.origin.z,
        }
    }

    /// Coordinate of direction along axis
    #[inline]
    pub fn d(&self, axis: Axis) -> Float {
        match axis {
            Axis::X => self.dir.x,
            Axis::Y => self.dir.y,
            Axis::Z => self.dir.z,
        }
    }
}
