#![allow(dead_code)]
use crate::{DVec3, DVec2};
use std::f64::consts::PI;
use crate::onb::Onb;
use crate::rand_utils;
use crate::tracer::hit::Hit;
use crate::tracer::object::Object;

pub trait Pdf {
    /// Generates a random direction according to the sampling strategy
    ///
    /// # Arguments
    /// * `rand_sq` - Random point on the unit square.
    fn generate_dir(&self, rand_sq: DVec2) -> DVec3;
    /// Computes the probability of the given direction
    ///
    /// # Arguments
    /// * `dir` - Direction to compute probability for
    /// * `_h` - ???
    fn pdf_val(&self, dir: DVec3, _h: &Hit) -> f64;
}

/// Cosine weighed samples on hemisphere pointing towards `z` of the ONB
pub struct CosPdf {
    uvw: Onb,
}

impl CosPdf {
    /// # Arguments
    ///
    /// * `w` - Normal at the point of sampling directions.
    pub fn new(w: DVec3) -> Self {
        Self {
            uvw: Onb::new(w),
        }
    }
}

impl Pdf for CosPdf {
    fn generate_dir(&self, rand_sq: DVec2) -> DVec3 {
        self.uvw.to_uvw_basis(rand_utils::sq_to_cos_unit_hemisphere(rand_sq))
    }

    fn pdf_val(&self, dir: DVec3, _h: &Hit) -> f64 {
        let cos_theta = self.uvw.w.dot(dir.normalize());
        // double check math
        if cos_theta > 0.0 { cos_theta * PI.recip() } else { 0.0 }
    }
}

/// Randomly samples a direction towards a point on the object that is visible
pub struct ObjectPdf<'a> {
    /// Object to do sampling from
    object: &'a Box<dyn Object>,
    /// Point from where the object should be visible
    p: DVec3,
}

impl<'a> ObjectPdf<'a> {
    pub fn new(object: &'a Box<dyn Object>, p: DVec3) -> Self {
        Self {
            object,
            p,
         }
    }
}

impl Pdf for ObjectPdf<'_> {
    fn generate_dir(&self, rand_sq: DVec2) -> DVec3 {
        self.object.sample_towards(self.p, rand_sq)
    }

    fn pdf_val(&self, dir: DVec3, h: &Hit) -> f64 {
        self.object.sample_pdf(self.p, dir, h)
    }
}

/// Combination of multiple PDFs. Chooses one uniformly at random. BROKEN.
pub struct MixedPdf {
    /// Vector of the PDFs to choose from
    pdfs: Vec<Box<dyn Pdf>>,
}

impl MixedPdf {
    pub fn new(pdfs: Vec<Box<dyn Pdf>>) -> Self {
        Self {
            pdfs,
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

/// Unit PDF with delta distribution. Glass might want own PDF?
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
