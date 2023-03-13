use crate::DVec3;
use crate::perlin::Perlin;
use crate::consts::CHECKER_SCALE;

/// Defines a texture to choose a colour of material at each point.
pub enum Texture {
    /// Solid colour.
    Solid(DVec3),
    /* box avoids having to define lifetime all the way to objects.
     * should texture be a struct instead? */
    /// Checkerboard of textures. Float defines scale,
    /// bigger scale = smaller boxes.
    Checkerboard(Box<Texture>, Box<Texture>, f64),
    /// Marble like texture generated from Perlin noise.
    Marble(Perlin),
}

impl Texture {
    /// Colour at point `p`, should transform to texture coords of object??
    pub fn albedo_at(&self, p: DVec3) -> DVec3 {
        match self {
            Texture::Solid(c) => c.clone(),
            Texture::Marble(pn) => pn.albedo_at(p),
            Texture::Checkerboard(t1, t2, s) => {
                if self.checkers_phase(p * (*s)) > 0.0 {
                    t1.albedo_at(p)
                } else {
                    t2.albedo_at(p)
                }
            }
        }
    }

    fn checkers_phase(&self, p: DVec3) -> f64 {
        let ps = CHECKER_SCALE*p;
        (ps.x).sin() * (ps.y).sin() * (ps.z).sin()
    }
}
