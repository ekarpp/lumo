use glam::f64::DVec3;
use crate::tracer::ray::Ray;

pub struct Camera {
    vp_height: f64,
    vp_width: f64,
    origin: DVec3,
    pub bot_left_corner: DVec3
}

impl Camera {
    pub fn ray_at(&self, u: f64, v: f64) -> Ray {
        Ray {
            origin: self.origin,
            dir: self.bot_left_corner - self.origin
                + DVec3::new(
                    u*self.vp_width,
                    v*self.vp_height,
                    0.0
                )
        }
    }
}

pub fn default() -> Camera {
    /* viewport height */
    let h = 2.0;
    /* viewport width */
    let w = h * 16.0 / 9.0;
    /* focal length */
    let f = 1.0;
    let origin = DVec3::ZERO;

    Camera {
        vp_height: h,
        vp_width: w,
        origin: origin,
        bot_left_corner: origin - 0.5*DVec3::new(w, h, 2.0*f)
    }
}
