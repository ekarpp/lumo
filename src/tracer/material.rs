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

    fn reflect(&self, _h: &Hit) -> Option<Ray> {
        None
    }

    fn transmit(&self, _h: &Hit) -> Option<Ray> {
        None
    }
}

pub struct Mirror {}

impl Material for Mirror {
    fn shade(&self, _h: &Hit) -> Vec3 {
        Vec3::ZERO
    }

    fn reflect(&self, h: &Hit) -> Option<Ray> {
        Some(Ray{
            origin: h.p + 0.001 * h.n,
            dir: h.p - 2.0 * h.p.dot(h.n).max(0.0) * h.n
        })
    }

    fn transmit(&self, _h: &Hit) -> Option<Ray> {
        None
    }
}
