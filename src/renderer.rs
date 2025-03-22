use crate::{
    formatting, rng::Xorshift, Vec2, Float, ToneMap, SamplerType
};
use crate::tracer::{
    Camera, Film, FilmSample,
    Integrator, Scene, FilmTile
};
use crate::pool::{Executor, ThreadPool};
use crate::math::vec2::UVec2;
use std::{
    sync::Arc, io::Write, time::{Duration, Instant}
};
use itertools::Itertools;

const TILE_SIZE: u64 = 16;
// should be a square for samplers
pub const SAMPLES_INCREMENT: u64 = 256;
const PROGRESS_BAR_LEN: usize = 16;

const DEFAULT_NUM_SAMPLES: u64 = 1;
const DEFAULT_THREADS: usize = 4;

mod task;

/// Configures the image to be rendered
pub struct Renderer {
    scene: Arc<Scene>,
    camera: Arc<Camera>,
    resolution: UVec2,
    num_samples: u64,
    integrator: Integrator,
    tone_map: ToneMap,
    sampler: SamplerType,
    threads: usize,
    seed: u64,
}

impl Renderer {
    /// Constructs a new renderer.
    pub fn new(mut scene: Scene, camera: Camera) -> Self {
        scene.build();
        assert!(scene.num_lights() != 0);

        let scene = Arc::new(scene);
        let camera = Arc::new(camera);

        let resolution = camera.get_resolution();
        let num_samples = DEFAULT_NUM_SAMPLES;

        let seed = crate::rng::gen_seed();

        Self {
            scene,
            camera,
            resolution,
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

    /// Sets number of samples per pixel
    pub fn samples(mut self, samples: u64) -> Self {
        self.num_samples = samples;
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

    fn output_begin(&self, film: &Film) {
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
                  \t Shadow rays: {}\n\
                  \t Integrator: {}\n\
                  \t Primitives: {}\n\
                  \t Lights: {}\n\
                  \t Sampler: {}\n\
                  \t Tone map: {}\n\
                  \t Film: [{}] \n\
                  \t Seed: {}\n\
                  \t Threads: {}",
                 self.resolution.x, self.resolution.y,
                 self.num_samples,
                 if matches!(self.integrator, Integrator::BDPathTrace) {
                     1
                 } else {
                     self.scene.num_shadow_rays()
                 },
                 self.integrator,
                 self.scene.num_primitives(),
                 self.scene.num_lights(),
                 self.sampler,
                 self.tone_map,
                 film,
                 self.seed,
                 self.threads,
        );
    }

    fn output_progress(
        &self,
        rays_traced: u64,
        rays_total: u64,
        dt: Duration,
    ) {
        let bar_step = rays_total / PROGRESS_BAR_LEN as u64;
        let steps = rays_traced / bar_step;
        let prog = rays_total as Float / rays_traced as Float;
        print!("\r{}", " ".repeat(PROGRESS_BAR_LEN + 32));
        print!("\r[{}{}] (~{} left)",
               "+".repeat(steps as usize),
               "-".repeat(PROGRESS_BAR_LEN - steps as usize),
               formatting::fmt_elapsed(dt.mul_f64(prog as f64 - 1.0)),
        );
        let _ = std::io::stdout().flush();
    }

    /// Starts the rendering process and returns the rendered image
    pub fn render(&mut self) -> Film {
        let start = Instant::now();

        let mut film = self.camera.create_film(self.num_samples);
        self.output_begin(&film);

        let mut rng = Xorshift::new(self.seed);
        let executor = task::RenderTaskExecutor::new(
            Arc::clone(&self.camera),
            Arc::clone(&self.scene),
            self.sampler.clone(),
            self.integrator.clone(),
            self.tone_map.clone(),
        );

        let pool = ThreadPool::new(
            self.threads,
            executor,
        );

        let tiles_x = self.resolution.x.div_ceil(TILE_SIZE);
        let tiles_y = self.resolution.y.div_ceil(TILE_SIZE);

        let mut samples_taken = 0;
        while samples_taken < self.num_samples {
            let prev = samples_taken;
            let batch = samples_taken / SAMPLES_INCREMENT;
            samples_taken += SAMPLES_INCREMENT;
            samples_taken = samples_taken.min(self.num_samples);
            let samples = samples_taken - prev;

            (0..tiles_y).cartesian_product(0..tiles_x).for_each(|(y, x): (u64, u64)| {
                let px_min = UVec2::new(x, y) * TILE_SIZE;
                let px_max = (px_min + TILE_SIZE).min(self.resolution);
                let tile = film.create_tile(px_min, px_max);

                let task = task::RenderTask::new(
                    tile,
                    batch,
                    samples,
                    self.num_samples,
                    rng.gen_u64()
                );
                pool.publish(task);
            });
        }

        pool.all_published();

        let camera_rays_total = self.num_samples
            * self.resolution.x
            * self.resolution.y;
        let mut camera_rays_traced = 0;
        let mut rays_total = 0;
        let mut finished = 0;
        let mut tiles_added = 0;
        while finished < self.threads {
            let result = pool.pop_result();
            if let Some(result) = result {
                let tile = result.tile;
                film.add_tile(tile);
                camera_rays_traced += result.num_camera_rays;
                rays_total += result.num_rays;
                tiles_added += 1;
                let output = tiles_added % self.resolution.x == 0
                    || tiles_added == self.threads as u64;
                if output {
                    self.output_progress(
                        camera_rays_traced,
                        camera_rays_total,
                        start.elapsed()
                    );
                }
            } else {
                finished += 1;
            }
        }

        println!("\rFinished rendering in {} ({} camera rays, {} total rays)",
                 formatting::fmt_elapsed(start.elapsed()),
                 formatting::fmt_si(camera_rays_total),
                 formatting::fmt_si(rays_total)
        );

        film
    }
}
