use crate::{
    Normal, Direction, Point,
    Float, Vec2, rand_utils
};
use crate::tracer::{
    bxdfs, microfacet::MfDistribution, Color,
    object::Sampleable, onb::Onb, ray::Ray
};

#[cfg(test)]
mod pdf_tests;

/// Assumes that each generation and evaluation has same starting point.
pub trait Pdf {
    /// Generates a random direction according to the sampling strategy
    ///
    /// # Arguments
    /// * `rand_sq` - Random point on the unit square.
    fn sample_direction(&self, rand_sq: Vec2) -> Option<Direction>;
    /// Computes the probability of the given direction w.r.t solid angle.
    ///
    /// # Arguments
    /// * `ri` - Ray to compute probability for
    /// * `swap_dir` - Do we swap `wi` and `v`.
    /// Only makes a difference in non-symmetric PDFs (MFD refraction).
    fn value_for(&self, ri: &Ray, swap_dir: bool) -> Float;
}

/// Cosine weighed hemisphere sampling
pub struct CosPdf {
    uvw: Onb,
}

impl CosPdf {
    pub fn new(ns: Normal) -> Self {
        let uvw = Onb::new(ns);
        Self { uvw }
    }
}

impl Pdf for CosPdf {
    fn sample_direction(&self, rand_sq: Vec2) -> Option<Direction> {
        Some(self.uvw.to_world(rand_utils::square_to_cos_hemisphere(rand_sq)))
    }

    fn value_for(&self, ri: &Ray, _swap_dir: bool) -> Float {
        let wi = ri.dir;
        let cos_theta = self.uvw.w.dot(wi);
        if cos_theta > 0.0 {
            cos_theta / crate::PI
        } else {
            0.0
        }
    }
}

/// Randomly samples a direction towards a point on the object that is visible
pub struct ObjectPdf<'a> {
    /// Object to do sampling from
    object: &'a dyn Sampleable,
    /// Point from where the object should be visible
    xo: Point,
}

impl<'a> ObjectPdf<'a> {
    pub fn new(object: &'a dyn Sampleable, xo: Point) -> Self {
        Self { object, xo }
    }
}

impl Pdf for ObjectPdf<'_> {
    fn sample_direction(&self, rand_sq: Vec2) -> Option<Direction> {
        Some( self.object.sample_towards(self.xo, rand_sq) )
    }

    fn value_for(&self, ri: &Ray, _swap_dir: bool) -> Float {
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
    wi: Direction,
}

impl DeltaPdf {
    pub fn new(wi: Direction) -> Self {
        Self { wi }
    }
}

impl Pdf for DeltaPdf {
    fn sample_direction(&self, _rand_sq: Vec2) -> Option<Direction> {
        Some( self.wi )
    }

    // symmetric
    fn value_for(&self, ri: &Ray, _swap_dir: bool) -> Float {
        let wi = ri.dir;
        if wi.dot(self.wi) >= 1.0 - crate::EPSILON {
            1.0
        } else {
            0.0
        }
    }
}

/// PDF for volumetric mediums
pub struct VolumetricPdf {
    /// ONB for view direction
    uvw: Onb,
    /// Scattering parameter
    g: Float,
}

impl VolumetricPdf {
    pub fn new(v: Direction, g: Float) -> Self {
        Self {
            g,
            uvw: Onb::new(v),
        }
    }
}

impl Pdf for VolumetricPdf {
    fn sample_direction(&self, rand_sq: Vec2) -> Option<Direction> {

        let cos_theta = if self.g.abs() < 1e-3 {
            1.0 - 2.0 * rand_sq.x
        } else {
            let g2 = self.g * self.g;
            let fract = (1.0 - g2) / (1.0 - self.g + 2.0 * self.g * rand_sq.x);
            (1.0 + g2 - fract * fract) / (2.0 * self.g)
        };
        let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();

        let phi = 2.0 * crate::PI * rand_sq.y;

        let wi = self.uvw.to_world(Direction::new(
            sin_theta * phi.cos(),
            sin_theta * phi.sin(),
            cos_theta
        ));

        Some( wi )
    }

    // symmetric
    fn value_for(&self, ri: &Ray, _swap_dir: bool) -> Float {
        let wi = ri.dir;
        let cos_theta = wi.dot(self.uvw.w);

        let denom = 1.0 + self.g * self.g + 2.0 * self.g * cos_theta;

        (1.0 - self.g * self.g)
            / (4.0 * crate::PI * denom * denom.max(0.0).sqrt())
    }
}

/// PDF for microfacet distribution.
pub struct MfdPdf {
    /// Direction from point of impact to viewer
    v: Direction,
    /// Macrosurface shading normal. Same hemisphere as `v`.
    ns: Normal,
    /// Macrosurface geometric normal. Points outside of surface.
    ng: Normal,
    /// Probability to sample ray from NDF
    ndf_sample_prob: Float,
    /// ONB for macrosurface normal
    uvw: Onb,
    /// The microfacet distribution of the surface
    mfd: MfDistribution,
}

impl MfdPdf {
    pub fn new(
        v: Direction,
        ns: Normal,
        ng: Normal,
        albedo: Color,
        mfd: MfDistribution
    ) -> Self {
        // refraction needs v and wh to be in same hemisphere so we do this
        let ns = if v.dot(ng) < 0.0 { -ns } else { ns };
        let uvw = Onb::new(ns);

        Self {
            v,
            uvw,
            ndf_sample_prob: mfd.probability_ndf_sample(albedo),
            ns,
            ng,
            mfd,
        }
    }

    /// Samples randomly from the hemisphere with cos weighing
    fn sample_cos_hemisphere(&self, rand_sq: Vec2) -> Option<Direction> {
        Some(
            self.uvw.to_world(rand_utils::square_to_cos_hemisphere(rand_sq))
        )
    }

    /// Samples a random microfacet normal and mirrors direction around it
    fn sample_ndf_scatter(&self, rand_sq: Vec2) -> Option<Direction> {
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
    fn sample_ndf_refract(&self, rand_sq: Vec2) -> Option<Direction> {
        let local_v = self.uvw.to_local(self.v);
        let local_wh = self.mfd.sample_normal(local_v, rand_sq).normalize();
        let wh = self.uvw.to_world(local_wh).normalize();

        let inside = self.ng.dot(self.v) < 0.0;
        let eta_ratio = if inside {
            self.mfd.get_rfrct_idx()
        } else {
            1.0 / self.mfd.get_rfrct_idx()
        };

        Some( bxdfs::refract(eta_ratio, self.v, wh) )
    }

    /// PDF for NDF scattering
    fn sample_ndf_scatter_pdf(&self, wh: Normal) -> Float {
        let wh_dot_v = self.v.dot(wh);

        // probability to sample wh w.r.t. to v.
        // wh and v always same hemisphere. ns flipped to same in constructor.
        self.mfd.sample_normal_pdf(wh, self.v, self.ns)
            // jacobian
            / (4.0 * wh_dot_v)
    }

    /// PDF for hemisphere cos sampling
    fn sample_cos_hemisphere_pdf(&self, wi: Direction) -> Float {
        let cos_theta = self.ns.dot(wi);
        if cos_theta > 0.0 {
            cos_theta / crate::PI
        } else {
            0.0
        }
    }

    /// PDF for NDF refraction. Non-symmetric
    fn sample_ndf_refract_pdf(&self, wi: Direction, swap_dir: bool) -> Float {
        let (v, wi) = if swap_dir { (wi, self.v) } else { (self.v, wi) };

        let v_inside = self.ng.dot(v) < 0.0;
        let wi_inside = self.ng.dot(wi) < 0.0;

        if v_inside == wi_inside {
            let wh = (v + wi).normalize();
            let wh_dot_v = wh.dot(v);
            let sin2_to = 1.0 - wh_dot_v * wh_dot_v;
            let sin2_ti = sin2_to * self.mfd.get_rfrct_idx().powi(2);

            if v_inside && sin2_ti > 1.0 {
                let wh_dot_v = wh.dot(v);
                self.mfd.sample_normal_pdf(wh, v, self.ns)
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
            let wh = -(v + wi * eta_ratio).normalize();
            let wh_dot_wi = wi.dot(wh);
            let wh_dot_v = wh.dot(v);

            if wh_dot_wi * wh_dot_v > 0.0 {
                // same hemisphere w.r.t wh, zero probability for refraction
                0.0
            } else {
                // wh and ns need to be in same hemisphere, hemisphere of v makes
                // no difference.
                let wh = if self.ns.dot(wh) < 0.0 { -wh } else { wh };
                self.mfd.sample_normal_pdf(wh, v, self.ns)
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
    fn sample_direction(&self, rand_sq: Vec2) -> Option<Direction> {
        if rand_utils::rand_float() < self.ndf_sample_prob {
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

    fn value_for(&self, ri: &Ray, swap_dir: bool) -> Float {
        let wi = ri.dir;
        let wh = (self.v + wi).normalize();

        // probability to sample wh w.r.t. to v
        let ndf_pdf = self.sample_ndf_scatter_pdf(wh);

        // refraction / cos hemisphere sample probability
        let hr_pdf = if !self.mfd.is_transparent() {
            self.sample_cos_hemisphere_pdf(wi)
        } else {
            self.sample_ndf_refract_pdf(wi, swap_dir)
        };

        self.ndf_sample_prob * ndf_pdf
            + (1.0 - self.ndf_sample_prob) * hr_pdf
    }
}
