use std::sync::{Arc, Condvar, Mutex};
use std::collections::VecDeque

pub struct Sender<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Sender<T> {
    pub fn send(&mut self, t: T) {
        //take the lock
        let queue = self.inner.queue.lock().unwrap();
        //add to the queue
        queue.push_back(t);
    }
}

pub struct Receiver<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> T {
        //take the lock
        let queue = self.inner.queue.lock().unwrap();
        //take fromto the queue
        queue.pop_front();
    }
}
//shared items in the channel
//basically a queue
// where the sender puts data and
//the receiver takes data
struct Inner<T> {
    queue: Mutex<VecDeque<T>>,
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        queue: Mutex::default(),
    };

    let inner = Arc::new(inner);
    (
        Sender {
            inner: inner.clone(),
        },
        Receiver {
            inner: inner.clone(),
        },
    )
}
