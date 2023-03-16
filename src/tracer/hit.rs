use crate::DVec3;
use crate::tracer::object::Object;

/// Stores information about a hit between a ray and an object.
pub struct Hit<'a> {
    /// The `t` value of ray at which the hit occurred.
    pub t: f64,
    /// The object which got hit
    pub object: &'a dyn Object,
    /// 3D point where object was hit
    pub p: DVec3,
    /// Normal of the object at the point of impact.
    pub norm: DVec3,
}

impl<'a> Hit<'a> {
    /// # Arguments
    ///
    /// * `t` - Value of ray at which hit occurred.
    /// * `o` - The object which got hit.
    /// * `r` - The ray which hit the object.
    pub fn new(t: f64, object: &'a dyn Object, xi: DVec3, ni: DVec3)
               -> Option<Self> {
        /* p and n not always needed. computing for every hit slows rendering */
        Some(Self {
            t,
            object,
            p: xi,
            norm: ni,
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
