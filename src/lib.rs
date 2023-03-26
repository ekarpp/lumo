//! Just a path tracer :)
#![warn(missing_docs)]

use glam::DVec3;

pub use image::Image;
pub use scene::Scene;
pub use camera::Camera;
pub use consts::*;
pub use perlin::Perlin;
pub use renderer::Renderer;
pub use samplers::{UniformSampler, JitteredSampler};
pub use tone_mapping::ToneMap;

/// Wrapper around rand. Provides functions to sample from various geometrics.
pub mod rand_utils;
/// The heart.
pub mod tracer;
/// .OBJ file loader
pub mod obj;

/// Command line interface
mod cli;
/// Abstraction for a camera
mod camera;
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
/// Scene that describes the 3D world to render.
mod scene;

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

/// Maps linear RGB value to luminance
pub fn rgb_to_luminance(rgb: DVec3) -> f64 {
    rgb.dot(DVec3::new(0.2126, 0.7152, 0.0722))
}
