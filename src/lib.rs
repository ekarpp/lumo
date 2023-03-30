//! Just a path tracer :)
#![warn(missing_docs)]

use glam::DVec3;

/// Epsilon to avoid self intersection of objects
const EPSILON: f64 = 1e-14;

pub use cli::TracerCli;
pub use image::Image;
pub use perlin::Perlin;
pub use renderer::Renderer;
pub use tone_mapping::ToneMap;

/// .OBJ file loader. Supports loading only vertices, normals, and faces.
pub mod obj;
/// The heart.
pub mod tracer;

/// Command line interface
mod cli;
/// Wrapper for writing image buffer to file.
mod image;
/// Perlin noise generator.
mod perlin;
/// Wrapper around rand. Provides functions to sample from various geometrics.
mod rand_utils;
/// Configures and computes the image.
mod renderer;
/// Different iterators that stream values sampled from the unit square.
mod samplers;

/// Tone mapping functions
mod tone_mapping;

/// Decodes 8-bit sRGB encoded `r`, `g`, and `b` channels to linear RGB.
pub fn srgb_to_linear(r: u8, g: u8, b: u8) -> DVec3 {
    DVec3::new(
        (r as f64 / 255.0).powf(2.2),
        (g as f64 / 255.0).powf(2.2),
        (b as f64 / 255.0).powf(2.2),
    )
}
