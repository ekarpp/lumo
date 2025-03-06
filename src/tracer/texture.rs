use crate::{ Float, Image, Point, perlin::Perlin };
use crate::math::complex::Complex;
use crate::tracer::{Color, ColorWavelength, Spectrum, hit::Hit};

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

/// Maximum number of Mandelbrot iterations
const MANDELBROT_DEPTH: usize = 256;
/// Escape radius for Mandelbrot
const MANDELBROT_R: Float = 64.0;
/// Escape radius squared
const MANDELBROT_R2: Float = MANDELBROT_R * MANDELBROT_R;

/// Defines a texture to choose a colour of material at each point.
pub enum Texture {
    /// Solid colour.
    Solid(Spectrum),
    /* box avoids having to define lifetime all the way to objects.
     * should texture be a struct instead? */
    /// Checkerboard of textures. Float defines scale,
    /// bigger scale = smaller boxes.
    Checkerboard(Box<Texture>, Box<Texture>, Float),
    /// Marble like texture generated from Perlin noise.
    /// Underlying color as argument.
    Marble(Perlin, Spectrum),
    /// Image texture loaded from a .png
    Image(Image),
    /// Cheap render of the Mandelbrot set
    Mandelbrot,
}

impl Default for Texture {
    fn default() -> Self { Self::Solid(Spectrum::WHITE) }
}

impl From<Spectrum> for Texture {
    fn from(spec: Spectrum) -> Self {
        Self::Solid(spec)
    }
}

impl Texture {
    /// Colour at hit `h`
    pub fn albedo_at(&self, lambda: &ColorWavelength, h: &Hit) -> Color {
        match self {
            Texture::Solid(spec) => spec.sample(lambda),
            Texture::Marble(pn, spec) => {
                let xo = h.p;
                let turb = Self::turbulence(pn, 0.0, MARBLE_SCALE * xo.abs(), 0);
                let scaled = 1.0 -
                    (0.5 + 0.5 * (MARBLE_FREQ * xo.x + MARBLE_AMP * turb).sin())
                    .powi(6);

                spec.sample(lambda) * scaled
            }
            Texture::Checkerboard(t1, t2, s) => {
                let uv = h.uv * (*s);
                if (uv.x.floor() + uv.y.floor()) as u64 % 2 == 0 {
                    t1.albedo_at(lambda, h)
                } else {
                    t2.albedo_at(lambda, h)
                }
            }
            Texture::Image(img) => {
                let uv = h.uv;
                let x = uv.x * img.width as Float;
                let x = x.floor() as usize;
                let y = uv.y * img.height as Float;
                let y = img.height - y.floor() as u32 - 1;
                img.buffer[x + (y*img.width) as usize].sample(lambda)
            }
            Texture::Mandelbrot => {
                let mut depth = 0;
                // [-1.5,0.5] x [-1.0,1.0]
                let c = 2.0 * Complex::new(h.uv.x - 0.75, h.uv.y - 0.5);
                let mut z = Complex::new(0.0, 0.0);

                while depth < MANDELBROT_DEPTH && z.norm_sqr() < MANDELBROT_R2 {
                    z = z * z + c;
                    depth += 1;
                }

                if depth == MANDELBROT_DEPTH {
                    Color::WHITE
                } else {
                    Color::BLACK
                }
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
