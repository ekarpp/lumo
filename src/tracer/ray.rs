use crate::DVec3;
use crate::pdfs::Pdf;

/// Ray abstraction
pub struct Ray {
    /// Point of origin of the ray
    pub origin: DVec3,
    /// Direction of the ray. Not neccesarily normalized.
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

/// Ray that got scattered and provides the related scatter pdf.
pub struct ScatterRay {
    /// Scattered ray.
    pub ray: Ray,
    /// PDF among which ray was scattered.
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
