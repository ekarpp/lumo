use crate::DVec3;
use std::f64::consts::PI;
use crate::pdfs::Pdf;
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;
use crate::tracer::bxdfs;
use crate::tracer::texture::Texture;

/// Describes which material an object is made out of
pub enum Material {
    /// "Normal material"
    Diffuse(Texture),
    /// Emits light
    Light(Texture),
    /// Perfect mirror
    Mirror,
    /// Refracts light
    Glass,
    /// Not specified. Used with objects that are built on top of other objects.
    Blank,
}

impl Material {
    /// How much light emitted at `h`?
    pub fn emit(&self, h: &Hit) -> DVec3 {
        match self {
            Self::Light(t) => t.albedo_at(h.p),
            _ => DVec3::ZERO,
        }
    }

    /// What is the color at p?
    pub fn bsdf_f(&self, p: DVec3) -> DVec3 {
        match self {
            Self::Diffuse(t) => t.albedo_at(p) * PI.recip(),
            Self::Mirror | Self::Glass => DVec3::ONE,
            _ => DVec3::ZERO,
        }
    }

    /// How does `r` get scattered at `h`?
    pub fn bsdf_pdf(&self, ho: &Hit, ro: &Ray) -> Option<Box<dyn Pdf>> {
        match self {
            Self::Glass => bxdfs::bsdf_glass_pdf(ho, ro),
            Self::Mirror => bxdfs::bsdf_mirror_pdf(ho, ro),
            Self::Diffuse(_) => bxdfs::bsdf_diffuse_pdf(ho, ro),
            _ => None,
        }
    }
}
