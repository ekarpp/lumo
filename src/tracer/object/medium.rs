use super::*;

/// An object of type medium. Mediums represent space where rays can scatter
/// at random depending on density. Examples of real life mediums include smoke,
/// fog, and clouds.
pub struct Medium {
    /// Density of the medium along each RGB channel
    density: DVec3,
    /// Color of the medium
    color: DVec3,
    /// Material of the medium
    material: Material,
}

impl Medium {
    /// Constructs a medium contained in an invisible solid.
    ///
    /// # Arguments
    /// * `density` - Density of the medium along each RBG channel in `\[0,1\]^3`
    /// * `color` - Color of the medium
    pub fn new(density: DVec3, color: DVec3) -> Self {
        assert!(density.max_element() <= 1.0
                && density.min_element() >= 0.0);

        Self {
            density,
            color,
            material: Material::Volumetric(1.0),
        }
    }

    /// TODO
    pub fn transmittance(&self, h: &Hit) -> DVec3 {
        let t_delta = h.t;
        let transmittance = (-self.density * t_delta).exp();
        let density = if h.is_medium() {
            // we hit a medium
            self.density * transmittance
        } else {
            transmittance
        };

        let pdf = density.dot(DVec3::ONE) / 3.0;

        if h.is_medium() {
            self.color * transmittance / pdf
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
            f if f < 1.0 => self.density.x,
            f if f < 2.0 => self.density.y,
            _ => self.density.z,
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
