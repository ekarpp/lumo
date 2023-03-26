use glam::DVec3;
use crate::tracer::pdfs::Pdf;
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;
use crate::tracer::bxdfs;
use crate::tracer::texture::Texture;
use crate::tracer::microfacet::MfDistribution;

/// Describes which material an object is made out of
pub enum Material {
    /// Glossy
    Microfacet(Texture, MfDistribution),
    /// Emits light
    Light(Texture),
    /// Perfect mirror
    Mirror,
    /// Isotropic medium
    Isotropic(Texture),
    /// Refracts light
    Glass,
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
        Self::Microfacet(
            texture,
            MfDistribution::transparent(rfrct_idx, roughness)
        )
    }

    /// Is the material specular? i.e does it reflect/refract light
    pub fn is_specular(&self) -> bool {
        match self {
            Self::Glass | Self::Mirror => true,
            Self::Microfacet(_, mfd) => mfd.is_specular(),
            _ => false,
        }
    }

    /// Is the material diffuse? I.e. do shadow rays have effect on it.
    /// (CORRECT TERM?)
    pub fn is_diffuse(&self) -> bool {
        matches!(self, Self::Microfacet(..))
    }

    /// How much light emitted at `h`?
    pub fn emit(&self, h: &Hit) -> DVec3 {
        match self {
            Self::Light(t) => t.albedo_at(h.p),
            _ => DVec3::ZERO,
        }
    }

    /// What is the color at `ri.origin`?
    pub fn bsdf_f(&self, ro: &Ray, ri: &Ray, no: DVec3) -> DVec3 {
        let xo = ri.origin;
        match self {
            Self::Isotropic(t) => t.albedo_at(xo),
            Self::Mirror | Self::Glass => DVec3::ONE,
            Self::Microfacet(t, mfd) => {
                bxdfs::bsdf_microfacet(ro, ri, no, t.albedo_at(xo), mfd)
            }
            _ => DVec3::ZERO,
        }
    }

    /// How does `r` get scattered at `h`?
    pub fn bsdf_pdf(&self, ho: &Hit, ro: &Ray) -> Option<Box<dyn Pdf>> {
        match self {
            Self::Glass => bxdfs::btdf_glass_pdf(ho, ro),
            Self::Mirror => bxdfs::brdf_mirror_pdf(ho, ro),
            Self::Microfacet(t, mfd) => {
                let xo = ho.p;
                bxdfs::bsdf_microfacet_pdf(ho, ro, t.albedo_at(xo), mfd)
            }
            Self::Isotropic(_) => bxdfs::bsdf_isotropic_pdf(ho, ro),
            _ => None,
        }
    }
}
