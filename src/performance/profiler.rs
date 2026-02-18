use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct ProfileEvent {
    pub name: String,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub duration: Option<Duration>,
    pub memory_before: Option<usize>,
    pub memory_after: Option<usize>,
    pub thread_id: u64,
    pub call_stack: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProfileMetrics {
    pub total_calls: usize,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub memory_peak: usize,
    pub memory_average: usize,
}

pub struct Profiler {
    events: Arc<Mutex<Vec<ProfileEvent>>>,
    active_events: Arc<Mutex<HashMap<String, ProfileEvent>>>,
    enabled: bool,
    memory_tracking: bool,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            active_events: Arc::new(Mutex::new(HashMap::new())),
            enabled: true,
            memory_tracking: false,
        }
    }

    pub fn with_memory_tracking(mut self, enable: bool) -> Self {
        self.memory_tracking = enable;
        self
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn start_profile(&self, name: &str) -> Option<String> {
        if !self.enabled {
            return None;
        }

        let event_id = format!("{}_{}", name, std::time::Instant::now().elapsed().as_nanos());
        let thread_id = std::time::Instant::now().elapsed().as_nanos() as u64;
        
        let event = ProfileEvent {
            name: name.to_string(),
            start_time: Instant::now(),
            end_time: None,
            duration: None,
            memory_before: if self.memory_tracking { Some(Self::get_memory_usage()) } else { None },
            memory_after: None,
            thread_id,
            call_stack: Vec::new(), // Could be enhanced with actual stack traces
        };

        if let Ok(mut active) = self.active_events.lock() {
            active.insert(event_id.clone(), event);
        }

        Some(event_id)
    }

    pub fn end_profile(&self, event_id: &str) {
        if !self.enabled {
            return;
        }

        let end_time = Instant::now();
        let memory_after = if self.memory_tracking { Some(Self::get_memory_usage()) } else { None };

        if let Ok(mut active) = self.active_events.lock() {
            if let Some(mut event) = active.remove(event_id) {
                event.end_time = Some(end_time);
                event.duration = Some(end_time.duration_since(event.start_time));
                event.memory_after = memory_after;

                if let Ok(mut events) = self.events.lock() {
                    events.push(event);
                }
            }
        }
    }

    pub fn profile_scope<F, R>(&self, name: &str, f: F) -> R 
    where 
        F: FnOnce() -> R
    {
        let event_id = self.start_profile(name);
        let result = f();
        if let Some(id) = event_id {
            self.end_profile(&id);
        }
        result
    }

    pub fn get_metrics(&self) -> HashMap<String, ProfileMetrics> {
        let mut metrics = HashMap::new();
        
        if let Ok(events) = self.events.lock() {
            for event in events.iter() {
                let entry = metrics.entry(event.name.clone()).or_insert_with(|| ProfileMetrics {
                    total_calls: 0,
                    total_duration: Duration::ZERO,
                    average_duration: Duration::ZERO,
                    min_duration: Duration::MAX,
                    max_duration: Duration::ZERO,
                    memory_peak: 0,
                    memory_average: 0,
                });

                entry.total_calls += 1;
                
                if let Some(duration) = event.duration {
                    entry.total_duration += duration;
                    entry.min_duration = entry.min_duration.min(duration);
                    entry.max_duration = entry.max_duration.max(duration);
                }

                if let (Some(before), Some(after)) = (event.memory_before, event.memory_after) {
                    let memory_used = after.saturating_sub(before);
                    entry.memory_peak = entry.memory_peak.max(memory_used);
                    entry.memory_average = (entry.memory_average + memory_used) / 2;
                }
            }

            // Calculate averages
            for metrics in metrics.values_mut() {
                if metrics.total_calls > 0 {
                    metrics.average_duration = metrics.total_duration / metrics.total_calls as u32;
                }
            }
        }

        metrics
    }

    pub fn generate_report(&self) -> String {
        let metrics = self.get_metrics();
        let mut report = String::new();
        
        report.push_str("Performance Profile Report\n");
        report.push_str("========================\n\n");

        for (name, metric) in metrics {
            report.push_str(&format!("Function: {}\n", name));
            report.push_str(&format!("  Total Calls: {}\n", metric.total_calls));
            report.push_str(&format!("  Total Duration: {:?}\n", metric.total_duration));
            report.push_str(&format!("  Average Duration: {:?}\n", metric.average_duration));
            report.push_str(&format!("  Min Duration: {:?}\n", metric.min_duration));
            report.push_str(&format!("  Max Duration: {:?}\n", metric.max_duration));
            if metric.memory_peak > 0 {
                report.push_str(&format!("  Memory Peak: {} bytes\n", metric.memory_peak));
                report.push_str(&format!("  Memory Average: {} bytes\n", metric.memory_average));
            }
            report.push('\n');
        }

        report
    }

    pub fn clear(&self) {
        if let Ok(mut events) = self.events.lock() {
            events.clear();
        }
        if let Ok(mut active) = self.active_events.lock() {
            active.clear();
        }
    }

    fn get_memory_usage() -> usize {
        // Simple memory usage estimation
        std::mem::size_of::<usize>() * 1024 // Placeholder
    }
}

// Global profiler instance
lazy_static::lazy_static! {
    static ref GLOBAL_PROFILER: Arc<Profiler> = Arc::new(Profiler::new());
}

pub fn get_global_profiler() -> Arc<Profiler> {
    GLOBAL_PROFILER.clone()
}

// Profiling macros for easy use
#[macro_export]
macro_rules! profile {
    ($name:expr, $block:expr) => {{
        let profiler = $crate::performance::profiler::get_global_profiler();
        profiler.profile_scope($name, $block)
    }};
}

#[macro_export]
macro_rules! profile_start {
    ($name:expr) => {{
        let profiler = $crate::performance::profiler::get_global_profiler();
        profiler.start_profile($name)
    }};
}

#[macro_export]
macro_rules! profile_end {
    ($event_id:expr) => {{
        let profiler = $crate::performance::profiler::get_global_profiler();
        profiler.end_profile($event_id);
    }};
}

// Performance monitoring for specific language components
pub struct LanguageProfiler;

impl LanguageProfiler {
    pub fn profile_lexer<F, R>(f: F) -> R 
    where 
        F: FnOnce() -> R
    {
        profile!("lexer", f)
    }

    pub fn profile_parser<F, R>(f: F) -> R 
    where 
        F: FnOnce() -> R
    {
        profile!("parser", f)
    }

    pub fn profile_runtime<F, R>(f: F) -> R 
    where 
        F: FnOnce() -> R
    {
        profile!("runtime", f)
    }

    pub fn profile_stdlib<F, R>(namespace: &str, f: F) -> R 
    where 
        F: FnOnce() -> R
    {
        profile!(&format!("stdlib_{}", namespace), f)
    }
}

// Memory profiling utilities
pub struct MemoryProfiler {
    allocations: Arc<Mutex<HashMap<String, usize>>>,
    deallocations: Arc<Mutex<HashMap<String, usize>>>,
}

impl MemoryProfiler {
    pub fn new() -> Self {
        Self {
            allocations: Arc::new(Mutex::new(HashMap::new())),
            deallocations: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn track_allocation(&self, type_name: &str, size: usize) {
        if let Ok(mut allocations) = self.allocations.lock() {
            *allocations.entry(type_name.to_string()).or_insert(0) += size;
        }
    }

    pub fn track_deallocation(&self, type_name: &str, size: usize) {
        if let Ok(mut deallocations) = self.deallocations.lock() {
            *deallocations.entry(type_name.to_string()).or_insert(0) += size;
        }
    }

    pub fn get_memory_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Memory Allocation Report\n");
        report.push_str("=======================\n\n");

        if let Ok(allocations) = self.allocations.lock() {
            if let Ok(deallocations) = self.deallocations.lock() {
                for (type_name, allocated) in allocations.iter() {
                    let deallocated = deallocations.get(type_name).unwrap_or(&0);
                    let current = allocated.saturating_sub(*deallocated);
                    
                    report.push_str(&format!("Type: {}\n", type_name));
                    report.push_str(&format!("  Total Allocated: {} bytes\n", allocated));
                    report.push_str(&format!("  Total Deallocated: {} bytes\n", deallocated));
                    report.push_str(&format!("  Current Usage: {} bytes\n", current));
                    report.push('\n');
                }
            }
        }

        report
    }
}
