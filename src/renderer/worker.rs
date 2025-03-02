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
        rng: Xorshift,
        result_tx: mpsc::Sender<RenderTaskResult>,
        task_queue: Arc<Mutex<queue::RenderQueue<RenderTask>>>,
        camera: Arc<Camera>,
        scene: Arc<Scene>,
        exec: Arc<RenderTaskExecutor>,
    ) -> Self {
        let thread = thread::spawn(move || {
            let rng = RefCell::new(rng);
            loop {
                let task = task_queue.lock().unwrap().pop();
                let result = exec(task, &rng, &camera, &scene);

                let was_null = result.is_null();
                result_tx.send(result).unwrap();

                if was_null { break; }
            }
        });
        Self {
            idx, thread,
        }
    }
}
