use glam::f64::DVec3;

use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;
use crate::tracer::illumination::{phong_illum, reflect_ray, refract_ray};

pub enum Material {
    Default(DVec3),
    Mirror,
    Glass,
}

impl Material {
    pub fn shade(&self, h: &Hit) -> DVec3 {
        let q = 3.0;
        let spec_coeff = DVec3::splat(0.9);
        match self {
            Material::Default(c) => phong_illum(c.clone(), h, spec_coeff, q),
            _ => DVec3::ZERO,
        }
    }

    pub fn reflect(&self, h: &Hit) -> Option<Ray> {
        match self {
            Material::Mirror => reflect_ray(h),
            _ => None,
        }
    }

    pub fn refract(&self, h: &Hit, r: &Ray) -> Option<Ray> {
        match self {
            Material::Glass => refract_ray(h, r),
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
