use super::*;

#[cfg(test)]
mod sphere_tests;

pub struct Sphere {
    pub origin: DVec3,
    pub radius: f64,
    material: Material,
}

impl Sphere {
    /* assume r != 0 */
    pub fn new(origin: DVec3, r: f64, mat: Material) -> Box<Self> {
        Box::new(Self {
            origin: origin,
            radius: r,
            material: mat,
        })
    }
}

impl Object for Sphere {
    fn inside(&self, r: &Ray) -> bool {
        self.origin.distance_squared(r.origin + crate::EPSILON*r.dir)
            < self.radius*self.radius
    }

    fn material(&self) -> &Material { &self.material }

    fn normal_for_at(&self, r: &Ray, p: DVec3) -> DVec3 {
        _orient_normal((p - self.origin) / self.radius, r)
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
        Hit::new(
            t,
            self,
            r,
        )
     }
}
