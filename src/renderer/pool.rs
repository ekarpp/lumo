use super::*;

pub struct ThreadPool {
    workers: Vec<worker::Worker>,
    task_tx: mpsc::Sender<task::RenderTask>,
    result_queue: queue::RenderQueue<result::RenderResult>,
}

impl ThreadPool {
    pub fn new(
        workers: usize,
        camera: Arc<Camera>,
        scene: Arc<Scene>,
        integrator: Integrator,
        sampler: SamplerType,
        tone_map: ToneMap,
    ) -> Self {
        assert!(workers > 0);

        let (result_queue, result_tx) = queue::RenderQueue::<result::RenderResult>::new();
        let (task_queue, task_tx) = queue::RenderQueue::<task::RenderTask>::new();
        // shared between workers, wrap it in a mutex arc
        let task_queue = Arc::new(Mutex::new(task_queue));

        let workers = (0..workers)
            .map(|i| worker::Worker::new(
                i,
                Arc::clone(&task_queue),
                result_tx.clone(),
                Arc::clone(&camera),
                Arc::clone(&scene),
                integrator.clone(),
                sampler.clone(),
                tone_map.clone(),
            ))
            .collect();

        Self { workers, task_tx, result_queue }
    }

    /// Publish `task` to the pool
    pub fn publish(&self, task: task::RenderTask) {
        self.task_tx.send(task).unwrap();
    }

    /// Inform workers that all tasks are published
    pub fn all_published(&self) {
        for _ in 0..self.workers.len() {
            self.publish(task::RenderTask::null());
        }
    }

    pub fn pop_result(&self) -> result::RenderResult {
        self.result_queue.pop()
    }
}
