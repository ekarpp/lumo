use crate::perlin::Perlin;
use crate::DVec3;
use crate::tracer::object::{Object, Sphere, Plane, Triangle};
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;
use crate::tracer::material::Material;
use crate::tracer::texture::Texture;

#[cfg(test)]
mod scene_tests;

pub struct Scene {
    pub light: DVec3,
    pub ambient: DVec3,
    objects: Vec<Box<dyn Object>>,
}

impl Scene {
    pub fn size(&self) -> usize { self.objects.len() }

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
        let block_light = |h: &Hit| -> bool {
            !h.object.material().is_translucent()
                && (h.p - r.origin).length_squared() <
                (self.light - r.origin).length_squared()
        };

        for object in &self.objects {
            let h = object.hit(r);
            // h.is_some_and
            if h.filter(block_light).is_some() {
                return false;
            }
        }
        true
    }

    pub fn default() -> Scene {
        let l = DVec3::new(-0.3, 0.2, -0.1);
        Scene {
            light: l,
            ambient: DVec3::splat(0.15),
            objects: vec![
                // floor
                Plane::new(
                    DVec3::new(0.0, -0.5, 0.0),
                    DVec3::new(0.0, 1.0, 0.0),
                    Material::Phong(Texture::Checkerboard(
                        Box::new(Texture::Solid(DVec3::ZERO)),
                        Box::new(Texture::Marble(
                            Perlin::new(DVec3::ONE)
                        ))
                    )),
                ),
                // right
                Plane::new(
                    DVec3::new(3.0, 0.0, -3.0),
                    DVec3::new(-1.0, 0.0, 1.0),
                    Material::Phong(Texture::Solid(DVec3::new(0.0, 0.0, 1.0))),
                ),
                // left
                Plane::new(
                    DVec3::new(-3.0, 0.0, -3.0),
                    DVec3::new(1.0, 0.0, 1.0),
                    Material::Phong(Texture::Solid(DVec3::new(1.0, 0.0, 0.0))),
                ),
                // behind
                Plane::new(
                    DVec3::new(0.0, 0.0, 1.0),
                    DVec3::new(0.0, 0.0, -1.0),
                    Material::Phong(Texture::Solid(DVec3::new(1.0, 0.0, 1.0))),
                ),
                Sphere::new(
                    DVec3::new(0.0, 0.0, -1.0),
                    0.5,
                    Material::Phong(Texture::Solid(
                        DVec3::new(136.0, 8.0, 8.0) / 255.9
                    )),
                ),
                Triangle::new(
                    DVec3::new(-1.5, 1.5, -2.5),
                    DVec3::new(-1.8, 1.8, -2.0),
                    DVec3::new(-1.2, 1.8, -2.0),
                    Material::Phong(Texture::Marble(Perlin::new(DVec3::ONE))),
                ),
                Sphere::new(
                    DVec3::new(-0.9, 0.0, -1.0),
                    0.1,
                    Material::Mirror,
                ),
                Sphere::new(
                    DVec3::new(-0.4, -0.12, -0.5),
                    0.1,
                    Material::Glass,
                ),
                Sphere::new(
                    DVec3::new(0.4, -0.2, -0.5),
                    0.1,
                    Material::Phong(Texture::Marble(Perlin::new(
                        DVec3::new(255.0, 182.0, 193.0) / 255.9
                    ))),
                ),
            ]
        }
    }
}
