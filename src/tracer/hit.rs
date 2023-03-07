use crate::DVec3;
use crate::tracer::object::Object;
use crate::tracer::ray::Ray;

pub struct Hit<'a> {
    pub t: f64,
    pub object: &'a dyn Object,
    /* point where ray hit object */
    pub p: DVec3,
    /* sphere normal at hit point pointing towards incoming ray.
     * for sphere, points always away from origin */
    pub norm: DVec3,
}

impl<'a, 'b> Hit<'_> {
    /* why can't use Self type here?? cus self cant have lifetime?? */
    pub fn new(t: f64, o: &'a dyn Object, r: &'b Ray) -> Option<Hit<'a>> {
        /* p and n not always needed. computing for every hit slows rendering */
        Some(Hit {
            t: t,
            object: o,
            p: r.at(t),
            norm: o.normal_for_at(r, r.at(t)),
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
