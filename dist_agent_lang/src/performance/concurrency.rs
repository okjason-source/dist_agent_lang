use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct ThreadPoolStats {
    pub total_threads: usize,
    pub active_threads: usize,
    pub idle_threads: usize,
    pub queued_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub average_task_duration: Duration,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: std::sync::mpsc::Sender<Message>,
    stats: Arc<Mutex<ThreadPoolStats>>,
}

enum Message {
    NewJob(Box<dyn FnOnce() + Send + 'static>),
    Terminate,
}

struct Worker {
    #[allow(dead_code)]
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = std::sync::mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let stats = Arc::new(Mutex::new(ThreadPoolStats {
            total_threads: size,
            active_threads: 0,
            idle_threads: size,
            queued_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            average_task_duration: Duration::ZERO,
        }));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver), Arc::clone(&stats)));
        }

        ThreadPool {
            workers,
            sender,
            stats,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
        
        // Update stats
        if let Ok(mut stats) = self.stats.lock() {
            stats.queued_tasks += 1;
        }
    }

    pub fn get_stats(&self) -> ThreadPoolStats {
        if let Ok(stats) = self.stats.lock() {
            stats.clone()
        } else {
            ThreadPoolStats {
                total_threads: 0,
                active_threads: 0,
                idle_threads: 0,
                queued_tasks: 0,
                completed_tasks: 0,
                failed_tasks: 0,
                average_task_duration: Duration::ZERO,
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Worker {
    fn new(
        id: usize,
        receiver: Arc<Mutex<std::sync::mpsc::Receiver<Message>>>,
        stats: Arc<Mutex<ThreadPoolStats>>,
    ) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();

                match message {
                    Message::NewJob(job) => {
                        // Update stats: worker becomes active
                        if let Ok(mut stats) = stats.lock() {
                            stats.active_threads += 1;
                            stats.idle_threads = stats.idle_threads.saturating_sub(1);
                        }

                        let start_time = Instant::now();
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(job));
                        let duration = start_time.elapsed();

                        // Update stats: job completed
                        if let Ok(mut stats) = stats.lock() {
                            stats.active_threads = stats.active_threads.saturating_sub(1);
                            stats.idle_threads += 1;
                            stats.queued_tasks = stats.queued_tasks.saturating_sub(1);
                            
                            if result.is_ok() {
                                stats.completed_tasks += 1;
                            } else {
                                stats.failed_tasks += 1;
                            }

                            // Update average duration
                            let total_duration = stats.average_task_duration * stats.completed_tasks as u32;
                            let new_total = total_duration + duration;
                            stats.average_task_duration = new_total / (stats.completed_tasks + 1) as u32;
                        }
                    }
                    Message::Terminate => {
                        break;
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

// Async task scheduler
pub struct AsyncScheduler {
    thread_pool: ThreadPool,
    task_queue: Arc<Mutex<VecDeque<AsyncTask>>>,
    running_tasks: Arc<Mutex<Vec<RunningTask>>>,
}

pub struct AsyncTask {
    pub id: usize,
    pub name: String,
    pub priority: TaskPriority,
    pub task: Box<dyn FnOnce() -> Result<(), String> + Send + 'static>,
    pub created_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[derive(Debug)]
struct RunningTask {
    id: usize,
    #[allow(dead_code)]
    start_time: Instant,
    #[allow(dead_code)]
    thread_id: usize,
}

impl AsyncScheduler {
    pub fn new(thread_count: usize) -> Self {
        Self {
            thread_pool: ThreadPool::new(thread_count),
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            running_tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn schedule<F>(&self, name: &str, priority: TaskPriority, task: F) -> usize
    where
        F: FnOnce() -> Result<(), String> + Send + 'static,
    {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let task_id = NEXT_ID.fetch_add(1, Ordering::SeqCst);

        let async_task = AsyncTask {
            id: task_id,
            name: name.to_string(),
            priority,
            task: Box::new(task),
            created_at: Instant::now(),
        };

        if let Ok(mut queue) = self.task_queue.lock() {
            // Insert based on priority (higher priority first)
            let mut insert_index = None;
            for (i, existing_task) in queue.iter().enumerate() {
                if priority > existing_task.priority {
                    insert_index = Some(i);
                    break;
                }
            }
            
            if let Some(index) = insert_index {
                queue.insert(index, async_task);
            } else {
                queue.push_back(async_task);
            }
        }

        self.process_queue();
        task_id
    }

    fn process_queue(&self) {
        if let Ok(mut queue) = self.task_queue.lock() {
            if let Some(task) = queue.pop_front() {
                let task_id = task.id;
                let task_name = task.name.clone();
                
                let running_tasks = self.running_tasks.clone();
                self.thread_pool.execute(move || {
                    let start_time = Instant::now();
                    
                    // Add to running tasks
                    if let Ok(mut running) = running_tasks.lock() {
                        running.push(RunningTask {
                            id: task_id,
                            start_time,
                            thread_id: std::time::Instant::now().elapsed().as_nanos() as usize,
                        });
                    }

                    // Execute task
                    let result = (task.task)();

                    // Remove from running tasks
                    if let Ok(mut running) = running_tasks.lock() {
                        running.retain(|t| t.id != task_id);
                    }

                    match result {
                        Ok(_) => println!("Task '{}' completed successfully", task_name),
                        Err(e) => println!("Task '{}' failed: {}", task_name, e),
                    }
                });
            }
        }
    }

    pub fn get_task_status(&self, task_id: usize) -> TaskStatus {
        // Check if task is running
        if let Ok(running) = self.running_tasks.lock() {
            if running.iter().any(|t| t.id == task_id) {
                return TaskStatus::Running;
            }
        }

        // Check if task is queued
        if let Ok(queue) = self.task_queue.lock() {
            if queue.iter().any(|t| t.id == task_id) {
                return TaskStatus::Queued;
            }
        }

        TaskStatus::Completed
    }

    pub fn cancel_task(&self, task_id: usize) -> bool {
        // Remove from queue
        if let Ok(mut queue) = self.task_queue.lock() {
            queue.retain(|t| t.id != task_id);
        }

        // Note: Can't cancel running tasks in this simple implementation
        true
    }
}

#[derive(Debug, Clone)]
pub enum TaskStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

// Parallel execution utilities
pub struct ParallelExecutor;

impl ParallelExecutor {
    pub fn map_parallel<T, R, F>(items: Vec<T>, f: F, num_threads: usize) -> Vec<R>
    where
        T: Send + Sync + Clone + 'static,
        R: Send + Sync + 'static,
        F: Fn(T) -> R + Send + Sync + 'static,
    {
        let thread_pool = ThreadPool::new(num_threads);
        let (sender, receiver) = std::sync::mpsc::channel();
        let items = Arc::new(items);
        let f = Arc::new(f);

        // Submit tasks
        for i in 0..items.len() {
            let items = Arc::clone(&items);
            let f = Arc::clone(&f);
            let sender = sender.clone();
            
            thread_pool.execute(move || {
                let item = items[i].clone();
                let result = f(item);
                sender.send((i, result)).unwrap();
            });
        }

        // Collect results
        let mut results = Vec::new();
        results.resize_with(items.len(), || unsafe { std::mem::zeroed() });
        
        for _ in 0..items.len() {
            if let Ok((index, result)) = receiver.recv() {
                results[index] = result;
            }
        }

        results
    }

    pub fn reduce_parallel<T, F>(items: Vec<T>, f: F, num_threads: usize) -> T
    where
        T: Send + Sync + Clone + 'static,
        F: Fn(T, T) -> T + Send + Sync + Clone + 'static,
    {
        if items.is_empty() {
            panic!("Cannot reduce empty vector");
        }

        if items.len() == 1 {
            return items[0].clone();
        }

        // Divide and conquer approach
        let chunk_size = items.len().div_ceil(num_threads);
        let chunks: Vec<Vec<T>> = items
            .chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        let thread_pool = ThreadPool::new(num_threads);
        let (sender, receiver) = std::sync::mpsc::channel();

        let chunks_len = chunks.len();
        // Process each chunk
        for chunk in chunks {
            let sender = sender.clone();
            let f_clone = f.clone();
            thread_pool.execute(move || {
                let result = chunk.into_iter().reduce(f_clone).unwrap();
                sender.send(result).unwrap();
            });
        }

        // Combine results
        let mut final_result = receiver.recv().unwrap();
        for _ in 1..chunks_len {
            if let Ok(result) = receiver.recv() {
                final_result = f(final_result, result);
            }
        }

        final_result
    }
}

// Performance monitoring for concurrency
pub struct ConcurrencyProfiler {
    thread_pool_stats: Arc<Mutex<Vec<ThreadPoolStats>>>,
    task_execution_times: Arc<Mutex<Vec<Duration>>>,
}

impl ConcurrencyProfiler {
    pub fn new() -> Self {
        Self {
            thread_pool_stats: Arc::new(Mutex::new(Vec::new())),
            task_execution_times: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn record_thread_pool_stats(&self, stats: ThreadPoolStats) {
        if let Ok(mut stats_vec) = self.thread_pool_stats.lock() {
            stats_vec.push(stats);
            // Keep only last 100 entries
            if stats_vec.len() > 100 {
                stats_vec.remove(0);
            }
        }
    }

    pub fn record_task_execution_time(&self, duration: Duration) {
        if let Ok(mut times) = self.task_execution_times.lock() {
            times.push(duration);
            // Keep only last 1000 entries
            if times.len() > 1000 {
                times.remove(0);
            }
        }
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Concurrency Performance Report\n");
        report.push_str("=============================\n\n");

        // Thread pool statistics
        if let Ok(stats_vec) = self.thread_pool_stats.lock() {
            if let Some(latest_stats) = stats_vec.last() {
                report.push_str("Thread Pool Status:\n");
                report.push_str(&format!("  Total Threads: {}\n", latest_stats.total_threads));
                report.push_str(&format!("  Active Threads: {}\n", latest_stats.active_threads));
                report.push_str(&format!("  Idle Threads: {}\n", latest_stats.idle_threads));
                report.push_str(&format!("  Queued Tasks: {}\n", latest_stats.queued_tasks));
                report.push_str(&format!("  Completed Tasks: {}\n", latest_stats.completed_tasks));
                report.push_str(&format!("  Failed Tasks: {}\n", latest_stats.failed_tasks));
                report.push_str(&format!("  Average Task Duration: {:?}\n", latest_stats.average_task_duration));
            }
        }

        // Task execution time statistics
        if let Ok(times) = self.task_execution_times.lock() {
            if !times.is_empty() {
                let total_time: Duration = times.iter().sum();
                let avg_time = total_time / times.len() as u32;
                let min_time = times.iter().min().unwrap();
                let max_time = times.iter().max().unwrap();

                report.push_str("\nTask Execution Times:\n");
                report.push_str(&format!("  Total Tasks: {}\n", times.len()));
                report.push_str(&format!("  Average Time: {:?}\n", avg_time));
                report.push_str(&format!("  Min Time: {:?}\n", min_time));
                report.push_str(&format!("  Max Time: {:?}\n", max_time));
            }
        }

        report
    }
}
