use memory_stats::memory_stats;

pub struct AlgorithmMetrics {
    pub time: usize,
    pub peak_memory: usize,
}

/// Get initial memory usage
pub fn start_memory_tracking() -> usize {
    if let Some(usage) = memory_stats() {
        usage.physical_mem
    } else {
        println!("Warning: Couldn't get memory stats");
        0
    }
}

/// Calculate the delta (might not be perfectly accurate but gives a good estimation)
pub fn update_peak_memory(start_memory: usize, current_peak: &mut usize) {
    if let Some(usage) = memory_stats() {
        let current = usage.physical_mem;
        if current > start_memory {
            let delta = current - start_memory;
            *current_peak = (*current_peak).max(delta);
        }
    }
}

pub fn print_metrics(metrics: &AlgorithmMetrics) {
    println!("Time: {} μs", metrics.time);
    println!("Peak memory: {} KiB", metrics.peak_memory / 1024);
}
