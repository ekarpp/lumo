use super::*;

#[derive(Clone)]
pub struct RenderTaskExecutor {
    camera: Arc<Camera>,
    scene: Arc<Scene>,
    sampler: SamplerType,
    integrator: Integrator,
    tone_map: ToneMap,
}

impl RenderTaskExecutor {
    pub fn new(
        camera: Arc<Camera>,
        scene: Arc<Scene>,
        sampler: SamplerType,
        integrator: Integrator,
        tone_map: ToneMap,
    ) -> Self {
        Self { camera, scene, sampler, integrator, tone_map }
    }
}

impl Executor<RenderTask, RenderTaskResult> for RenderTaskExecutor {
    fn exec(&mut self, task: RenderTask) -> RenderTaskResult {
        let mut tile = task.tile;
        let mut rng = Xorshift::new(task.seed);
        let mut ns = [0; SAMPLES_INCREMENT as usize];
        let mut fs = [0.0; SAMPLES_INCREMENT as usize];
        let mut ptr = 0;
        let mut num_rays = 0;

        let (mi_y, mx_y) = (tile.px_min.y, tile.px_max.y);
        let (mi_x, mx_x) = (tile.px_min.x, tile.px_max.x);

        (mi_y..mx_y).cartesian_product(mi_x..mx_x)
            .for_each(|(y, x): (u64, u64)| {
                let xy = Vec2::new(x as Float, y as Float);
                self.sampler.new(task.batch, task.total_samples, rng.gen_u64())
                    .flat_map(|rand_sq: Vec2| {
                        let raster_xy = xy + rand_sq;
                        let f = (0..task.samples)
                            .fold(0.0, |acc, i| acc + fs[i as usize]);
                        let f2 = (0..task.samples)
                            .fold(0.0, |acc, i| acc + fs[i as usize] * fs[i as usize]);
                        let var = f2 - f * f / task.samples as Float;
                        let delta = if var <= 0.0 {
                            1e-5
                        } else {
                            let cost = (0..task.samples)
                                .fold(0, |acc, i| acc + ns[i as usize]);
                            (var / cost as Float).sqrt()
                        };

                        let samples = self.integrator.integrate(
                            &self.scene,
                            &self.camera,
                            &mut rng,
                            delta,
                            raster_xy
                        );

                        // main sample stored in last position, BDPT splats for RR?
                        let sample = &samples[samples.len() - 1];
                        num_rays += sample.cost;
                        ns[ptr] = sample.cost;
                        fs[ptr] = sample.color.luminance(&sample.lambda);
                        ptr += 1;
                        ptr %= task.samples as usize;

                        samples
                    })
                    .for_each(|mut sample: FilmSample| {
                        sample.color = self.tone_map.map(&sample);
                        tile.add_sample(&sample)
                    })
            });

        let num_camera_rays = (mx_x - mi_x) * (mx_y - mi_y) * task.samples;
        RenderTaskResult::new(tile, num_camera_rays, num_rays as u64)
    }
}

pub struct RenderTask {
    pub tile: FilmTile,
    pub batch: u64,
    pub samples: u64,
    pub total_samples: u64,
    pub seed: u64,
}

impl RenderTask {
    pub fn new(
        tile: FilmTile,
        batch: u64,
        samples: u64,
        total_samples: u64,
        seed: u64
    ) -> Self {
        Self { tile, batch, samples, total_samples, seed }
    }
}

pub struct RenderTaskResult {
    pub tile: FilmTile,
    pub num_camera_rays: u64,
    pub num_rays: u64,
}

impl RenderTaskResult {
    pub fn new(tile: FilmTile, num_camera_rays: u64, num_rays: u64) -> Self {
        Self { tile: tile, num_rays, num_camera_rays }
    }
}
