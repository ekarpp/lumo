use super::*;

pub struct ThreadPool {
    workers: Vec<worker::Worker>,
    task_tx: mpsc::Sender<RenderTask>,
    result_queue: queue::RenderQueue<RenderTaskResult>,
}

impl ThreadPool {
    pub fn new(
        workers: usize,
        mut rng: Xorshift,
        camera: Arc<Camera>,
        scene: Arc<Scene>,
        exec: Arc<RenderTaskExecutor>,
    ) -> Self {
        assert!(workers > 0);

        let (result_queue, result_tx) = queue::RenderQueue::<RenderTaskResult>::new();
        let (task_queue, task_tx) = queue::RenderQueue::<RenderTask>::new();
        // shared between workers, wrap it in a mutex arc
        let task_queue = Arc::new(Mutex::new(task_queue));

        let workers = (0..workers)
            .map(|i| worker::Worker::new(
                i,
                Xorshift::new(rng.gen_u64()),
                result_tx.clone(),
                Arc::clone(&task_queue),
                Arc::clone(&camera),
                Arc::clone(&scene),
                Arc::clone(&exec),
            ))
            .collect();

        Self { workers, task_tx, result_queue }
    }

    /// Publish `task` to the pool
    pub fn publish(&self, task: RenderTask) {
        self.task_tx.send(task).unwrap();
    }

    /// Inform workers that all tasks are published
    pub fn all_published(&self) {
        for _ in 0..self.workers.len() {
            self.publish(RenderTask::null());
        }
    }

    pub fn pop_result(&self) -> RenderTaskResult {
        self.result_queue.pop()
    }
}
