use glam::f64::DVec3;
use crate::tracer::ray::Ray;
use crate::tracer::hit::Hit;
use crate::tracer::material::Material;

#[cfg(test)]
mod sphere_tests;
#[cfg(test)]
mod plane_tests;

pub trait Object {
    // unit length normal at p
    fn normal_at(&self, p: DVec3) -> DVec3;
    fn hit(&self, r: &Ray) -> Option<Hit>;
    fn material(&self) -> &Material;
    fn debug_light(&self) -> bool;
}

pub struct Plane {
    norm: DVec3,
    material: Material,
    d: f64, // for hit calc, store instead of point
}

impl Plane {
    pub fn new(p: DVec3, n: DVec3, m: Material) -> Box<Plane> {
        let norm = n.normalize();
        Box::new(Plane {
            norm: norm,
            material: m,
            d: p.dot(-norm),
        })
    }
}

impl Object for Plane {
    fn debug_light(&self) -> bool { true }
    fn material(&self) -> &Material { &self.material }
    // check that point is on plane?? or assume we are smart
    fn normal_at(&self, _p: DVec3) -> DVec3 { self.norm }

    fn hit(&self, r: &Ray) -> Option<Hit> {
        /* check if plane and ray are parallel. use epsilon instead?
         * or fail only if we get div by zero?? */
        if self.norm.dot(r.dir) == 0.0 {
            return None;
        }

        let t = -(self.d + self.norm.dot(r.origin)) / self.norm.dot(r.dir);
        if t < crate::EPSILON {
            None
        } else {
            Some(Hit::new(
                t,
                self,
                r.at(t),
                self.normal_at(r.at(t))
            ))
        }
    }
}

pub struct Sphere {
    pub origin: DVec3,
    pub radius: f64,
    material: Material,
}

impl Sphere {
    pub fn new(origin: DVec3, r: f64, mat: Material) -> Box<Sphere> {
        Box::new(Sphere {
            origin: origin,
            radius: r,
            material: mat,
        })
    }
}

impl Object for Sphere {
    fn debug_light(&self) -> bool { self.radius != crate::DEBUG_R }
    fn material(&self) -> &Material { &self.material }

    fn normal_at(&self, p: DVec3) -> DVec3 {
        (p - self.origin) / self.radius
    }

    fn hit(&self, r: &Ray) -> Option<Hit> {
        let tmp = r.origin - self.origin;
        // coefficients of "hit quadratic"
        // .dot faster than .length_squared, recheck
        let a = r.dir.dot(r.dir);
        let half_b = tmp.dot(r.dir);
        let c = tmp.dot(tmp) - self.radius*self.radius;
        let disc = half_b*half_b - a*c;

        if disc < 0.0 {
            return None;
        }
        let disc_root = disc.sqrt();
        let mut t = (-half_b - disc_root) / a;
        if t < crate::EPSILON {
            t = (-half_b + disc_root) / a;
            if t < crate::EPSILON {
                return None;
            }
        }
        Some(Hit::new(
            t,
            self,
            r.at(t),
            self.normal_at(r.at(t))
        ))
     }
}
