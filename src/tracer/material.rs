use crate::{ Normal, Direction, Transport, Float, Vec2, Image };
use crate::tracer::{
    Color, ColorWavelength, color::illuminants, Spectrum, hit::Hit, microfacet::MfDistribution,
    color::DenseSpectrum, texture::Texture, bsdf::BSDF, bxdf::BxDF, onb::Onb,
};

#[cfg(test)]
mod white_furnace_tests;

/// Describes which material an object is made out of
pub enum Material {
    /// Materials with standard BSDF
    Standard(BSDF, Option<Image<Normal>>),
    /// Emits light
    Light(Texture, &'static DenseSpectrum, Float, bool),
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
        bump_map: Option<Image<Normal>>,
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
        Self::Standard(bsdf, bump_map)
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
            None,
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
            None,
        )
    }

    /// Plain lambertian diffuse material
    pub fn lambertian(spec: Spectrum) -> Self {
        Self::Standard(BSDF::new(BxDF::Lambertian(spec)), None)
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
            None,
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
            None,
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
            None,
        )
    }

    /// Volumetric material for mediums
    pub fn volumetric(
        g: Float,
        t_scale: Float,
        sigma_t: Spectrum,
        sigma_s: Spectrum
    ) -> Self {
        let bsdf = BSDF::new(BxDF::Volumetric(g, t_scale, sigma_t, sigma_s));
        Self::Volumetric(bsdf)
    }

    /// Creates a light material
    pub fn light(ke: Texture) -> Self {
        Self::light_scale(ke, 1.0)
    }

    /// Create a light with emittance scaled by `scale`
    pub fn light_scale(ke: Texture, scale: Float) -> Self {
        Material::Light(ke, illuminants::D65, scale, false)
    }

    /// Are we a light?
    #[inline]
    pub fn is_light(&self) -> bool {
        matches!(self, Material::Light(..))
    }

    /// Is the material specular? I.e. reflects light
    pub fn is_specular(&self) -> bool {
        match self {
            Self::Volumetric(..) => true,
            Self::Standard(bsdf, _) => bsdf.is_specular(),
            _ => false,
        }
    }

    /// Does the material scattering follow delta distribution?
    /// Dumb hack to make delta things not have shadows in path trace.
    #[inline]
    pub fn is_delta(&self) -> bool {
        match self {
            Self::Standard(bsdf, _) => bsdf.is_delta(),
            _ => false,
        }
    }

    /// How much light emitted at `h`?
    #[inline]
    pub fn emit(&self, lambda: &ColorWavelength, h: &Hit) -> Color {
        match self {
            Self::Light(t, e, s, ts) => {
                if !ts && h.backface {
                    Color::BLACK
                } else {
                    *s * t.albedo_at(lambda, h.uv) * e.sample(lambda)
                }
            }
            _ => Color::BLACK,
        }
    }

    /// Power of light material
    #[inline]
    pub fn power(&self, lambda: &ColorWavelength) -> Color {
        match self {
            Self::Light(t, e, s, ts) => {
                let phi = *s * t.power(lambda) * e.sample(lambda);
                if !ts { phi } else { 2.0 * phi }
            }
            _ => Color::BLACK,
        }
    }

    /// BSDF evaluated at `h` for incoming `wo` and outgoing `wi` while
    /// transporting `mode`
    #[inline]
    pub fn bsdf_f(
        &self,
        wo: Direction,
        wi: Direction,
        lambda: &ColorWavelength,
        mode: Transport,
        h: &Hit
    ) -> Color {
        match self {
            Self::Volumetric(bsdf) => {
                bsdf.f(wo, wi, lambda, h.backface, h.t, h.ng, h.ns, h.uv, mode)
            }
            Self::Standard(bsdf, normal_map) => {
                let ns = Self::map_normal(h.ns, h.uv, normal_map.as_ref());
                bsdf.f(wo, wi, lambda, h.backface, h.t, h.ng, ns, h.uv, mode)
            }
            _ => Color::BLACK,
        }
    }

    /// Samples leaving direction from `h` from incoming direction `wo`
    #[inline]
    pub fn bsdf_sample(
        &self,
        wo: Direction,
        h: &Hit,
        rand_u: Float,
        rand_sq: Vec2,
    ) -> Option<Direction> {
        match self {
            Self::Volumetric(bsdf) => bsdf.sample(wo, h.ns, h.backface, rand_u, rand_sq),
            Self::Standard(bsdf, normal_map) => {
                let ns = Self::map_normal(h.ns, h.uv, normal_map.as_ref());
                bsdf.sample(wo, ns, h.backface, rand_u, rand_sq)
            }
            _ => None,
        }
    }

    /// PDF for direction `wi` at `h` with incoming direction `wo`
    #[inline]
    pub fn bsdf_pdf(
        &self,
        wo: Direction,
        wi: Direction,
        h: &Hit,
        swap_dir: bool
    ) -> Float {
        let (wo, wi) = if swap_dir { (wi, wo) } else { (wo, wi) };
        match self {
            Self::Volumetric(bsdf) => bsdf.pdf(wo, wi, h.ng, h.ns),
            Self::Standard(bsdf, normal_map) => {
                let ns = Self::map_normal(h.ns, h.uv, normal_map.as_ref());
                bsdf.pdf(wo, wi, h.ng, ns)
            }
            _ => 0.0,
        }
    }

    /// Computes the shading cosine coefficient per material
    #[inline]
    pub fn shading_cosine(&self, wi: Direction, ns: Normal) -> Float {
        match self {
            Self::Standard(..) => ns.dot(wi).abs(),
            _ => 1.0,
        }
    }

    #[inline(always)]
    fn map_normal(ns: Normal, uv: Vec2, normal_map: Option<&Image<Normal>>) -> Normal {
        if let Some(normal_map) = normal_map {
            let onb = Onb::new(ns);
            onb.to_world(normal_map.value_at(uv)).normalize()
        } else {
            ns
        }
    }
}
