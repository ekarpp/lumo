use glam::f32::Vec3;
use crate::tracer::ray::Ray;

pub struct Camera {
    vp_height: f32,
    vp_width: f32,
    origin: Vec3,
    pub bot_left_corner: Vec3
}

impl Camera {
    pub fn ray_at(&self, u: f32, v: f32) -> Ray {
        Ray {
            origin: self.origin,
            dir: self.bot_left_corner - self.origin
                + Vec3::new(
                    u*self.vp_width,
                    v*self.vp_height,
                    0.0
                )
        }
    }
}

pub fn def() -> Camera {
    let h = 2.0;
    let w = h * 16.0 / 9.0;
    let f = 1.0;

    Camera {
        vp_height: h,
        vp_width: w,
        origin: Vec3::ZERO,
        bot_left_corner: -0.5*Vec3::new(w, h, 2.0*f)
    }
}
