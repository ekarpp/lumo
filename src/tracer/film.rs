use crate::tracer::{filter::Filter, Color};
use crate::{Float, Vec2};
use glam::IVec2;
use png::{BitDepth, ColorType, Encoder, EncodingError};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// Sample for the film
pub struct FilmSample {
    /// Raster coordinate `x` of the sample
    pub raster_xy: Vec2,
    /// Color of the sample
    pub color: Color,
    /// "Splat" sample i.e. from sampling camera
    pub splat: bool,
}

impl Default for FilmSample {
    fn default() -> Self {
        Self {
            raster_xy: Vec2::NEG_ONE,
            color: Color::BLACK,
            splat: true,
        }
    }
}

impl FilmSample {
    /// Creates a sample of `color` at raster `(x,y)`
    pub fn new(color: Color, raster_xy: Vec2, splat: bool) -> Self {
        Self {
            raster_xy, color, splat,
        }
    }
}

#[derive(Clone)]
struct Pixel {
    pub color: Color,
    pub filter_weight_sum: Float,
}

impl Default for Pixel {
    fn default() -> Self {
        Pixel { color: Color::BLACK, filter_weight_sum: 0.0 }
    }
}

/// Film that contains the image being rendered
pub struct Film {
    samples: Vec<Pixel>,
    filter: Box<dyn Filter>,
    /// Width of the image
    pub width: i32,
    /// Height of the image
    pub height: i32,
}

impl Film {
    /// Creates a new empty film
    pub fn new(width: i32, height: i32, filter: Box<dyn Filter>) -> Self {
        let n = width * height;
        Self {
            samples: vec![Pixel::default(); n as usize],
            filter,
            width,
            height,
        }
    }

    /// Adds a sample to the film
    pub fn add_sample(&mut self, sample: FilmSample) {
        let raster = sample.raster_xy.floor().as_ivec2();
        if !(0..self.width).contains(&raster.x) || !(0..self.height).contains(&raster.y) {
            return;
        }

        let px = sample.raster_xy - 0.5;
        let p0 = (px - self.filter.radius()).ceil()
            .as_ivec2().max(IVec2::ZERO);
        let p1 = ((px + self.filter.radius()).floor()
            .as_ivec2()).min(IVec2::new(self.width, self.height));

        for y in p0.y..p1.y {
            for x in p0.x..p1.x {
                let idx = (x + self.width * y) as usize;
                if sample.splat {
                    self.samples[idx].color += sample.color;
                } else {
                    let pn = Vec2::new(x as Float, y as Float);
                    let weight = self.filter.eval(pn - px);
                    self.samples[idx].filter_weight_sum += weight;
                    self.samples[idx].color += sample.color * weight;
                }
            }
        }
    }

    /// Empties the vector and adds each sample to film
    pub fn add_samples(&mut self, mut samples: Vec<FilmSample>) {
        while let Some(sample) = samples.pop() {
            self.add_sample(sample);
        }
    }

    fn rgb_image(&self) -> Vec<u8> {
        let mut img = Vec::new();

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (x + y * self.width) as usize;
                let px = self.samples[idx].color
                    / self.samples[idx].filter_weight_sum;
                let (r, g, b) = px.gamma_enc();
                img.push(r);
                img.push(g);
                img.push(b);
            }
        }

        img
    }

    /// Saves the film to a .png file
    pub fn save(&self, fname: &str) -> Result<(), EncodingError> {
        println!("Saving to \"{}\"", fname);
        let path = Path::new(fname);

        let mut binding = BufWriter::new(File::create(path)?);
        let mut encoder = Encoder::new(
            &mut binding,
            self.width as u32,
            self.height as u32,
        );
        encoder.set_color(ColorType::Rgb);
        encoder.set_depth(BitDepth::Eight);

        encoder.write_header()?.write_image_data(&self.rgb_image())?;
        Ok(())
    }
}
