use png::{BitDepth, ColorType, Decoder, DecodingError};
use std::fs::File;
use crate::tracer::Spectrum;

/// Loaded texture images stored in a Rust vector
#[derive(Clone)]
pub struct Image {
    /// Image buffer storing color values
    pub buffer: Vec<Spectrum>,
    /// Width of rendered image.
    pub width: u32,
    /// Height of rendered image.
    pub height: u32,
}

impl Image {
    /// Creates an `image` struct from a file at `path`
    pub fn from_path(path: &str) -> Result<Self, DecodingError> {
        println!("Decoding \"{}\"", path);
        Self::from_file(File::open(path)?)
    }

    /// Creates an `image` from `file`
    pub fn from_file(file: File) -> Result<Self, DecodingError> {
        let decoder = Decoder::new(file);
        let mut reader = decoder.read_info()?;
        let mut bytes = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut bytes)?;

        assert!(info.bit_depth == BitDepth::Eight);

        let buffer = match info.color_type {
            ColorType::Rgb => {
                bytes[..info.buffer_size()]
                    .chunks(3)
                    .map(|rgb| Spectrum::from_srgb(rgb[0], rgb[1], rgb[2]))
                    .collect()
            }
            ColorType::Rgba => {
                bytes[..info.buffer_size()]
                    .chunks(4)
                    .map(|rgba| Spectrum::from_srgb(rgba[0], rgba[1], rgba[2]))
                    .collect()
            }
            _ => panic!("unsupported image type {:?}", info.color_type),
        };

        let width = info.width;
        let height = info.height;

        // maybe not correct for textures, but we do it anyway
        // assert!(width == height);
        // not correct for textures!
        println!("Decoded succesfully");
        Ok(Self {
            buffer,
            width,
            height,
        })
    }
}
