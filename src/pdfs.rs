#![allow(dead_code)]
use crate::{DVec3, DVec2};
use std::f64::consts::PI;
use crate::onb;
use crate::rand_utils;
use crate::tracer::hit::Hit;
use crate::tracer::object::Object;

pub trait Pdf {
    fn generate_dir(&self, rand_sq: DVec2) -> DVec3;
    fn pdf_val(&self, dir: DVec3, _h: &Hit) -> f64;
}

/* cosine weighed samples on hemisphere pointing towards w */
pub struct CosPdf {
    u: DVec3,
    v: DVec3,
    w: DVec3,
}

impl CosPdf {
    pub fn new(norm: DVec3) -> Self {
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

    fn pdf_val(&self, dir: DVec3, _h: &Hit) -> f64 {
        self.w.dot(dir.normalize()) * PI.recip()
    }
}

/* randomly sample point from the area of the object that is visible from p.
 * norm is the normal at p */
pub struct ObjectPdf<'a> {
    object: &'a Box<dyn Object>,
    p: DVec3,
}

impl<'a> ObjectPdf<'a> {
    pub fn new(o: &'a Box<dyn Object>, p: DVec3) -> Self {
        Self {
            object: o,
            p: p,
         }
    }
}

impl Pdf for ObjectPdf<'_> {
    fn generate_dir(&self, rand_sq: DVec2) -> DVec3 {
        self.object.sample_from(self.p, rand_sq)
    }

    fn pdf_val(&self, dir: DVec3, h: &Hit) -> f64 {
        self.object.sample_pdf(self.p, dir, h)
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

    fn uniform_choose(&self) -> &Box<dyn Pdf> {
        let idx = (self.pdfs.len() as f64 * rand_utils::rand_f64())
            .floor() as usize;
        &self.pdfs[idx]
    }
}

impl Pdf for MixedPdf {
    fn generate_dir(&self, rand_sq: DVec2) -> DVec3 {
        self.uniform_choose().generate_dir(rand_sq)
    }

    fn pdf_val(&self, dir: DVec3, h: &Hit) -> f64 {
        self.pdfs.iter().fold(0.0, |acc, pdf| acc + pdf.pdf_val(dir, h))
            / self.pdfs.len() as f64
    }
}

/* for glass and mirror. glass might want own pdf.. */
pub struct UnitPdf {}

impl UnitPdf {
    pub fn new() -> Self {
        Self {}
    }
}

impl Pdf for UnitPdf {
    /* just make sure it never gets called */
    fn generate_dir(&self, _rand_sq: DVec2) -> DVec3 { todo!() }

    fn pdf_val(&self, _dir: DVec3, _h: &Hit) -> f64 { 1.0 }
}
