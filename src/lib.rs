//! Just a path tracer :)
use glam::{UVec3, f64::{DVec3, DMat3, DVec2, DAffine3}};
pub use glam;

pub use consts::*;
pub use image::*;
pub use obj::*;
pub use camera::*;
pub use scene::*;
pub use onb::*;
pub use pdfs::*;
pub use perlin::*;
pub use rand_utils::*;
pub use renderer::*;
pub use samplers::*;
pub use tracer::*;

/// Scene that describes the 3D world to render.
pub mod scene;
/// Abstraction for a camera
pub mod camera;


/// .OBJ file loader
mod obj;
/// Utility struct for orthonormal basis.
mod onb;
/// Implementation of different probability density functions for sampling.
mod pdfs;
/// Wrapper for writing image buffer to file.
mod image;
/// The heart.
mod tracer;
/// Various constants used around the crate.
mod consts;
/// Perlin noise generator.
mod perlin;
/// Different iterators that stream values sampled from the unit square.
mod samplers;
/// Main renderer.
mod renderer;
/// Wrapper around rand. Provides functions to sample from various geometrics.
mod rand_utils;
