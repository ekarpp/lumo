use super::*;

#[derive(Clone)]
pub struct TilePixel {
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

impl AddAssign<&TilePixel> for Pixel {
    fn add_assign(&mut self, rhs: &TilePixel) {
        self.color += &rhs.color;
        self.color_weight += rhs.color_weight;
    }
}

pub struct TileSplat {
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
    /// Pixels of the tile
    pub pixels: Vec<TilePixel>,
    /// Splat samples of the tile
    pub splats: Vec<TileSplat>,
    resolution: UVec2,
    cs: &'static ColorSpace,
    filter: Arc<PixelFilter>,
    white_balance: Mat3,
}

impl FilmTile {
    /// Creates a new tile `[px_min.x, px_max.x) x [px_min.y, px_max.y)` with `filter`
    pub fn new(
        px_min: UVec2,
        px_max: UVec2,
        resolution: UVec2,
        cs: &'static ColorSpace,
        white_balance: Mat3,
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
            white_balance,
            resolution,
            pixels: vec![TilePixel::default(); (width * height) as usize],
            splats: vec![],
        }
    }

    /// Adds a sample to the tile
    pub fn add_sample(&mut self, sample: &FilmSample) {
        let rgb = self.cs.from_color(&sample.color, &sample.lambda, &self.white_balance);

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
