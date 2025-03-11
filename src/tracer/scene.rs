use crate::{ Float, rng::Xorshift };
use crate::tracer::{
    hit::Hit, ray::Ray, Material, Texture, Color, BVH, Sphere, color::illuminants,
    ColorWavelength, Medium, Object, Rectangle, Sampleable, Instanceable
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
    pub objects: BVH<Box<dyn Object>>,
    /// Contains all lights in the scene.
    pub lights: BVH<Box<dyn Sampleable>>,
    /// Medium that the scene is filled with
    pub medium: Option<Medium>,
    /// Texture to use for environmental light
    pub environment_map: Option<Material>,
}

impl Scene {
    /// Build the scene
    pub fn build(&mut self) {
        self.objects.build();

        if let Some(env_map) = self.environment_map.take() {
            let bounds = self.objects.bounding_box()
                .merge(&self.lights.bounding_box());
            let center = bounds.center();
            let radius = center.distance(bounds.ax_min);
            self.add_light(
                Sphere::new(radius, env_map).translate(center.x, center.y, center.z)
            );
        }

        self.lights.build();

        if let Some(medium) = self.medium.as_mut() {
            let bounds = self.objects.bounding_box()
                .merge(&self.lights.bounding_box());
            let extent = bounds.extent();
            medium.set_extent(extent);
        }
    }

    /// Add a non-light object to the scene
    pub fn add(&mut self, obj: Box<dyn Object>) {
        // how to check material is not light?
        self.objects.add(obj);
    }

    /// Adds a light to the scene
    pub fn add_light(&mut self, light: Box<dyn Sampleable>) {
        // how to check material is light?
        // trait upcasting experimental, store lights and objects in separate BVHs
        self.lights.add(light);
    }

    /// Sets the volumetric medium of the scene
    pub fn set_medium(&mut self, medium: Medium) {
        self.medium = Some(medium);
    }

    /// Set the texture to use for environment light
    pub fn set_environment_map(&mut self, env_map: Texture, scale: Float) {
        self.environment_map = Some(
            Material::Light(env_map, illuminants::D65, scale, true)
        );
    }

    /// Returns number of lights in the scene
    pub fn num_lights(&self) -> usize {
        self.lights.num_objects()
    }

    /// Return the number of objects in the scene
    pub fn num_primitives(&self) -> usize {
        self.objects.num_primitives() + self.lights.num_primitives()
    }

    /// Number of shadow rays that should be shot in the scene
    pub fn num_shadow_rays(&self) -> usize {
        self.num_lights().ilog2().max(1) as usize
    }

    /// Choose one of the lights uniformly at random.
    /// Return reference and probability of sample.
    pub fn sample_light(&self, rand_u: Float) -> usize {
        self.lights.sample_light(rand_u)
    }

    /// Get light with BVH index `idx` and probability for it to get samped
    pub fn get_light(&self, idx: usize) -> (&dyn Sampleable, Float) {
        self.lights.get_light(idx)
    }

    /// Get light from BVH at hit `h` and probability for it to get sampled
    pub fn get_light_at(&self, h: &Hit) -> Option<usize> {
        self.lights.get_light_at(h)
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

        h = self.objects.hit(r, 0.0, t_max).or(h);
        t_max = h.as_ref().map_or(t_max, |hit| hit.t);

        h = self.lights.hit(r, 0.0, t_max).or(h);

        #[cfg(debug_assertions)]
        {
            h = h.inspect(|h| {
                if self.medium.is_none() && h.t < crate::EPSILON.powf(1.5) {
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

        t = t.min(self.objects.hit_t(r, 0.0, t));

        t = t.min(self.lights.hit_t(r, 0.0, t));

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

        if self.objects.hit_t(r, 0.0, t_max) < t_max {
            return None;
        }

        if self.lights.hit_t(r, 0.0, t_max) < t_max {
            return None;
        }

        Some( light_hit )
    }
}
