use glam::f64::DVec3;
use crate::tracer::ray::Ray;

pub struct Camera {
    origin: DVec3,
    horiz: DVec3,
    vert: DVec3,
    /* bottom left corner */
    blc: DVec3
}

impl Camera {
    pub fn ray_at(&self, u: f64, v: f64) -> Ray {
        Ray {
            origin: self.origin,
            dir: self.blc + u*self.horiz + v*self.vert - self.origin
        }
    }

    pub fn new(ar: f64, from: DVec3, towards: DVec3, up: DVec3) -> Camera {
        /* viewport height */
        let vph = 2.0;
        /* viewport width */
        let vpw = vph * ar;

        let w = (from - towards).normalize();
        let u = up.cross(w).normalize();
        let v = w.cross(u);

        let horiz = u * vpw;
        let vert = v * vph;

        Camera {
            origin: from,
            horiz: horiz,
            vert: vert,
            blc: from - (horiz + vert) / 2.0 - w
        }
    }
}
