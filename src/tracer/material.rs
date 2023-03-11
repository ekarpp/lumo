use crate::DVec3;

use crate::tracer::scene::Scene;
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;
use crate::tracer::texture::Texture;
use crate::tracer::illumination;

pub enum Material {
    Diffuse(Texture),
    Light(Texture),
    Mirror,
    Glass,
    Blank, // used for recursive objects. make translucent?
    /* yes, can use it as black background with plane */
}

impl Material {
    pub fn color(&self, h: &Hit, s: &Scene, r: &Ray) -> DVec3 {
        match self {
            Self::Diffuse(t) => illumination::illuminate(t, h, s),
            Self::Light(t) => t.albedo_at(h.p),
            Self::Mirror => illumination::reflect_ray(h, r).color(s),
            Self::Glass => illumination::refract_ray(h, r).color(s),
            _ => DVec3::ZERO,
        }
    }

    pub fn is_translucent(&self) -> bool {
        match self {
            Self::Glass | Self::Blank => true,
            _ => false,
        }
    }
}
