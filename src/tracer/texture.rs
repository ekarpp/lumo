use crate::Image;
use crate::perlin::Perlin;
use crate::tracer::hit::Hit;
use glam::DVec3;

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
    /// Image texture loaded from a .png
    Image(Image),
}

impl Texture {

    /// Colour at hit `h`
    pub fn albedo_at(&self, h: &Hit) -> DVec3 {
        match self {
            Texture::Solid(c) => *c,
            Texture::Marble(pn) => pn.albedo_at(h.p),
            Texture::Checkerboard(t1, t2, s) => {
                let uv = (*s) * h.uv;
                if (uv.x.floor() + uv.y.floor()) as i32 % 2 == 0 {
                    t1.albedo_at(h)
                } else {
                    t2.albedo_at(h)
                }
            }
            Texture::Image(img) => {
                let uv = h.uv;
                let x = uv.x * img.width as f64;
                let x = x.floor() as usize;
                let y = uv.y * img.height as f64;
                let y = img.height - y.floor() as u32 - 1;
                img.buffer[x + (y*img.width) as usize]
            }
        }
    }
}
