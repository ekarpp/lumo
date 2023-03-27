use glam::{DVec3, DVec2};
use std::f64::consts::PI;
use crate::EPSILON;
use crate::tracer::onb::Onb;
use crate::rand_utils;
use crate::tracer::bxdfs;
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
    xo: DVec3,
    wi: DVec3,
}

impl DeltaPdf {
    pub fn new(xo: DVec3, wi: DVec3) -> Self {
        Self {
            xo,
            wi,
        }
    }
}

impl Pdf for DeltaPdf {
    fn sample_ray(&self, _rand_sq: DVec2) -> Ray {
        Ray::new(self.xo, self.wi)
    }

    fn value_for(&self, ri: &Ray) -> f64 {
        let wi = ri.dir;
        if wi.dot(self.wi) >= 1.0 - EPSILON {
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
    /// Direction from point of impact to viewer
    v: DVec3,
    /// Macrosurface normal. Same hemisphere as `v`.
    no: DVec3,
    /// Material albedo at point of impact
    albedo: DVec3,
    /// ONB for macrosurface normal
    uvw: Onb,
    /// The microfacet distribution of the surface
    mfd: MfDistribution,
    /// Perfect reflection and refraction of roughness below threshold
    delta_pdf: DeltaPdf,
}
// add cos pdf. add reflection pdf. reflection pdf has delta pdf?
impl MfdPdf {
    pub fn new(
        xo: DVec3,
        v: DVec3,
        no: DVec3,
        albedo: DVec3,
        mfd: MfDistribution,
    ) -> Self {
        let uvw = Onb::new(no);

        let wi = if !mfd.is_transparent() {
            bxdfs::reflect(v, no)
        } else {
            let inside = no.dot(v) < 0.0;
            let eta_ratio = if inside {
                mfd.get_rfrct_idx()
            } else {
                1.0 / mfd.get_rfrct_idx()
            };

            if inside {
                bxdfs::refract(eta_ratio, v, -no)
            } else {
                bxdfs::refract(eta_ratio, v, no)
            }
        };

        Self {
            delta_pdf: DeltaPdf::new(xo, wi),
            xo,
            v,
            uvw,
            albedo,
            no,
            mfd,
        }
    }
}

/// Threshold for roughness of microfacet at which we switch to delta pdf
const DELTA_THRESHOLD: f64 = 0.001;

impl Pdf for MfdPdf {
    /// Sample microsurface normal from the distribution. Mirror direction from
    /// camera around the normal. Better and more complex method of sampling
    /// only visible normals due to Heitz 2014.
    fn sample_ray(&self, rand_sq: DVec2) -> Ray {
        let prob_ndf = self.mfd.probability_ndf_sample(self.albedo);

        let wi = if rand_utils::rand_f64() < prob_ndf {
            // mirror??
            let wm = self.uvw.to_uvw_basis(
                self.mfd.sample_normal(rand_sq)
            ).normalize();
            let wi = bxdfs::reflect(self.v, wm);
            // if angle between wm and wo > 90 deg, its bad.
            // VNDF fixes this?
            if wi.dot(self.no) < 0.0 { -wi } else { wi }
        } else if !self.mfd.is_transparent() {
            self.uvw.to_uvw_basis(
                rand_utils::square_to_cos_hemisphere(rand_sq)
            )
        } else {
            if self.mfd.get_roughness() < DELTA_THRESHOLD {
                self.delta_pdf.sample_ray(rand_sq).dir
            } else {
                let inside = self.no.dot(self.v) < 0.0;
                let eta_ratio = if inside {
                    self.mfd.get_rfrct_idx()
                } else {
                    1.0 / self.mfd.get_rfrct_idx()
                };
                let wh = self.uvw.to_uvw_basis(
                    self.mfd.sample_normal(rand_sq)
                ).normalize();
                let wh = if inside { -wh } else { wh };

                bxdfs::refract(eta_ratio, self.v, wh)
            }
        };

        Ray::new(self.xo, wi)
    }

    /// Read it directly from the NFD and do change of variables
    /// from `wh` to `wi`.
    fn value_for(&self, ri: &Ray) -> f64 {
        let wi = ri.dir;
        let wh = (self.v + wi).normalize();
        let wh_dot_no = wh.dot(self.no);
        let wh_dot_v = self.v.dot(wh);
        // probability to sample wh w.r.t. to wo. mirror??
        let ndf = self.mfd.d(wh, self.no) * wh_dot_no.abs()
            / (4.0 * wh_dot_v);

        // transmission / scatter probability
        let st = if !self.mfd.is_transparent() {
            let cos_theta = self.uvw.w.dot(wi);
            if cos_theta > 0.0 { cos_theta * PI.recip() } else { 0.0 }
        } else if self.v.dot(wi) > 0.0 {
            // in the same hemisphere, zero probability for transmission
            0.0
        } else {
            if self.mfd.get_roughness() < DELTA_THRESHOLD {
                self.delta_pdf.value_for(ri)
            } else {
                let inside = self.no.dot(self.v) < 0.0;
                let eta_ratio = if inside {
                    1.0 / self.mfd.get_rfrct_idx()
                } else {
                    self.mfd.get_rfrct_idx()
                };
                let wh = (self.v + wi * eta_ratio).normalize();
                let wh_dot_wi = wi.dot(wh);
                let wh_dot_v = wh.dot(self.v);

                self.mfd.d(wh, self.no) * wh_dot_no.abs()
                    * (eta_ratio * eta_ratio * wh_dot_wi).abs()
                    / (wh_dot_v + eta_ratio * wh_dot_wi).powi(2)
            }
        };

        let prob_ndf = ndf * ndf / (ndf * ndf + st * st);

        prob_ndf * ndf + (1.0 - prob_ndf) * st
    }
}
