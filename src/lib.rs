//! Just a path tracer :)
#![warn(missing_docs)]

pub use glam::{UVec3, f64::{DVec3, DMat3, DVec2, DAffine3}};

pub use obj::*;
pub use image::*;
pub use scene::*;
pub use camera::*;
pub use consts::*;
pub use perlin::*;
pub use tracer::*;
pub use renderer::*;
pub use samplers::*;
pub use rand_utils::*;
pub use tone_mapping::*;

/// Command line interface
mod cli;
/// Abstraction for a camera
mod camera;
/// Various constants used around the crate.
mod consts;
/// Wrapper for writing image buffer to file.
mod image;
/// .OBJ file loader
mod obj;
/// Perlin noise generator.
mod perlin;
/// Wrapper around rand. Provides functions to sample from various geometrics.
mod rand_utils;
/// Configures and computes the image.
mod renderer;
/// Different iterators that stream values sampled from the unit square.
mod samplers;
/// Scene that describes the 3D world to render.
mod scene;
/// The heart.
mod tracer;
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
