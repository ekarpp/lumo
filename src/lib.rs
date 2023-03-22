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
