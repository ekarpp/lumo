use super::*;

#[cfg(test)]
mod plane_tests;

/// Infinite plane defined by a single point and a normal
pub struct Plane {
    /// Unidirectional normal
    normal: DVec3,
    /// Material of the plane
    material: Material,
    /// `p.dot(-norm)`, used for fast hit calculations
    d: EFloat64,
}

impl Plane {
    /// Constructs an infinite plane given a point and a normal
    pub fn new(p: DVec3, n: DVec3, material: Material) -> Box<Self> {
        assert!(n.dot(n) != 0.0);
        let normal = n.normalize();
        let nx = EFloat64::from(normal.x); let ny = EFloat64::from(normal.y);
        let nz = EFloat64::from(normal.z); let px = EFloat64::from(p.x);
        let py = EFloat64::from(p.y); let pz = EFloat64::from(p.z);

        // p.dot(-normal)
        let d = px * (-nx) + py * (-ny) + pz * (-nz);

        Box::new(Self {
            normal,
            material,
            d,
        })
    }
}

impl Object for Plane {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        let xo = r.origin;
        let wi = r.dir;

        // co planar to plane
        if self.normal.dot(wi).abs() < EPSILON {
            return None;
        }

        let dx = EFloat64::from(wi.x); let dy = EFloat64::from(wi.y);
        let dz = EFloat64::from(wi.z); let ox = EFloat64::from(xo.x);
        let oy = EFloat64::from(xo.y); let oz = EFloat64::from(xo.z);

        let nx = EFloat64::from(self.normal.x);
        let ny = EFloat64::from(self.normal.y);
        let nz = EFloat64::from(self.normal.z);

        let t = -(self.d + nx * ox + ny * oy + nz * oz)
            / (nx * dx + ny * dy + nz * dz);

        if t.high >= t_max || t.low <= t_min {
            None
        } else {
            let xi = r.at(t.value);
            let err = DVec3::new(
                (ox + dx * t).abs_error(),
                (oy + dy * t).abs_error(),
                (oz + dz * t).abs_error(),
            );

            let (u, v) = self.normal.any_orthonormal_pair();
            let uv = DVec2::new(u.dot(xi), v.dot(xi)).fract();

            Hit::new(
                t.value,
                &self.material,
                r.backface(self.normal),
                xi,
                err,
                self.normal,
                self.normal,
                uv
            )
        }
    }
}
