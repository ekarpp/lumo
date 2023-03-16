use super::*;

use crate::tracer::object::sphere::Sphere;
use crate::tracer::texture::Texture;

pub struct Medium {
    density: f64,
    boundary: Sphere,
    isotropic: Material,
}

impl Medium {
    pub fn new(density: f64, origin: DVec3, radius: f64, color: DVec3)
               -> Box<Self> {
        Box::new(Self {
            density,
            boundary: *Sphere::new(origin, radius, Material::Blank),
            isotropic: Material::Isotropic(Texture::Solid(color)),
        })
    }
}

impl Object for Medium {
    fn density(&self) -> f64 { self.density }
    fn inside(&self, p: DVec3) -> bool { self.boundary.inside(p) }
    fn size(&self) -> usize { self.boundary.size() }
    fn area(&self) -> f64 { self.boundary.area() }
    fn material(&self) -> &Material { &self.isotropic }
    fn normal_at(&self, p: DVec3) -> DVec3 { p }
    fn sample_on(&self, _rand_sq: DVec2) -> DVec3 { unimplemented!() }
    fn sample_towards(&self, _xo: DVec3, _rand_sq: DVec2) -> Ray {
        unimplemented!()
    }

    fn hit(&self, ro: &Ray) -> Option<Hit> {
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
            Hit::new(to + hit_dist / ray_length, self, ro)
        }
    }
}
