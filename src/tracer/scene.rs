use glam::f64::DVec3;
use crate::tracer::object::{Object, Sphere};
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;
use crate::tracer::material::Material;

pub struct Scene {
    pub light: DVec3,
    pub ambient: DVec3,
    objects: Vec<Box<dyn Object>>,
}

impl Scene {
    pub fn hit(&self, r: &Ray) -> Option<Hit> {
        let mut closest_hit: Option<Hit> = None;
        for sphere in &self.objects {
            let h = sphere.hit(r);
            // make cleaner?
            if closest_hit.is_none() {
                closest_hit = h;
            }
            else if h.is_some() && h < closest_hit {
                closest_hit = h;
            }
        }
        closest_hit
    }

    pub fn hit_light(&self, r: &Ray) -> bool {
        for sphere in &self.objects {
            let h = sphere.hit(r);
            // h.is_some_and
            if h.filter(|x| !x.object.material().is_translucent()).is_some() {
                return false;
            }
        }
        true
    }

    pub fn default() -> Scene {
        Scene {
            light: DVec3::new(-0.25, 0.35, -0.2),
            ambient: DVec3::splat(0.15),
            objects: vec![
                Sphere::new(
                    DVec3::new(0.0, -100.5, -1.0),
                    Material::Default(
                        DVec3::new(124.0, 252.0, 0.0) / 255.9
                    ),
                    100.0
                ),
                Sphere::new(
                    DVec3::new(0.0, 0.0, -1.0),
                    Material::Default(
                        DVec3::new(136.0, 8.0, 8.0) / 255.9
                    ),
                    0.5
                ),
                Sphere::new(
                    DVec3::new(-0.9, 0.0, -1.0),
                    Material::Mirror,
                    0.1
                ),
                Sphere::new(
                    DVec3::new(-0.4, -0.12, -0.5),
                    Material::Glass,
                    0.1
                ),
                Sphere::new(
                    DVec3::new(0.4, 0.0, -0.5),
                    Material::Glass,
                    0.1
                )
            ]
        }
    }
}
