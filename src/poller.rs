use std::sync::{Mutex, mpsc};
use std::time;
use std::thread;

enum Command {
    Stop,
}

/// Helper for launching poll operations in a separate thread.
pub struct Poller {
    handle: Option<thread::JoinHandle<()>>,
    handle_tx: Mutex<mpsc::Sender<Command>>,
}

impl Poller {
    pub fn new<F>(interval: time::Duration, f: F) -> Self
    where
        F: Fn() -> (),
        F: Send + 'static,
    {
        let (tx, rx) = mpsc::channel::<Command>();
        let handle = thread::spawn(move || Self::poll(interval, rx, f));
        Poller {
            handle: Some(handle),
            handle_tx: Mutex::new(tx),
        }
    }

    fn poll<F>(interval: time::Duration, control_channel: mpsc::Receiver<Command>, f: F)
    where
        F: Fn() -> (),
        F: Send + 'static,
    {
        loop {
            f();
            match control_channel.recv_timeout(interval) {
                Result::Ok(Command::Stop) => return,
                Result::Err(_) => (),
            }
        }
    }
}

impl Drop for Poller {
    fn drop(&mut self) {
        if let Some(join_handler) = self.handle.take() {
            if let Result::Ok(handle_tx) = self.handle_tx.lock() {
                if (*handle_tx).send(Command::Stop).is_ok() {
                    let _ = join_handler.join();
                }
            }
        }
    }
}
