use crate::{
    Vec2, Float, ToneMap, SamplerType
};
use crate::tracer::{
    Camera, Film, FilmSample,
    Integrator, Scene, PixelFilter, FilmTile, ColorSpace
};
use glam::IVec2;
use std::{io::Write, sync::{Arc, mpsc, Mutex}, time::Instant};
use itertools::Itertools;

const TILE_SIZE: i32 = 16;
// should be a square for samplers
const SAMPLES_INCREMENT: u32 = 256;
const PROGRESS_BAR_LEN: usize = 16;

const DEFAULT_NUM_SAMPLES: u32 = 1;
const DEFAULT_THREADS: usize = 4;

mod result;
mod pool;
mod queue;
mod task;
mod worker;

/// Configures the image to be rendered
pub struct Renderer {
    scene: Arc<Scene>,
    camera: Arc<Camera>,
    resolution: IVec2,
    num_samples: u32,
    integrator: Integrator,
    tone_map: ToneMap,
    film: Film,
    sampler: SamplerType,
    threads: usize,
}

impl Renderer {
    /// Constructs a new renderer. Defaults to 1000x1000 image with 1 sample
    /// per pixel and path tracing as the integrator. Configured through the CLI
    /// or the setter functions of the struct.
    pub fn new(scene: Scene, camera: Camera) -> Self {
        assert!(scene.num_lights() != 0);

        let scene = Arc::new(scene);
        let camera = Arc::new(camera);

        let resolution = camera.get_resolution();
        let num_samples = DEFAULT_NUM_SAMPLES;
        let film = Film::new(
            resolution.x,
            resolution.y,
            num_samples,
            ColorSpace::default(),
            PixelFilter::default(),
        );


        Self {
            scene,
            camera,
            resolution,
            film,
            num_samples,
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
    pub fn samples(mut self, samples: u32) -> Self {
        self.num_samples = samples;
        self.film.set_samples(samples);
        self
    }

    /// Sets the integrator used to render the image
    pub fn integrator(mut self, integrator: Integrator) -> Self {
        self.integrator = integrator;
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
                  \t Film: [{}] \n\
                  \t Threads: {}",
                 self.resolution.x, self.resolution.y,
                 self.num_samples,
                 self.integrator,
                 self.sampler,
                 self.film,
                 self.threads,
        );
    }

    fn output_progress(
        &self,
        rays_traced: i32,
        rays_total: i32,
        dt: Float,
    ) {
        let bar_step = rays_total / PROGRESS_BAR_LEN as i32;
        let steps = rays_traced / bar_step;
        let prog = rays_total as Float / rays_traced as Float;
        print!("\r{}", " ".repeat(PROGRESS_BAR_LEN + 32));
        print!("\r[{}{}] (~{} left)",
               "+".repeat(steps as usize),
               "-".repeat(PROGRESS_BAR_LEN - steps as usize),
               self.fmt_ms(dt as Float * (prog - 1.0), false),
        );
        let _ = std::io::stdout().flush();
    }

    fn fmt_si(&self, val: i32) -> String {
        let val = val as Float;

        if val > 1e9 {
            format!("{:.2} G", val / 1e9)
        } else if val > 1e6 {
            format!("{:.2} M", val / 1e6)
        } else if val > 1e3 {
            format!("{:.2} k", val / 1e3)
        } else {
            format!("{:.2}", val)
        }
    }

    fn fmt_ms(&self, ms: Float, accurate: bool) -> String {
        let sec = ms / 1e3;
        if sec <= 60.0 {
            if accurate {
                format!("{:.3} s", sec)
            } else {
                format!("{:.0} s", sec)
            }
        } else if sec <= 60.0 * 60.0 {
            format!("{:.1} m", sec / 60.0)
        } else {
            format!("{:.1} h", sec / 3600.0)
        }
    }


    /// Starts the rendering process and returns the rendered image
    pub fn render(&mut self) -> &Film {
        self.output_begin();
        let start = Instant::now();

        let pool = pool::ThreadPool::new(
            self.threads,
            Arc::clone(&self.camera),
            Arc::clone(&self.scene),
            self.integrator.clone(),
            self.sampler.clone(),
            self.tone_map.clone(),
        );

        let tiles_x = (self.resolution.x + TILE_SIZE - 1) / TILE_SIZE;
        let tiles_y = (self.resolution.y + TILE_SIZE - 1) / TILE_SIZE;

        let mut samples_taken = 0;
        while samples_taken < self.num_samples {
            let prev = samples_taken;
            samples_taken += SAMPLES_INCREMENT;
            samples_taken = samples_taken.min(self.num_samples);
            let samples = samples_taken - prev;

            (0..tiles_y).cartesian_product(0..tiles_x).for_each(|(y, x): (i32, i32)| {
                let px_min = IVec2::new(x, y) * TILE_SIZE;
                let px_max = (px_min + TILE_SIZE).min(self.resolution);
                let tile = self.film.create_tile(px_min, px_max);

                let task = task::RenderTask::new(tile, samples);
                pool.publish(task);
            });
        }

        pool.all_published();

        let rays_total = self.num_samples as i32 * self.resolution.x * self.resolution.y;
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
                    || tiles_added == self.threads as i32;
                if output {
                    let dt = start.elapsed().as_millis() as Float;
                    self.output_progress(rays_traced, rays_total, dt);
                }
            } else {
                finished += 1;
            }
        }

        println!("\rFinished rendering in {} ({} camera rays)",
                 self.fmt_ms(start.elapsed().as_millis() as Float, true),
                 self.fmt_si(rays_total),
        );

        &self.film
    }
}
