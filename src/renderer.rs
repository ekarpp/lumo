use crate::cli::TracerCli;
use crate::image::Image;
use crate::samplers::JitteredSampler;
use crate::tone_mapping::ToneMap;
use crate::tracer::Camera;
use crate::tracer::Integrator;
use crate::tracer::Scene;
use glam::{DVec2, DVec3};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::time::Instant;

type PxSampler = JitteredSampler;

/// Configures the image to be rendered
pub struct Renderer {
    scene: Scene,
    camera: Camera,
    img_width: i32,
    img_height: i32,
    num_samples: u32,
    integrator: Integrator,
    tone_map: ToneMap,
}

impl Renderer {
    /// Constructs a new renderer. Defaults to 1000x1000 image with 1 sample
    /// per pixel and path tracing as the integrator. Configured through the CLI
    /// or the setter functions of the struct.
    pub fn new(scene: Scene, camera: Camera) -> Self {
        assert!(scene.num_lights() != 0);

        let cli_args: TracerCli = argh::from_env();
        cli_args.set_threads();

        Self {
            scene,
            camera,
            img_width: cli_args.width,
            img_height: cli_args.height,
            num_samples: cli_args.samples,
            integrator: cli_args.get_integrator(),
            tone_map: ToneMap::NoMap,
        }
    }

    /// Sets the tone mapping algorithm used
    pub fn set_tone_map(&mut self, tone_map: ToneMap) {
        self.tone_map = tone_map;
    }

    /// Sets number of samples per pixel
    pub fn set_samples(&mut self, samples: u32) {
        self.num_samples = samples;
    }

    /// Sets the width of the image
    pub fn set_width(&mut self, width: i32) {
        self.img_width = width;
    }

    /// Sets the height of the image
    pub fn set_height(&mut self, height: i32) {
        self.img_height = height;
    }

    /// Sets the integrator used to render the image
    pub fn set_integrator(&mut self, integrator: Integrator) {
        self.integrator = integrator;
    }

    /// Starts the rendering process and returns the rendered image
    pub fn render(&self) -> Image {
        println!(
            "Rendering scene as a {} x {} image \
                  with {} thread(s) and {} sample(s) per pixel using {}",
            self.img_width,
            self.img_height,
            rayon::current_num_threads(),
            self.num_samples,
            self.integrator,
        );

        let start = Instant::now();

        let buffer: Vec<DVec3> = (0..self.img_height)
            .into_par_iter()
            .flat_map(|y: i32| {
                (0..self.img_width)
                    .map(|x: i32| self.get_color(x, y))
                    .collect::<Vec<DVec3>>()
            })
            .collect();

        println!("Finished rendering in {:#?}", start.elapsed());

        Image::new(buffer, self.img_width, self.img_height)
    }

    /// Sends `num_samples` rays towards the given pixel and averages the result
    fn get_color(&self, x: i32, y: i32) -> DVec3 {
        let max_dim = self.img_height.max(self.img_width) as f64;
        /* (u,v) in [-1, 1]^2 */
        let u = (2 * x + 1 - self.img_width) as f64 / max_dim;
        let v = (2 * (self.img_height - y) - 1 - self.img_height) as f64 / max_dim;

        PxSampler::new(self.num_samples)
            .map(|rand_sq: DVec2| {
                // random offsets
                let ou = rand_sq.x / max_dim;
                let ov = rand_sq.y / max_dim;
                let rgb = self.integrator.integrate(
                    &self.scene,
                    self.camera.generate_ray(u + ou, v + ov)
                );

                self.tone_map.map(rgb)
            })
            .fold(DVec3::ZERO, |acc, c| acc + c)
            / self.num_samples as f64
    }
}
