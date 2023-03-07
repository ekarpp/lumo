use crate::DVec3;
use crate::perlin::Perlin;
use Texture::*;

const CHECKER_SCALE: f64 = 13.0;

pub enum Texture {
    Solid(DVec3),
    /* box avoids having to define lifetime all the way to objects.
     * should texture be a struct instead? */
    Checkerboard(Box<Texture>, Box<Texture>),
    Marble(Perlin),
}

impl Texture {
    pub fn color_at(&self, p: DVec3) -> DVec3 {
        match self {
            Solid(c) => c.clone(),
            Marble(pn) => pn.color_at(p),
            Checkerboard(t1, t2) => {
                if self.checkers_phase(p) > 0.0 {
                    t1.color_at(p)
                } else {
                    t2.color_at(p)
                }
            }
        }
    }

    fn checkers_phase(&self, p: DVec3) -> f64 {
        let t = CHECKER_SCALE*p;
        (t.x).sin() * (t.y).sin() * (t.z).sin()
    }
}
