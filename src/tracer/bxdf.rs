use crate::{
    Direction, Normal, Transport, Float,
    Vec2, rng, math::spherical_utils,
};
use crate::tracer::{
    Color, ColorWavelength, Spectrum,
    microfacet::MfDistribution, onb::Onb
};

mod microfacet;
mod scatter;
mod volumetric;

#[cfg(test)]
mod sampling_tests;

#[cfg(test)]
mod chi2_tests;

pub enum BxDF {
    Lambertian(Spectrum),
    /// Lambertian diffuse
    MfDiffuse(MfDistribution),
    /// Microfacet mirror
    MfConductor(MfDistribution),
    /// Microfacet glass
    MfDielectric(MfDistribution),
    /// Volumetric medium[scattering_parameter, t_scale, sigma_t, sigma_s]
    Volumetric(Float, Float, Spectrum, Spectrum),
    None,
}

impl BxDF {
    #[inline]
    pub fn is_specular(&self) -> bool {
        match self {
            Self::Volumetric(..) | Self::MfDielectric(_) => true,
            Self::MfConductor(mfd) => mfd.is_specular(),
            _ => false,
        }
    }

    #[inline]
    pub fn is_diffuse(&self) -> bool { !self.is_specular() }

    #[allow(clippy::match_like_matches_macro)]
    #[inline]
    pub fn is_transmission(&self) -> bool {
        match self {
            Self::MfDielectric(_) | Self::Volumetric(..) => true,
            _ => false
        }
    }

    #[inline]
    pub fn is_reflection(&self) -> bool { !self.is_transmission() }

    #[inline]
    pub fn is_delta(&self, lambda: &ColorWavelength) -> bool {
        match self {
            Self::MfConductor(mfd) => mfd.is_delta(),
            Self::MfDielectric(mfd) => {
                mfd.is_delta() || mfd.eta_at(lambda.leading_sample()) == 1.0
            }
            _ => false,
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub fn f(
        &self,
        wo: Direction,
        wi: Direction,
        lambda: &ColorWavelength,
        reflection: bool,
        backface: bool,
        t: Float,
        uv: Vec2,
        mode: Transport
    ) -> Color {
        if (!reflection || backface) && self.is_reflection() {
            return Color::BLACK;
        }
        match self {
            Self::Lambertian(spec) => scatter::lambertian::f(spec, lambda),
            Self::MfDiffuse(mfd) => {
                microfacet::diffuse::f(wo, wi, lambda, uv, mfd)
            }
            Self::MfConductor(mfd) => {
                microfacet::conductor::f(wo, wi, lambda, uv, mfd)
            }
            Self::MfDielectric(mfd) => {
                microfacet::dielectric::f(wo, wi, lambda, reflection, uv, mfd, mode)
            }
            Self::Volumetric(_, t_scale, sigma_t, sigma_s) => {
                volumetric::f(lambda, *t_scale * t, sigma_t, sigma_s)
            }
            Self::None => Color::BLACK,
        }
    }

    #[inline]
    pub fn sample(
        &self,
        wo: Direction,
        backface: bool,
        lambda: &mut ColorWavelength,
        rand_u: Float,
        rand_sq: Vec2,
    ) -> Option<Direction> {
        if backface && self.is_reflection() {
            return None;
        }
        match self {
            Self::Lambertian(_) => scatter::lambertian::sample(rand_sq),
            //Self::MfDiffuse(_) => scatter::lambertian::pdf(wo, wi),
            Self::MfDiffuse(mfd) => microfacet::diffuse::sample(wo, mfd, rand_u, rand_sq),
            Self::MfConductor(mfd) => microfacet::conductor::sample(wo, mfd, rand_sq),
            Self::MfDielectric(mfd) => {
                microfacet::dielectric::sample(wo, mfd, lambda, rand_u, rand_sq)
            }
            Self::Volumetric(g, ..) => volumetric::sample(wo, *g, rand_sq),
            Self::None => None,
        }
    }

    #[inline]
    pub fn pdf(
        &self,
        wo: Direction,
        wi: Direction,
        reflection: bool,
        lambda: &ColorWavelength
    ) -> Float {
        if !reflection && self.is_reflection() {
            // backfaces too? or explicitly in pdfs?
            return 0.0;
        }
        match self {
            Self::Lambertian(_) => scatter::lambertian::pdf(wo, wi),
            //Self::MfDiffuse(_) => scatter::lambertian::pdf(wo, wi),
            Self::MfDiffuse(mfd) => microfacet::diffuse::pdf(wo, wi, mfd),
            Self::MfConductor(mfd) => microfacet::conductor::pdf(wo, wi, mfd),
            Self::MfDielectric(mfd) => {
                microfacet::dielectric::pdf(wo, wi, reflection, lambda, mfd)
            }
            Self::Volumetric(g, ..) => volumetric::pdf(wo, wi, *g),
            Self::None => 0.0,
        }
    }
}
