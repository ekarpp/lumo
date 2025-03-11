use crate::tracer::{
    filter::PixelFilter, ColorSpace, color::DenseSpectrum,
    RGB, Color, ColorWavelength
};
use crate::{Float, Mat3, Vec2};
use crate::math::vec2::UVec2;
use png::{BitDepth, ColorType, Encoder, EncodingError};
use std::{fmt, fs::File, io::BufWriter, sync::Arc, path::Path, ops::AddAssign};

pub use tile::FilmTile;

mod tile;

/// Sample for the film
pub struct FilmSample {
    /// Raster coordinate `x` of the sample
    pub raster_xy: Vec2,
    /// Color of the sample
    pub color: Color,
    /// Sampled wavelengths of the color
    pub lambda: ColorWavelength,
    /// "Splat" sample i.e. from sampling camera
    pub splat: bool,
}

impl Default for FilmSample {
    fn default() -> Self {
        Self {
            raster_xy: -Vec2::ONE,
            color: Color::BLACK,
            lambda: ColorWavelength::default(),
            splat: true,
        }
    }
}

impl FilmSample {
    /// Creates a sample of `color` at raster `(x,y)`
    pub fn new(color: Color, lambda: ColorWavelength, raster_xy: Vec2, splat: bool) -> Self {
        Self {
            raster_xy, color, splat, lambda,
        }
    }
}

#[derive(Clone)]
struct Pixel {
    pub color: RGB,
    pub splat: RGB,
    pub color_weight: Float,
}

impl Default for Pixel {
    fn default() -> Self {
        Pixel {
            color: RGB::BLACK,
            splat: RGB::BLACK,
            color_weight: 0.0,
        }
    }
}

/// Film that contains the image being rendered
pub struct Film {
    pixels: Vec<Pixel>,
    /// Image resolution
    pub resolution: UVec2,
    splat_scale: Float,
    filter: Arc<PixelFilter>,
    cs: &'static ColorSpace,
    white_balance: Mat3,
}

impl fmt::Display for Film {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Filter: {}, Color space: {}", self.filter, self.cs)
    }
}

impl Film {
    /// Creates a new empty film
    pub fn new(
        resolution: UVec2,
        samples: u64,
        cs: &'static ColorSpace,
        filter: PixelFilter,
        illuminant: &'static DenseSpectrum,
    ) -> Self {
        let n = resolution.x * resolution.y;

        let filter = Arc::new(filter);
        let white_balance = cs.wb_matrix(illuminant);

        Self {
            pixels: vec![Pixel::default(); n as usize],
            splat_scale: 1.0 / samples as Float,
            filter,
            cs,
            resolution,
            white_balance,
        }
    }

    /// Create a tile of the film for block rendering
    pub fn create_tile(&self, px_min: UVec2, px_max: UVec2) -> FilmTile {
        FilmTile::new(
            px_min, px_max, self.resolution, self.cs,
            // TODO: don't clone, give reference to self?
            self.white_balance.clone(), Arc::clone(&self.filter),
        )
    }

    /// Add samples from `tile` to self.
    pub fn add_tile(&mut self, tile: FilmTile) {
        let px_offset = tile.px_max - tile.px_min;
        for y in 0..px_offset.y {
            for x in 0..px_offset.x {
                let px = UVec2::new(x, y);
                let idx_tile = (px.x + px.y * tile.width) as usize;
                let raster = px + tile.px_min;
                let idx_film = (raster.x + raster.y * self.resolution.x) as usize;
                self.pixels[idx_film] += &tile.pixels[idx_tile];
            }
        }

        for splat in tile.splats {
            let idx = (splat.x + self.resolution.x * splat.y) as usize;
            self.pixels[idx].splat += &splat.color;
        }
    }

    fn rgb_image(&self) -> Vec<u8> {
        let mut img = Vec::new();

        for y in 0..self.resolution.y {
            for x in 0..self.resolution.x {
                let idx = (x + y * self.resolution.x) as usize;
                let pix = &self.pixels[idx];

                let dir = if pix.color_weight == 0.0 {
                    RGB::BLACK
                } else {
                    pix.color.clone() / pix.color_weight
                };
                let splt = self.splat_scale * pix.splat.clone() / self.filter.integral();
                let col = dir + splt;

                let (r, g, b) = self.cs.encode(col);
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
