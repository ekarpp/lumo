use glam::DVec3;
use png::{BitDepth, ColorType, Encoder, EncodingError};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// Sample for the film
pub struct FilmSample {
    pub x: i32,
    pub y: i32,
    pub color: DVec3,
}

impl FilmSample {
    pub fn new(x: i32, y: i32, color: DVec3) -> Self {
        Self {
            x, y, color
        }
    }
}

/// Film that contains the image being rendered
pub struct Film {
    samples: Vec<DVec3>,
    num_samples: Vec<u64>,
    pub width: i32,
    pub height: i32,
}

impl Film {
    /// Creates a new empty film
    pub fn new(width: i32, height: i32) -> Self {
        let n = width * height;
        Self {
            samples: vec![DVec3::ZERO; n as usize],
            num_samples: vec![0; n as usize],
            width,
            height,
        }
    }

    /// Adds a sample to the film
    pub fn add_sample(&mut self, sample: FilmSample) {
        let idx = (sample.x + self.width * sample.y) as usize;
        self.samples[idx] += sample.color;
        self.num_samples[idx] += 1;
    }

    fn rgb_image(&self) -> Vec<u8> {
        let mut img = Vec::new();

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (x + y * self.width) as usize;
                let px = self.samples[idx] / self.num_samples[idx] as f64;

                img.push(self.lin_to_srgb(px.x));
                img.push(self.lin_to_srgb(px.y));
                img.push(self.lin_to_srgb(px.z));
            }
        }

        img
    }

    /// 2.2 Gamma encodes the linear RGB channel
    fn lin_to_srgb(&self, c: f64) -> u8 {
        (c.powf(1.0 / 2.2) * 255.0) as u8
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
