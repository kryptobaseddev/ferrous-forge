//! Simple in-memory cache with TTL support

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Cache entry with value and insertion time
struct CacheEntry<V> {
    value: V,
    inserted_at: Instant,
}

/// Simple cache with time-to-live (TTL) support
pub struct Cache<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
    ttl: Duration,
}

impl<K, V> Cache<K, V>
where
    K: Eq + std::hash::Hash,
    V: Clone,
{
    /// Create a new cache with the specified TTL
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
        }
    }
    
    /// Get a value from the cache
    pub fn get(&self, key: &K) -> Option<V> {
        self.entries.get(key).and_then(|entry| {
            if entry.inserted_at.elapsed() < self.ttl {
                Some(entry.value.clone())
            } else {
                None
            }
        })
    }
    
    /// Insert a value into the cache
    pub fn insert(&mut self, key: K, value: V) {
        self.entries.insert(
            key,
            CacheEntry {
                value,
                inserted_at: Instant::now(),
            },
        );
    }
    
    /// Clear the cache
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    
    /// Remove expired entries
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        self.entries.retain(|_, entry| {
            now.duration_since(entry.inserted_at) < self.ttl
        });
    }
    
    /// Invalidate a specific entry
    pub fn invalidate(&mut self, key: &K) {
        self.entries.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_cache_basic() {
        let mut cache = Cache::new(Duration::from_secs(60));
        
        cache.insert("key1", "value1");
        assert_eq!(cache.get(&"key1"), Some("value1"));
        assert_eq!(cache.get(&"key2"), None);
    }
    
    #[test]
    fn test_cache_expiration() {
        let mut cache = Cache::new(Duration::from_millis(50));
        
        cache.insert("key1", "value1");
        assert_eq!(cache.get(&"key1"), Some("value1"));
        
        thread::sleep(Duration::from_millis(60));
        assert_eq!(cache.get(&"key1"), None);
    }
    
    #[test]
    fn test_cache_cleanup() {
        let mut cache = Cache::new(Duration::from_millis(50));
        
        cache.insert("key1", "value1");
        cache.insert("key2", "value2");
        
        thread::sleep(Duration::from_millis(60));
        
        cache.insert("key3", "value3");
        cache.cleanup();
        
        assert_eq!(cache.entries.len(), 1);
        assert_eq!(cache.get(&"key3"), Some("value3"));
    }
}
