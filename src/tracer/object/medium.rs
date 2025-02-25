use super::*;
use crate::tracer::Color;

#[cfg(test)]
mod medium_test;

/// A homogenous participating media. Mediums represent space where rays get
/// absorbed and can scatter at random depending on density.
/// Examples of real life mediums include smoke, fog, and clouds.
pub struct Medium {
    /// Transmittance of the medium, defined as `sigma_a + sigma_s`, where
    /// `sigma_a` tells how much each RGB channel gets absorbed while traversing the medium
    sigma_t: Vec3,
    /// Material of the medium
    material: Material,
}

impl Medium {
    /// Constructs a medium contained in an invisible solid.
    ///
    /// # Arguments
    /// * `absorption` - How much of each RGB channel gets absorbed while
    ///   traversing the medium. Value in `\[0,1\]^3`
    /// * `scattering` - How much of each RGB channel gets scattered on hit
    /// * `scatter_param` - Scattering parameter to Henyey-Greenstein in `(-1,1)`
    pub fn new(absorption: Vec3, scattering: Vec3, scatter_param: Float) -> Self {
        assert!(-1.0 < scatter_param && scatter_param < 1.0);
        assert!(scattering.min_element() >= 0.0);
        assert!(absorption.max_element() <= 1.0
                && absorption.min_element() >= 0.0);

        let sigma_s = Color::from(scattering);
        let sigma_t = scattering + absorption;

        Self {
            sigma_t,
            material: Material::volumetric(scatter_param, sigma_t, sigma_s),
        }
    }

    /// Computes the transmittance for the distance `t`.
    pub fn transmittance(&self, t_delta: Float) -> Color {
        // need to move some of the stuff to bsdf?
        let transmittance = (-self.sigma_t * t_delta).exp();

        let pdf = transmittance.dot(Vec3::ONE) / 3.0;

        if pdf == 0.0 {
            // this medium does not do much...
            Color::WHITE
        } else {
            Color::from(transmittance / pdf)
        }
    }
}

impl Object for Medium {
    fn hit(&self, ro: &Ray, t_min: Float, t_max: Float) -> Option<Hit> {
        // choose a random color channel from density
        let density = match 3.0 * rand_utils::rand_float() {
            f if f < 1.0 => self.sigma_t.x,
            f if f < 2.0 => self.sigma_t.y,
            _ => self.sigma_t.z,
        };

        // this channel never gets hit
        if density == 0.0 {
            return None;
        }

        let ray_length = ro.dir.length();
        let inside_dist = (t_max - t_min) * ray_length;

        let hit_dist = -(1.0 - rand_utils::rand_float()).ln() / density;
        // this way, the scale of the world matters.
        // doubt there are alternative ways?
        if hit_dist > inside_dist {
            None
        } else {
            let t = t_min + hit_dist / ray_length;
            let xi = ro.at(t);
            // need shading normal to cancel out the dot product in integrator.
            let ns = Normal::Z;
            let ng = Normal::Z;
            let uv = Vec2::ZERO;
            let err = Vec3::ZERO;

            Hit::new(t, &self.material, -ng, xi, err, ns, ng, uv)
        }
    }
}
