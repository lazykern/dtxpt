use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, TryRecvError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundPoll<T> {
    Pending,
    Ready(T),
    Disconnected,
}

#[derive(Debug)]
pub struct BackgroundTask<T> {
    pub active: bool,
    receiver: Option<Mutex<Receiver<T>>>,
}

impl<T> Default for BackgroundTask<T> {
    fn default() -> Self {
        Self {
            active: false,
            receiver: None,
        }
    }
}

impl<T: Send + 'static> BackgroundTask<T> {
    pub fn start<F>(&mut self, work: F)
    where
        F: FnOnce() -> T + Send + 'static,
    {
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = sender.send(work());
        });
        self.active = true;
        self.receiver = Some(Mutex::new(receiver));
    }

    pub fn poll(&mut self) -> BackgroundPoll<T> {
        let received = match self.receiver.as_ref() {
            None => return BackgroundPoll::Pending,
            Some(mutex) => mutex.lock().unwrap().try_recv(),
        };
        match received {
            Ok(value) => {
                self.active = false;
                self.receiver = None;
                BackgroundPoll::Ready(value)
            }
            Err(TryRecvError::Empty) => BackgroundPoll::Pending,
            Err(TryRecvError::Disconnected) => {
                self.active = false;
                self.receiver = None;
                BackgroundPoll::Disconnected
            }
        }
    }

    pub fn reset(&mut self) {
        self.active = false;
        self.receiver = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poll_returns_pending_while_waiting() {
        let (tx, rx) = mpsc::channel::<i32>();
        let mut task = BackgroundTask {
            active: true,
            receiver: Some(Mutex::new(rx)),
        };
        assert_eq!(task.poll(), BackgroundPoll::Pending);
        tx.send(42).unwrap();
        assert_eq!(task.poll(), BackgroundPoll::Ready(42));
        assert!(!task.active);
    }

    #[test]
    fn reset_clears_receiver() {
        let mut task = BackgroundTask::<i32>::default();
        task.start(|| 1);
        task.reset();
        assert!(!task.active);
        assert_eq!(task.poll(), BackgroundPoll::Pending);
    }
}
