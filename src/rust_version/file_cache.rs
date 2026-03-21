//! Local file cache manager for GitHub API responses
//!
//! Provides persistent caching with 24-hour TTL for offline support.
//!
//! @task T024
//! @epic T014

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Default cache TTL (24 hours)
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);

/// Cache file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// When the cache entry was created
    pub created_at: SystemTime,
    /// Data stored in the cache
    pub data: Vec<u8>,
    /// Content type (e.g., "application/json")
    pub content_type: String,
}

/// File-based cache manager for GitHub API responses
pub struct FileCache {
    cache_dir: PathBuf,
    ttl: Duration,
}

impl FileCache {
    /// Create a new file cache
    ///
    /// # Errors
    ///
    /// Returns an error if the cache directory cannot be created.
    pub fn new(cache_dir: impl AsRef<Path>, ttl: Duration) -> Result<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        fs::create_dir_all(&cache_dir)
            .map_err(|e| Error::io(format!("Failed to create cache directory: {e}")))?;

        Ok(Self { cache_dir, ttl })
    }

    /// Create a default cache in the user's cache directory
    ///
    /// # Errors
    ///
    /// Returns an error if the cache directory cannot be determined or created.
    pub fn default() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| Error::config("Could not determine cache directory"))?
            .join("ferrous-forge")
            .join("github");

        Self::new(cache_dir, DEFAULT_CACHE_TTL)
    }

    /// Get a cached entry
    ///
    /// Returns `None` if the entry doesn't exist or has expired.
    pub fn get(&self, key: &str) -> Option<CacheEntry> {
        let path = self.cache_path(key);

        if !path.exists() {
            return None;
        }

        let entry = match self.read_entry(&path) {
            Ok(e) => e,
            Err(_) => return None,
        };

        if self.is_expired(&entry) {
            let _ = fs::remove_file(&path);
            return None;
        }

        Some(entry)
    }

    /// Store data in the cache
    ///
    /// # Errors
    ///
    /// Returns an error if the cache file cannot be written.
    pub fn set(&self, key: &str, data: Vec<u8>, content_type: &str) -> Result<()> {
        let path = self.cache_path(key);
        let entry = CacheEntry {
            created_at: SystemTime::now(),
            data,
            content_type: content_type.to_string(),
        };

        let json = serde_json::to_vec(&entry)
            .map_err(|e| Error::Validation(format!("Failed to serialize cache entry: {e}")))?;

        fs::write(&path, json)
            .map_err(|e| Error::io(format!("Failed to write cache file: {e}")))?;

        Ok(())
    }

    /// Check if offline mode should be used (cache-only)
    ///
    /// Returns true if we should operate in offline mode due to
    /// network unavailability or explicit offline flag.
    pub fn should_use_offline(&self) -> bool {
        // Check for environment variable override
        if std::env::var("FERROUS_FORGE_OFFLINE").is_ok() {
            return true;
        }

        // Check if we have recent cached data
        // If we have valid cache, we can work offline
        self.has_valid_cache()
    }

    /// Check if any valid cached data exists
    fn has_valid_cache(&self) -> bool {
        let Ok(entries) = fs::read_dir(&self.cache_dir) else {
            return false;
        };

        for entry in entries.flatten() {
            if let Ok(cache_entry) = self.read_entry(&entry.path()) {
                if !self.is_expired(&cache_entry) {
                    return true;
                }
            }
        }

        false
    }

    /// Get the cache file path for a key
    fn cache_path(&self, key: &str) -> PathBuf {
        // Sanitize key to create safe filename
        let safe_key = key.replace(['/', '\\', ':', ' '], "_");
        self.cache_dir.join(format!("{safe_key}.json"))
    }

    /// Read a cache entry from disk
    fn read_entry(&self, path: &Path) -> Result<CacheEntry> {
        let data =
            fs::read(path).map_err(|e| Error::io(format!("Failed to read cache file: {e}")))?;

        serde_json::from_slice(&data)
            .map_err(|e| Error::parse(format!("Failed to parse cache entry: {e}")))
    }

    /// Check if a cache entry has expired
    fn is_expired(&self, entry: &CacheEntry) -> bool {
        SystemTime::now()
            .duration_since(entry.created_at)
            .map(|elapsed| elapsed > self.ttl)
            .unwrap_or(true)
    }

    /// Clear all cached entries
    ///
    /// # Errors
    ///
    /// Returns an error if the cache directory cannot be cleared.
    pub fn clear(&self) -> Result<()> {
        let entries = fs::read_dir(&self.cache_dir)
            .map_err(|e| Error::io(format!("Failed to read cache directory: {e}")))?;

        for entry in entries.flatten() {
            let _ = fs::remove_file(entry.path());
        }

        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let mut stats = CacheStats {
            total_entries: 0,
            valid_entries: 0,
            expired_entries: 0,
            total_size: 0,
        };

        let Ok(entries) = fs::read_dir(&self.cache_dir) else {
            return stats;
        };

        for entry in entries.flatten() {
            stats.total_entries += 1;

            if let Ok(metadata) = entry.metadata() {
                stats.total_size += metadata.len();
            }

            if let Ok(cache_entry) = self.read_entry(&entry.path()) {
                if self.is_expired(&cache_entry) {
                    stats.expired_entries += 1;
                } else {
                    stats.valid_entries += 1;
                }
            }
        }

        stats
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of cache entries
    pub total_entries: usize,
    /// Number of valid (non-expired) entries
    pub valid_entries: usize,
    /// Number of expired entries
    pub expired_entries: usize,
    /// Total size in bytes
    pub total_size: u64,
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache: {} total, {} valid, {} expired, {} bytes",
            self.total_entries, self.valid_entries, self.expired_entries, self.total_size
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cache_basic() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache = FileCache::new(temp_dir.path(), Duration::from_secs(60)).unwrap();

        // Store data
        cache
            .set("test-key", b"test data".to_vec(), "text/plain")
            .unwrap();

        // Retrieve data
        let entry = cache.get("test-key").unwrap();
        assert_eq!(entry.data, b"test data");
        assert_eq!(entry.content_type, "text/plain");
    }

    #[test]
    fn test_cache_expiration() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache = FileCache::new(temp_dir.path(), Duration::from_millis(50)).unwrap();

        cache
            .set("test-key", b"test data".to_vec(), "text/plain")
            .unwrap();

        // Should exist immediately
        assert!(cache.get("test-key").is_some());

        // Wait for expiration
        thread::sleep(Duration::from_millis(60));

        // Should be expired
        assert!(cache.get("test-key").is_none());
    }

    #[test]
    fn test_cache_stats() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache = FileCache::new(temp_dir.path(), Duration::from_secs(60)).unwrap();

        cache.set("key1", b"data1".to_vec(), "text/plain").unwrap();
        cache.set("key2", b"data2".to_vec(), "text/plain").unwrap();

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.valid_entries, 2);
        assert_eq!(stats.expired_entries, 0);
    }
}
