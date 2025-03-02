use crate::{ Float, rng::Xorshift };
use crate::tracer::{
    hit::Hit, ray::Ray, Material, Texture, Color,
    ColorWavelength, Medium, Object, Rectangle, Sampleable
};

#[cfg(test)]
mod scene_tests;

/// Empty cornell box, custom material for floor, and left and right walls.
mod empty_box;

mod cornell_box;

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
    pub fn uniform_random_light(&self, rand_u: Float) -> &dyn Sampleable {
        let idx = (rand_u * self.lights.len() as Float).floor() as usize;
        self.lights[idx].as_ref()
    }

    /// Returns the transmittance due to volumetric medium
    pub fn transmittance(&self, lambda: &ColorWavelength, t: Float) -> Color {
        match &self.medium {
            None => Color::WHITE,
            Some(medium) => medium.transmittance(lambda, t),
        }
    }

    /// Returns the closest object `r` hits and `None` if no hits
    pub fn hit(&self, r: &Ray, rng: &mut Xorshift) -> Option<Hit> {
        let mut t_max = crate::INF;
        let mut h = None;

        if let Some(medium) = &self.medium {
            // if we hit an object, it must be closer than what we have
            h = medium.hit(r, rng, 0.0, t_max).or(h);
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

        #[cfg(debug_assertions)]
        {
            h = h.inspect(|h| {
                if self.medium.is_none() && h.t < crate::EPSILON {
                    println!("Suspiciously close hit ({}) at {}", h.t, h.p);
                }
            });
        }

        h
    }

    /// Distance to nearest object for `r`, `INF` if no intersections.
    pub fn hit_t(&self, r: &Ray, rng: &mut Xorshift) -> Float {
        let mut t = crate::INF;

        if let Some(medium) = &self.medium {
            t = t.min(medium.hit_t(r, rng, 0.0, t));
        }

        for object in &self.objects {
            t = t.min(object.hit_t(r, 0.0, t));
        }

        for light in &self.lights {
            t = t.min(light.hit_t(r, 0.0, t));
        }

        t
    }

    /// Does ray `r` reach the light object `light`?
    pub fn hit_light<'a>(
        &'a self,
        r: &Ray,
        rng: &mut Xorshift,
        light: &'a dyn Sampleable,
    ) -> Option<Hit<'a>> {
        let light_hit = light.hit(r, 0.0, crate::INF)?;
        let t_max = light_hit.t - crate::EPSILON;

        if let Some(medium) = &self.medium {
            if medium.hit_t(r, rng, 0.0, t_max) < t_max {
                return None;
            }
        }

        for object in &self.objects {
            if object.hit_t(r, 0.0, t_max) < t_max {
                return None;
            }
        }

        for light in &self.lights {
            if light.hit_t(r, 0.0, t_max) < t_max {
                return None;
            }
        }

        Some( light_hit )
    }
}
