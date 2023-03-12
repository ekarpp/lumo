use crate::{DVec3, DVec2};
use std::f64::consts::PI;
use crate::onb;
use crate::rand_utils;
use crate::tracer::object::Object;

pub trait Pdf {
    fn generate_dir(&self, rand_sq: DVec2) -> DVec3;
    fn pdf_val(&self, dir: DVec3) -> f64;
}

/* cosine weighed samples on hemisphere pointing towards w */
pub struct CosPdf {
    u: DVec3,
    v: DVec3,
    w: DVec3,
}

impl CosPdf {
    fn new(norm: DVec3) -> Self {
        let (u, v) = onb::uvw_basis(norm);
        Self {
            u: u,
            v: v,
            w: norm,
        }
    }
}

impl Pdf for CosPdf {
    fn generate_dir(&self, rand_sq: DVec2) -> DVec3 {
        onb::to_uvw_basis(
            rand_utils::sq_to_cos_unit_hemisphere(rand_sq),
            self.u,
            self.v,
            self.w,
        )
    }

    fn pdf_val(&self, dir: DVec3) -> f64 {
        dir.dot(self.w) * PI.recip()
    }
}

/* randomly sample point from the area of the object that is visible from p.
 * norm is the normal at p */
pub struct ObjectPdf {
    object: &dyn Object,
    p: DVec3,
    norm: DVec3,
}

impl ObjectPdf {
    pub fn new(o: &dyn Object, p: DVec3, norm: DVec3) -> Self {
        Self {
            object: o,
            p: p,
            norm: norm,
        }
    }
}

impl Pdf for ObjectPdf {
    fn generate_dir(&self, rand_sq: DVec2) -> DVec3 {
        self.object.sample_from(self.p, rand_sq)
    }

    fn pdf_val(&self, dir: DVec3) -> f64 {
        self.object.sample_pdf(self.p, self.norm, dir)
    }
}

pub struct MixedPdf {
    pdfs: Vec<Box<dyn Pdf>>,
}

impl MixedPdf {
    pub fn new(pdfs: Vec<Box<dyn Pdf>>) -> Self {
        Self {
            pdfs: pdfs,
        }
    }

    fn uniform_choose(&self) -> Box<dyn Pdf> {
        let idx = (self.pdfs.len() as f64 * rand_utils::rand_f64())
            .floor() as usize;
        self.pdfs[idx]
    }
}

impl Pdf for MixedPdf {
    fn generate_dir(&self, rand_sq: DVec2) -> DVec3 {
        self.uniform_choose().generate_dir(rand_sq)
    }

    fn pdf_val(&self, dir: DVec3) -> f64 {
        self.pdfs.iter().fold(0.0, |acc, pdf| acc + pdf.pdf_val(dir))
            / self.pdfs.len() as f64
    }
}
