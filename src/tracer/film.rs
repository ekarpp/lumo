use crate::tracer::{filter::Filter, Color};
use crate::{Float, Vec2};
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

/// Film that contains the image being rendered
pub struct Film {
    samples: Vec<Color>,
    num_samples: Vec<u32>,
    /// Width of the image
    pub width: i32,
    /// Height of the image
    pub height: i32,
}

impl Film {
    /// Creates a new empty film
    pub fn new(width: i32, height: i32) -> Self {
        let n = width * height;
        Self {
            samples: vec![Color::BLACK; n as usize],
            num_samples: vec![0; n as usize],
            width,
            height,
        }
    }

    /// Adds a sample to the film
    pub fn add_sample(&mut self, sample: FilmSample) {
        let raster = sample.raster_xy.floor().as_ivec2();
        if !(0..self.width).contains(&raster.x)
            || !(0..self.height).contains(&raster.y) {
            return;
        }
        let idx = (raster.x + self.width * raster.y) as usize;
        self.samples[idx] += sample.color;
        if !sample.splat {
            self.num_samples[idx] += 1;
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
                let px = self.samples[idx] / self.num_samples[idx] as Float;
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
