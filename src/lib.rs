//! Just a path tracer :)
#![warn(missing_docs)]

use glam::DVec3;

/// Epsilon to avoid self intersection of objects
const EPSILON: f64 = 1e-10;

pub use cli::TracerCli;
pub use image::Image;
pub use perlin::Perlin;
pub use renderer::Renderer;
pub use tone_mapping::ToneMap;

/// Wavefront .mtl and .obj parser
pub mod parser;
/// The heart.
pub mod tracer;

/// Command line interface
mod cli;
/// `f64` with built in tracking of floating point error
mod efloat;
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

/// Maps linear RGB value to luminance
pub fn rgb_to_luminance(rgb: DVec3) -> f64 {
    rgb.dot(DVec3::new(0.2126, 0.7152, 0.0722))
}

/// Enum to determine from which direction we are tracing rays
#[derive(Copy, Clone)]
pub enum Transport {
    /// Starting from camera
    Radiance = 0,
    /// Starting from light
    Importance = 1,
}

/// Represents an axis in the cartesian coordinate system
#[derive(Copy, Clone)]
pub enum Axis {
    /// X-axis
    X,
    /// Y-axis
    Y,
    /// Z-axis
    Z,
}
