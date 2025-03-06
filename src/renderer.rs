use crate::{
    formatting, rng::Xorshift, Vec2, Float, ToneMap, SamplerType
};
use crate::tracer::{
    Camera, Film, FilmSample,
    Integrator, Scene, PixelFilter, FilmTile, ColorSpace
};
use crate::math::vec2::UVec2;
use std::{
    sync::{Arc, mpsc, Mutex}, cell::RefCell,
    io::Write, time::Instant
};
use itertools::Itertools;

const TILE_SIZE: u64 = 16;
// should be a square for samplers
const SAMPLES_INCREMENT: u64 = 256;
const PROGRESS_BAR_LEN: usize = 16;

const DEFAULT_NUM_SAMPLES: u64 = 1;
const DEFAULT_THREADS: usize = 4;

use task::{ RenderTask, RenderTaskResult, RenderTaskExecutor };

mod pool;
mod queue;
mod task;
mod worker;

/// Configures the image to be rendered
pub struct Renderer {
    scene: Arc<Scene>,
    camera: Arc<Camera>,
    resolution: UVec2,
    num_samples: u64,
    integrator: Integrator,
    tone_map: ToneMap,
    film: Film,
    sampler: SamplerType,
    threads: usize,
    seed: u64,
}

impl Renderer {
    /// Constructs a new renderer.
    pub fn new(scene: Scene, camera: Camera) -> Self {
        assert!(scene.num_lights() != 0);

        let scene = Arc::new(scene);
        let camera = Arc::new(camera);

        let resolution = camera.get_resolution();
        let num_samples = DEFAULT_NUM_SAMPLES;
        let film = Film::new(
            resolution,
            num_samples,
            ColorSpace::default(),
            PixelFilter::default(),
        );

        let seed = crate::rng::gen_seed();

        Self {
            scene,
            camera,
            resolution,
            film,
            num_samples,
            seed,
            threads: DEFAULT_THREADS,
            sampler: SamplerType::default(),
            integrator: Integrator::default(),
            tone_map: ToneMap::default(),
        }
    }

    /// Sets the tone mapping algorithm used
    pub fn tone_map(mut self, tone_map: ToneMap) -> Self {
        self.tone_map = tone_map;
        self
    }

    /// Sets the pixel filter
    pub fn filter(mut self, filter: PixelFilter) -> Self {
        self.film.set_filter(filter);
        self
    }

    /// Sets number of samples per pixel
    pub fn samples(mut self, samples: u64) -> Self {
        self.num_samples = samples;
        self.film.set_samples(samples);
        self
    }

    /// Sets the integrator used to render the image
    pub fn integrator(mut self, integrator: Integrator) -> Self {
        self.integrator = integrator;
        self
    }

    /// Set the seed used for random number generation
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Set the sampler used in rendering
    pub fn sampler(mut self, sampler: SamplerType) -> Self {
        self.sampler = sampler;
        self
    }

    /// Sets the number of threads used for rendering
    pub fn threads(mut self, threads: usize) -> Self {
        self.threads = threads;
        self
    }

    /// Sets the color space used to develop the film
    pub fn color_space(mut self, color_space: ColorSpace) -> Self {
        self.film.set_color_space(color_space);
        self
    }

    fn output_begin(&self) {
        #[cfg(debug_assertions)]
        println!("Debug assertions enabled");

        if matches!(self.integrator, Integrator::BDPathTrace)
            && self.scene.medium.is_some() {
                println!("Volumetric mediums not currently supported with BDPT, \
                          rendering anyways");
            }
        println!("Starting to render the scene:\n\
                  \t Resolution: {} x {}\n\
                  \t Samples: {}\n\
                  \t Integrator: {}\n\
                  \t Sampler: {}\n\
                  \t Tone map: {}\n\
                  \t Film: [{}] \n\
                  \t Seed: {}\n\
                  \t Threads: {}",
                 self.resolution.x, self.resolution.y,
                 self.num_samples,
                 self.integrator,
                 self.sampler,
                 self.tone_map,
                 self.film,
                 self.seed,
                 self.threads,
        );
    }

    fn output_progress(
        &self,
        rays_traced: u64,
        rays_total: u64,
        dt: Float,
    ) {
        let bar_step = rays_total / PROGRESS_BAR_LEN as u64;
        let steps = rays_traced / bar_step;
        let prog = rays_total as Float / rays_traced as Float;
        print!("\r{}", " ".repeat(PROGRESS_BAR_LEN + 32));
        print!("\r[{}{}] (~{} left)",
               "+".repeat(steps as usize),
               "-".repeat(PROGRESS_BAR_LEN - steps as usize),
               formatting::fmt_ms(dt as Float * (prog - 1.0), false),
        );
        let _ = std::io::stdout().flush();
    }

    fn render_executor(&self) -> Arc<RenderTaskExecutor> {
        let integrator = self.integrator.clone();
        let sampler = self.sampler.clone();
        let tone_map = self.tone_map.clone();

        Arc::new(
            move |task, rng, camera, scene| -> RenderTaskResult {
                let Some(mut tile) = task.tile else { return RenderTaskResult::null(); };

                let (mi_y, mx_y) = (tile.px_min.y, tile.px_max.y);
                let (mi_x, mx_x) = (tile.px_min.x, tile.px_max.x);

                (mi_y..mx_y).cartesian_product(mi_x..mx_x)
                    .for_each(|(y, x): (u64, u64)| {
                        let xy = Vec2::new(x as Float, y as Float);
                        sampler.new(task.samples, rng)
                            .flat_map(|rand_sq: Vec2| {
                                let raster_xy = xy + rand_sq;
                                integrator.integrate(scene, camera, rng, raster_xy)
                            })
                            .for_each(|mut sample: FilmSample| {
                                tone_map.map(&mut sample);
                                tile.add_sample(&sample)
                            })
                    });

                let num_rays = (mx_x - mi_x) * (mx_y - mi_y) * task.samples;
                RenderTaskResult::new(tile, num_rays)
            }
        )
    }


    /// Starts the rendering process and returns the rendered image
    pub fn render(&mut self) -> &Film {
        self.output_begin();
        let start = Instant::now();

        let pool = pool::ThreadPool::new(
            self.threads,
            Xorshift::new(self.seed),
            Arc::clone(&self.camera),
            Arc::clone(&self.scene),
            self.render_executor(),
        );

        let tiles_x = self.resolution.x.div_ceil(TILE_SIZE);
        let tiles_y = self.resolution.y.div_ceil(TILE_SIZE);

        let mut samples_taken = 0;
        while samples_taken < self.num_samples {
            let prev = samples_taken;
            samples_taken += SAMPLES_INCREMENT;
            samples_taken = samples_taken.min(self.num_samples);
            let samples = samples_taken - prev;

            (0..tiles_y).cartesian_product(0..tiles_x).for_each(|(y, x): (u64, u64)| {
                let px_min = UVec2::new(x, y) * TILE_SIZE;
                let px_max = (px_min + TILE_SIZE).min(self.resolution);
                let tile = self.film.create_tile(px_min, px_max);

                let task = task::RenderTask::new(tile, samples);
                pool.publish(task);
            });
        }

        pool.all_published();

        let rays_total = self.num_samples
            * self.resolution.x
            * self.resolution.y;
        let mut rays_traced = 0;
        let mut finished = 0;
        let mut tiles_added = 0;
        while finished < self.threads {
            let result = pool.pop_result();
            if let Some(tile) = result.tile {
                self.film.add_tile(tile);
                rays_traced += result.num_rays;
                tiles_added += 1;
                let output = tiles_added % self.resolution.x == 0
                    || tiles_added == self.threads as u64;
                if output {
                    let dt = start.elapsed().as_millis() as Float;
                    self.output_progress(rays_traced, rays_total, dt);
                }
            } else {
                finished += 1;
            }
        }

        println!("\rFinished rendering in {} ({} camera rays)",
                 formatting::fmt_ms(start.elapsed().as_millis() as Float, true),
                 formatting::fmt_si(rays_total),
        );

        &self.film
    }
}
