use std::sync::mpsc;

pub struct RenderQueue<T> {
    rx: mpsc::Receiver<T>,
}

impl<T> RenderQueue<T> {
    pub fn new() -> (Self, mpsc::Sender<T>) {
        let (tx, rx) = mpsc::channel();
        (Self { rx }, tx)
    }

    pub fn pop(&self) -> T {
        self.rx.recv().unwrap()
    }
}
