use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

pub struct Sender<T> {
    inner: Arc<Inner<T>>,
}

//implement clone for Sender
impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Sender {
            //clone the Arc not the thing inside the arc
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> Sender<T> {
    pub fn send(&mut self, t: T) {
        //take the lock
        let mut queue = self.inner.queue.lock().unwrap();
        //add to the queue
        queue.push_back(t);
        //let go of the lock as it needs to be passed to any
        //receivers
        drop(queue);
        //notify waiting receivers when it sends
        self.inner.available.notify_one();
    }
}

pub struct Receiver<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> T {
        //take the lock
        let mut queue = self.inner.queue.lock().unwrap();
        loop {
            //take from the queue
            //use condvar to implement blocking
            match queue.pop_front() {
                Some(t) => return t,
                None => {
                    //sleep the thread until we need to recieve
                    //hand over the lock since we have been woken
                    queue = self.inner.available.wait(queue).unwrap();
                }
            }
        }
    }
}
//shared items in the channel
//basically a queue
// where the sender puts data and
//the receiver takes data
struct Inner<T> {
    //use a VecDeque to enable use to push to the back
    // and pop from the front
    queue: Mutex<VecDeque<T>>,
    // is outside the mutex to avoid deadlocks
    available: Condvar,
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        queue: Mutex::default(),
        available: Condvar::new(),
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ping_pong() {
        let (mut tx, mut rx) = channel();
        tx.send(43);
        assert_eq!(rx.recv(), 43);
    }
}
