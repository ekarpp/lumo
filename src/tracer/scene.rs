use crate::{DVec3, DMat3, DAffine3};
use std::f64::{INFINITY, consts::PI};
use crate::rand_utils;
#[allow(unused_imports)]
use crate::perlin::Perlin;
use crate::consts::EPSILON;
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;
use crate::tracer::texture::Texture;
use crate::tracer::material::Material;
use crate::tracer::object::{Object, plane::Plane, rectangle::Rectangle};
use crate::tracer::object::{cuboid::Cuboid, sphere::Sphere};

#[cfg(test)]
mod scene_tests;

/// "Cornell Box"
pub mod box_scene;
/// Scene showing capabilities of the renderer
pub mod default_scene;

/// Defines a scene in 3D space
pub struct Scene {
    /// All of the objects in the scene.
    pub objects: Vec<Box<dyn Object>>,
    /// Contains indices to all of the lights in `objects`
    pub lights: Vec<usize>,

}

impl Scene {
    pub fn new(objects: Vec<Box<dyn Object>>) -> Self {
        let lights = (0..objects.len()).map(|i: usize| {
            match objects[i].material() {
                Material::Light(_) => i,
                _ => objects.len(),
            }
        }).filter(|i: &usize| *i != objects.len()).collect();

        Self {
            objects,
            lights,
        }
    }

    /// Choose one of the lights uniformly at random.
    pub fn uniform_random_light(&self) -> &dyn Object {
        let rnd = rand_utils::rand_f64();
        let idx = (rnd * self.lights.len() as f64).floor() as usize;
        self.objects[self.lights[idx]].as_ref()
    }

    /// Returns the closest object `r` hits and `None` if no hits
    pub fn hit(&self, r: &Ray) -> Option<Hit> {
        self.objects.iter()
            .map(|obj| obj.hit(r, 0.0, INFINITY))
            .fold(None, |closest, hit| {
                if closest.is_none() || (hit.is_some() && hit < closest) {
                    hit
                } else {
                    closest
                }
            })
    }

    /// Does ray `r` reach the light object `light`? TODO: rewrite
    pub fn hit_light<'a>(&'a self, r: &Ray, light: &'a dyn Object)
                     -> Option<Hit> {
        let light_hit = light.hit(r, 0.0, INFINITY).map(|mut h| {
            h.t -= EPSILON;
            h
        });

        // ...
        let no_block_light = |obj: &&dyn Object| -> bool {
            let hi = obj.hit(r, 0.0, INFINITY);
            hi.is_none() || hi > light_hit
        };

        let reached_light = self.objects.iter()
            .map(|obj| &**obj) // ...
            .take_while(no_block_light)
            .count() == self.objects.len();

        if reached_light { light_hit } else { None }
    }
}
