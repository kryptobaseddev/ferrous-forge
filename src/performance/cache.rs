//! Caching strategies for performance optimization

use dashmap::DashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, trace};

/// Generic cache entry with expiration
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}

/// Thread-safe cache with TTL support
pub struct Cache<K, V>
where
    K: Eq + Hash,
{
    storage: Arc<DashMap<K, CacheEntry<V>>>,
    ttl: Duration,
    hits: Arc<std::sync::atomic::AtomicU64>,
    misses: Arc<std::sync::atomic::AtomicU64>,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Create a new cache with specified TTL
    pub fn new(ttl: Duration) -> Self {
        Self {
            storage: Arc::new(DashMap::new()),
            ttl,
            hits: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            misses: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Get a value from the cache
    pub fn get(&self, key: &K) -> Option<V> {
        if let Some(entry) = self.storage.get(key) {
            if !entry.is_expired() {
                self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                trace!("Cache hit for key");
                return Some(entry.value.clone());
            } else {
                // Remove expired entry
                drop(entry);
                self.storage.remove(key);
                debug!("Cache entry expired, removing");
            }
        }

        self.misses
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        trace!("Cache miss for key");
        None
    }

    /// Insert a value into the cache
    pub fn insert(&self, key: K, value: V) {
        let entry = CacheEntry::new(value, self.ttl);
        self.storage.insert(key, entry);
        trace!("Cached value inserted");
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        self.storage.clear();
        debug!("Cache cleared");
    }

    /// Remove expired entries
    pub fn evict_expired(&self) {
        let now = Instant::now();
        self.storage.retain(|_, entry| entry.expires_at > now);
        debug!("Evicted expired cache entries");
    }

    /// Get cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(std::sync::atomic::Ordering::Relaxed);
        let misses = self.misses.load(std::sync::atomic::Ordering::Relaxed);
        let total = hits + misses;

        if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.storage.len(),
            hits: self.hits.load(std::sync::atomic::Ordering::Relaxed),
            misses: self.misses.load(std::sync::atomic::Ordering::Relaxed),
            hit_rate: self.hit_rate(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of entries in cache
    pub entries: usize,
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Hit rate (0.0 - 1.0)
    pub hit_rate: f64,
}

/// Validation result cache
pub struct ValidationCache {
    cache: Cache<String, Vec<crate::validation::Violation>>,
}

impl ValidationCache {
    /// Create a new validation cache
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Cache::new(ttl),
        }
    }

    /// Get cached validation results
    pub fn get_results(&self, file_path: &str) -> Option<Vec<crate::validation::Violation>> {
        self.cache.get(&file_path.to_string())
    }

    /// Cache validation results
    pub fn cache_results(&self, file_path: String, violations: Vec<crate::validation::Violation>) {
        self.cache.insert(file_path, violations);
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.cache.stats()
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.cache.clear();
    }
}

/// File content cache for lazy parsing
pub struct FileCache {
    cache: Cache<std::path::PathBuf, String>,
}

impl FileCache {
    /// Create a new file cache
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Cache::new(ttl),
        }
    }

    /// Get cached file content
    pub fn get_content(&self, path: &std::path::Path) -> Option<String> {
        self.cache.get(&path.to_path_buf())
    }

    /// Cache file content
    pub fn cache_content(&self, path: std::path::PathBuf, content: String) {
        self.cache.insert(path, content);
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.cache.stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let cache = Cache::<String, String>::new(Duration::from_secs(60));

        cache.insert("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cache.get(&"key2".to_string()), None);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = Cache::<String, String>::new(Duration::from_millis(1));

        cache.insert("key1".to_string(), "value1".to_string());
        std::thread::sleep(Duration::from_millis(2));
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_cache_statistics() {
        let cache = Cache::<String, String>::new(Duration::from_secs(60));

        cache.insert("key1".to_string(), "value1".to_string());
        let _ = cache.get(&"key1".to_string()); // Hit
        let _ = cache.get(&"key2".to_string()); // Miss

        let stats = cache.stats();
        assert_eq!(stats.entries, 1);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 0.5);
    }

    #[test]
    fn test_validation_cache() {
        let cache = ValidationCache::new(Duration::from_secs(60));
        let violations = vec![];

        cache.cache_results("test.rs".to_string(), violations.clone());
        assert_eq!(cache.get_results("test.rs"), Some(violations));
    }
}
