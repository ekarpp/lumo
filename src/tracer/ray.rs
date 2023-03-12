use crate::DVec3;
use crate::pdfs::Pdf;

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


pub struct ScatterRay {
    pub ray: Ray,
    pub pdf: Box<dyn Pdf>,
}

impl ScatterRay {
    pub fn new(r: Ray, pdf: Box<dyn Pdf>) -> Option<Self> {
        Some(Self {
            ray: r,
            pdf: pdf,
        })
    }
}
