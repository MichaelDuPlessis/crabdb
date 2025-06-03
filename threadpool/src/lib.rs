use std::{
    collections::VecDeque,
    num::NonZero,
    sync::{Arc, Condvar, Mutex},
    thread,
};

/// The default number of threads for the threadpool
const DEFAULT_NUM_THREADS: NonZero<usize> = unsafe { NonZero::new_unchecked(4) }; // TODO: Determine whether new(<num>).unwrap() is better

/// A simple implementation of a fifo queue
struct Queue<T> {
    /// The underlying data structure used for the queue
    container: VecDeque<T>, // TODO: Is a VecDeque the best container to use?
}

impl<T> Queue<T> {
    /// Create a new empty queue
    fn new() -> Self {
        Self {
            container: VecDeque::new(),
        }
    }

    /// Add an element to the queue
    fn enqueue(&mut self, item: T) {
        self.container.push_back(item);
    }

    /// Remove an element from the queue
    fn dequeue(&mut self) -> Option<T> {
        self.container.pop_front()
    }
}

/// A worker represents a thread
struct Worker {
    /// Used to signal to the thread that is should wake up
    job_signal: Arc<(Mutex<()>, Condvar)>,
    /// The thread that the worker manages
    thread: thread::JoinHandle<()>,
}

impl Worker {
    /// Creates a new worker
    fn new(signal: Arc<(Mutex<()>, Condvar)>) -> Self {
        let thread = thread::spawn(|| {});

        Self {
            job_signal: signal,
            thread,
        }
    }
}

/// A threadpool
pub struct ThreadPool {
    /// Used to single to the threads that there is new code to be executed
    job_signal: Arc<(Mutex<()>, Condvar)>,
    /// The workers belonging to the threadpool
    workers: Vec<Worker>,
}

impl ThreadPool {
    /// Create a new threadpool with the specified number of threads
    pub fn new(num_threads: usize) -> Self {
        let job_signal = Arc::new((Mutex::new(()), Condvar::new()));
        let mut workers = Vec::with_capacity(num_threads);

        // creating the worker threads
        for _ in 0..num_threads {
            let job_signal = Arc::clone(&job_signal);
            workers.push(Worker::new(job_signal));
        }

        Self {
            job_signal,
            workers,
        }
    }

    /// Send a function to be excecuted on the threadpool
    pub fn execute<F>(&mut self, func: F)
    where
        F: FnOnce() -> () + Send + 'static,
    {
    }
}
