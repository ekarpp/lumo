#![allow(dead_code)]
use crate::{DVec3, DVec2};
use std::f64::consts::PI;
use crate::onb::Onb;
use crate::rand_utils;
use rand_utils::RandomShape;
use crate::consts::EPSILON;
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;
use crate::tracer::object::Object;

/// Assumes that each generation and evaluation has same starting point. DO AS ENUM?
pub trait Pdf {
    /// Generates a random direction according to the sampling strategy
    ///
    /// # Arguments
    /// * `rand_sq` - Random point on the unit square.
    fn sample_ray(&self, rand_sq: DVec2) -> Ray;
    /// Computes the probability of the given direction.
    /// CAN REFACTORIZE THIS TO JUST TAKE THE GENERATED RAY
    ///
    /// # Arguments
    /// * `wi` - Direction to compute probability for
    /// * `hi` - Optional hit on the object direction sampled towards
    fn value_for(&self, wi: DVec3, hi: Option<Hit>) -> f64;
}

/// Cosine weighed samples on hemisphere pointing towards `z` of the ONB
pub struct CosPdf {
    xo: DVec3,
    uvw: Onb,
}

impl CosPdf {
    /// # Arguments
    ///
    /// * `xo` - Point where sampling is done.
    /// * `no` - Normal at the point of sampling directions.
    pub fn new(xo: DVec3, no: DVec3) -> Self {
        Self {
            xo,
            uvw: Onb::new(no),
        }
    }
}

impl Pdf for CosPdf {
    fn sample_ray(&self, rand_sq: DVec2) -> Ray {
        let wi = self.uvw.to_uvw_basis(
            RandomShape::gen_3d(RandomShape::CosHemisphere(rand_sq))
        );

        Ray::new(self.xo, wi)
    }

    fn value_for(&self, wi: DVec3, _hi: Option<Hit>) -> f64 {
        let cos_theta = self.uvw.w.dot(wi.normalize());
        if cos_theta > 0.0 { cos_theta * PI.recip() } else { 0.0 }
    }
}

pub struct IsotropicPdf {
    xo: DVec3,
}

impl IsotropicPdf {
    pub fn new(xo: DVec3) -> Self {
        Self {
            xo,
        }
    }
}

impl Pdf for IsotropicPdf {
    fn sample_ray(&self, rand_sq: DVec2) -> Ray {
        let wi = RandomShape::square_to_sphere(rand_sq);
        Ray::new(self.xo, wi)
    }

    fn value_for(&self, _wi: DVec3, hi_opt: Option<Hit>) -> f64 {
        hi_opt.map_or(0.0, |hi| {
            let d = hi.object.density();
            d * (-d * hi.t).exp()
        })
    }
}

/// WITH OBJECT PDF HAVE TO MAKE SURE THAT IT HITS. SHOULD REFACTOR IT HERE,
/// TO PDF VAL CALC.
/// Randomly samples a direction towards a point on the object that is visible
pub struct ObjectPdf<'a> {
    /// Object to do sampling from
    object: &'a dyn Object,
    /// Point from where the object should be visible
    xo: DVec3,
}

impl<'a> ObjectPdf<'a> {
    pub fn new(object: &'a dyn Object, xo: DVec3) -> Self {
        Self {
            object,
            xo,
         }
    }
}

impl Pdf for ObjectPdf<'_> {
    fn sample_ray(&self, rand_sq: DVec2) -> Ray {
        self.object.sample_towards(self.xo, rand_sq)
    }

    fn value_for(&self, wi: DVec3, hi_opt: Option<Hit>) -> f64 {
        hi_opt.map_or(0.0, |hi| self.object.sample_area_pdf(self.xo, wi, &hi))
    }
}

/*
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

    fn pdf_val(&self, wi: DVec3) -> f64 {
        self.pdfs.iter().fold(0.0, |acc, pdf| acc + pdf.pdf_val(wi))
            / self.pdfs.len() as f64
    }
}
*/

/// Delta distribution PDF. Always samples the same ray. For glass/mirror.
pub struct DeltaPdf {
    ri: Ray,
}

impl DeltaPdf {
    pub fn new(xo: DVec3, wi: DVec3) -> Self {
        Self {
            ri: Ray::new(xo, wi),
        }
    }
}

impl Pdf for DeltaPdf {
    fn sample_ray(&self, _rand_sq: DVec2) -> Ray {
        // lazy
        Ray::new(self.ri.origin, self.ri.dir)
    }

    fn value_for(&self, wi: DVec3, _hi: Option<Hit>) -> f64 {
        if wi.normalize().dot(self.ri.dir.normalize()).abs() >= 1.0 - EPSILON {
            1.0
        } else {
            0.0
        }
    }
}
