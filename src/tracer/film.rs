use crate::tracer::{filter::Filter, Color};
use crate::{Float, Vec2};
use glam::IVec2;
use png::{BitDepth, ColorType, Encoder, EncodingError};
use std::{fs::File, io::BufWriter, path::Path, ops::AddAssign};

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

impl AddAssign<&Pixel> for Pixel {
    fn add_assign(&mut self, rhs: &Self) {
        self.color += rhs.color;
        self.filter_weight_sum += rhs.filter_weight_sum;
    }
}

pub struct FilmTile {
    pub px_min: IVec2,
    pub px_max: IVec2,
    pub width: i32,
    pixels: Vec<Pixel>,
    filter: Filter,
}

impl FilmTile {
    pub fn new(px_min: IVec2, px_max: IVec2, filter: Filter) -> Self {
        let pxs = px_max - px_min;
        let width = pxs.x;
        Self {
            px_min,
            px_max,
            filter,
            width,
            pixels: vec![Pixel::default(); (pxs.x * pxs.y) as usize],
        }
    }

    /// Adds a sample to the film
    pub fn add_sample(&mut self, sample: FilmSample) {
        let raster = sample.raster_xy.floor().as_ivec2();
        if !(self.px_min.x..self.px_max.x).contains(&raster.x) {
            return;
        }
        if !(self.px_min.y..self.px_max.y).contains(&raster.y) {
            return;
        }

        let mid = Vec2::new(raster.x as Float, raster.y as Float) + 0.5;
        let offset = mid - sample.raster_xy;
        let weight = self.filter.eval(2.0 * offset);

        let raster = raster - self.px_min;
        let idx = (raster.x + self.width * raster.y) as usize;
        self.pixels[idx].filter_weight_sum += weight;
        self.pixels[idx].color += sample.color * weight;
    }
}

/// Film that contains the image being rendered
pub struct Film {
    pixels: Vec<Pixel>,
    /// Image resolution
    pub resolution: IVec2,
}

impl Film {
    /// Creates a new empty film
    pub fn new(width: i32, height: i32) -> Self {
        let n = width * height;
        let resolution = IVec2::new(width, height);
        Self {
            pixels: vec![Pixel::default(); n as usize],
            resolution,
        }
    }

    pub fn add_tile(&mut self, tile: FilmTile) {
        let px_offset = tile.px_max - tile.px_min;
        for y in 0..px_offset.y {
            for x in 0..px_offset.x {
                let px = IVec2::new(x, y);
                let idx_tile = (px.x + px.y * tile.width) as usize;
                let raster = px + tile.px_min;
                let idx_film = (raster.x + raster.y * self.resolution.x) as usize;
                self.pixels[idx_film] += &tile.pixels[idx_tile];
            }
        }
    }

    fn rgb_image(&self) -> Vec<u8> {
        let mut img = Vec::new();

        for y in 0..self.resolution.y {
            for x in 0..self.resolution.x {
                let idx = (x + y * self.resolution.x) as usize;
                let px = self.pixels[idx].color
                    / self.pixels[idx].filter_weight_sum;
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
            self.resolution.x as u32,
            self.resolution.y as u32,
        );
        encoder.set_color(ColorType::Rgb);
        encoder.set_depth(BitDepth::Eight);

        encoder.write_header()?.write_image_data(&self.rgb_image())?;
        Ok(())
    }
}
