use crate::{ Direction, Transport, Float, Vec2, Normal };
use crate::tracer::{ Color, ColorWavelength, bxdf::BxDF, onb::Onb };

pub struct BSDF {
    BxDF: BxDF
}

impl BSDF {
    /// Construct new empty BSDF
    #[inline]
    pub fn new(BxDF: BxDF) -> Self {
        Self { BxDF }
    }

    #[inline]
    pub fn is_specular(&self) -> bool {
        self.BxDF.is_specular()
    }

    #[inline]
    pub fn is_delta(&self, lambda: &ColorWavelength) -> bool {
        self.BxDF.is_delta(lambda)
    }

    /// Evaluate the BSDF
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub fn f(
        &self,
        wo: Direction,
        wi: Direction,
        lambda: &ColorWavelength,
        backface: bool,
        t: Float,
        ng: Normal,
        ns: Normal,
        uv: Vec2,
        mode: Transport
    ) -> Color {
        let reflection = Self::is_reflection(wo, wi, ng);
        let uvw = Onb::new(ns);

        let wo_local = uvw.to_local(wo);
        let wi_local = uvw.to_local(wi);

        self.BxDF.f(wo_local, wi_local, lambda, reflection, backface, t, uv, mode)
    }

    /// Sample direction from a random BxDF
    #[inline]
    pub fn sample(
        &self,
        wo: Direction,
        ns: Normal,
        backface: bool,
        lambda: &mut ColorWavelength,
        rand_u: Float,
        rand_sq: Vec2,
    ) -> Option<Direction> {
        let uvw = Onb::new(ns);

        let wo_local = uvw.to_local(wo);

        self.BxDF.sample(wo_local, backface, lambda, rand_u, rand_sq)
            .map(|wi| uvw.to_world(wi))
    }

    /// PDF for the BSDF
    #[inline]
    pub fn pdf(
        &self,
        wo: Direction,
        wi: Direction,
        ng: Normal,
        ns: Normal,
        lambda: &ColorWavelength,
    ) -> Float {
        let reflection = Self::is_reflection(wo, wi, ng);
        let uvw = Onb::new(ns);

        let wo_local = uvw.to_local(wo);
        let wi_local = uvw.to_local(wi);

        self.BxDF.pdf(wo_local, wi_local, reflection, lambda)
    }

    #[inline(always)]
    fn is_reflection(wo: Direction, wi: Direction, ng: Normal) -> bool {
        ng.dot(wi) * ng.dot(wo) >= 0.0
    }
}
