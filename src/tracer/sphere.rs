use glam::f32::Vec3;
use crate::tracer::ray::Ray;

pub struct Sphere {
    pub origin: Vec3,
    pub radius: f32
}

impl Sphere {
    pub fn hit(&self, r: &Ray) -> bool {
        let tmp = r.origin - self.origin;
        // coefficients of "hit quadratic"
        let a = r.dir.dot(r.dir);
        let b = 2.0 * tmp.dot(r.dir);
        let c = tmp.dot(tmp) - self.radius*self.radius;
        let disc = b*b - 4.0*a*c;

        if disc < 0.0 {
            return false;
        }
        let disc_root = disc.sqrt();
        let roots = [(-b + disc_root) / (2.0*a), (-b - disc_root) / (2.0*a)];
        !roots.iter().all(|&v| v < 0.0)
    }
}
