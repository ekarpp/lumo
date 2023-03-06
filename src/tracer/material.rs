use crate::DVec3;

use crate::tracer::scene::Scene;
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;
use crate::tracer::texture::Texture;
use crate::tracer::illumination;

pub enum Material {
    Phong(Texture),
    Mirror,
    Glass,
}

impl Material {
    pub fn shade(&self, h: &Hit, s: &Scene) -> Option<DVec3> {
        /* see phong_illum for meaning */
        let q = 5.0;
        let sc = DVec3::splat(0.15);

        match self {
            // return opt directlY??
            Material::Phong(t) => Some(
                illumination::phong_illum(t, h, sc, q, s)
            ),
            _ => None,
        }
    }

    pub fn reflect(&self, h: &Hit) -> Option<Ray> {
        match self {
            Material::Mirror => illumination::reflect_ray(h),
            _ => None,
        }
    }

    pub fn refract(&self, h: &Hit, r: &Ray) -> Option<Ray> {
        match self {
            Material::Glass => illumination::refract_ray(h, r),
            _ => None,
        }
    }

    pub fn is_translucent(&self) -> bool {
        match self {
            Material::Glass => true,
            _ => false,
        }
    }
}
