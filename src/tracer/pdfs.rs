#![allow(dead_code)]
use crate::{DVec3, DVec2};
use std::f64::consts::PI;
use crate::tracer::onb::Onb;
use crate::rand_utils;
use crate::consts::EPSILON;
use crate::tracer::ray::Ray;
use crate::tracer::object::Object;
use crate::tracer::microfacet::MfDistribution;

/// Assumes that each generation and evaluation has same starting point. DO AS ENUM?
pub trait Pdf {
    /// Generates a random direction according to the sampling strategy
    ///
    /// # Arguments
    /// * `rand_sq` - Random point on the unit square.
    fn sample_ray(&self, rand_sq: DVec2) -> Ray;
    /// Computes the probability of the given direction.
    ///
    /// # Arguments
    /// * `ri` - Ray to compute probability for
    fn value_for(&self, ri: &Ray) -> f64;
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
            rand_utils::square_to_cos_hemisphere(rand_sq)
        );

        Ray::new(self.xo, wi)
    }

    fn value_for(&self, ri: &Ray) -> f64 {
        let wi = ri.dir;
        let cos_theta = self.uvw.w.dot(wi.normalize());
        if cos_theta > 0.0 { cos_theta * PI.recip() } else { 0.0 }
    }
}

/// TODO
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
        let wi = rand_utils::square_to_sphere(rand_sq);
        Ray::new(self.xo, wi)
    }

    fn value_for(&self, _ri: &Ray) -> f64 {
        let d: f64 = 0.1;//hi.object.density();
        // hi.t = 1.5
        d * (-d * 1.5).exp()
    }
}

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

    fn value_for(&self, ri: &Ray) -> f64 {
        self.object.sample_towards_pdf(ri)
    }
}

/// Delta distribution PDF. Always samples the same ray. For glass/mirror.
pub struct DeltaPdf {
    r: Ray,
}

impl DeltaPdf {
    pub fn new(xo: DVec3, wi: DVec3) -> Self {
        Self {
            r: Ray::new(xo, wi),
        }
    }
}

impl Pdf for DeltaPdf {
    fn sample_ray(&self, _rand_sq: DVec2) -> Ray {
        // lazy
        Ray::new(self.r.origin, self.r.dir)
    }

    fn value_for(&self, ri: &Ray) -> f64 {
        let wi = ri.dir;
        if wi.normalize().dot(self.r.dir.normalize()).abs() >= 1.0 - EPSILON {
            1.0
        } else {
            0.0
        }
    }
}

/// PDF for microfacet distribution.
pub struct MfdPdf {
    /// Point of impact
    xo: DVec3,
    /// Direction from point of impact to camera
    wo: DVec3,
    /// Macrosurface normal
    no: DVec3,
    /// ONB for macrosurface normal
    uvw: Onb,
    /// The microfacet distribution of the surface
    mfd: MfDistribution,
}

impl MfdPdf {
    pub fn new(xo: DVec3, wo: DVec3, no: DVec3, mfd: MfDistribution) -> Self {
        Self {
            xo,
            wo: wo.normalize(),
            uvw: Onb::new(no),
            no,
            mfd,
        }
    }
}

impl Pdf for MfdPdf {
    /// Sample microsurface normal from the distribution. Mirror direction from
    /// camera around the normal. Better and more complex method of sampling
    /// only visible normals due to Heitz 2014.
    fn sample_ray(&self, rand_sq: DVec2) -> Ray {
        let prob_ndf = self.mfd.probability_ndf_sample();

        let wi = if rand_utils::rand_f64() < prob_ndf {
            let wm = self.uvw.to_uvw_basis(
                self.mfd.sample_normal(rand_sq)
            ).normalize();
            let wi = 2.0 * self.wo.dot(wm) * wm - self.wo;
            // if angle between wm and wo > 90 deg, its bad.
            // VNDF fixes this?
            if wi.dot(self.no) < 0.0 { -wi } else { wi }
        } else if !self.mfd.is_transparent() {
            // use cosPdf?
            self.uvw.to_uvw_basis(
                rand_utils::square_to_cos_hemisphere(rand_sq)
            )
        } else {
            let inside = self.no.dot(self.wo) < 0.0;
            let eta_ratio = if inside {
                self.mfd.get_rfrct_idx()
            } else {
                1.0 / self.mfd.get_rfrct_idx()
            };
            let wm = self.uvw.to_uvw_basis(
                self.mfd.sample_normal(rand_sq)
            ).normalize();
            let wm = if inside { -wm } else { wm };

            /* Snell-Descartes law */
            let cos_in = wm.dot(self.wo).min(1.0);
            let sin_out = (1.0 - cos_in * cos_in) * eta_ratio * eta_ratio;

            /* total reflection */
            if sin_out > 1.0 {
                2.0 * self.wo.project_onto(wm) - self.wo
            } else {
                eta_ratio * self.wo + wm *
                    (eta_ratio * cos_in - (1.0 - sin_out).sqrt())
            }
        };

        Ray::new(self.xo, wi)
    }

    /// Read it directly from the NFD and do change of variables
    /// from `wh` to `wi`.
    fn value_for(&self, ri: &Ray) -> f64 {
        let wi = ri.dir.normalize();
        let wh = (self.wo + wi).normalize();

        let ndf = self.mfd.d(wh, self.no) * wh.dot(self.no).abs()
            / (4.0 * self.wo.dot(wh));

        // incorrect, should be different for transparent materials
        let hemisphere = wi.dot(self.no).max(0.0) / PI;
        let prob_ndf = ndf * ndf / (ndf * ndf + hemisphere * hemisphere);

        prob_ndf * ndf + (1.0 - prob_ndf) * hemisphere
    }
}
