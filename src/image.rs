use png::{BitDepth, ColorType, Decoder, DecodingError};
use std::{ io::{BufRead, BufReader, Read}, fs::{self, File} };
use crate::{Float, Normal, Vec2, Vec3};
use crate::tracer::{Color, ColorWavelength, Spectrum, RGB};

/// Loaded texture images stored in a Rust vector
#[derive(Clone)]
pub struct Image<T> {
    /// Image buffer storing color values
    pub buffer: Vec<T>,
    /// Width of rendered image.
    pub width: u32,
    /// Height of rendered image.
    pub height: u32,
    mean: T,
}

impl<T> Image<T> {
    fn decode_png<R: Read>(read: R) -> Result<(Vec<[u8; 3]>, png::OutputInfo), DecodingError> {
        let decoder = Decoder::new(read);
        let mut reader = decoder.read_info()?;
        let mut bytes = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut bytes)?;

        let buffer = match info.color_type {
            ColorType::Indexed => {
                let palette = reader.info().palette.as_ref().unwrap();

                (0..info.height * info.width)
                    .map(|idx| {
                        let (bidx, rss, msk) = match info.bit_depth {
                            BitDepth::One => {
                                (idx / 8, idx % 8, (1 << 1) - 1)
                            }
                            BitDepth::Two => {
                                (idx / 4, 2 * (idx % 4), (1 << 2) - 1)
                            }
                            BitDepth::Four => {
                                (idx / 2, 4 * (idx % 2), (1 << 4) - 1)
                            }
                            BitDepth::Eight => {
                                (idx, 0, 0xFF)
                            }
                            BitDepth::Sixteen => unreachable!(),
                        };
                        let byte = bytes[bidx as usize];
                        let pidx = (byte >> rss) & msk;
                        let pidx = pidx as usize;
                        [palette[3*pidx + 0], palette[3*pidx + 1], palette[3*pidx + 2]]
                    })
                    .collect()
            }
            ColorType::Grayscale | ColorType::GrayscaleAlpha => {
                let chunk_size = if info.color_type == ColorType::GrayscaleAlpha
                { 2 } else { 1 };

                bytes[..info.buffer_size()]
                    .chunks(chunk_size)
                    .map(|gs| [gs[0], gs[0], gs[0]])
                    .collect()
            }
            ColorType::Rgb | ColorType::Rgba => {
                let chunk_size = {
                    let bytes = 1;
                    // only support rgb/rgba
                    let channels = if info.color_type == ColorType::Rgb { 3 } else { 4 };
                    bytes * channels
                };

                bytes[..info.buffer_size()]
                    .chunks(chunk_size)
                    .map(|rgb| [rgb[0], rgb[1], rgb[2]])
                    .collect()
            }
        };

        Ok((buffer, info))
    }

    /// Get the mean Vec3 of rgb channels from the image
    pub fn mean_vec3_from_file<R: Read>(read: R) -> Result<Vec3, DecodingError> {
        let (pixels, info) = Self::decode_png(read)?;
        let scale = 1.0 / (info.width * info.height) as Float;
        Ok(
            pixels.iter()
                .fold(Vec3::ZERO, |acc, rgb| {
                    let map_byte = |c: u8| { scale * c as Float / 256.0};

                    acc + Vec3::new(
                        map_byte(rgb[0]),
                        map_byte(rgb[1]),
                        map_byte(rgb[2]),
                    )
                })
        )
    }

    #[inline]
    fn bilin_interp<R, F: Fn(&T, &T, Float) -> R>(&self, uv: Vec2, lerp: F) -> (R, R, Float) {
        let (w, h) = (self.width as Float, self.height as Float);

        let xy = Vec2::new(
            uv.x * w,
            (1.0 - uv.y) * h,
        );
        let xoyo = (xy - 0.5).floor();

        let x1y1 = xy - xoyo - 0.5;
        let x0y0 = 1.0 - x1y1;

        #[cfg(debug_assertions)]
        {
            assert!(x1y1.x >= 0.0 && x1y1.x <= 1.0);
            assert!(x1y1.y >= 0.0 && x1y1.y <= 1.0);
        }

        let xo = (xoyo.x + w) as u32 % self.width;
        let yo = (xoyo.y + h) as u32 % self.height;
        let xi = (xo + 1) % self.width;
        let yi = (yo + 1) % self.height;

        let xy00 = &self.buffer[(xo + yo * self.width) as usize];
        let xy10 = &self.buffer[(xi + yo * self.width) as usize];
        let xy01 = &self.buffer[(xo + yi * self.width) as usize];
        let xy11 = &self.buffer[(xi + yi * self.width) as usize];

        (lerp(xy00, xy10, x0y0.x), lerp(xy01, xy11, x0y0.x), x0y0.y)
    }
}

impl Image<Normal> {
    /// Bilinearly interpolated normal at `uv`
    #[inline]
    pub fn value_at(&self, uv: Vec2) -> Normal {
        let lerp = |n0: &Normal, n1: &Normal, v: Float| -> Normal {
            (*n0 * v + *n1 * (1.0 - v)).normalize()
        };
        let (y0, y1, v) = self.bilin_interp(uv, lerp);
        lerp(&y0, &y1, v)
    }

    /// Parse a bump map from file
    pub fn bump_from_file<R: Read>(read: R) -> Result<Self, DecodingError> {
        let (pixels, info) = Self::decode_png(read)?;

        let buffer = pixels.iter()
            .map(|rgb| {
                let map_byte = |c: u8| {
                    c as Float / 128.0 - 1.0
                };

                Normal::new(
                    map_byte(rgb[0]),
                    map_byte(rgb[1]),
                    map_byte(rgb[2]),
                ).normalize()
            })
            .collect();

        let width = info.width;
        let height = info.height;

        Ok(Self {
            buffer,
            width,
            height,
            mean: Normal::Z,
        })
    }
}

impl Image<Spectrum> {
    #[inline]
    /// Bilinearly interpolate the texture at `uv` for `lambda`
    pub fn value_at(&self, uv: Vec2, lambda: &ColorWavelength) -> Color {
        let lerp = |s0: &Spectrum, s1: &Spectrum, v: Float| -> Color {
            s0.sample(lambda) * v + s1.sample(lambda) * (1.0 - v)
        };
        let (y0, y1, v) = self.bilin_interp(uv, lerp);
        let c = y0 * v + y1 * (1.0 - v);
        #[cfg(debug_assertions)]
        assert!(!c.is_neg());
        c
    }

    /// Get the power of the texture, or average over all of the pixels
    pub fn power(&self, lambda: &ColorWavelength) -> Color {
        self.mean.sample(lambda)
    }

    /// Creates an `image` struct from a file at `path`
    pub fn from_path(path: &str) -> Result<Self, DecodingError> {
        println!("Decoding \"{}\"", path);
        Self::from_file(File::open(path)?)
    }

    /// Create image file from a Radiance HDR image file
    pub fn from_hdri(path: &str) -> Result<Self, std::io::Error> {
        println!("Decoding HDR image \"{}\"", path);
        let bytes = fs::read(path)?;
        Self::from_hdri_bytes(bytes.as_slice())
    }

    /// Radiance HDR image file byte content
    pub fn from_hdri_bytes<R: BufRead>(bytes: R) -> Result<Self, std::io::Error> {
        let mut reader = BufReader::new(bytes);
        let mut buffer = Vec::new();

        let ascii_lf = 10;
        let ascii_plus = 43;
        let ascii_minus = 45;
        reader.read_until(ascii_lf, &mut buffer)?;
        let header = std::str::from_utf8(&buffer).unwrap();
        assert!(header.trim() == "#?RADIANCE");

        let width;
        let height;
        loop {
            buffer.clear();
            reader.read_until(ascii_lf, &mut buffer)?;
            if buffer[0] == ascii_plus || buffer[0] == ascii_minus {
                let fields: Vec<&str> = std::str::from_utf8(&buffer).unwrap()
                    .split_whitespace().collect();
                assert!(fields[0].chars().nth(0) == Some('-'));
                assert!(fields[2].chars().nth(0) == Some('+'));
                height = fields[1].parse::<u32>().unwrap();
                width = fields[3].parse::<u32>().unwrap();
                break;
            }
        }

        let mut bytes = Vec::new();
        while let Ok(len) = reader.read_until(0x00, &mut bytes) {
            if len == 0 { break; }
        }

        let chunk_size = 4;
        assert!(bytes.len() as u32 == width * height * chunk_size as u32);

        let sum_rgb = bytes.chunks(chunk_size).fold(RGB::BLACK, |acc, chk| {
            acc + RGB::from_rgbe(chk[0], chk[1], chk[2], chk[3])
        });
        let mean = Spectrum::from_rgb(sum_rgb / (width * height) as Float);

        let buffer: Vec<Spectrum> = bytes.chunks(chunk_size)
            .map(|chk| Spectrum::from_rgb(RGB::from_rgbe(chk[0], chk[1], chk[2], chk[3])))
            .collect();

        assert!(buffer.len() == (width * height) as usize);
        println!("Decoded succesfully");
        Ok(Self { width, height, buffer, mean })
    }

    /// Creates an `image` from `file`
    pub fn from_file<R: Read>(read: R) -> Result<Self, DecodingError> {
        let (pixels, info) = Self::decode_png(read)?;

        let sum_rgb = pixels.iter()
            .fold(RGB::BLACK, |acc, rgb| acc + RGB::from_srgb(rgb[0], rgb[1], rgb[2]));
        let mean = Spectrum::from_rgb(sum_rgb / pixels.len() as Float);

        let buffer = pixels.iter()
            .map(|rgb| Spectrum::from_srgb(rgb[0], rgb[1], rgb[2]))
            .collect();

        let width = info.width;
        let height = info.height;

        println!("Decoded succesfully");
        Ok(Self {
            buffer,
            width,
            height,
            mean,
        })
    }
}
