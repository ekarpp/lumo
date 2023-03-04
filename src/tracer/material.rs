use glam::f32::Vec3;
use crate::tracer::hit::Hit;

pub trait Material {
    fn shade(&self, h: &Hit) -> Vec3;
    fn reflect(&self, h: &Hit) -> Vec3;
    fn transmit(&self, h: &Hit) -> Vec3;
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

    fn reflect(&self, h: &Hit) -> Vec3 {
        Vec3::ZERO
    }

    fn transmit(&self, h: &Hit) -> Vec3 {
        Vec3::ZERO
    }
}
