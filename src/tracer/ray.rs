use glam::{DAffine3, DVec3};
use crate::Axis;

/// Ray abstraction
pub struct Ray {
    /// Point of origin of the ray
    pub origin: DVec3,
    /// Direction of the ray. Normalized.
    pub dir: DVec3,
}

impl Ray {
    /// Constructs a ray. Normalize direction?
    pub fn new(origin: DVec3, dir: DVec3) -> Self {
        Self {
            origin,
            dir: dir.normalize(),
        }
    }

    /// Applies `transformation` to `self`. Direction unnormalized to guarantee
    /// correct computations in Instance.
    pub fn transform(&self, transformation: DAffine3) -> Self {
        Self {
            origin: transformation.transform_point3(self.origin),
            dir: transformation.transform_vector3(self.dir),
        }
    }

    /// Position of the ray at time `t`
    pub fn at(&self, t: f64) -> DVec3 {
        self.origin + t * self.dir
    }

    /// Coordinate of origin along axis
    pub fn o(&self, axis: Axis) -> f64 {
        match axis {
            Axis::X => self.origin.x,
            Axis::Y => self.origin.y,
            Axis::Z => self.origin.z,
        }
    }

    /// Coordinate of direction along axis
    pub fn d(&self, axis: Axis) -> f64 {
        match axis {
            Axis::X => self.dir.x,
            Axis::Y => self.dir.y,
            Axis::Z => self.dir.z,
        }
    }
}
