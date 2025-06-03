use std::{
    collections::VecDeque,
    num::NonZero,
    sync::{Arc, Condvar, Mutex},
    thread,
};

/// The default number of threads for the threadpool
const DEFAULT_NUM_THREADS: NonZero<usize> = unsafe { NonZero::new_unchecked(4) }; // TODO: Determine whether new(<num>).unwrap() is better

/// The kinds of signals that can be sent to threads
enum Signal {
    /// There is a job to process
    Job(Box<dyn FnOnce() -> () + Send + 'static>),
    /// The thread should shutdown
    Shutdown,
}

/// A type alias for the object used for signaling the threads
type JobSignal = Arc<(Mutex<Queue<Signal>>, Condvar)>;

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

    /// Returns the number of items in the queue
    fn len(&self) -> usize {
        self.container.len()
    }

    /// Returns true if the queue has no elements otherwise false
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A worker represents a thread
struct Worker {
    /// The thread that the worker manages
    thread: thread::JoinHandle<()>,
}

impl Worker {
    /// Creates a new worker
    fn new(job_signal: JobSignal) -> Self {
        let thread = thread::spawn(move || {
            loop {
                // getting the job
                let signal = {
                    // aquiring lock
                    let (lock, cvar) = &*job_signal;

                    // aquiring queue
                    let mut queue = lock.lock().unwrap();

                    // making sure there is a job to execute
                    while queue.is_empty() {
                        queue = cvar.wait(queue).unwrap();
                    }

                    // getting the job
                    let signal = queue.dequeue();
                    signal
                };

                if let Some(signal) = signal {
                    match signal {
                        // run the job
                        Signal::Job(job) => job(),
                        // shutdown the thread
                        Signal::Shutdown => break,
                    }
                }
            }
        });

        Self { thread }
    }

    /// Wait for the threaed to finish executing
    fn join(self) {
        self.thread.join().unwrap();
    }
}

/// A threadpool
pub struct ThreadPool {
    /// Used to single to the threads that there is new code to be executed
    job_signal: JobSignal,
    /// The workers belonging to the threadpool
    workers: Vec<Worker>,
}

impl ThreadPool {
    /// Create a new threadpool with the specified number of threads
    pub fn new(num_threads: NonZero<usize>) -> Self {
        let num_threads = num_threads.get();

        let queue = Queue::new();
        let job_signal = Arc::new((Mutex::new(queue), Condvar::new()));
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

    /// sends a signal to the worker threads
    fn send_signal(&mut self, signal: Signal) {
        let (lock, cvar) = &*self.job_signal;
        // adding job to queue
        let mut queue = lock.lock().unwrap();
        queue.enqueue(signal);
        // unlocking mutex
        drop(queue);

        // notifying a thread
        cvar.notify_one();
    }

    /// Send a function to be excecuted on the threadpool
    pub fn execute<F>(&mut self, func: F)
    where
        F: FnOnce() -> () + Send + 'static,
    {
        let job = Box::new(func);
        self.send_signal(Signal::Job(job));
    }

    /// Wait for all threads for finish executing
    pub fn join(mut self) {
        // sending signals equal to the number of workers to shutdown
        for _ in 0..self.workers.len() {
            self.send_signal(Signal::Shutdown);
        }

        // waiting for workers to finish
        for worker in self.workers {
            worker.join();
        }
    }
}

impl Default for ThreadPool {
    fn default() -> Self {
        Self::new(DEFAULT_NUM_THREADS)
    }
}
