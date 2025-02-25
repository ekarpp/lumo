//! Just a path tracer :)
#![warn(missing_docs)]
#![allow(clippy::needless_range_loop)]

pub use cli::TracerCli;
pub use image::Image;
pub use perlin::Perlin;
pub use renderer::Renderer;
pub use samplers::SamplerType;
pub use tone_mapping::ToneMap;

/// Wavefront .mtl and .obj parser
pub mod parser;
/// The heart.
pub mod tracer;

/// Complex numbers
mod complex;
/// Command line interface
mod cli;
/// `Float` with built in tracking of floating point error
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
/// Utility functions when working with vectors in shading space
mod spherical_utils;
/// Tone mapping functions
mod tone_mapping;

#[cfg(test)]
/// Implementation of adaptive simpson for numerical integration. Used in tests only.
mod simpson_integration;

#[cfg(test)]
/// Chi2 CDF, used only in tests
mod chi2;

type Transform = glam::DAffine3;
type Vec4 = glam::DVec4;
type Mat4 = glam::DMat4;
type Vec2 = glam::DVec2;
/// 3x3 matrix type alias
pub type Mat3 = glam::DMat3;
/// 3 element vector type alias
pub type Vec3 = glam::DVec3;
/// Float type alias
pub type Float = f64;

/// easy as ...
pub const PI: Float = std::f64::consts::PI;
const INF: Float = f64::INFINITY;
const NEG_INF: Float = f64::NEG_INFINITY;
const EPSILON: Float = 1e-10;

type Normal = Vec3;
type Direction = Vec3;
type Point = Vec3;


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
