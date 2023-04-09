#![allow(unused_variables, dead_code)]
use super::*;

use crate::tracer::object::Solid;
use crate::tracer::texture::Texture;

/// An object of type medium. Mediums represent space where rays can scatter
/// at random depending on density. Examples of real life mediums include smoke,
/// fog, and clouds.
pub struct Medium {
    /// Density of the medium
    density: f64,
    /// Bounding object of the medium
    boundary: Box<dyn Solid>,
    /// Material of the medium
    isotropic: Material,
}

impl Medium {
    /// Constructs a medium contained in an invisible solid.
    ///
    /// # Arguments
    /// * `boundary` - Bounding object of the medium
    /// * `density` - Density of the medium. In range \[0,1\]
    /// * `color` - Color of the medium
    pub fn new(boundary: Box<dyn Solid>, density: f64, color: DVec3) -> Box<Self> {
        assert!((0.0..1.0).contains(&density));
        Box::new(Self {
            density,
            boundary,
            isotropic: Material::Isotropic(Texture::Solid(color), density),
        })
    }
}

impl Object for Medium {
    fn material(&self) -> &Material {
        &self.isotropic
    }

    fn hit(&self, ro: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        let to = match self.boundary.hit(ro, -INFINITY, INFINITY) {
            None => return None,
            Some(ho) => ho.t,
        };

        let ti = match self.boundary.hit(ro, to + EPSILON, INFINITY) {
            None => return None,
            Some(hi) => hi.t,
        };

        let to = to.max(t_min);
        let ti = ti.min(t_max);

        // medium is behind
        if to > ti {
            return None;
        }

        let ray_length = ro.dir.length();
        let inside_dist = (ti - to) * ray_length;

        let hit_dist = -(1.0 - rand_utils::rand_f64()).ln() / self.density;

        // this way, the scale of the world matters. doubt there are alternative
        // ways?
        if hit_dist > inside_dist {
            None
        } else {
            let t = to + hit_dist / ray_length;
            // store the ts in geometric normal, it's not needed :-)
            // need shading normal to cancel out the dot product in integrator.
            Hit::new(t, self, ro.at(t), DVec3::X, DVec3::new(to, t, ti))
        }
    }
}
