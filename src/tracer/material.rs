use crate::{ Normal, Direction, Transport, Float, Vec2 };
use crate::tracer::{
    Color, ColorWavelength, Spectrum, hit::Hit, microfacet::MfDistribution,
    texture::Texture, bsdf::BSDF, bxdf::BxDF
};

#[cfg(test)]
mod white_furnace_tests;

/// Describes which material an object is made out of
pub enum Material {
    /// Materials with standard BSDF
    Standard(BSDF),
    /// Emits light
    Light(Texture),
    /// Volumetric material for mediums. `scatter_param`, `sigma_t`, `sigma_s`
    Volumetric(BSDF),
    /// Not specified. Used with objects that are built on top of other objects.
    Blank,
}

impl Material {
    /// General microfacet constructor
    #[allow(clippy::too_many_arguments)]
    pub fn microfacet(
        roughness: Float,
        eta: Float,
        k: Float,
        is_transparent: bool,
        fresnel_enabled: bool,
        kd: Texture,
        ks: Texture,
        tf: Texture,
    ) -> Self {
        let mfd = MfDistribution::new(roughness, eta, k, kd, ks, tf);
        // dirty dirty...
        let bsdf = if is_transparent {
            BSDF::new(BxDF::MfDielectric(mfd))
        } else if fresnel_enabled {
            BSDF::new(BxDF::MfConductor(mfd))
        } else {
            BSDF::new(BxDF::MfDiffuse(mfd))
        };
        Self::Standard(bsdf)
    }

    /// Microfacet mirror with assignable roughness
    pub fn metal(ks: Texture, roughness: Float, eta: Float, k: Float) -> Self {
        let kd = Texture::from(Spectrum::WHITE);
        let tf = Texture::from(Spectrum::BLACK);

        let is_transparent = false;
        let fresnel_enabled = true;

        Self::microfacet(
            roughness,
            eta,
            k,
            is_transparent,
            fresnel_enabled,
            kd, ks, tf,
        )
    }

    /// Diffuse material with a microfacet based BxDF
    pub fn diffuse(kd: Texture) -> Self {
        let ks = Texture::from(Spectrum::WHITE);
        let tf = Texture::from(Spectrum::BLACK);

        let roughness = 1.0;
        let eta = 1.5;
        let k = 0.0;
        let is_transparent = false;
        let fresnel_enabled = false;

        Self::microfacet(
            roughness,
            eta,
            k,
            is_transparent,
            fresnel_enabled,
            kd, ks, tf,
        )
    }

    /// Plain lambertian diffuse material
    pub fn lambertian(spec: Spectrum) -> Self {
        Self::Standard(BSDF::new(BxDF::Lambertian(spec)))
    }

    /// Transparent material
    pub fn transparent(tf: Texture, roughness: Float, eta: Float) -> Self {
        let kd = Texture::from(Spectrum::BLACK);
        let ks = Texture::from(Spectrum::WHITE);

        let k = 0.0;
        let is_transparent = true;
        let fresnel_enabled = true;

        Self::microfacet(
            roughness,
            eta,
            k,
            is_transparent,
            fresnel_enabled,
            kd, ks, tf,
        )
    }

    /// Perfect reflection
    pub fn mirror() -> Self {
        let kd = Texture::from(Spectrum::WHITE);
        let ks = Texture::from(Spectrum::WHITE);
        let tf = Texture::from(Spectrum::BLACK);

        let roughness = 0.0;
        let eta = 1e5;
        let k = 0.0;
        let is_transparent = false;
        let fresnel_enabled = true;

        Self::microfacet(
            roughness,
            eta,
            k,
            is_transparent,
            fresnel_enabled,
            kd, ks, tf,
        )
    }

    /// Perfect refraction
    pub fn glass() -> Self {
        let kd = Texture::from(Spectrum::WHITE);
        let ks = Texture::from(Spectrum::WHITE);
        let tf = Texture::from(Spectrum::WHITE);

        let eta = 1.5;
        let roughness = 0.0;
        let k = 0.0;
        let is_transparent = true;
        let fresnel_enabled = true;

        Self::microfacet(
            roughness,
            eta,
            k,
            is_transparent,
            fresnel_enabled,
            kd, ks, tf,
        )
    }

    /// Volumetric material for mediums
    pub fn volumetric(g: Float, sigma_t: Spectrum, sigma_s: Spectrum) -> Self {
        let bsdf = BSDF::new(BxDF::Volumetric(g, sigma_t, sigma_s));
        Self::Volumetric(bsdf)
    }

    /// Is the material specular? I.e. reflects light
    pub fn is_specular(&self) -> bool {
        match self {
            Self::Volumetric(..) => true,
            Self::Standard(bsdf) => bsdf.is_specular(),
            _ => false,
        }
    }

    /// Does the material scattering follow delta distribution?
    /// Dumb hack to make delta things not have shadows in path trace.
    pub fn is_delta(&self) -> bool {
        match self {
            Self::Standard(bsdf) => bsdf.is_delta(),
            _ => false,
        }
    }


    /// How much light emitted at `h`?
    pub fn emit(&self, lambda: &ColorWavelength, h: &Hit) -> Color {
        match self {
            Self::Light(t) => if h.backface {
                Color::BLACK
            } else {
                t.albedo_at(lambda, h)
            },
            _ => Color::BLACK
        }
    }

    /// BSDF evaluated at `h` for incoming `wo` and outgoing `wi` while
    /// transporting `mode`
    pub fn bsdf_f(
        &self,
        wo: Direction,
        wi: Direction,
        lambda: &ColorWavelength,
        mode: Transport,
        h: &Hit
    ) -> Color {
        match self {
            Self::Volumetric(bsdf) | Self::Standard(bsdf) => bsdf.f(wo, wi, lambda, h, mode),
            _ => Color::BLACK,
        }
    }

    /// Samples leaving direction from `h` from incoming direction `wo`
    pub fn bsdf_sample(
        &self,
        wo: Direction,
        h: &Hit,
        rand_sq: Vec2
    ) -> Option<Direction> {
        match self {
            Self::Volumetric(bsdf) | Self::Standard(bsdf) => bsdf.sample(wo, h, rand_sq),
            _ => None,
        }
    }

    /// PDF for direction `wi` at `h` with incoming direction `wo`
    pub fn bsdf_pdf(
        &self,
        wo: Direction,
        wi: Direction,
        h: &Hit,
        swap_dir: bool
    ) -> Float {
        let (wo, wi) = if swap_dir { (wi, wo) } else { (wo, wi) };
        match self {
            Self::Volumetric(bsdf) | Self::Standard(bsdf) => bsdf.pdf(wo, wi, h),
            _ => 0.0,
        }
    }

    /// Computes the shading cosine coefficient per material
    pub fn shading_cosine(&self, wi: Direction, ns: Normal) -> Float {
        match self {
            Self::Standard(_) => ns.dot(wi).abs(),
            _ => 1.0,
        }
    }
}
