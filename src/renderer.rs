use crate::{DVec3, DVec2};
use rayon::iter::{ParallelIterator, IntoParallelIterator};
#[allow(unused_imports)]
use crate::samplers::{JitteredSampler, UniformSampler};
use crate::tracer::scene::Scene;
use crate::tracer::camera::Camera;
#[allow(unused_imports)]
use crate::tracer::integrator::Integrator;

type PxSampler = JitteredSampler;

pub fn _render(
    img_height: usize,
    px_height: f64,
    img_width: usize,
    px_width: f64,
    num_samples: usize,
    cam: Camera,
    scene: &Scene,
) -> Vec<DVec3> {
    let integrator = Integrator::PathTrace;
    (0..img_height).into_par_iter().flat_map(|y: usize| {
        (0..img_width).map(|x: usize| {
            let u = x as f64 * px_width;
            let v = (img_height - 1 - y) as f64 * px_height;

            PxSampler::new(num_samples).map(|rand_sq: DVec2| {
                integrator.integrate(
                    scene,
                    &cam.ray_at(u + rand_sq.x*px_width, v + rand_sq.y*px_height),
                )
            }).fold(DVec3::ZERO, |acc: DVec3, c: DVec3| acc + c)
                / num_samples as f64
        }).collect::<Vec<DVec3>>()
    }).collect()
}
