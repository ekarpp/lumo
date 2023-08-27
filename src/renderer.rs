use crate::cli::TracerCli;
use crate::samplers::JitteredSampler;
use crate::tone_mapping::ToneMap;
use crate::tracer::Camera;
use crate::tracer::Film;
use crate::tracer::FilmSample;
use crate::tracer::Integrator;
use crate::tracer::Scene;
use crate::tracer::{TriangleFilter, GaussianFilter, BoxFilter};
use glam::IVec2;
use crate::{Vec2, Float};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::time::Instant;

type PxSampler = JitteredSampler;

/// Configures the image to be rendered
pub struct Renderer {
    scene: Scene,
    camera: Camera,
    resolution: IVec2,
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
    pub fn render(&self) -> Film {
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
        let samples: Vec<FilmSample> = (0..self.resolution.y)
            .into_par_iter()
            .map(|y: i32| {
                (0..self.resolution.x)
                    .flat_map(move |x: i32| self.get_samples(x, y))
            })
            .flatten_iter()
            .collect();

        let mut film = Film::new(
            self.resolution.x,
            self.resolution.y,
            Box::new(BoxFilter::new(1)),
            //Box::new(GaussianFilter::new(1, 50.0)),
        );
        film.add_samples(samples);
        println!("Finished rendering in {:#?}", start.elapsed());
        film
    }

    /// Sends `num_samples` rays towards the given pixel and averages the result
    fn get_samples(&self, x: i32, y: i32) -> Vec<FilmSample> {
        let xy = Vec2::new(x as Float, y as Float);
        PxSampler::new(self.num_samples)
            .flat_map(|rand_sq: Vec2| {
                let raster_xy = xy + rand_sq;
                self.integrator.integrate(
                    &self.scene,
                    &self.camera,
                    raster_xy,
                    self.camera.generate_ray(raster_xy),
                )
            })
            .map(|mut sample: FilmSample| {
                sample.color = self.tone_map.map(sample.color);
                sample
            })
            .collect()
    }
}
