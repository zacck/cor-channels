use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

//implement clone for Sender
impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        //increase number of senders
        let mut inner = self.shared.inner.lock().unwrap();
        inner.senders += 1;
        drop(inner);
        Sender {
            //clone the Arc not the thing inside the arc
            shared: Arc::clone(&self.shared),
        }
    }
}

//descrease count of senders when one goes out
impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.senders -= 1;
        let was_last = inner.senders == 0;
        drop(inner);
        //if we are the last sender lets announce that we are gone
        if was_last {
            self.shared.available.notify_one()
        }
    }
}

impl<T> Sender<T> {
    pub fn send(&mut self, t: T) {
        //take the lock
        let mut inner = self.shared.inner.lock().unwrap();
        //add to the queue
        inner.queue.push_back(t);
        //let go of the lock as it needs to be passed to any
        //receivers
        drop(inner);
        //notify waiting receivers when it sends
        self.shared.available.notify_one();
    }
}

pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> Option<T> {
        //take the lock
        let mut inner = self.shared.inner.lock().unwrap();
        loop {
            //take from the queue
            //use condvar to implement blocking
            match inner.queue.pop_front() {
                Some(t) => return Some(t),
                None if inner.senders == 0 => return None,
                None => {
                    //sleep the thread until we need to recieve
                    //hand over the lock since we have been woken
                    inner = self.shared.available.wait(inner).unwrap();
                }
            }
        }
    }
}
//shared items in the channel
//basically a queue
// where the sender puts data and
//the receiver takes data
struct Shared<T> {
    inner: Mutex<Inner<T>>,
    // is outside the mutex to avoid deadlocks
    available: Condvar,
}

//so we can guard the q and the number of senders
//so we can notify the receiver when the channel is closed
//by detection no more senders
struct Inner<T> {
    //use a VecDeque to enable use to push to the back
    // and pop from the front
    queue: VecDeque<T>,
    senders: usize,
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        queue: VecDeque::default(),
        senders: 1,
    };
    let shared = Shared {
        inner: Mutex::new(inner),
        available: Condvar::new(),
    };

    let shared = Arc::new(shared);
    (
        Sender {
            shared: shared.clone(),
        },
        Receiver {
            shared: shared.clone(),
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
        assert_eq!(rx.recv().unwrap(), 43);
    }

    #[test]
    fn closed() {
        let (tx, mut rx) = channel::<()>();
        drop(tx);
        assert_eq!(rx.recv(), None);
    }
}
