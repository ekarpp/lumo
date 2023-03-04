use glam::f32::Vec3;
use crate::tracer::sphere::Sphere;
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;

pub struct Scene {
    pub light: Vec3,
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

pub fn def() -> Scene {
    Scene {
        light: Vec3::new(-0.0, 0.5, -0.5),
        objects: vec![
            Sphere{
                origin: Vec3::new(0.0, -100.5, -1.0),
                color: Vec3::new(124.0, 252.0, 0.0) / 256.0,
                radius: 100.0
            },
            Sphere{
                origin: Vec3::new(0.0, 0.0, -1.0),
                color: Vec3::new(136.0, 8.0, 8.0) / 256.0,
                radius: 0.5
            }
        ]
    }
}
