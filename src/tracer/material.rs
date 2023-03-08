use crate::DVec3;

use crate::tracer::scene::Scene;
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;
use crate::tracer::texture::Texture;
use crate::tracer::illumination;

pub enum Material {
    Phong(Texture),
    Light(Texture),
    Mirror,
    Glass,
    Blank, // used for recursive objects. make translucent?
}

impl Material {

    pub fn color(&self, h: &Hit, s: &Scene, r: &Ray) -> DVec3 {
        /* see phong_illum for meaning */
        let q = 5.0;
        let sc = DVec3::splat(0.15);

        match self {
            Self::Phong(t) => illumination::phong_illum(t, h, sc, q, s),
            Self::Light(t) => t.color_at(h.p),
            Self::Mirror => illumination::reflect_ray(h, r).color(s),
            Self::Glass => illumination::refract_ray(h, r).color(s),
            _ => DVec3::ZERO,
        }
    }

    pub fn is_translucent(&self) -> bool {
        match self {
            Self::Glass => true,
            _ => false,
        }
    }
}
