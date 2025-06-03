use std::{num::NonZero, thread};

/// The default number of threads for the threadpool
const DEFAULT_NUM_THREADS: NonZero<usize> = unsafe { NonZero::new_unchecked(4) }; // TODO: Determine whether new(<num>).unwrap() is better

/// A worker represents a thread
struct Worker {
    /// The thread that the worker manages
    thread: thread::JoinHandle<()>,
}

impl Worker {
    /// Creates a new worker
    fn new() -> Self {
        let thread = thread::spawn(|| {});

        Self { thread }
    }
}

/// A threadpool
pub struct ThreadPool {
    /// The workers belonging to the threadpool
    workers: Vec<Worker>,
}

impl ThreadPool {
    /// Create a new threadpool with the specified number of threads
    pub fn new(num_threads: usize) -> Self {
        // creating the worker threads
        let mut workers = Vec::with_capacity(num_threads);

        for _ in 0..num_threads {
            workers.push(Worker::new());
        }

        Self { workers }
    }

    /// Send a function to be excecuted on the threadpool
    pub fn execute<F>(&mut self, func: F)
    where
        F: FnOnce() -> () + Send + 'static,
    {
    }
}
