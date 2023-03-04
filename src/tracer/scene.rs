use glam::f64::DVec3;
use crate::tracer::sphere::Sphere;
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;
use crate::tracer::material::{Default, Mirror};

pub struct Scene {
    pub light: DVec3,
    pub ambient: DVec3,
    objects: Vec<Sphere>
}

impl Scene {
    pub fn hit(&self, r: &Ray) -> Option<Hit> {
        let mut closest_hit: Option<Hit> = None;
        for sphere in &self.objects {
            let h = sphere.hit(r);
            if closest_hit.is_none() {
                closest_hit = h;
            }
            else if h.is_some() && h < closest_hit {
                closest_hit = h;
            }
        }
        closest_hit
    }

    pub fn hit_shadow(&self, r: &Ray) -> bool {
        for sphere in &self.objects {
            let h = sphere.hit(r);
            if h.is_some() {
                return true;
            }
        }
        false
    }
}

pub fn default() -> Scene {
    Scene {
        light: DVec3::new(-0.25, 0.35, -0.2),
        ambient: DVec3::splat(0.15),
        objects: vec![
            Sphere {
                origin: DVec3::new(0.0, -100.5, -1.0),
                color: DVec3::new(124.0, 252.0, 0.0) / 256.0,
                material: Box::new(Default {}),
                radius: 100.0
            },
            Sphere {
                origin: DVec3::new(0.0, 0.0, -1.0),
                color: DVec3::new(136.0, 8.0, 8.0) / 256.0,
                material: Box::new(Default {}),
                radius: 0.5
            },
            Sphere {
                origin: DVec3::new(-0.9, 0.0, -1.0),
                color: DVec3::splat(211.0) / 255.9,
                material: Box::new(Mirror {}),
                radius: 0.1
            }
        ]
    }
}
