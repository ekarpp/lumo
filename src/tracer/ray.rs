use crate::DVec3;

pub struct Ray {
    pub origin: DVec3,
    /* should not be neccesarily normalized. go through code to verify */
    pub dir: DVec3,
}

impl Ray {
    pub fn new(origin: DVec3, dir: DVec3) -> Self {
        Self {
            origin: origin,
            dir: dir,
        }
    }

    pub fn at(&self, t: f64) -> DVec3 {
        self.origin + t*self.dir
    }
}
