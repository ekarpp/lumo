use super::*;

#[cfg(test)]
mod plane_tests;

/// Plane defined by a single point and a normal
pub struct Plane {
    /// Unidirectional normal
    norm: DVec3,
    material: Material,
    /// `p.dot(-norm)`, used for fast hit calculations
    d: f64,
}

impl Plane {
    /* assume n != 0 */
    pub fn new(p: DVec3, n: DVec3, material: Material) -> Box<Self> {
        assert!(n.dot(n) != 0.0);
        let norm = n.normalize();
        Box::new(Self {
            norm,
            material,
            d: p.dot(-norm),
        })
    }
}

impl Object for Plane {
    fn material(&self) -> &Material { &self.material }

    fn area(&self) -> f64 { f64::INFINITY }

    // check that point is on plane?? or assume we are smart
    fn normal_at(&self, _p: DVec3) -> DVec3 {
        self.norm
    }

    fn hit(&self, r: &Ray) -> Option<Hit> {
        /* check if plane and ray are parallel. use epsilon instead?
         * or fail only if we get div by zero?? */
        if self.norm.dot(r.dir) == 0.0 {
            return None;
        }

        let t = -(self.d + self.norm.dot(r.origin)) / self.norm.dot(r.dir);
        if t < EPSILON {
            None
        } else {
            Hit::new(
                t,
                self,
                r,
            )
        }
    }

    fn sample_towards(&self, _ho: &Hit, _rand_sq: DVec2) -> (Ray, f64) {
        unimplemented!()
    }
    fn sample_on(&self, _rand_sq: DVec2) -> DVec3 { unimplemented!() }
}
