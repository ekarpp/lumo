use std::sync::{Arc, mpsc, Mutex};

mod queue;
mod worker;

pub trait Executor<T, R>: Clone + Send + Sync + 'static {
    fn exec(&mut self, task: T) -> R;
}

pub struct ThreadPool<T, R> {
    workers: Vec<worker::Worker>,
    task_tx: mpsc::Sender<Option<T>>,
    result_queue: queue::RenderQueue<Option<R>>,
}

impl<T: Send + Sync + 'static, R: Send + Sync + 'static> ThreadPool<T, R> {
    pub fn new<E: Executor<T, R>>(
        workers: usize,
        executor: E,
    ) -> Self {
        assert!(workers > 0);

        let (result_queue, result_tx) = queue::RenderQueue::<Option<R>>::new();
        let (task_queue, task_tx) = queue::RenderQueue::<Option<T>>::new();
        // shared between workers, wrap it in a mutex arc
        let task_queue = Arc::new(Mutex::new(task_queue));

        let workers = (0..workers)
            .map(|i| worker::Worker::new(
                i,
                result_tx.clone(),
                Arc::clone(&task_queue),
                executor.clone(),
            ))
            .collect();

        Self { workers, task_tx, result_queue }
    }

    /// Publish `task` to the pool
    pub fn publish(&self, task: T) {
        self.task_tx.send(Some(task)).unwrap();
    }

    /// Inform workers that all tasks are published
    pub fn all_published(&self) {
        for _ in 0..self.workers.len() {
            self.task_tx.send(None).unwrap()
        }
    }

    pub fn pop_result(&self) -> Option<R> {
        self.result_queue.pop()
    }
}
