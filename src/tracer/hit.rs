use crate::DVec3;
use crate::tracer::object::Object;

pub struct Hit<'a> {
    pub t: f64,
    pub object: &'a dyn Object,
    /* point where ray hit object */
    pub p: DVec3,
    /* sphere normal at hit point.
     * if inside points towards origin otherwise not */
    pub n: DVec3,
}

impl Hit<'_> {
    /* why can't use Self type here?? */
    pub fn new(t: f64, o: &dyn Object, p: DVec3) -> Option<Hit> {
        /* p and n not always needed. computing for every hit slows rendering */
        Some(Hit {
            t: t,
            object: o,
            p: p,
            n: o.normal_at(p),
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
