use glam::f32::Vec3;
use crate::tracer::sphere::Sphere;

pub struct Hit<'a> {
    pub t: f32,
    pub sphere: &'a Sphere,
    /* hit point */
    pub p: Vec3,
    /* vector to light */
    pub l: Vec3,
    /* sphere normal at hit point. pointing away from origin*/
    pub n: Vec3,
    /* is the hit inside the sphere? */
    pub inside: bool

}

impl Hit<'_> {
    pub fn new(t: f32, sphere: &Sphere) -> Hit {
        Hit {
            t: t,
            sphere: sphere,
            p: Vec3::ZERO,
            l: Vec3::ZERO,
            n: Vec3::ZERO,
            inside: false
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
