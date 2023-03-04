use crate::tracer::sphere::Sphere;

pub struct Hit<'a> {
    pub t: f32,
    pub sphere: &'a Sphere
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
