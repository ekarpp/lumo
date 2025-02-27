use super::*;
use std::thread;

#[allow(dead_code)]
pub struct Worker {
    idx: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        idx: usize,
        task_queue: Arc<Mutex<queue::RenderQueue<task::RenderTask>>>,
        result_tx: mpsc::Sender<result::RenderResult>,
        camera: Arc<Camera>,
        scene: Arc<Scene>,
        integrator: Integrator,
        sampler: SamplerType,
        tone_map: ToneMap,
    ) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let task = task_queue.lock().unwrap().pop();
                let Some(mut tile) = task.tile else { break; };

                let (mi_y, mx_y) = (tile.px_min.y, tile.px_max.y);
                let (mi_x, mx_x) = (tile.px_min.x, tile.px_max.x);

                (mi_y..mx_y).cartesian_product(mi_x..mx_x)
                    .for_each(|(y, x): (i32, i32)| {
                        let xy = Vec2::new(x as Float, y as Float);
                        sampler.new(task.samples)
                            .flat_map(|rand_sq: Vec2| {
                                let raster_xy = xy + rand_sq;
                                integrator.integrate(&scene, &camera, raster_xy)
                            })
                            .for_each(|mut sample: FilmSample| {
                                tone_map.map(&mut sample);
                                tile.add_sample(&sample)
                            })
                    });

                let num_rays = (mx_x - mi_x) * (mx_y - mi_y) * task.samples as i32;
                result_tx.send(result::RenderResult::new(tile, num_rays)).unwrap();
            }
            result_tx.send(result::RenderResult::null()).unwrap();
        });
        Self {
            idx, thread,
        }
    }
}
