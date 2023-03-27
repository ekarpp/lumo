use glam::DVec3;

/// Small utility struct for orthonormal basis
pub struct Onb {
    /// `x` (or `y`?) direction
    pub u: DVec3,
    /// `y` (or `x`?) direction
    pub v: DVec3,
    /// `z` direction
    pub w: DVec3,
}

impl Onb {
    /// Creates a new ONB.
    ///
    /// # Arguments
    /// * `dir` - Direction of `z` axis. Not necessarily normalized.
    pub fn new(dir: DVec3) -> Self {
        let w = dir.normalize();
        let (u, v) = w.any_orthonormal_pair();
        Self { u, v, w }
    }

    /// Translate from canonical basis to our ONB.
    ///
    /// # Arguments
    /// * `v` - Vector in canonical basis.
    pub fn to_uvw_basis(&self, v: DVec3) -> DVec3 {
        v.x * self.u + v.y * self.v + v.z * self.w
    }
}
