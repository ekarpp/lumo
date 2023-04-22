use super::*;

/// An object of type medium. Mediums represent space where rays can scatter
/// at random depending on density. Examples of real life mediums include smoke,
/// fog, and clouds.
pub struct Medium {
    /// Density of the medium
    density: f64,
    /// Percentage of how much each RGB channel gets absorbed by the medium
    absorption: DVec3,
    /// Material of the medium
    material: Material,
}

impl Medium {
    /// Constructs a medium contained in an invisible solid.
    ///
    /// # Arguments
    /// * `density` - Density of the medium. In range \[0,1\]
    /// * `absoprition` - Percentage of how much each RGB channel gets absorbed
    /// * `color` - Color of the medium
    pub fn new(density: f64, absorption: DVec3, color: DVec3) -> Self {
        assert!((0.0..1.0).contains(&density));
        assert!(absorption.max_element() <= 1.0
                && absorption.min_element() >= 0.0);
        // absorption = multi channel density
        Self {
            density,
            absorption,
            material: Material::Volumetric(color),
        }
    }

    /// TODO
    pub fn transmittance(&self, h: &Hit) -> DVec3 {
        let t_delta = h.t;
        let transmittance = (-self.density * t_delta).exp();
        let pdf = if h.is_medium() {
            self.density * transmittance
        } else {
            transmittance
        };
        DVec3::splat(transmittance) / pdf
    }
}

impl Object for Medium {
    fn material(&self) -> &Material {
        &self.material
    }

    fn hit(&self, ro: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        let ray_length = ro.dir.length();
        let inside_dist = (t_max - t_min) * ray_length;

        let hit_dist = -(1.0 - rand_utils::rand_f64()).ln() / self.density;
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
