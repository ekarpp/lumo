use glam::{DVec3, DMat3};
use std::f64::INFINITY;
use crate::rand_utils;
use crate::srgb_to_lin;
use crate::consts::EPSILON;
use crate::tracer::{ray::Ray, hit::Hit, Texture, Material};
use crate::tracer::{Object, Plane, Rectangle};

#[cfg(test)]
mod scene_tests;

/// Empty cornell box, custom material for floor, and left and right walls.
mod empty_box;

/// Defines a scene in 3D space
#[derive(Default)]
pub struct Scene {
    /// All of the objects in the scene.
    pub objects: Vec<Box<dyn Object>>,
    /// Contains indices to all of the lights in `objects`
    pub lights: Vec<usize>,

}

impl Scene {
    /// Add an object to the scene
    pub fn add(&mut self, obj: Box<dyn Object>) {
        // add index to light vector if object is light
        if matches!(obj.material(), Material::Light(_)) {
            self.lights.push(self.objects.len());
        }
        self.objects.push(obj);
    }

    /// Returns number of lights in the scene
    pub fn num_lights(&self) -> usize {
        self.lights.len()
    }

    /// Choose one of the lights uniformly at random. Crash if no lights.
    pub fn uniform_random_light(&self) -> &dyn Object {
        let rnd = rand_utils::rand_f64();
        let idx = (rnd * self.lights.len() as f64).floor() as usize;
        self.objects[self.lights[idx]].as_ref()
    }

    /// Returns the closest object `r` hits and `None` if no hits
    pub fn hit(&self, r: &Ray) -> Option<Hit> {
        let mut t_max = INFINITY;
        let mut h = None;

        for object in &self.objects {
            // if we hit an object, it must be closer than what we have
            h = object.hit(r, 0.0, t_max)
                .or(h);
            // update distance to closest found so far
            t_max = h.as_ref().map_or(t_max, |hit| hit.t);
        }

        h
    }

    /// Does ray `r` reach the light object `light`? TODO: rewrite
    pub fn hit_light<'a>(&'a self, r: &Ray, light: &'a dyn Object)
                     -> Option<Hit> {
        let light_hit = light.hit(r, 0.0, INFINITY);
        let t_max = light_hit.as_ref().map_or(INFINITY, |hit| hit.t - EPSILON);

        if t_max == INFINITY {
            return None;
        }

        for object in &self.objects {
            if object.hit(r, 0.0, t_max).is_some() {
                return None;
            }
        }

        light_hit
    }
}
