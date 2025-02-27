use crate::{ Direction, Normal, Transport, Float, Vec2, rand_utils, spherical_utils };
use crate::tracer::{ Color, ColorWavelength, Spectrum, hit::Hit, microfacet::MfDistribution, onb::Onb };

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
    /// Volumetric medium
    Volumetric(Float, Spectrum, Spectrum),
    None,
}

impl BxDF {
    pub fn is_specular(&self) -> bool {
        match self {
            Self::Volumetric(..) | Self::MfDielectric(_) => true,
            Self::MfConductor(mfd) => mfd.is_specular(),
            _ => false,
        }
    }

    pub fn is_diffuse(&self) -> bool { !self.is_specular() }

    #[allow(clippy::match_like_matches_macro)]
    pub fn is_transmission(&self) -> bool {
        match self {
            Self::MfDielectric(_) | Self::Volumetric(..) => true,
            _ => false
        }
    }

    pub fn is_reflection(&self) -> bool { !self.is_transmission() }

    pub fn is_delta(&self) -> bool {
        match self {
            Self::MfConductor(mfd) | Self::MfDielectric(mfd) => mfd.is_delta(),
            _ => false,
        }
    }

    pub fn f(
        &self,
        wo: Direction,
        wi: Direction,
        lambda: &ColorWavelength,
        reflection: bool,
        h: &Hit,
        mode: Transport
    ) -> Color {
        if (!reflection || h.backface) && self.is_reflection() {
            return Color::BLACK;
        }
        match self {
            Self::Lambertian(spec) => scatter::lambertian_f(spec, lambda),
            Self::MfDiffuse(mfd) => microfacet::diffuse_f(wo, wi, lambda, h, mfd),
            Self::MfConductor(mfd) => microfacet::conductor_f(wo, wi, lambda, h, mfd),
            Self::MfDielectric(mfd) => {
                microfacet::dielectric_f(wo, wi, lambda, reflection, h, mfd, mode)
            }
            Self::Volumetric(_, sigma_t, sigma_s) => {
                volumetric::f(lambda, h, sigma_t, sigma_s)
            }
            Self::None => Color::BLACK,
        }
    }

    pub fn sample(&self, wo: Direction, backface: bool, rand_sq: Vec2) -> Option<Direction> {
        if backface && self.is_reflection() {
            return None;
        }
        match self {
            Self::Lambertian(_) => scatter::lambertian_sample(rand_sq),
            Self::MfDiffuse(_) => scatter::lambertian_sample(rand_sq),
            Self::MfConductor(mfd) => microfacet::conductor_sample(wo, mfd, rand_sq),
            Self::MfDielectric(mfd) => microfacet::dielectric_sample(wo, mfd, rand_sq),
            Self::Volumetric(g, ..) => volumetric::sample(wo, *g, rand_sq),
            Self::None => None,
        }
    }

    pub fn pdf(&self, wo: Direction, wi: Direction, reflection: bool) -> Float {
        if !reflection && self.is_reflection() {
            // backfaces too? or explicitly in pdfs?
            return 0.0;
        }
        match self {
            Self::Lambertian(_) => scatter::lambertian_pdf(wo, wi),
            Self::MfDiffuse(_) => scatter::lambertian_pdf(wo, wi),
            Self::MfConductor(mfd) => microfacet::conductor_pdf(wo, wi, mfd),
            Self::MfDielectric(mfd) => {
                microfacet::dielectric_pdf(wo, wi, reflection, mfd)
            }
            Self::Volumetric(g, ..) => volumetric::pdf(wo, wi, *g),
            Self::None => 0.0,
        }
    }
}
