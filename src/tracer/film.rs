use crate::tracer::{filter::PixelFilter, ColorSpace, RGB, Color, ColorWavelength};
use crate::{Float, Vec2};
use crate::math::vec2::UVec2;
use png::{BitDepth, ColorType, Encoder, EncodingError};
use std::{fmt, fs::File, io::BufWriter, sync::Arc, path::Path, ops::AddAssign};

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

impl AddAssign<&TilePixel> for Pixel {
    fn add_assign(&mut self, rhs: &TilePixel) {
        self.color += &rhs.color;
        self.color_weight += rhs.color_weight;
    }
}


#[derive(Clone)]
struct TilePixel {
    pub color: RGB,
    pub color_weight: Float,
}

impl Default for TilePixel {
    fn default() -> Self {
        TilePixel {
            color: RGB::BLACK,
            color_weight: 0.0,
        }
    }
}

struct TileSplat {
    /// Filter weighed sRGB color of the splat
    pub color: RGB,
    /// `x` coordinate in raster space
    pub x: u64,
    /// `y` coordinate in raster space
    pub y: u64,
}

impl TileSplat {
    pub fn new(color: RGB, x: u64, y: u64) -> Self {
        Self { color, x, y }
    }
}

/// FilmTile given to a thread to avoid synchronization issues
pub struct FilmTile {
    /// Minimum coordinates of tile in raster space
    pub px_min: UVec2,
    /// Maximum coordinates of tile in raster space
    pub px_max: UVec2,
    /// Width of the tile
    pub width: u64,
    pixels: Vec<TilePixel>,
    splats: Vec<TileSplat>,
    resolution: UVec2,
    cs: Arc<ColorSpace>,
    filter: Arc<PixelFilter>,
}

impl FilmTile {
    /// Creates a new tile `[px_min.x, px_max.x) x [px_min.y, px_max.y)` with `filter`
    pub fn new(
        px_min: UVec2,
        px_max: UVec2,
        resolution: UVec2,
        cs: Arc<ColorSpace>,
        filter: Arc<PixelFilter>,
    ) -> Self {
        let radius = filter.r_disc();
        let filt_min = px_min - radius;
        let filt_max = (px_max + radius).min(resolution);
        let UVec2 { x: width, y: height } = filt_max - filt_min;

        Self {
            px_min,
            px_max,
            filter,
            cs,
            width,
            resolution,
            pixels: vec![TilePixel::default(); (width * height) as usize],
            splats: vec![],
        }
    }

    /// Adds a sample to the tile
    pub fn add_sample(&mut self, sample: &FilmSample) {
        let rgb = self.cs.from_color(&sample.color, &sample.lambda);
        let px = UVec2::new(
            sample.raster_xy.floor().x as u64,
            sample.raster_xy.floor().y as u64,
        );

        let r = self.filter.r_disc();
        let UVec2 { x: mi_x, y: mi_y } = if sample.splat {
            px - r
        } else {
            (px - r).max(self.px_min)
        };
        let UVec2 { x: mx_x, y: mx_y } = if sample.splat {
            (px + r).min(self.resolution - 1)
        } else {
            (px + r).min(self.px_max - 1)
        };

        for flt_y in mi_y..=mx_y {
            for flt_x in mi_x..=mx_x {
                let flt_mid = 0.5 + Vec2::new(
                    flt_x as Float,
                    flt_y as Float,
                );

                let v = sample.raster_xy - flt_mid;
                let w = self.filter.eval(v);

                if w != 0.0 {
                    if sample.splat {
                        self.splats.push(TileSplat::new(
                            rgb.clone() * w,
                            flt_x,
                            flt_y,
                        ))
                    } else {
                        let px_x = flt_x - self.px_min.x;
                        let px_y = flt_y - self.px_min.y;
                        let idx = (px_x + self.width * px_y) as usize;
                        self.pixels[idx].color += &(rgb.clone() * w);
                        self.pixels[idx].color_weight += w;
                    }
                }
            }
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
    cs: Arc<ColorSpace>,
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
        cs: ColorSpace,
        filter: PixelFilter,
    ) -> Self {
        let n = resolution.x * resolution.y;

        let cs = Arc::new(cs);
        let filter = Arc::new(filter);

        Self {
            pixels: vec![Pixel::default(); n as usize],
            splat_scale: 1.0 / samples as Float,
            filter,
            cs,
            resolution,
        }
    }

    /// Set the color space used by the film
    pub fn set_color_space(&mut self, cs: ColorSpace) {
        self.cs = Arc::new(cs);
    }

    /// Set the pixel filter used by the film
    pub fn set_filter(&mut self, filter: PixelFilter) {
        self.filter = Arc::new(filter);
    }

    /// Set the number of samples per pixel for correct splat scaling
    pub fn set_samples(&mut self, samples: u64) {
        self.splat_scale = 1.0 / samples as Float;
    }

    /// Create a tile of the film for block rendering
    pub fn create_tile(&self, px_min: UVec2, px_max: UVec2) -> FilmTile {
        FilmTile::new(
            px_min, px_max, self.resolution,
            Arc::clone(&self.cs),
            Arc::clone(&self.filter),
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
