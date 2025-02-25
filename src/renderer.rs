use crate::{
    Vec2, Float, TracerCli, ToneMap, SamplerType, rand_utils,
};
use crate::tracer::{
    Camera, Film, FilmSample, ColorWavelength,
    Integrator, Scene, PixelFilter, FilmTile, ColorSpace
};
use glam::IVec2;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{io::Write, sync::Mutex, time::Instant};

const TILE_SIZE: i32 = 16;
// should be a square for samplers
const SAMPLES_INCREMENT: u32 = 256;
const PROGRESS_BAR_LEN: usize = 16;

/// Configures the image to be rendered
pub struct Renderer {
    scene: Scene,
    camera: Camera,
    resolution: IVec2,
    num_samples: u32,
    integrator: Integrator,
    tone_map: ToneMap,
    filter: PixelFilter,
    color_space: ColorSpace,
    sampler: SamplerType,
    threads: usize,
}

impl Renderer {
    /// Constructs a new renderer. Defaults to 1000x1000 image with 1 sample
    /// per pixel and path tracing as the integrator. Configured through the CLI
    /// or the setter functions of the struct.
    pub fn new(scene: Scene, camera: Camera) -> Self {
        assert!(scene.num_lights() != 0);

        let cli_args: TracerCli = argh::from_env();
        let resolution = camera.get_resolution();

        Self {
            scene,
            camera,
            resolution,
            threads: cli_args.threads,
            filter: PixelFilter::default(),
            num_samples: cli_args.samples,
            sampler: SamplerType::MultiJittered,
            integrator: cli_args.get_integrator(),
            tone_map: ToneMap::NoMap,
            color_space: ColorSpace::default(),
        }
    }

    /// Sets the tone mapping algorithm used
    pub fn tone_map(mut self, tone_map: ToneMap) -> Self {
        self.tone_map = tone_map;
        self
    }

    /// Sets the pixel filter
    pub fn filter(mut self, filter: PixelFilter) -> Self {
        self.filter = filter;
        self
    }

    /// Sets number of samples per pixel
    pub fn samples(mut self, samples: u32) -> Self {
        self.num_samples = samples;
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
        self.color_space = color_space;
        self
    }

    /// Starts the rendering process and returns the rendered image
    pub fn render(&self) -> Film {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.threads)
            .build()
            .unwrap();

        if matches!(self.integrator, Integrator::BDPathTrace)
            && self.scene.medium.is_some() {
                println!("Volumetric mediums not currently supported with BDPT, \
                          rendering anyways");
            }
        println!("Starting to render the scene:\n\
                  \t Resolution: {} x {}\n\
                  \t Samples: {}\n\
                  \t Integrator: {}\n\
                  \t Filter: {}\n\
                  \t Sampler: {}\n\
                  \t Color space: {}\n\
                  \t Threads: {}",
                 self.resolution.x, self.resolution.y,
                 self.num_samples,
                 self.integrator,
                 self.filter,
                 self.sampler,
                 self.color_space,
                 pool.current_num_threads(),
        );

        let start = Instant::now();
        let rays = self.num_samples as i32 * self.resolution.x * self.resolution.y;

        let mut film = Film::new(
            self.resolution.x,
            self.resolution.y,
            self.num_samples,
            &self.color_space,
            &self.filter,
        );

        let fmt_ms = |ms: Float| -> String {
            let sec = ms / 1e3;
            if sec <= 60.0 {
                format!("{:.0} s", sec)
            } else if sec <= 60.0 * 60.0 {
                format!("{:.1} m", sec / 60.0)
            } else {
                format!("{:.1} h", sec / 3600.0)
            }
        };

        pool.install(|| {
            let mutex_film = Mutex::new(&mut film);

            let tiles_x = (self.resolution.x + TILE_SIZE - 1) / TILE_SIZE;
            let tiles_y = (self.resolution.y + TILE_SIZE - 1) / TILE_SIZE;

            let mutex_progr = Mutex::new(0);

            let mut samples_taken = 0;
            while samples_taken < self.num_samples {
                let prev = samples_taken;
                samples_taken += SAMPLES_INCREMENT;
                samples_taken = samples_taken.min(self.num_samples);
                let samples = samples_taken - prev;

                (0..tiles_y).into_par_iter()
                    .for_each(|y: i32| {
                        (0..tiles_x).for_each(|x: i32| {
                            let px_min = IVec2::new(x, y) * TILE_SIZE;
                            let px_max = px_min + TILE_SIZE;
                            let mut tile = self.get_tile(px_min, px_max);

                            for y in tile.px_min.y..tile.px_max.y {
                                for x in tile.px_min.x..tile.px_max.x {
                                    self.get_samples(&mut tile, samples, x, y)
                                }
                            }

                            mutex_film.lock().unwrap().add_tile(tile);
                            {
                                let mut ray_counter = mutex_progr.lock().unwrap();
                                *ray_counter += samples * (TILE_SIZE as u32).pow(2);
                                let y0 = tiles_x.min(pool.current_num_threads() as i32 + 1);

                                if x == 0 || (y == 0 && x == y0) {
                                    let count = *ray_counter as i32;
                                    let n = count / (rays / PROGRESS_BAR_LEN as i32);
                                    let dt = start.elapsed().as_millis() as i32;
                                    let prog = rays as Float / count as Float;
                                    print!("\r{}", " ".repeat(PROGRESS_BAR_LEN + 32));
                                    print!("\r[{}{}] (~{} left)",
                                           "+".repeat(n as usize),
                                           "-".repeat(PROGRESS_BAR_LEN - n as usize),
                                           fmt_ms(dt as Float * (prog - 1.0)),
                                    );
                                    let _ = std::io::stdout().flush();
                                }
                            }
                        })
                    });
            }
        });

        let fmt_si = |rays: i32| -> String {
            let r = rays as Float;

            if r > 1e9 {
                format!("{:.2} G", r / 1e9)
            } else if r > 1e6 {
                format!("{:.2} M", r / 1e6)
            } else if r > 1e3 {
                format!("{:.2} k", r / 1e3)
            } else {
                format!("{:.2}", r)
            }
        };

        println!("\rFinished rendering in {} ({} camera rays)",
                 fmt_ms(start.elapsed().as_millis() as Float),
                 fmt_si(rays),
        );

        film
    }

    fn get_tile(&self, px_min: IVec2, px_max: IVec2) -> FilmTile {
        FilmTile::new(
            px_min,
            px_max.min(self.resolution),
            self.resolution,
            &self.color_space,
            &self.filter,
        )
    }

    /// Sends `num_samples` rays towards the given pixel and averages the result
    fn get_samples(&self, tile: &mut FilmTile, num_samples: u32, x: i32, y: i32) {
        let xy = Vec2::new(x as Float, y as Float);
        self.sampler.new(num_samples)
            .for_each(|rand_sq: Vec2| {
                let raster_xy = xy + rand_sq;
                let lambda = ColorWavelength::sample(rand_utils::rand_float());
                self.integrator.integrate(
                    &self.scene,
                    &self.camera,
                    lambda,
                    raster_xy,
                    self.camera.generate_ray(raster_xy),
                )
                    .iter_mut().for_each(|sample: &mut FilmSample| {
                        sample.color = self.tone_map.map(sample.color, &sample.lambda);
                        tile.add_sample(sample)
                    })
            })
    }
}
