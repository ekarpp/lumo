use crate::{ Float, rand_utils };
use crate::tracer::{
    hit::Hit, ray::Ray, Material, Texture, Color,
    Medium, Object, Rectangle, Sampleable
};

#[cfg(test)]
mod scene_tests;

/// Empty cornell box, custom material for floor, and left and right walls.
mod empty_box;

/// Defines a scene in 3D space
#[derive(Default)]
pub struct Scene {
    /// All of the objects in the scene.
    pub objects: Vec<Box<dyn Object>>,
    /// Contains all lights in the scene.
    pub lights: Vec<Box<dyn Sampleable>>,
    /// Medium that the scene is filled with
    pub medium: Option<Medium>,
}

impl Scene {
    /// Add a non-light object to the scene
    pub fn add(&mut self, obj: Box<dyn Object>) {
        // how to check material is not light?
        self.objects.push(obj);
    }

    /// Adds a light to the scene
    pub fn add_light(&mut self, light: Box<dyn Sampleable>) {
        // how to check material is light?
        self.lights.push(light);
    }

    /// Sets the volumetric medium of the scene
    pub fn set_medium(&mut self, medium: Medium) {
        self.medium = Some(medium);
    }

    /// Returns number of lights in the scene
    pub fn num_lights(&self) -> usize {
        self.lights.len()
    }

    /// Choose one of the lights uniformly at random. Crash if no lights.
    pub fn uniform_random_light(&self) -> &dyn Sampleable {
        let rnd = rand_utils::rand_float();
        let idx = (rnd * self.lights.len() as Float).floor() as usize;
        self.lights[idx].as_ref()
    }

    /// Returns the transmittance due to volumetric medium
    pub fn transmittance(&self, h: &Hit) -> Color {
        match &self.medium {
            None => Color::WHITE,
            Some(medium) => medium.transmittance(h),
        }
    }

    /// Returns the closest object `r` hits and `None` if no hits
    pub fn hit(&self, r: &Ray) -> Option<Hit> {
        let mut t_max = crate::INF;
        let mut h = None;

        if let Some(medium) = &self.medium {
            // if we hit an object, it must be closer than what we have
            h = medium.hit(r, 0.0, t_max).or(h);
            // update distance to closest found so far
            t_max = h.as_ref().map_or(t_max, |hit| hit.t);
        }

        for object in &self.objects {
            // if we hit an object, it must be closer than what we have
            h = object.hit(r, 0.0, t_max).or(h);
            // update distance to closest found so far
            t_max = h.as_ref().map_or(t_max, |hit| hit.t);
        }

        // lazy, something better should be done.
        // use enum wrapper? have issues with instances..
        for light in &self.lights {
            h = light.hit(r, 0.0, t_max).map(|mut hit| {
                t_max = hit.t;
                hit.light = Some(light.as_ref());
                hit
            }).or(h);
        }

        h
    }

    /// Does ray `r` reach the light object `light`?
    pub fn hit_light<'a>(&'a self, r: &Ray, light: &'a dyn Sampleable) -> Option<Hit> {
        let light_hit = match light.hit(r, 0.0, crate::INF) {
            None => return None,
            Some(hi) => hi,
        };
        // consider also checking medium
        let t_max = light_hit.t - crate::EPSILON;

        for object in &self.objects {
            if object.hit(r, 0.0, t_max).is_some() {
                return None;
            }
        }

        for light in &self.lights {
            if light.hit(r, 0.0, t_max).is_some() {
                return None;
            }
        }

        Some( light_hit )
    }
}
