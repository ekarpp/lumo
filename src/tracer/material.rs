use glam::f32::Vec3;
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;

pub trait Material {
    fn shade(&self, h: &Hit) -> Vec3;
    fn reflect(&self, h: &Hit) -> Option<Ray>;
    fn transmit(&self, h: &Hit) -> Option<Ray>;
}

pub struct Default {}

impl Material for Default {
    fn shade(&self, h: &Hit) -> Vec3 {
        crate::tracer::phong::phong_shading(
            h,
            Vec3::splat(0.9),
            3.0
        )
    }

    fn reflect(&self, h: &Hit) -> Option<Ray> {
        None
    }

    fn transmit(&self, h: &Hit) -> Option<Ray> {
        None
    }
}
