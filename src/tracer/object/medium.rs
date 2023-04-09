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
    /// Constructs a medium contained in an invisible sphere.
    ///
    /// # Arguments
    /// * `density` - Density of the medium. In range \[0,1\]
    /// * `boundary` - Bounding object of the medium
    /// * `color` - Color of the medium
    pub fn new(density: f64, boundary: Box<dyn Solid>, color: DVec3) -> Box<Self> {
        assert!((0.0..1.0).contains(&density));
        Box::new(Self {
            density,
            boundary,
            isotropic: Material::Isotropic(Texture::Solid(color)),
        })
    }
}

impl Object for Medium {
    fn material(&self) -> &Material {
        &self.isotropic
    }

    fn hit(&self, ro: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        None
        /*
        let to = if self.boundary.inside(ro.origin) {
            0.0
        } else {
            match self.boundary.hit(ro) {
                None => return None,
                Some(ho) => ho.t,
            }
        };

        let ri = Ray::new(ro.at(to), ro.dir);

        let ti = match self.boundary.hit(&ri) {
            None => panic!(),
            Some(hi) => hi.t,
        };

        let ray_length = ro.dir.length();
        let inside_dist = (ti - to) * ray_length;

        let hit_dist = -(1.0 - rand_utils::rand_f64()).ln() / self.density;

        if hit_dist > inside_dist {
            None
        } else {
            let t = to + hit_dist / ray_length;
            Hit::new(t, self, ro.at(t), DVec3::ZERO)
        }
        */
    }
}
