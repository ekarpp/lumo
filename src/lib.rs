//! Just a path tracer :)
#![warn(missing_docs)]
#![allow(clippy::needless_range_loop)]

pub use image::Image;
pub use perlin::Perlin;
pub use renderer::Renderer;
pub use samplers::SamplerType;
pub use tone_mapping::ToneMap;

/// Wavefront .mtl and .obj parser
pub mod parser;
/// The heart.
pub mod tracer;

/// `Float` with built in tracking of floating point error
mod efloat;
/// Utility functions to format output
mod formatting;
/// Wrapper for writing image buffer to file.
mod image;
/// Math utilities
mod math;
/// Perlin noise generator.
mod perlin;
/// Random number generator and utility functions to sample different geometrics
mod rng;
/// Configures and computes the image.
mod renderer;
/// Different iterators that stream values sampled from the unit square.
mod samplers;
/// Tone mapping functions
mod tone_mapping;

type Transform = math::transform::Transform;
type Vec4 = math::mat4::Vec4;
type Mat4 = math::mat4::Mat4;
type Vec2 = math::vec2::Vec2;
/// 3x3 matrix type alias
pub type Mat3 = math::mat3::Mat3;
/// 3 element vector type alias
pub type Vec3 = math::vec3::Vec3;
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
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub enum Axis {
    /// X-axis
    X = 0,
    /// Y-axis
    Y = 1,
    /// Z-axis
    Z = 2,
}
