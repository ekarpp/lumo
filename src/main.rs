//! Ray tracer. WIP
use glam::{UVec3, f64::{DVec3, DMat3, DVec2, DAffine3}};
use crate::renderer::Renderer;
use crate::image::Image;
use crate::tracer::scene::Scene;
use crate::tracer::camera::Camera;

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

/// Default output filename
const FNAME: &str = "render.png";

fn main() {
    let fl = 1.0;
    let scene = Scene::obj_scene(fl);
    let cam = Camera::new(
        1.0,
        90.0,
        DVec3::new(0.0, 0.0, 0.0), /* origin */
        DVec3::new(0.0, 0.0, -1.0), /* towards */
        DVec3::new(0.0, 1.0, 0.0), /* up */
        fl, /* focal length */
    );

    let mut renderer = Renderer::new(scene, cam);
    let start_img = std::time::SystemTime::now();
    let image_buffer = renderer.render();
    match start_img.elapsed() {
        Ok(v) => println!("rendered scene in {v:?}"),
        Err(e) => println!("rendering done, error measuring duration {e:?}"),
    }

    let image = Image::new(
        image_buffer,
        1000,
        1000,
        String::from(FNAME),
    );

    let start_png = std::time::SystemTime::now();
    match image.save() {
        Ok(()) => (),
        Err(e) => println!("error during png encoding {e:?}"),
    }
    match start_png.elapsed() {
        Ok(v) => println!("created png in {v:?}"),
        Err(e) => println!("png done, error measuring duration {e:?}"),
    }
}
