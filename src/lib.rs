//! Just a path tracer :)
#![warn(missing_docs)]

use glam::DVec3;

pub use consts::*;
pub use image::Image;
pub use perlin::Perlin;
pub use renderer::Renderer;
pub use tone_mapping::ToneMap;


/// The heart.
pub mod tracer;
/// .OBJ file loader
pub mod obj;

/// Wrapper around rand. Provides functions to sample from various geometrics.
mod rand_utils;
/// Command line interface
mod cli;
/// Various constants used around the crate.
mod consts;
/// Wrapper for writing image buffer to file.
mod image;
/// Perlin noise generator.
mod perlin;
/// Configures and computes the image.
mod renderer;
/// Different iterators that stream values sampled from the unit square.
mod samplers;

/// Tone mapping functions
mod tone_mapping;

/// Decodes 8-bit sRGB encoded `r`, `g`, and `b` channels to linear RGB.
pub fn srgb_to_lin(r: u8, g: u8, b: u8) -> DVec3 {
    DVec3::new(
        (r as f64 / 255.0).powf(2.2),
        (g as f64 / 255.0).powf(2.2),
        (b as f64 / 255.0).powf(2.2),
    )
}
