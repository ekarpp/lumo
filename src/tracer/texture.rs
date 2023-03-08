use crate::DVec3;
use crate::perlin::Perlin;
use crate::consts::CHECKER_SCALE;

pub enum Texture {
    Solid(DVec3),
    /* box avoids having to define lifetime all the way to objects.
     * should texture be a struct instead? */
    /* f64 for scale to make recursion work */
    Checkerboard(Box<Texture>, Box<Texture>, f64),
    Marble(Perlin),
}

impl Texture {
    /* should use texture coordinates instead of hit point here */
    pub fn color_at(&self, p: DVec3) -> DVec3 {
        match self {
            Texture::Solid(c) => c.clone(),
            Texture::Marble(pn) => pn.color_at(p),
            Texture::Checkerboard(t1, t2, s) => {
                if self.checkers_phase(p * (*s)) > 0.0 {
                    t1.color_at(p)
                } else {
                    t2.color_at(p)
                }
            }
        }
    }

    fn checkers_phase(&self, p: DVec3) -> f64 {
        let ps = CHECKER_SCALE*p;
        (ps.x).sin() * (ps.y).sin() * (ps.z).sin()
    }
}
