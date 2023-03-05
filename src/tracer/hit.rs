use glam::f64::DVec3;
use crate::tracer::sphere::Sphere;

pub struct Hit<'a> {
    pub t: f64,
    pub sphere: &'a Sphere,
    /* hit point */
    pub p: DVec3,
    /* sphere normal at hit point.
     * if inside points towards origin otherwise not */
    pub n: DVec3,
}

impl Hit<'_> {
    pub fn new(t: f64, sphere: &Sphere) -> Hit {
        Hit {
            t: t,
            sphere: sphere,
            p: DVec3::ZERO,
            n: DVec3::ZERO
        }
    }
}

impl PartialEq for Hit<'_> {
    fn eq(&self, other: &Hit) -> bool {
        self.t == other.t
    }
}

impl PartialOrd for Hit<'_> {
    fn partial_cmp(&self, other: &Hit) -> Option<std::cmp::Ordering> {
        self.t.partial_cmp(&other.t)
    }
}
