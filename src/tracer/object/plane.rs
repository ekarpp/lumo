use super::*;

#[cfg(test)]
mod plane_tests;

/// Plane defined by a single point and a normal
pub struct Plane {
    /// Unidirectional normal
    normal: DVec3,
    material: Material,
    /// `p.dot(-norm)`, used for fast hit calculations
    d: f64,
}

impl Plane {
    /// Constructs an infinite plane given a point and a normal
    pub fn new(p: DVec3, n: DVec3, material: Material) -> Box<Self> {
        assert!(n.dot(n) != 0.0);
        let normal = n.normalize();
        Box::new(Self {
            normal,
            material,
            d: p.dot(-normal),
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
        if self.normal.dot(r.dir) == 0.0 {
            return None;
        }

        let t = -(self.d + self.normal.dot(r.origin)) / self.normal.dot(r.dir);
        if t < t_min + EPSILON || t > t_max {
            None
        } else {
            let xi = r.at(t);
            let (u, v) = self.normal.any_orthonormal_pair();
            let uv = DVec2::new(u.dot(xi), v.dot(xi)).fract();
            Hit::new(t, self, xi, self.normal, self.normal, uv)
        }
    }
}
