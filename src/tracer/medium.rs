use crate::{ Float, Vec3, Normal, rng::Xorshift, Vec2 };
use crate::tracer::{
    RGB, Color, ColorWavelength, Material, ray::Ray, hit::Hit, Spectrum
};

#[cfg(test)]
mod medium_test;

/// A homogenous participating media. Mediums represent space where rays get
/// absorbed and can scatter at random depending on density.
/// Examples of real life mediums include smoke, fog, and clouds.
pub struct Medium {
    /// Transmittance of the medium, defined as `sigma_a + sigma_s`, where
    /// `sigma_a` tells how much each RGB channel gets absorbed while traversing the medium
    sigma_t: Spectrum,
    g: Float,
    sigma_s: Spectrum,
    /// Material of the medium
    material: Material,
    /// Transform to make the scene fit inside an unit cube
    t_scale: Float,
}

impl Medium {
    /// Constructs a medium contained in an invisible solid.
    ///
    /// # Arguments
    /// * `absorption` - How much of each RGB channel gets absorbed while
    ///   traversing the medium. Value in `\[0,1\]^3`
    /// * `scattering` - How much of each RGB channel gets scattered on hit
    /// * `scatter_param` - Scattering parameter to Henyey-Greenstein in `(-1,1)`
    pub fn new(absorption: RGB, scattering: RGB, scatter_param: Float) -> Self {
        assert!(-1.0 < scatter_param && scatter_param < 1.0);
        let sigma_t = Spectrum::from_rgb(scattering.clone() + absorption);
        let sigma_s = Spectrum::from_rgb(scattering);
        let t_scale = 1.0;

        Self {
            sigma_t: sigma_t.clone(),
            sigma_s: sigma_s.clone(),
            material: Material::volumetric(scatter_param, t_scale, sigma_t, sigma_s),
            g: scatter_param,
            t_scale,
        }
    }

    /// Set the extent of the scene
    pub fn set_extent(&mut self, extent: Vec3) {
        self.t_scale = 1.0 / extent.max_element();
        self.material = Material::volumetric(
            self.g,
            self.t_scale,
            self.sigma_t.clone(),
            self.sigma_s.clone(),
        );
    }

    /// Computes the transmittance for the distance `t`.
    pub fn transmittance(&self, lambda: &ColorWavelength, t_delta: Float) -> Color {
        let t_delta = t_delta * self.t_scale;
        // need to move some of the stuff to bsdf?
        let transmittance = (-self.sigma_t.sample(lambda) * t_delta).exp();

        let pdf = transmittance.mean();

        if pdf == 0.0 {
            // this medium does not do much...
            Color::WHITE
        } else {
            transmittance / pdf
        }
    }

    /// Get a hit to `self` for `r`, if any
    pub fn hit(
        &self,
        r: &Ray,
        rng: &mut Xorshift,
        t_min: Float,
        t_max: Float
    ) -> Option<Hit> {
        let t = self.hit_t(r, rng, t_min, t_max);

        if t <= t_min || t >= t_max {
            None
        } else {
            let xi = r.at(t);
            // need shading normal to cancel out the dot product in integrator.
            let ns = Normal::Z;
            let ng = Normal::Z;
            let uv = Vec2::ZERO;
            let err = Vec3::ZERO;

            Hit::new(t, &self.material, -ng, xi, err, ns, ng, uv)
        }
    }

    /// Get a distance for hit to `self` for `r`, `INF` if no hit
    pub fn hit_t(&self, r: &Ray, rng: &mut Xorshift, t_min: Float, t_max: Float) -> Float {
        // choose a random color channel from density
        let lambda = ColorWavelength::sample_one_uniform(rng.gen_float());
        let density = self.sigma_t.sample_one(lambda);

        // this channel never gets hit
        if density == 0.0 {
            crate::INF
        } else {
            // distance inside the medium
            let inside_t = -(1.0 - rng.gen_float()).ln() / density;

            let ray_length = r.dir.length() * self.t_scale;
            let inside_dist = (t_max - t_min) * ray_length;

            // this way, the scale of the world matters
            if inside_t > inside_dist {
                crate::INF
            } else {
                let ray_length = r.dir.length() * self.t_scale;
                let t = t_min + inside_t / ray_length;
                if t <= t_min || t >= t_max {
                    crate::INF
                } else {
                    t
                }
            }
        }
    }
}
