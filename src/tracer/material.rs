use crate::tracer::bxdfs;
use crate::tracer::hit::Hit;
use crate::tracer::microfacet::MfDistribution;
use crate::tracer::pdfs::Pdf;
use crate::tracer::ray::Ray;
use crate::tracer::texture::Texture;
use glam::DVec3;
use std::f64::consts::PI;

/// Describes which material an object is made out of
pub enum Material {
    /// Glossy
    Microfacet(Texture, MfDistribution),
    /// Emits light
    Light(Texture),
    /// Perfect reflection
    Mirror,
    /// Perfect refraction with refraction index as argument
    Glass(f64),
    /// Volumetric material for mediums
    Volumetric(f64),
    /// Not specified. Used with objects that are built on top of other objects.
    Blank,
}

impl Material {
    /// Metallic microfacet material
    pub fn metal(texture: Texture, roughness: f64) -> Self {
        Self::Microfacet(texture, MfDistribution::metallic(roughness))
    }

    /// Specular microfacet material
    pub fn specular(texture: Texture, roughness: f64) -> Self {
        Self::Microfacet(texture, MfDistribution::specular(roughness))
    }

    /// Diffuse material
    pub fn diffuse(texture: Texture) -> Self {
        Self::Microfacet(texture, MfDistribution::diffuse())
    }

    /// Transparent material
    pub fn transparent(texture: Texture, rfrct_idx: f64, roughness: f64) -> Self {
        Self::Microfacet(texture, MfDistribution::transparent(rfrct_idx, roughness))
    }

    /// Perfect reflection
    pub fn mirror() -> Self {
        Self::Mirror
    }

    /// Perfect refraction
    pub fn glass(refraction_index: f64) -> Self {
        assert!(refraction_index >= 1.0);
        Self::Glass(refraction_index)
    }

    /// Is the material specular? I.e. reflects light
    pub fn is_specular(&self) -> bool {
        match self {
            Self::Mirror | Self::Glass(..) => true,
            Self::Microfacet(_, mfd) => mfd.is_specular(),
            _ => false,
        }
    }

    /// Does the material scattering follow delta distribution?
    /// Dumb hack to make delta things not have shadows in path trace.
    pub fn is_delta(&self) -> bool {
        matches!(self, Self::Mirror | Self::Glass(_))
    }


    /// How much light emitted at `h`?
    pub fn emit(&self, h: &Hit) -> DVec3 {
        match self {
            Self::Light(t) => t.albedo_at(h),
            _ => DVec3::ZERO,
        }
    }

    /// What is the color at `h`?
    pub fn bsdf_f(&self, wo: DVec3, wi: DVec3, h: &Hit) -> DVec3 {
        let ns = h.ns;
        let ng = h.ng;
        match self {
            // cancel the applied shading cosine for mirror and glass
            Self::Mirror | Self::Glass(..) => DVec3::ONE / ns.dot(wi).abs(),
            Self::Volumetric(..) => DVec3::ONE / (4.0 * PI * ns.dot(wi).abs()),
            Self::Microfacet(t, mfd) => {
                bxdfs::bsdf_microfacet(wo, wi, ng, t.albedo_at(h), mfd)
            }
            _ => DVec3::ZERO,
        }
    }

    /// How does `ro` get scattered at `ho`?
    pub fn bsdf_pdf(&self, ho: &Hit, ro: &Ray) -> Option<Box<dyn Pdf>> {
        match self {
            Self::Mirror => bxdfs::brdf_mirror_pdf(ho, ro),
            Self::Glass(ridx) => bxdfs::btdf_glass_pdf(ho, ro, *ridx),
            Self::Volumetric(_) => bxdfs::brdf_volumetric_pdf(ro),
            Self::Microfacet(t, mfd) => {
                bxdfs::bsdf_microfacet_pdf(ho, ro, t.albedo_at(ho), mfd)
            }
            _ => None,
        }
    }
}
