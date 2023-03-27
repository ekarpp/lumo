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
    /// Constructs an infinite plane given a point and a normal
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
    fn material(&self) -> &Material {
        &self.material
    }

    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        /* check if plane and ray are parallel. use epsilon instead?
         * or fail only if we get div by zero?? */
        if self.norm.dot(r.dir) == 0.0 {
            return None;
        }

        let t = -(self.d + self.norm.dot(r.origin)) / self.norm.dot(r.dir);
        if t < t_min + EPSILON || t > t_max - EPSILON {
            None
        } else {
            Hit::new(t, self, r.at(t), self.norm)
        }
    }

    fn sample_towards(&self, _xo: DVec3, _rand_sq: DVec2) -> Ray {
        unimplemented!()
    }
    fn sample_towards_pdf(&self, _ri: &Ray) -> f64 {
        unimplemented!()
    }
    fn sample_on(&self, _rand_sq: DVec2) -> DVec3 {
        unimplemented!()
    }
}
