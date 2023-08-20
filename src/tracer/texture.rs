use crate::{Float, Image, Point};
use crate::perlin::Perlin;
use crate::tracer::{Color, hit::Hit};

/// Scale of points in perlin. bigger = more noticeable effect
const MARBLE_SCALE: Float = 4.0;
/// Frequency of noise in perlin noise. bigger = more frequent
const MARBLE_FREQ: Float = 60.0;
/// Amplitude of the noise pattern in perlin noise
const MARBLE_AMP: Float = 20.0;
/// Recursion depth in perlin turbulence
const MARBLE_OCTAVES: i32 = 6;
/// Scale of each term in turbulence. should be less than 1.0
const MARBLE_GAIN: Float = 0.5;

/// Defines a texture to choose a colour of material at each point.
pub enum Texture {
    /// Solid colour.
    Solid(Color),
    /* box avoids having to define lifetime all the way to objects.
     * should texture be a struct instead? */
    /// Checkerboard of textures. Float defines scale,
    /// bigger scale = smaller boxes.
    Checkerboard(Box<Texture>, Box<Texture>, Float),
    /// Marble like texture generated from Perlin noise.
    /// Underlying color as argument.
    Marble(Perlin, Color),
    /// Image texture loaded from a .png
    Image(Image),
}

impl Texture {
    /// Colour at hit `h`
    pub fn albedo_at(&self, h: &Hit) -> Color {
        match self {
            Texture::Solid(c) => *c,
            Texture::Marble(pn, c) => {
                let xo = h.p;
                let turb = Self::turbulence(pn, 0.0, MARBLE_SCALE * xo.abs(), 0);
                let scaled = 1.0 -
                    (0.5 + 0.5 * (MARBLE_FREQ * xo.x + MARBLE_AMP * turb).sin())
                    .powi(6);

                *c * scaled
            }
            Texture::Checkerboard(t1, t2, s) => {
                let uv = h.uv * (*s);
                if (uv.x.floor() + uv.y.floor()) as i32 % 2 == 0 {
                    t1.albedo_at(h)
                } else {
                    t2.albedo_at(h)
                }
            }
            Texture::Image(img) => {
                let uv = h.uv;
                let x = uv.x * img.width as Float;
                let x = x.floor() as usize;
                let y = uv.y * img.height as Float;
                let y = img.height - y.floor() as u32 - 1;
                img.buffer[x + (y*img.width) as usize]
            }
        }
    }

    /// Computes the turbulence for the noise. I.e. absolute values of the
    /// noise at different octaves are summed together.
    fn turbulence(pn: &Perlin, acc: Float, p: Point, depth: i32) -> Float {
        if depth >= MARBLE_OCTAVES {
            return acc;
        }
        let w = MARBLE_GAIN.powi(depth);

        Self::turbulence(pn, acc + w * pn.noise_at(p).abs(), 2.0 * p, depth + 1)
    }
}
