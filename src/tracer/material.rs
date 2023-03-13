use crate::DVec3;
use crate::tracer::hit::Hit;
use crate::tracer::ray::{Ray, ScatterRay};
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
    /// How much light emitted at h?
    pub fn emit(&self, h: &Hit) -> DVec3 {
        match self {
            Self::Light(t) => t.albedo_at(h.p),
            _ => DVec3::ZERO,
        }
    }

    /// What is the color at p?
    pub fn albedo_at(&self, p: DVec3) -> DVec3 {
        match self {
            Self::Diffuse(t) => t.albedo_at(p),
            Self::Mirror | Self::Glass => DVec3::ONE,
            _ => DVec3::ZERO,
        }
    }

    /// How does `r` get scattered at `h`?
    pub fn bsdf(&self, h: &Hit, r: &Ray) -> Option<ScatterRay> {
        match self {
            Self::Glass => bxdfs::glass_bsdf(h, r),
            Self::Mirror => bxdfs::mirror_bsdf(h, r),
            Self::Diffuse(_) => bxdfs::diffuse_bsdf(h, r),
            _ => None,
        }
    }

    /// Does light pass through the material?
    pub fn is_translucent(&self) -> bool {
        match self {
            Self::Glass | Self::Blank => true,
            _ => false,
        }
    }
}
