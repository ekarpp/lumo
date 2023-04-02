use crate::tracer::object::Object;
use glam::DVec3;

/// Stores information about a hit between a ray and an object.
pub struct Hit<'a> {
    /// The `t` value of ray at which the hit occurred.
    pub t: f64,
    /// The object which got hit
    pub object: &'a dyn Object,
    /// 3D point where object was hit
    pub p: DVec3,
    /// Normal of the surface used for shading calculations
    pub ns: DVec3,
    /// Geometric normal of the surface used for scattering calculations
    pub ng: DVec3,
}

impl<'a> Hit<'a> {
    /// # Arguments
    ///
    /// * `t` - Value of ray at which hit occurred.
    /// * `object` - The object which got hit.
    /// * `xi` - Point in world space at which object got hit
    /// * `ns` - Shading normal of the object at the point of impact
    /// * `ng` - Geometric normal of the object at the point of impact
    pub fn new(
        t: f64,
        object: &'a dyn Object,
        xi: DVec3,
        ns: DVec3,
        ng: DVec3,
    ) -> Option<Self> {
        Some(Self {
            t,
            object,
            p: xi,
            ns,
            ng,
        })
    }
}

impl PartialEq for Hit<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.t == other.t
    }
}

impl PartialOrd for Hit<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.t.partial_cmp(&other.t)
    }
}
