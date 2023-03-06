use crate::DVec3;
use Texture::*;

pub enum Texture {
    Solid(DVec3),
    Checkerboard,
}

impl Texture {
    pub fn color_at(&self, p: DVec3) -> DVec3 {
        match self {
            Solid(c) => c.clone(),
            Checkerboard => {
                // very dependant on s
                let s = 17.0;
                if (s*p.x).sin() * (s*p.y).sin() * (s*p.z).sin() > 0.0 {
                    DVec3::ZERO
                } else {
                    DVec3::ONE
                }
            }
        }
    }
}
