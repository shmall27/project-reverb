use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

pub struct ThreadSafeQueue<T> {
    queue: Mutex<VecDeque<T>>,
    available: Condvar,
    max_size: usize,
}

impl<T> ThreadSafeQueue<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            available: Condvar::new(),
            max_size,
        }
    }

    pub fn push(&self, value: T) {
        let mut queue = self.queue.lock().unwrap();
        if queue.len() >= self.max_size {
            // Drop the oldest element to make room for the new one.
            queue.pop_front();
        }
        queue.push_back(value);
        self.available.notify_one();
    }

    pub fn pop(&self) -> T {
        let mut queue = self.queue.lock().unwrap();
        // wait until there is a value available
        while queue.is_empty() {
            queue = self.available.wait(queue).unwrap();
        }
        queue.pop_front().unwrap()
    }
}

