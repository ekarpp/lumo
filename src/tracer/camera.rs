use crate::DVec3;
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
        Ray::new(
            self.origin,
            self.blc + x*self.horiz + y*self.vert - self.origin,
            0,
        )
    }

    pub fn new(
        aspect_ratio: f64,
        vfov: f64,
        from: DVec3,
        towards: DVec3,
        up: DVec3,
        focal_length: f64
    ) -> Self {
        let h = (vfov.to_radians() / 2.0).tan();
        /* viewport height */
        let vph = 2.0 * h;
        /* viewport width */
        let vpw = vph * aspect_ratio;

        let z = (from - towards).normalize();
        let x = up.cross(z).normalize();
        let y = z.cross(x);

        let horiz = x * vpw * focal_length;
        let vert = y * vph * focal_length;

        Self {
            origin: from,
            horiz: horiz,
            vert: vert,
            blc: from - (horiz + vert) / 2.0 - z*focal_length,
        }
    }
}
