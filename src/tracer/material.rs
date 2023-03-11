use crate::DVec3;
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;
use crate::tracer::scattering;
use crate::tracer::texture::Texture;

pub enum Material {
    Diffuse(Texture),
    Light(Texture),
    Mirror,
    Glass,
    Blank, // used for recursive objects. make translucent?
    /* yes, can use it as black background with plane */
}

impl Material {
    /* only lights emit */
    pub fn emit(&self, h: &Hit) -> DVec3 {
        match self {
            Self::Light(t) => t.albedo_at(h.p),
            _ => DVec3::ZERO,
        }
    }

    pub fn albedo(&self, h: &Hit) -> DVec3 {
        match self {
            Self::Diffuse(t) => t.albedo_at(h.p),
            Self::Mirror | Self::Glass => DVec3::ONE,
            _ => DVec3::ZERO,
        }
    }

    pub fn scatter(&self, h: &Hit, r: &Ray) -> Option<Ray> {
        match self {
            Self::Glass => scattering::refract_ray(h, r),
            Self::Mirror => scattering::reflect_ray(h, r),
            Self::Diffuse(_) => scattering::diffuse_scatter_ray(h, r),
            _ => None,
        }
    }

    pub fn is_translucent(&self) -> bool {
        match self {
            Self::Glass | Self::Blank => true,
            _ => false,
        }
    }
}
