use glam::f32::Vec3;
use crate::tracer::ray::Ray;
use crate::tracer::hit::Hit;

pub struct Sphere {
    pub origin: Vec3,
    pub color: Vec3,
    pub radius: f32
}

impl Sphere {
    pub fn hit(&self, r: &Ray) -> Option<Hit> {
        let tmp = r.origin - self.origin;
        // coefficients of "hit quadratic"
        let a = r.dir.dot(r.dir);
        let b = 2.0 * tmp.dot(r.dir);
        let c = tmp.dot(tmp) - self.radius*self.radius;
        let disc = b*b - 4.0*a*c;

        if disc < 0.0 {
            return None;
        }
        let disc_root = disc.sqrt();
        let mut t = (-b - disc_root) / (2.0*a);
        let eps = 0.001;
        if t < eps {
            t = (-b + disc_root) / (2.0*a);
            if t < eps {
                return None;
            }
        }
        Some(Hit{
            t: t,
            sphere: self
        })
    }
}
