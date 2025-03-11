//! Just a path tracer :)
#![warn(missing_docs)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::identity_op)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::upper_case_acronyms)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

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
/// 2 element float vector type
pub type Vec2 = math::vec2::Vec2;
/// 3x3 float matrix type
pub type Mat3 = math::mat3::Mat3;
/// 3 element float vector type
pub type Vec3 = math::vec3::Vec3;
/// Float type alias
pub type Float = f64;

/// easy as ...
pub const PI: Float = if Float::DIGITS == 6 {
    std::f32::consts::PI as Float
} else {
    std::f64::consts::PI as Float
};
const INF: Float = Float::INFINITY;
const NEG_INF: Float = Float::NEG_INFINITY;
const EPSILON: Float = if Float::DIGITS == 6 { 1e-5 } else { 1e-10 };

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
