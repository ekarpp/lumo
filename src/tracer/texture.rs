use crate::DVec3;
use crate::perlin::Perlin;
use Texture::*;

pub enum Texture {
    Solid(DVec3),
    Checkerboard,
    Marble(Perlin),
}

impl Texture {
    pub fn color_at(&self, p: DVec3) -> DVec3 {
        match self {
            Solid(c) => c.clone(),
            Checkerboard => Self::checkers_at(p),
            Marble(pn) => pn.color_at(p),
        }
    }

    fn checkers_at(p: DVec3) -> DVec3 {
        // if looks bad, try to tune s
        let s = 13.0;
        if (s*p.x).sin() * (s*p.y).sin() * (s*p.z).sin() > 0.0 {
            DVec3::ZERO
        } else {
            DVec3::ONE
        }
    }
}
