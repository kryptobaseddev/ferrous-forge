//! Performance optimizations for Ferrous Forge
//!
//! This module provides performance enhancements including:
//! - Parallel validation execution
//! - Caching strategies
//! - Lazy file parsing
//! - Memory pooling

pub mod cache;
pub mod parallel;

use std::sync::Arc;
use std::time::Duration;

/// Performance configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Enable parallel execution
    pub parallel_enabled: bool,
    /// Number of worker threads (0 = auto)
    pub thread_count: usize,
    /// Enable caching
    pub cache_enabled: bool,
    /// Cache TTL in seconds
    pub cache_ttl: Duration,
    /// Enable lazy parsing
    pub lazy_parsing: bool,
    /// Memory pool size in MB
    pub memory_pool_size: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            parallel_enabled: true,
            thread_count: 0, // Auto-detect
            cache_enabled: true,
            cache_ttl: Duration::from_secs(300), // 5 minutes
            lazy_parsing: true,
            memory_pool_size: 100, // 100MB
        }
    }
}

/// Performance metrics
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    /// Total validation time in milliseconds
    pub validation_time_ms: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Number of files processed
    pub files_processed: usize,
    /// Cache hit rate (0.0 - 1.0)
    pub cache_hit_rate: f64,
    /// Parallel speedup factor
    pub parallel_speedup: f64,
}

impl PerformanceMetrics {
    /// Check if performance targets are met
    pub fn meets_targets(&self) -> bool {
        self.validation_time_ms < 2000 // < 2s
            && self.memory_usage_bytes < 100 * 1024 * 1024 // < 100MB
    }

    /// Generate performance report
    pub fn report(&self) -> String {
        format!(
            "Performance Metrics:\n\
             - Validation time: {}ms\n\
             - Memory usage: {:.2}MB\n\
             - Files processed: {}\n\
             - Cache hit rate: {:.1}%\n\
             - Parallel speedup: {:.2}x",
            self.validation_time_ms,
            self.memory_usage_bytes as f64 / (1024.0 * 1024.0),
            self.files_processed,
            self.cache_hit_rate * 100.0,
            self.parallel_speedup
        )
    }
}

/// Performance monitor
pub struct PerformanceMonitor {
    #[allow(dead_code)]
    config: Arc<PerformanceConfig>,
    metrics: Arc<dashmap::DashMap<String, PerformanceMetrics>>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config: Arc::new(config),
            metrics: Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Start monitoring a task
    pub fn start_task(&self, task_name: &str) -> TaskMonitor {
        TaskMonitor::new(task_name.to_string(), self.metrics.clone())
    }

    /// Get metrics for a task
    pub fn get_metrics(&self, task_name: &str) -> Option<PerformanceMetrics> {
        self.metrics.get(task_name).map(|m| m.clone())
    }

    /// Get all metrics
    pub fn all_metrics(&self) -> Vec<(String, PerformanceMetrics)> {
        self.metrics
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }
}

/// Task-specific performance monitor
pub struct TaskMonitor {
    task_name: String,
    start_time: std::time::Instant,
    metrics_map: Arc<dashmap::DashMap<String, PerformanceMetrics>>,
}

impl TaskMonitor {
    fn new(
        task_name: String,
        metrics_map: Arc<dashmap::DashMap<String, PerformanceMetrics>>,
    ) -> Self {
        Self {
            task_name,
            start_time: std::time::Instant::now(),
            metrics_map,
        }
    }

    /// Complete the task and record metrics
    pub fn complete(self, files_processed: usize) {
        let elapsed = self.start_time.elapsed();
        let metrics = PerformanceMetrics {
            validation_time_ms: elapsed.as_millis() as u64,
            memory_usage_bytes: estimate_memory_usage(),
            files_processed,
            cache_hit_rate: 0.0,   // Will be updated by cache module
            parallel_speedup: 1.0, // Will be updated by parallel module
        };
        self.metrics_map.insert(self.task_name, metrics);
    }
}

/// Estimate current memory usage
fn estimate_memory_usage() -> u64 {
    // This is a simplified estimation
    // In production, we'd use more sophisticated memory tracking

    // For now, use a simple heuristic based on thread count
    let thread_count = rayon::current_num_threads();
    let base_memory = 20 * 1024 * 1024; // 20MB base
    let per_thread = 5 * 1024 * 1024; // 5MB per thread

    base_memory + (thread_count as u64 * per_thread)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_config_default() {
        let config = PerformanceConfig::default();
        assert!(config.parallel_enabled);
        assert!(config.cache_enabled);
        assert_eq!(config.thread_count, 0);
    }

    #[test]
    fn test_performance_metrics() {
        let metrics = PerformanceMetrics {
            validation_time_ms: 1500,
            memory_usage_bytes: 50 * 1024 * 1024,
            ..Default::default()
        };

        assert!(metrics.meets_targets());

        let metrics = PerformanceMetrics {
            validation_time_ms: 3000,
            memory_usage_bytes: 50 * 1024 * 1024,
            ..Default::default()
        };
        assert!(!metrics.meets_targets());
    }

    #[test]
    fn test_performance_monitor() {
        let config = PerformanceConfig::default();
        let monitor = PerformanceMonitor::new(config);

        let task = monitor.start_task("test_task");
        task.complete(10);

        let metrics = monitor.get_metrics("test_task");
        assert!(metrics.is_some());
    }
}
