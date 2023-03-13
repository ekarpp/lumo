use super::*;

#[cfg(test)]
mod plane_tests;

pub struct Plane {
    norm: DVec3,
    material: Material,
    d: f64, // for hit calc, store instead of point
}

impl Plane {
    /* assume n != 0 */
    pub fn new(p: DVec3, n: DVec3, m: Material) -> Box<Self> {
        let norm = n.normalize();
        Box::new(Self {
            norm: norm,
            material: m,
            d: p.dot(-norm),
        })
    }
}

impl Object for Plane {
    fn material(&self) -> &Material { &self.material }

    fn area(&self) -> f64 { f64::INFINITY }

    // check that point is on plane?? or assume we are smart
    fn normal_for_at(&self, r: &Ray, _p: DVec3) -> DVec3 {
        _orient_normal(self.norm, r)
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
}
