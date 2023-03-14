use crate::DVec3;

/// Ray abstraction
pub struct Ray {
    /// Point of origin of the ray
    pub origin: DVec3,
    /// Direction of the ray. Not neccesarily normalized.
    pub dir: DVec3,
}

impl Ray {
    pub fn new(origin: DVec3, dir: DVec3) -> Self {
        Self {
            origin,
            dir,
        }
    }

    pub fn at(&self, t: f64) -> DVec3 {
        self.origin + t*self.dir
    }
}
