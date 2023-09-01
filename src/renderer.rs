use crate::{
    Vec2, Float, TracerCli,
    samplers::JitteredSampler, ToneMap
};
use crate::tracer::{
    Camera, Film, FilmSample,
    Integrator, Scene, Filter, FilmTile
};
use glam::IVec2;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{sync::{Mutex, Arc}, time::Instant};

type PxSampler = JitteredSampler;

const TILE_SIZE: i32 = 16;

/// Configures the image to be rendered
pub struct Renderer {
    scene: Scene,
    camera: Camera,
    resolution: IVec2,
    num_samples: u32,
    integrator: Integrator,
    tone_map: ToneMap,
    filter: Filter,
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
            filter: Filter::Box,
            num_samples: cli_args.samples,
            integrator: cli_args.get_integrator(),
            tone_map: ToneMap::NoMap,
        }
    }

    /// Sets the tone mapping algorithm used
    pub fn set_tone_map(&mut self, tone_map: ToneMap) {
        self.tone_map = tone_map;
    }

    /// Sets the pixel filter
    pub fn set_filter(&mut self, filter: Filter) {
        self.filter = filter;
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
        let mut film = Film::new(
            self.resolution.x,
            self.resolution.y,
            self.filter,
        );

        let mutex = Mutex::new(&mut film);

        let tiles_x = (self.resolution.x + TILE_SIZE - 1) / TILE_SIZE;
        let tiles_y = (self.resolution.y + TILE_SIZE - 1) / TILE_SIZE;
        (0..tiles_y).into_par_iter()
            .for_each(|y: i32| {
                (0..tiles_x).for_each(|x: i32| {
                    let px_min = IVec2::new(x, y) * TILE_SIZE;
                    let px_max = px_min + TILE_SIZE;
                    let mut tile = mutex.lock().unwrap().get_tile(px_min, px_max);
                    for y in tile.px_min.y..tile.px_max.y {
                        for x in tile.px_min.x..tile.px_max.x {
                            self.get_samples(&mut tile, x, y)
                        }
                    }
                    mutex.lock().unwrap().add_tile(tile);
                })
            });
        println!("Finished rendering in {:#?}", start.elapsed());
        film
    }

    /// Sends `num_samples` rays towards the given pixel and averages the result
    fn get_samples(&self, tile: &mut FilmTile, x: i32, y: i32) {
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
            .for_each(|mut sample: FilmSample| {
                sample.color = self.tone_map.map(sample.color);
                tile.add_sample(sample)
            })
    }
}
