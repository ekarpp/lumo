use crate::rand_utils;
use crate::tracer::bxdfs;
use crate::tracer::microfacet::MfDistribution;
use crate::tracer::object::Sampleable;
use crate::tracer::onb::Onb;
use crate::tracer::ray::Ray;
use crate::EPSILON;
use glam::{DVec2, DVec3};
use std::f64::consts::PI;

#[cfg(test)]
mod pdf_tests;

/// Assumes that each generation and evaluation has same starting point.
pub trait Pdf {
    /// Generates a random direction according to the sampling strategy
    ///
    /// # Arguments
    /// * `rand_sq` - Random point on the unit square.
    fn sample_direction(&self, rand_sq: DVec2) -> Option<DVec3>;
    /// Computes the probability of the given direction w.r.t solid angle.
    ///
    /// # Arguments
    /// * `ri` - Ray to compute probability for
    fn value_for(&self, ri: &Ray) -> f64;
}

/// Randomly samples a direction towards a point on the object that is visible
pub struct ObjectPdf<'a> {
    /// Object to do sampling from
    object: &'a dyn Sampleable,
    /// Point from where the object should be visible
    xo: DVec3,
}

impl<'a> ObjectPdf<'a> {
    pub fn new(object: &'a dyn Sampleable, xo: DVec3) -> Self {
        Self { object, xo }
    }
}

impl Pdf for ObjectPdf<'_> {
    fn sample_direction(&self, rand_sq: DVec2) -> Option<DVec3> {
        Some( self.object.sample_towards(self.xo, rand_sq) )
    }

    fn value_for(&self, ri: &Ray) -> f64 {
        let (p, hi) = self.object.sample_towards_pdf(ri);
        if let Some(hi) = hi {
            // convert area measure to solid angle measure
            // other fields of hit might be in local instance coordinates
            let xi = hi.p;
            let ni = hi.ng;
            let wi = ri.dir;
            p * self.xo.distance_squared(xi) / ni.dot(wi).abs()
        } else {
            0.0
        }
    }
}

/// Delta distribution PDF. Always samples the same ray. For glass/mirror.
pub struct DeltaPdf {
    wi: DVec3,
}

impl DeltaPdf {
    pub fn new(wi: DVec3) -> Self {
        Self { wi }
    }
}

impl Pdf for DeltaPdf {
    fn sample_direction(&self, _rand_sq: DVec2) -> Option<DVec3> {
        Some( self.wi )
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
    /// Direction from point of impact to viewer
    v: DVec3,
    /// Macrosurface geometric normal. Same hemisphere as `v`.
    ng: DVec3,
    /// Probability to sample ray from NDF
    ndf_sample_prob: f64,
    /// ONB for macrosurface normal
    uvw: Onb,
    /// The microfacet distribution of the surface
    mfd: MfDistribution,
}

impl MfdPdf {
    pub fn new(
        v: DVec3,
        ng: DVec3,
        albedo: DVec3,
        mfd: MfDistribution
    ) -> Self {
        // refraction needs v and wh to be in same hemisphere so we do this
        let w = if v.dot(ng) < 0.0 { -ng } else { ng };
        let uvw = Onb::new(w);

        Self {
            v,
            uvw,
            ndf_sample_prob: mfd.probability_ndf_sample(albedo),
            ng,
            mfd,
        }
    }

    /// Samples randomly from the hemisphere with cos weighing
    fn sample_cos_hemisphere(&self, rand_sq: DVec2) -> Option<DVec3> {
        Some(
            self.uvw.to_world(rand_utils::square_to_cos_hemisphere(rand_sq))
        )
    }

    /// Samples a random microfacet normal and mirrors direction around it
    fn sample_ndf_scatter(&self, rand_sq: DVec2) -> Option<DVec3> {
        let local_v = self.uvw.to_local(self.v);
        let local_wh = self.mfd.sample_normal(local_v, rand_sq).normalize();
        let local_wi = bxdfs::reflect(local_v, local_wh);

        if local_wi.z <= 0.0 {
            // bad sample, do something else?
            None
        } else {
            Some( self.uvw.to_world(local_wi) )
        }
    }

    /// Samples a random microfacet normal and refracts direction around it
    fn sample_ndf_refract(&self, rand_sq: DVec2) -> Option<DVec3> {
        let inside = self.ng.dot(self.v) < 0.0;
        let eta_ratio = if inside {
            self.mfd.get_rfrct_idx()
        } else {
            1.0 / self.mfd.get_rfrct_idx()
        };
        let local_v = self.uvw.to_local(self.v);
        let local_wh = self.mfd.sample_normal(local_v, rand_sq).normalize();

        let wh = self.uvw.to_world(local_wh).normalize();

        bxdfs::refract(eta_ratio, self.v, wh)
    }

    /// PDF for NDF scattering
    fn sample_ndf_scatter_pdf(&self, wh: DVec3) -> f64 {
        let wh_dot_v = self.v.dot(wh);
        // wh and v always same hemisphere. need to make sure ng is there too
        let ng = if self.ng.dot(self.v) < 0.0 { -self.ng } else { self.ng };
        // probability to sample wh w.r.t. to v
        self.mfd.sample_normal_pdf(wh, self.v, ng)
            // jacobian
            / (4.0 * wh_dot_v)
    }

    /// PDF for hemisphere cos sampling
    fn sample_cos_hemisphere_pdf(&self, wi: DVec3) -> f64 {
        let cos_theta = self.ng.dot(wi);
        if cos_theta > 0.0 {
            cos_theta / PI
        } else {
            0.0
        }
    }

    /// PDF for NDF refraction
    fn sample_ndf_refract_pdf(&self, wi: DVec3) -> f64 {
        let ng_dot_wi = self.ng.dot(wi);
        let ng_dot_v = self.ng.dot(self.v);
        let v_inside = ng_dot_v < 0.0;
        let wi_inside = ng_dot_wi < 0.0;

        if v_inside == wi_inside {
            let wh = (self.v + wi).normalize();
            let wh_dot_v = wh.dot(self.v);
            let sin2_to = 1.0 - wh_dot_v * wh_dot_v;
            let sin2_ti = sin2_to * self.mfd.get_rfrct_idx().powi(2);

            if v_inside && sin2_ti > 1.0 {
                let wh_dot_v = wh.dot(self.v);
                self.mfd.sample_normal_pdf(wh, self.v, -self.ng)
                    / (4.0 * wh_dot_v)
            } else {
                // wi and v same hemisphere but not total internal reflection.
                // impossible
                0.0
            }
        } else {
            let eta_ratio = if v_inside {
                1.0 / self.mfd.get_rfrct_idx()
            } else {
                self.mfd.get_rfrct_idx()
            };
            let wh = -(self.v + wi * eta_ratio).normalize();
            let wh_dot_wi = wi.dot(wh);
            let wh_dot_v = wh.dot(self.v);

            if wh_dot_wi * wh_dot_v > 0.0 {
                // same hemisphere w.r.t wh, zero probability for refraction
                0.0
            } else {
                // wh and ng need to be in same hemisphere, hemisphere of v makes
                // no difference.
                self.mfd.sample_normal_pdf(wh, self.v, self.ng)
                    // jacobian
                    * (eta_ratio * eta_ratio * wh_dot_wi).abs()
                    / (wh_dot_v + eta_ratio * wh_dot_wi).powi(2)
            }
        }
    }
}

impl Pdf for MfdPdf {
    /// Sample microsurface normal from the distribution. Mirror direction from
    /// camera around the normal. GGX uses VNDF sampling, Beckmann NDF sampling.
    /// Importance sample with hemisphere scattering / refraction.
    fn sample_direction(&self, rand_sq: DVec2) -> Option<DVec3> {
        if rand_utils::rand_f64() < self.ndf_sample_prob {
            // NDF sample
            self.sample_ndf_scatter(rand_sq)
        } else if !self.mfd.is_transparent() {
            // Opaque materials sample hemisphere
            self.sample_cos_hemisphere(rand_sq)
        } else {
            // Transparent materials refract
            self.sample_ndf_refract(rand_sq)
        }
    }

    /// Read it directly from the NFD and do change of variables
    /// from `wh` to `wi`.
    fn value_for(&self, ri: &Ray) -> f64 {
        let wi = ri.dir;
        let wh = (self.v + wi).normalize();

        // probability to sample wh w.r.t. to v
        let ndf_pdf = self.sample_ndf_scatter_pdf(wh);

        // refraction / cos hemisphere sample probability
        let hr_pdf = if !self.mfd.is_transparent() {
            self.sample_cos_hemisphere_pdf(wi)
        } else {
            self.sample_ndf_refract_pdf(wi)
        };

        self.ndf_sample_prob * ndf_pdf
            + (1.0 - self.ndf_sample_prob) * hr_pdf
    }
}
