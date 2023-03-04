use glam::f32::Vec3;

pub struct Hit {
    pub t: f32,
    pub normal: Vec3
}

impl PartialEq for Hit {
    fn eq(&self, other: &Hit) -> bool {
        self.t == other.t
    }
}

impl PartialOrd for Hit {
    fn partial_cmp(&self, other: &Hit) -> Option<std::cmp::Ordering> {
        self.t.partial_cmp(&other.t)
    }
}
