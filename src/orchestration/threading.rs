//! # Threading Module
//!
//! Provides threading utilities for managing concurrent tasks in the Gooty Proxy system.
//!
//! ## Overview
//!
//! This module includes abstractions and helpers for:
//! - Spawning and managing threads
//! - Synchronizing shared data between threads
//! - Handling thread-safe operations
//!
//! ## Examples
//!
//! ```
//! use gooty_proxy::orchestration::threading;
//!
//! // Example of spawning a thread
//! let handle = threading::spawn(|| {
//!     println!("Thread running");
//! });
//! handle.join().unwrap();
//! ```

/// Provides threading utilities for orchestration.
///
/// This module contains helper functions and abstractions for managing
/// threads in the orchestration layer of the application.
///
/// # Examples
///
/// ```
/// use gooty_proxy::orchestration::threading;
///
/// threading::spawn_worker(|| {
///     println!("Worker thread running");
/// });
/// ```
use futures::{StreamExt, stream};
use std::future::Future;
use std::pin::Pin;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// Manages a collection of task handles for concurrent execution
#[derive(Default)]
pub struct TaskManager {
    tasks: Vec<JoinHandle<()>>,
}

impl TaskManager {
    /// Create a new task manager
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawn a new task and add it to the managed set
    pub fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let handle = tokio::spawn(future);
        self.tasks.push(handle);
    }

    /// Wait for all tasks to complete
    pub async fn join_all(&mut self) {
        while let Some(task) = self.tasks.pop() {
            let _ = task.await;
        }
    }

    /// Cancel all running tasks
    pub fn cancel_all(&mut self) {
        for task in self.tasks.drain(..) {
            task.abort();
        }
    }
}

/// Creates a set of worker tasks with a bounded channel for work distribution
pub fn create_worker_pool<T, F, Fut>(
    concurrency: usize,
    worker_fn: F,
) -> (mpsc::Sender<T>, TaskManager)
where
    T: Send + 'static,
    F: FnMut(T) -> Fut + Send + Clone + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let (tx, rx) = mpsc::channel::<T>(concurrency);
    let rx = std::sync::Arc::new(tokio::sync::Mutex::new(rx));

    let mut task_manager = TaskManager::new();

    for _ in 0..concurrency {
        let mut worker_fn = worker_fn.clone();
        let rx = rx.clone();

        task_manager.spawn(async move {
            loop {
                let message = {
                    let mut rx_lock = rx.lock().await;
                    rx_lock.recv().await
                };

                match message {
                    Some(item) => {
                        worker_fn(item).await;
                    }
                    None => break,
                }
            }
        });
    }

    (tx, task_manager)
}

/// Execute multiple futures concurrently with a limit on parallelism
///
/// # Panics
///
/// This function will panic if the semaphore is closed, which can happen
/// if the semaphore is dropped while permits are still active.
pub async fn execute_with_concurrency_limit<T, F, Fut>(
    items: Vec<T>,
    concurrency: usize,
    mut job_fn: F,
) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>>
where
    T: Send + 'static,
    F: FnMut(T) -> Fut + Send,
    Fut: Future<Output = ()> + Send + 'static,
{
    let mut futures = Vec::new();
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(concurrency));

    for item in items {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let future = job_fn(item);

        futures.push(Box::pin(async move {
            future.await;
            drop(permit);
        }) as Pin<Box<dyn Future<Output = ()> + Send>>);
    }

    futures
}

/// Run a batch of operations concurrently with limited parallelism.
///
/// This function takes a collection of items, a concurrency limit, and a job function.
/// It processes the items concurrently but limited to the specified level of parallelism,
/// returning the results when all operations are complete.
///
/// # Type Parameters
///
/// * `T` - The input item type
/// * `R` - The result type
/// * `F` - The function type that processes each item
/// * `Fut` - The future type returned by the function
///
/// # Arguments
///
/// * `items` - Vector of items to process
/// * `concurrency` - Maximum number of concurrent operations
/// * `job_fn` - Function that processes each item and returns a future
///
/// # Returns
///
/// A vector containing the results of all operations in the same order as the input items.
///
/// # Examples
///
/// ```
/// async fn process_item(item: u32) -> u32 {
///     // Some async processing
///     item * 2
/// }
///
/// let items = vec![1, 2, 3, 4, 5];
/// let concurrency = 2;
/// let results = run_concurrent_batch(items, concurrency, |item| async move {
///     process_item(item).await
/// }).await;
/// ```
pub async fn run_concurrent_batch<T, R, F>(
    items: Vec<T>,
    concurrency: usize,
    job_fn: &F,
) -> Vec<(R, bool)>
where
    T: Send + 'static,
    R: Send + 'static,
    F: Fn(T) -> Pin<Box<dyn Future<Output = (R, bool)> + Send>> + Send + Sync + Clone + 'static,
{
    // Create a buffered stream with the specified concurrency
    stream::iter(items)
        .map(|item| {
            let job = job_fn.clone();
            async move { job(item).await }
        })
        .buffer_unordered(concurrency.max(1)) // Ensure at least 1 concurrency
        .collect::<Vec<_>>()
        .await
}

/// Process items concurrently with a shared state
///
/// Similar to `run_concurrent_batch`, but allows for a shared state that
/// can be accessed and modified by each job.
///
/// # Type Parameters
///
/// * `T` - The type of items to process
/// * `R` - The result type returned by the job function
/// * `S` - The shared state type
/// * `F` - The job function type
///
/// # Arguments
///
/// * `items` - Vector of items to process
/// * `state` - Shared state accessible by all jobs (must be thread-safe)
/// * `concurrency` - Maximum number of concurrent operations
/// * `job_fn` - Function to process each item with access to the shared state
///
/// # Returns
///
/// A vector containing the results from processing each item
pub async fn run_concurrent_batch_with_state<T, R, S, F>(
    items: Vec<T>,
    state: S,
    concurrency: usize,
    job_fn: F,
) -> Vec<(R, bool)>
where
    T: Send + 'static,
    R: Send + 'static,
    S: Clone + Send + Sync + 'static,
    F: Fn(T, S) -> Pin<Box<dyn Future<Output = (R, bool)> + Send>> + Send + Sync + Clone + 'static,
{
    // Create a buffered stream with the specified concurrency
    stream::iter(items)
        .map(move |item| {
            let job = job_fn.clone();
            let state = state.clone();
            async move { job(item, state).await }
        })
        .buffer_unordered(concurrency.max(1)) // Ensure at least 1 concurrency
        .collect::<Vec<_>>()
        .await
}

/// Runs a batch of operations with progress reporting.
///
/// Similar to `run_concurrent_batch`, but also reports progress through a callback function.
/// This is useful for long-running operations where you want to update a progress bar
/// or log periodic status updates.
///
/// # Type Parameters
///
/// * `T` - The input item type
/// * `R` - The result type
/// * `F` - The function type that processes each item
/// * `Fut` - The future type returned by the function
/// * `P` - The progress callback function type
///
/// # Arguments
///
/// * `items` - Vector of items to process
/// * `concurrency` - Maximum number of concurrent operations
/// * `job_fn` - Function that processes each item and returns a future
/// * `progress_fn` - Callback function called after each item is processed
///
/// # Returns
///
/// A vector containing the results of all operations in the same order as the input items.
pub async fn run_concurrent_batch_with_progress<T, R, F, Fut, P>(
    items: Vec<T>,
    concurrency: usize,
    job_fn: impl Fn(T) -> Fut + Send + Sync + Clone + 'static,
    progress_fn: impl Fn(usize, &R) + Send + Sync + Clone + 'static,
) -> Vec<R>
where
    T: Send + 'static,
    R: Send + 'static,
    F: FnOnce(T) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = R> + Send,
    P: Fn(usize, &R) + Send + Sync + Clone + 'static,
{
    let mut results = Vec::with_capacity(items.len());

    // Process in batches to allow for progress reporting
    let mut iter = items.into_iter().enumerate();

    loop {
        let batch: Vec<(usize, T)> = iter.by_ref().take(concurrency).collect();
        if batch.is_empty() {
            break;
        }

        // Process this batch concurrently
        let batch_results = stream::iter(batch)
            .map(|(idx, item)| {
                let job = job_fn.clone();
                async move { (idx, job(item).await) }
            })
            .buffer_unordered(concurrency)
            .collect::<Vec<(usize, R)>>()
            .await;

        // Update progress for each result
        for (idx, result) in &batch_results {
            let progress = progress_fn.clone();
            progress(*idx, result);
        }

        // Store results
        results.extend(batch_results.into_iter().map(|(_, r)| r));
    }

    results
}
