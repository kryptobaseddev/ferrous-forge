//! Parallel execution for validation tasks

use crate::Result;
use crate::validation::Violation;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info};

/// Parallel validator for concurrent file processing
pub struct ParallelValidator {
    /// Number of threads to use (0 = auto)
    thread_count: usize,
}

impl ParallelValidator {
    /// Create a new parallel validator
    pub fn new(thread_count: usize) -> Self {
        if thread_count > 0 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(thread_count)
                .build_global()
                .ok();
        }
        Self { thread_count }
    }

    /// Validate multiple files in parallel
    pub async fn validate_files(&self, files: Vec<PathBuf>) -> Result<Vec<Violation>> {
        info!(
            "Validating {} files in parallel (threads: {})",
            files.len(),
            if self.thread_count == 0 {
                rayon::current_num_threads()
            } else {
                self.thread_count
            }
        );

        let start = std::time::Instant::now();

        // Process files in parallel
        let violations: Vec<Violation> = files
            .into_par_iter()
            .flat_map(|file| {
                debug!("Validating file: {}", file.display());
                match validate_single_file(&file) {
                    Ok(violations) => violations,
                    Err(e) => {
                        tracing::error!("Error validating {}: {}", file.display(), e);
                        vec![]
                    }
                }
            })
            .collect();

        let elapsed = start.elapsed();
        info!(
            "Parallel validation completed in {:.2}s ({} violations)",
            elapsed.as_secs_f64(),
            violations.len()
        );

        Ok(violations)
    }

    /// Calculate parallel speedup
    pub fn calculate_speedup(&self, serial_time: f64, parallel_time: f64) -> f64 {
        if parallel_time > 0.0 {
            serial_time / parallel_time
        } else {
            1.0
        }
    }
}

/// Validate a single file (used in parallel processing)
fn validate_single_file(path: &Path) -> Result<Vec<Violation>> {
    // Use sync validation for parallel processing
    use std::fs;

    let content = fs::read_to_string(path).map_err(crate::Error::Io)?;

    // Simple validation for now - check file size
    let mut violations = Vec::new();
    let line_count = content.lines().count();

    if line_count > 300 {
        violations.push(Violation {
            violation_type: crate::validation::ViolationType::FileTooLarge,
            file: path.to_path_buf(),
            line: line_count,
            message: format!("File has {} lines, maximum allowed is 300", line_count),
            severity: crate::validation::Severity::Error,
        });
    }

    Ok(violations)
}

/// Parallel safety check runner
pub struct ParallelSafetyRunner {
    thread_pool: Arc<rayon::ThreadPool>,
}

impl ParallelSafetyRunner {
    /// Create a new parallel safety runner
    pub fn new(thread_count: usize) -> Result<Self> {
        let pool = if thread_count > 0 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(thread_count)
                .build()
                .map_err(|e| crate::Error::config(format!("Failed to create thread pool: {}", e)))?
        } else {
            rayon::ThreadPoolBuilder::new()
                .build()
                .map_err(|e| crate::Error::config(format!("Failed to create thread pool: {}", e)))?
        };

        Ok(Self {
            thread_pool: Arc::new(pool),
        })
    }

    /// Run checks in parallel
    pub async fn run_checks_parallel<F, T>(&self, checks: Vec<F>) -> Vec<Result<T>>
    where
        F: Fn() -> Result<T> + Send + Sync + 'static,
        T: Send + 'static,
    {
        let pool = self.thread_pool.clone();

        tokio::task::spawn_blocking(move || {
            pool.install(|| checks.into_par_iter().map(|check| check()).collect())
        })
        .await
        .unwrap_or_else(|e| {
            tracing::error!("Parallel execution failed: {}", e);
            vec![]
        })
    }
}

/// Batch processor for file operations
pub struct BatchProcessor {
    batch_size: usize,
}

impl BatchProcessor {
    /// Create a new batch processor
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch_size: batch_size.max(1),
        }
    }

    /// Process items in batches
    pub fn process_batches<T, F, R>(&self, items: Vec<T>, processor: F) -> Vec<R>
    where
        T: Send + Clone,
        F: Fn(Vec<T>) -> Vec<R> + Send + Sync,
        R: Send,
    {
        let mut results = Vec::new();
        for chunk in items.chunks(self.batch_size) {
            results.extend(processor(chunk.to_vec()));
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_validator_creation() {
        let validator = ParallelValidator::new(4);
        assert_eq!(validator.thread_count, 4);
    }

    #[test]
    fn test_speedup_calculation() {
        let validator = ParallelValidator::new(0);
        let speedup = validator.calculate_speedup(10.0, 2.5);
        assert_eq!(speedup, 4.0);
    }

    #[test]
    fn test_batch_processor() {
        let processor = BatchProcessor::new(3);
        let items = vec![1, 2, 3, 4, 5, 6, 7, 8];

        let results =
            processor.process_batches(items, |batch| batch.iter().map(|x| x * 2).collect());

        assert_eq!(results, vec![2, 4, 6, 8, 10, 12, 14, 16]);
    }

    #[tokio::test]
    async fn test_parallel_safety_runner() {
        let runner = ParallelSafetyRunner::new(2).unwrap();

        let checks = vec![
            || Ok::<i32, crate::Error>(1),
            || Ok::<i32, crate::Error>(2),
            || Ok::<i32, crate::Error>(3),
        ];

        let results = runner.run_checks_parallel(checks).await;
        assert_eq!(results.len(), 3);
    }
}
