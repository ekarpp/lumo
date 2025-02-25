use crate::tracer::{filter::PixelFilter, Color};
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

// TODO: some memory saving available by making separate tile pixel (w.o. splat)
#[derive(Clone)]
struct Pixel {
    pub color: Color,
    pub splat: Color,
    pub color_weight: Float,
}

impl Default for Pixel {
    fn default() -> Self {
        Pixel {
            color: Color::BLACK,
            splat: Color::BLACK,
            color_weight: 0.0,
        }
    }
}

impl AddAssign<&Pixel> for Pixel {
    fn add_assign(&mut self, rhs: &Self) {
        self.color += rhs.color;
        self.splat += rhs.splat;
        self.color_weight += rhs.color_weight;
    }
}

/// FilmTile given to a thread to avoid synchronization issues
pub struct FilmTile<'a> {
    /// Minimum coordinates of tile in raster space
    pub px_min: IVec2,
    /// Maximum coordinates of tile in raster space
    pub px_max: IVec2,
    /// Width of the tile
    pub width: i32,
    pixels: Vec<Pixel>,
    splats: Vec<FilmSample>,
    resolution: IVec2,
    filter: &'a PixelFilter,
}

impl<'a> FilmTile<'a> {
    /// Creates a new tile `[px_min.x, px_max.x) x [px_min.y, px_max.y)` with `filter`
    pub fn new(px_min: IVec2, px_max: IVec2, res: IVec2, filter: &'a PixelFilter) -> Self {
        let radius = filter.r_disc();
        let px_min = (px_min - radius).max(IVec2::ZERO);
        let px_max = (px_max + radius).min(res);
        let pxs = px_max - px_min;
        let width = pxs.x;
        let height = pxs.y;

        Self {
            px_min,
            px_max,
            filter,
            width,
            resolution: res,
            pixels: vec![Pixel::default(); (width * height) as usize],
            splats: vec![],
        }
    }

    /// Adds a sample to the tile
    pub fn add_sample(&mut self, sample: FilmSample) {
        let px_xy = sample.raster_xy.floor().as_ivec2();
        // middle of pixel where sample is from
        let px_mid = 0.5 + sample.raster_xy.floor();

        let (mi,mx) = if sample.splat {
            (-px_xy, self.resolution - px_xy)
        } else {
            (self.px_min - px_xy, self.px_max - px_xy)
        };

        for (y, x) in self.filter.xys(mi, mx) {
            let px_x = px_xy.x + x;
            let px_y = px_xy.y + y;

            // middle of the pixel where filter will be evaluated
            let mid_xy = Vec2::new(
                px_mid.x + Float::from(x),
                px_mid.y + Float::from(y)
            );

            let v = sample.raster_xy - mid_xy;
            let w = self.filter.eval(v);

            if w != 0.0 {
                if sample.splat {
                    self.splats.push(FilmSample::new(
                        sample.color * w,
                        mid_xy,
                        true,
                    ))
                } else {
                    let px_x = px_x - self.px_min.x;
                    let px_y = px_y - self.px_min.y;
                    let idx = (px_x + self.width * px_y) as usize;
                    self.pixels[idx].color += sample.color * w;
                    self.pixels[idx].color_weight += w;
                }
            }
        }
    }
}

/// Film that contains the image being rendered
pub struct Film<'a> {
    pixels: Vec<Pixel>,
    /// Image resolution
    pub resolution: IVec2,
    splat_scale: Float,
    filter: &'a PixelFilter,
}

impl<'a> Film<'a> {
    /// Creates a new empty film
    pub fn new(width: i32, height: i32, samples: u32, filter: &'a PixelFilter) -> Self {
        let n = width * height;
        let resolution = IVec2::new(width, height);
        // TODO: where does 1.2^r come from?
        let splat_scale = 1.0 / (samples as Float * Float::powi(1.2, filter.r_disc()));
        Self {
            pixels: vec![Pixel::default(); n as usize],
            splat_scale,
            filter,
            resolution,
        }
    }

    /// Add samples from `tile` to self.
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

        for splat in tile.splats {
            let px_xy = splat.raster_xy.floor().as_ivec2();
            let idx = (px_xy.x + self.resolution.x * px_xy.y) as usize;
            self.pixels[idx].splat += splat.color;
        }
    }

    fn rgb_image(&self) -> Vec<u8> {
        let mut img = Vec::new();

        for y in 0..self.resolution.y {
            for x in 0..self.resolution.x {
                let idx = (x + y * self.resolution.x) as usize;
                let pix = &self.pixels[idx];

                let dir = if pix.color_weight == 0.0 {
                    Color::BLACK
                } else {
                    pix.color / pix.color_weight
                };
                let splt = self.splat_scale * pix.splat / self.filter.integral();
                let col = dir + splt;

                let (r, g, b) = col.gamma_enc();
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
