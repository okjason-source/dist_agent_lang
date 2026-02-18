use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_allocated: usize,
    pub total_freed: usize,
    pub current_usage: usize,
    pub peak_usage: usize,
    pub allocation_count: usize,
    pub deallocation_count: usize,
    pub fragmentation: f64,
}

#[derive(Debug, Clone)]
pub struct MemoryBlock {
    pub id: usize,
    pub size: usize,
    pub allocated_at: Instant,
    pub freed_at: Option<Instant>,
    pub lifetime: Option<Duration>,
    pub type_name: String,
}

pub struct MemoryManager {
    blocks: Arc<Mutex<HashMap<usize, MemoryBlock>>>,
    next_id: Arc<Mutex<usize>>,
    stats: Arc<Mutex<MemoryStats>>,
    object_pools: Arc<Mutex<HashMap<String, ObjectPool>>>,
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {
            blocks: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
            stats: Arc::new(Mutex::new(MemoryStats {
                total_allocated: 0,
                total_freed: 0,
                current_usage: 0,
                peak_usage: 0,
                allocation_count: 0,
                deallocation_count: 0,
                fragmentation: 0.0,
            })),
            object_pools: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn allocate(&self, size: usize, type_name: &str) -> usize {
        let id = {
            let mut next_id = self.next_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let block = MemoryBlock {
            id,
            size,
            allocated_at: Instant::now(),
            freed_at: None,
            lifetime: None,
            type_name: type_name.to_string(),
        };

        // Update blocks
        if let Ok(mut blocks) = self.blocks.lock() {
            blocks.insert(id, block);
        }

        // Update stats
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_allocated += size;
            stats.current_usage += size;
            stats.allocation_count += 1;
            stats.peak_usage = stats.peak_usage.max(stats.current_usage);
        }

        id
    }

    pub fn deallocate(&self, id: usize) -> bool {
        if let Ok(mut blocks) = self.blocks.lock() {
            if let Some(block) = blocks.get_mut(&id) {
                block.freed_at = Some(Instant::now());
                block.lifetime = Some(block.freed_at.unwrap().duration_since(block.allocated_at));

                // Update stats
                if let Ok(mut stats) = self.stats.lock() {
                    stats.total_freed += block.size;
                    stats.current_usage = stats.current_usage.saturating_sub(block.size);
                    stats.deallocation_count += 1;
                }

                return true;
            }
        }
        false
    }

    pub fn get_stats(&self) -> MemoryStats {
        if let Ok(stats) = self.stats.lock() {
            stats.clone()
        } else {
            MemoryStats {
                total_allocated: 0,
                total_freed: 0,
                current_usage: 0,
                peak_usage: 0,
                allocation_count: 0,
                deallocation_count: 0,
                fragmentation: 0.0,
            }
        }
    }

    pub fn get_memory_report(&self) -> String {
        let stats = self.get_stats();
        let mut report = String::new();

        report.push_str("Memory Management Report\n");
        report.push_str("======================\n\n");
        report.push_str(&format!(
            "Total Allocated: {} bytes\n",
            stats.total_allocated
        ));
        report.push_str(&format!("Total Freed: {} bytes\n", stats.total_freed));
        report.push_str(&format!("Current Usage: {} bytes\n", stats.current_usage));
        report.push_str(&format!("Peak Usage: {} bytes\n", stats.peak_usage));
        report.push_str(&format!("Allocation Count: {}\n", stats.allocation_count));
        report.push_str(&format!(
            "Deallocation Count: {}\n",
            stats.deallocation_count
        ));
        report.push_str(&format!("Fragmentation: {:.2}%\n", stats.fragmentation));

        // Type breakdown
        if let Ok(blocks) = self.blocks.lock() {
            let mut type_stats: HashMap<String, (usize, usize)> = HashMap::new();

            for block in blocks.values() {
                let entry = type_stats.entry(block.type_name.clone()).or_insert((0, 0));
                entry.0 += block.size;
                entry.1 += 1;
            }

            report.push_str("\nType Breakdown:\n");
            for (type_name, (total_size, count)) in type_stats {
                report.push_str(&format!(
                    "  {}: {} bytes ({} instances)\n",
                    type_name, total_size, count
                ));
            }
        }

        report
    }

    pub fn create_object_pool<T>(&self, name: &str, initial_capacity: usize) -> ObjectPool
    where
        T: Clone + Default + Send + Sync + 'static,
    {
        let pool = ObjectPool::new::<T>(name, initial_capacity);

        if let Ok(mut pools) = self.object_pools.lock() {
            pools.insert(name.to_string(), pool.clone());
        }

        pool
    }

    pub fn get_object_pool(&self, name: &str) -> Option<ObjectPool> {
        if let Ok(pools) = self.object_pools.lock() {
            pools.get(name).cloned()
        } else {
            None
        }
    }
}

// Global memory manager instance
lazy_static::lazy_static! {
    static ref GLOBAL_MEMORY_MANAGER: Arc<MemoryManager> = Arc::new(MemoryManager::new());
}

pub fn get_global_memory_manager() -> Arc<MemoryManager> {
    GLOBAL_MEMORY_MANAGER.clone()
}

// Object pooling for performance optimization
#[derive(Debug, Clone)]
pub struct ObjectPool {
    name: String,
    capacity: usize,
    available: Arc<Mutex<Vec<Box<dyn std::any::Any + Send + Sync>>>>,
    in_use: Arc<Mutex<Vec<Box<dyn std::any::Any + Send + Sync>>>>,
    created_count: Arc<Mutex<usize>>,
    reused_count: Arc<Mutex<usize>>,
}

impl ObjectPool {
    pub fn new<T>(name: &str, initial_capacity: usize) -> Self
    where
        T: Clone + Default + Send + Sync + 'static,
    {
        let mut available = Vec::with_capacity(initial_capacity);
        for _ in 0..initial_capacity {
            available.push(Box::new(T::default()) as Box<dyn Any + Send + Sync>);
        }

        Self {
            name: name.to_string(),
            capacity: initial_capacity,
            available: Arc::new(Mutex::new(available)),
            in_use: Arc::new(Mutex::new(Vec::new())),
            created_count: Arc::new(Mutex::new(0)),
            reused_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn acquire<T>(&self) -> PooledObject<T>
    where
        T: Clone + Default + Send + Sync + 'static,
    {
        if let Ok(mut available) = self.available.lock() {
            if let Some(obj) = available.pop() {
                if let Ok(obj) = obj.downcast::<T>() {
                    if let Ok(mut in_use) = self.in_use.lock() {
                        in_use.push(Box::new(obj.clone()));
                    }
                    if let Ok(mut reused) = self.reused_count.lock() {
                        *reused += 1;
                    }
                    return PooledObject {
                        value: *obj,
                        pool: self.clone(),
                    };
                }
            }
        }

        // Create new object if pool is empty
        let new_obj = T::default();
        if let Ok(mut created) = self.created_count.lock() {
            *created += 1;
        }
        if let Ok(mut in_use) = self.in_use.lock() {
            in_use.push(Box::new(new_obj.clone()));
        }

        PooledObject {
            value: new_obj,
            pool: self.clone(),
        }
    }

    pub fn get_stats(&self) -> PoolStats {
        let available_count = self.available.lock().map(|v| v.len()).unwrap_or(0);
        let in_use_count = self.in_use.lock().map(|v| v.len()).unwrap_or(0);
        let created_count = self.created_count.lock().map(|v| *v).unwrap_or(0);
        let reused_count = self.reused_count.lock().map(|v| *v).unwrap_or(0);

        PoolStats {
            name: self.name.clone(),
            capacity: self.capacity,
            available_count,
            in_use_count,
            created_count,
            reused_count,
            reuse_rate: if created_count > 0 {
                reused_count as f64 / created_count as f64
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub name: String,
    pub capacity: usize,
    pub available_count: usize,
    pub in_use_count: usize,
    pub created_count: usize,
    pub reused_count: usize,
    pub reuse_rate: f64,
}

pub struct PooledObject<T>
where
    T: Clone + Send + Sync + 'static,
{
    value: T,
    pool: ObjectPool,
}

impl<T: Clone + Send + Sync + 'static> PooledObject<T> {
    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    pub fn into_inner(self) -> T
    where
        T: Clone,
    {
        self.value.clone()
    }
}

impl<T: Clone + Send + Sync + 'static> Drop for PooledObject<T> {
    fn drop(&mut self) {
        // Return object to pool
        if let Ok(mut available) = self.pool.available.lock() {
            if let Ok(mut in_use) = self.pool.in_use.lock() {
                // Find and remove from in_use
                if let Some(index) = in_use.iter().position(|obj| {
                    if let Some(obj) = obj.downcast_ref::<T>() {
                        std::ptr::eq(obj, &self.value)
                    } else {
                        false
                    }
                }) {
                    let _ = in_use.remove(index);
                    available.push(Box::new(self.value.clone()));
                }
            }
        }
    }
}

// Garbage collection utilities
pub struct GarbageCollector {
    memory_manager: Arc<MemoryManager>,
    collection_threshold: usize,
    last_collection: Arc<Mutex<Instant>>,
}

impl GarbageCollector {
    pub fn new(memory_manager: Arc<MemoryManager>) -> Self {
        Self {
            memory_manager,
            collection_threshold: 1024 * 1024, // 1MB
            last_collection: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn with_threshold(mut self, threshold: usize) -> Self {
        self.collection_threshold = threshold;
        self
    }

    pub fn should_collect(&self) -> bool {
        let stats = self.memory_manager.get_stats();
        let time_since_last = {
            if let Ok(last) = self.last_collection.lock() {
                last.elapsed()
            } else {
                Duration::from_secs(0)
            }
        };

        stats.current_usage > self.collection_threshold || time_since_last > Duration::from_secs(30)
    }

    pub fn collect(&self) -> CollectionResult {
        let start_time = Instant::now();
        let stats_before = self.memory_manager.get_stats();

        // Simple mark-and-sweep implementation
        let mut collected_blocks = 0;
        let mut freed_memory = 0;

        if let Ok(mut blocks) = self.memory_manager.blocks.lock() {
            let mut to_remove = Vec::new();

            for (id, block) in blocks.iter() {
                if block.freed_at.is_some() {
                    to_remove.push(*id);
                    collected_blocks += 1;
                    freed_memory += block.size;
                }
            }

            for id in to_remove {
                blocks.remove(&id);
            }
        }

        let stats_after = self.memory_manager.get_stats();
        let duration = start_time.elapsed();

        // Update last collection time
        if let Ok(mut last) = self.last_collection.lock() {
            *last = Instant::now();
        }

        CollectionResult {
            duration,
            blocks_collected: collected_blocks,
            memory_freed: freed_memory,
            memory_before: stats_before.current_usage,
            memory_after: stats_after.current_usage,
            fragmentation_before: stats_before.fragmentation,
            fragmentation_after: stats_after.fragmentation,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CollectionResult {
    pub duration: Duration,
    pub blocks_collected: usize,
    pub memory_freed: usize,
    pub memory_before: usize,
    pub memory_after: usize,
    pub fragmentation_before: f64,
    pub fragmentation_after: f64,
}

// Memory optimization utilities
pub struct MemoryOptimizer;

impl MemoryOptimizer {
    pub fn optimize_string_storage(strings: &[String]) -> Vec<String> {
        // Simple string interning optimization
        let mut interned: HashMap<String, String> = HashMap::new();
        let mut optimized = Vec::new();

        for string in strings {
            if let Some(existing) = interned.get(string) {
                optimized.push(existing.clone());
            } else {
                interned.insert(string.clone(), string.clone());
                optimized.push(string.clone());
            }
        }

        optimized
    }

    pub fn estimate_memory_usage<T>(objects: &[T]) -> usize
    where
        T: Sized,
    {
        std::mem::size_of_val(objects)
    }

    pub fn suggest_pool_size(usage_pattern: &[usize]) -> usize {
        if usage_pattern.is_empty() {
            return 10;
        }

        let avg_usage = usage_pattern.iter().sum::<usize>() / usage_pattern.len();
        let max_usage = usage_pattern.iter().max().unwrap_or(&avg_usage);

        // Suggest pool size based on usage pattern
        if *max_usage > avg_usage * 2 {
            avg_usage * 2 // Conservative for spiky usage
        } else {
            avg_usage // Standard for consistent usage
        }
    }
}
