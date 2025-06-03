use std::num::NonZero;

/// The default number of threads for the threadpool
const DEFAULT_NUM_THREADS: NonZero<usize> = unsafe { NonZero::new_unchecked(4) }; // TODO: Determine whether new(<num>).unwrap() is better

/// A threadpool
pub struct ThreadPool {}

impl ThreadPool {
    /// Send a function to be excecuted on the threadpool
    pub fn execute<F>(&mut self, func: F)
    where
        F: FnOnce() -> () + Send + 'static,
    {
    }
}
