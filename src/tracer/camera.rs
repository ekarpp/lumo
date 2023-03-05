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
    pub fn ray_at(&self, x: f64, y: f64) -> Ray {
        Ray {
            origin: self.origin,
            dir: self.blc + x*self.horiz + y*self.vert - self.origin
        }
    }

    pub fn new(ar: f64, from: DVec3, towards: DVec3, up: DVec3) -> Camera {
        /* viewport height */
        let vph = 2.0;
        /* viewport width */
        let vpw = vph * ar;

        let z = (from - towards).normalize();
        let x = up.cross(z).normalize();
        let y = z.cross(x);

        let horiz = x * vpw;
        let vert = y * vph;

        Camera {
            origin: from,
            horiz: horiz,
            vert: vert,
            blc: from - (horiz + vert) / 2.0 - z
        }
    }
}
