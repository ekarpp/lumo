use crate::DVec3;

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

    /// Position of the ray at time `t`
    pub fn at(&self, t: f64) -> DVec3 {
        self.origin + t*self.dir
    }

    /// Coordinate of origin along axis
    pub fn o(&self, axis: usize) -> f64 {
        match axis {
            0 => self.origin.x,
            1 => self.origin.y,
            2 => self.origin.z,
            _ => unreachable!(),
        }
    }

    /// Coordinate of direction along axis
    pub fn d(&self, axis: usize) -> f64 {
        match axis {
            0 => self.dir.x,
            1 => self.dir.y,
            2 => self.dir.z,
            _ => unreachable!(),
        }
    }
}
