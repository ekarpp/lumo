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
    // 4x supersampling for now
    pub fn ss_rays_at(&self, pw: f64, ph: f64, x: f64, y: f64) -> Vec<Ray> {
        let pws = pw / 4.0;
        let phs = ph / 4.0;
        let pm = self.blc + x*self.horiz + y*self.vert - self.origin;
        (0..4).map(|i| {
            let xd = if i & 1 == 1 { -1.0 } else { 1.0 };
            let yd = if i & 2 == 2 { -1.0 } else { 1.0 };
            Ray::new(
                self.origin,
                pm + xd*pws + yd*phs
            )
        }).collect()
    }

    pub fn ray_at(&self, x: f64, y: f64) -> Ray {
        Ray::new(
            self.origin,
            self.blc + x*self.horiz + y*self.vert - self.origin
        )
    }

    pub fn new(ar: f64, vfov: f64, from: DVec3, towards: DVec3, up: DVec3)
               -> Camera {
        let h = (vfov.to_radians() / 2.0).tan();
        /* viewport height */
        let vph = 2.0 * h;
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
