#![allow(unused_variables, dead_code)]
use super::*;

/// An object of type medium. Mediums represent space where rays can scatter
/// at random depending on density. Examples of real life mediums include smoke,
/// fog, and clouds.
pub struct Medium {
    /// Density of the medium
    density: f64,
    /// Color of the medium
    color: DVec3,
    /// Material of the medium
    material: Material,
}

impl Medium {
    /// Constructs a medium contained in an invisible solid.
    ///
    /// # Arguments
    /// * `density` - Density of the medium. In range \[0,1\]
    /// * `color` - Color of the medium
    pub fn new(density: f64, color: DVec3) -> Box<Self> {
        assert!((0.0..1.0).contains(&density));
        Box::new(Self {
            density,
            color,
            material: Material::Volumetric,
        })
    }

    /// TODO
    pub fn transmittance(&self, t_delta: f64) -> DVec3 {
        self.color * (-self.density * t_delta).exp()
    }
}

impl Object for Medium {
    fn material(&self) -> &Material {
        &self.material
    }

    fn hit(&self, ro: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        let ray_length = ro.dir.length();
        let inside_dist = (t_min - t_max) * ray_length;

        let hit_dist = -(1.0 - rand_utils::rand_f64()).ln() / self.density;

        // this way, the scale of the world matters.
        // doubt there are alternative ways?
        if hit_dist > inside_dist {
            None
        } else {
            let t = t_min + hit_dist / ray_length;

            // need shading normal to cancel out the dot product in integrator.
            Hit::new(t, self, ro.at(t), DVec3::X, DVec3::NAN, DVec2::ZERO)
        }
    }
}
