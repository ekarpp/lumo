use glam::f64::DVec3;
use crate::tracer::ray::Ray;
use crate::tracer::hit::Hit;
use crate::tracer::material::Material;

pub struct Sphere {
    pub origin: DVec3,
    pub material: Material,
    pub radius: f64
}

impl Sphere {
    pub fn normal_at(&self, p: DVec3) -> DVec3 {
        (p - self.origin) / self.radius
    }

    pub fn hit(&self, r: &Ray) -> Option<Hit> {
        let tmp = r.origin - self.origin;
        // coefficients of "hit quadratic"
        // .dot faster than .length_squared, recheck
        let a = r.dir.dot(r.dir);
        let half_b = tmp.dot(r.dir);
        let c = tmp.dot(tmp) - self.radius*self.radius;
        let disc = half_b*half_b - a*c;

        if disc < 0.0 {
            return None;
        }
        let disc_root = disc.sqrt();
        let mut t = (-half_b - disc_root) / a;
        if t < crate::EPSILON {
            t = (-half_b + disc_root) / a;
            if t < crate::EPSILON {
                return None;
            }
        }
        Some(Hit::new(t, self))
     }
}
