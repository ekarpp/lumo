use crate::{DVec3, DVec2};
use std::time::Instant;
use rayon::iter::{ParallelIterator, IntoParallelIterator};
use crate::image::Image;
use crate::cli::TracerCli;
#[allow(unused_imports)]
use crate::samplers::{JitteredSampler, UniformSampler};
use crate::scene::Scene;
use crate::camera::Camera;
use crate::tracer::Integrator;


type PxSampler = JitteredSampler;

/// Configures the image to be rendered
pub struct Renderer {
    scene: Scene,
    camera: Camera,
    img_width: i32,
    img_height: i32,
    num_samples: u32,
    integrator: Integrator,
}

impl Renderer {

    /// Constructs a new renderer. Defaults to 1000x1000 image with 1 sample
    /// per pixel and path tracing as the integrator.
    pub fn new(scene: Scene, camera: Camera) -> Self {
        let cli_args: TracerCli = argh::from_env();

        cli_args.set_threads();
        cli_args.output_cfg();

        Self {
            scene,
            camera,
            img_width: cli_args.get_width(),
            img_height: cli_args.get_height(),
            num_samples: cli_args.get_samples(),
            integrator: Integrator::PathTrace,
        }
    }

    /// Sets the integration algorithm used
    pub fn set_integrator(&mut self, integrator: Integrator) {
        self.integrator = integrator;
    }

    /// Starts the rendering process and returns the rendered image
    pub fn render(&self) -> Image {
        let start = Instant::now();

        let buffer: Vec<DVec3> = (0..self.img_height).into_par_iter()
            .flat_map(|y: i32| {
                (0..self.img_width)
                    .map(|x: i32| self.get_color(x, y))
                    .collect::<Vec<DVec3>>()
            })
            .collect();

        println!("Finished rendering in {:#?}", start.elapsed());

        Image::new(
            buffer,
            self.img_width,
            self.img_height,
        )
    }

    /// Sends `num_samples` rays towards the given pixel and averages the result
    fn get_color(&self, x: i32, y: i32) -> DVec3 {
        let max_dim = self.img_height.max(self.img_width) as f64;
        /* (u,v) in [-1, 1]^2 */
        let u = (2*x + 1 - self.img_width) as f64 / max_dim;
        let v = (2 * (self.img_height - y) - 1 - self.img_height) as f64
            / max_dim;

        PxSampler::new(self.num_samples)
            .map(|rand_sq: DVec2| {
                // random offsets
                let ou = rand_sq.x / max_dim;
                let ov = rand_sq.y / max_dim;
                self.integrator.integrate(
                    &self.scene,
                    &self.camera.ray_at(
                        u + ou,
                        v + ov
                    ),
                )
            })
            .fold(DVec3::ZERO, |acc: DVec3, c: DVec3| acc + c)
            / self.num_samples as f64
    }
}
