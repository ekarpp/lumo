use crate::cli::TracerCli;
use crate::image::Image;
use crate::samplers::JitteredSampler;
use crate::tone_mapping::ToneMap;
use crate::tracer::Camera;
use crate::tracer::Integrator;
use crate::tracer::Scene;
use glam::{DVec2, IVec2, DVec3};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::time::Instant;

type PxSampler = JitteredSampler;

/// Configures the image to be rendered
pub struct Renderer {
    scene: Scene,
    camera: Box<dyn Camera>,
    resolution: IVec2,
    num_samples: u32,
    integrator: Integrator,
    tone_map: ToneMap,
}

impl Renderer {
    /// Constructs a new renderer. Defaults to 1000x1000 image with 1 sample
    /// per pixel and path tracing as the integrator. Configured through the CLI
    /// or the setter functions of the struct.
    pub fn new(scene: Scene, camera: Box<dyn Camera>) -> Self {
        assert!(scene.num_lights() != 0);

        let cli_args: TracerCli = argh::from_env();
        cli_args.set_threads();

        let resolution = camera.get_resolution();

        Self {
            scene,
            camera,
            resolution,
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

    /// Sets the integrator used to render the image
    pub fn set_integrator(&mut self, integrator: Integrator) {
        self.integrator = integrator;
    }

    /// Starts the rendering process and returns the rendered image
    pub fn render(&self) -> Image {


        println!(
            "Rendering scene as a {} x {} image \
                  with {} thread(s) and {} sample(s) per pixel using {}",
            self.resolution.x,
            self.resolution.y,
            rayon::current_num_threads(),
            self.num_samples,
            self.integrator,
        );

        let start = Instant::now();
        let buffer: Vec<DVec3> = (0..self.resolution.y)
            .into_par_iter()
            .flat_map(|y: i32| {
                (0..self.resolution.x)
                    .map(|x: i32| self.get_color(x, y))
                    .collect::<Vec<DVec3>>()
            })
            .collect();

        println!("Finished rendering in {:#?}", start.elapsed());

        Image::new(buffer, self.resolution.x, self.resolution.y)
    }

    /// Sends `num_samples` rays towards the given pixel and averages the result
    fn get_color(&self, x: i32, y: i32) -> DVec3 {
        let xy = DVec2::new(x as f64, y as f64);

        PxSampler::new(self.num_samples)
            .map(|rand_sq: DVec2| {
                let rgb = self.integrator.integrate(
                    &self.scene,
                    self.camera.generate_ray(xy + rand_sq)
                );

                self.tone_map.map(rgb)
            })
            .fold(DVec3::ZERO, |acc, c| acc + c)
            / self.num_samples as f64
    }
}
