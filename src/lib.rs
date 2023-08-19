//! Just a path tracer :)
#![warn(missing_docs)]

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
