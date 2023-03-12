use crate::DVec3;

pub struct Onb {
    pub u: DVec3,
    pub v: DVec3,
    pub w: DVec3,
}

/* orthonormal basis */
impl Onb {
    pub fn new(dir: DVec3) -> Self {
        let w = dir.normalize();

        let a = if w.x.abs() > 0.9 {
            DVec3::new(0.0, 1.0, 0.0)
        } else {
            DVec3::new(1.0, 0.0, 0.0)
        };

        let v = w.cross(a).normalize();
        let u = w.cross(v);

        Self {
            u: u,
            v: v,
            w: w,
        }
    }

    pub fn to_uvw_basis(&self, v: DVec3) -> DVec3 {
        v.x * self.u + v.y * self.v + v.z * self.w
    }
}
