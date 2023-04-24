use super::*;

/// An object of type medium. Mediums represent space where rays can scatter
/// at random depending on density. Examples of real life mediums include smoke,
/// fog, and clouds.
pub struct Medium {
    /// How much of each RGB channgel gets scattered when hitting the medium
    sigma_s: DVec3,
    /// Transmittance of the medium, defined as `sigma_a + sigma_s`, where
    /// `sigma_a` tells how much each RGB channel gets absorbed while
    /// traversing the medium
    sigma_t: DVec3,
    /// Material of the medium
    material: Material,
}

impl Medium {
    /// Constructs a medium contained in an invisible solid.
    ///
    /// # Arguments
    /// * `absorption` - How much of each RGB channel gets absorbed while
    /// traversing the medium. Value in `\[0,1\]^3`
    /// * `scattering` - How much of each RGB channel gets scattered on hit
    /// * `scatter_param` - Scattering parameter to Henyey-Greenstein in
    /// `(-1,1)`
    pub fn new(absorption: DVec3, scattering: DVec3, scatter_param: f64) -> Self {
        assert!(-1.0 < scatter_param && scatter_param < 1.0);
        assert!(scattering.min_element() >= 0.0);
        assert!(absorption.max_element() <= 1.0
                && absorption.min_element() >= 0.0);

        Self {
            sigma_s: scattering,
            sigma_t: scattering + absorption,
            material: Material::Volumetric(scatter_param),
        }
    }

    /// Computes the transmittance for the hit `h`. Checks if we hit the medium.
    pub fn transmittance(&self, h: &Hit) -> DVec3 {
        let t_delta = h.t;
        let transmittance = (-self.sigma_t * t_delta).exp();
        let density = if h.is_medium() {
            // we hit a medium
            self.sigma_t * transmittance
        } else {
            transmittance
        };

        let pdf = density.dot(DVec3::ONE) / 3.0;

        if h.is_medium() {
            self.sigma_s * transmittance / pdf
        } else {
            transmittance / pdf
        }
    }
}

impl Object for Medium {
    fn material(&self) -> &Material {
        &self.material
    }

    fn hit(&self, ro: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        // choose a random color channel from density
        let density = match 3.0 * rand_utils::rand_f64() {
            f if f < 1.0 => self.sigma_t.x,
            f if f < 2.0 => self.sigma_t.y,
            _ => self.sigma_t.z,
        };

        let ray_length = ro.dir.length();
        let inside_dist = (t_max - t_min) * ray_length;

        let hit_dist = -(1.0 - rand_utils::rand_f64()).ln() / density;
        // this way, the scale of the world matters.
        // doubt there are alternative ways?
        if hit_dist > inside_dist {
            None
        } else {
            let t = t_min + hit_dist / ray_length;
            // need shading normal to cancel out the dot product in integrator.
            // set geometric normal to zero, so hit does not do ray origin offset
            Hit::new(t, self, ro.at(t), DVec3::X, DVec3::ZERO, DVec2::ZERO)
        }
    }
}
