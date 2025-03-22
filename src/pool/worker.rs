use super::*;
use std::thread;

#[allow(dead_code)]
pub struct Worker {
    idx: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    #[allow(clippy::too_many_arguments)]
    pub fn new<T: Send + Sync + 'static, R: Send + Sync + 'static, E: Executor<T, R>>(
        idx: usize,
        result_tx: mpsc::Sender<Option<R>>,
        task_queue: Arc<Mutex<queue::RenderQueue<Option<T>>>>,
        mut exec: E,
    ) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let task = task_queue.lock().unwrap().pop();
                let Some(task) = task else { result_tx.send(None).unwrap(); break; };
                result_tx.send(Some( exec.exec(task) )).unwrap();
            }
        });
        Self {
            idx, thread,
        }
    }
}
