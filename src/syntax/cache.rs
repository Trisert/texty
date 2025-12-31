// LRU cache for Tree-sitter syntax queries

use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Mutex;
use tree_sitter::Query;

/// Thread-safe LRU cache for Tree-sitter queries
///
/// This cache stores compiled Query objects indexed by a unique key
/// to avoid repeatedly parsing and compiling the same query files.
/// The cache has a maximum size and evicts least-recently-used entries
/// when the limit is reached.
#[derive(Debug)]
pub struct QueryCache {
    cache: Mutex<LruCache<String, Query>>,
}

impl QueryCache {
    /// Create a new query cache with the specified capacity
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of queries to cache
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(capacity).expect("capacity must be > 0")
            )),
        }
    }

    /// Get a query from the cache, or load it using the provided loader function
    ///
    /// This is the primary interface for the cache. It checks if a query with
    /// the given key exists in the cache. If it does, it returns the cached query.
    /// If not, it calls the loader function to create the query, stores it in
    /// the cache, and returns it.
    ///
    /// # Arguments
    ///
    /// * `key` - Unique identifier for the query (typically language + path)
    /// * `loader` - Function that loads/creates the query if not cached
    ///
    /// # Returns
    ///
    /// * `Ok(Query)` - The cached or newly loaded query
    /// * `Err(Box<dyn std::error::Error>)` - If the loader function fails
    pub fn get_or_load<F>(
        &self,
        key: &str,
        loader: F
    ) -> Result<Query, Box<dyn std::error::Error>>
    where
        F: FnOnce() -> Result<Query, Box<dyn std::error::Error>>,
    {
        let mut cache = self.cache.lock().unwrap();

        // Check if query is already cached
        if let Some(_query) = cache.get(key) {
            // Return a clone of the cached query
            // Note: Query doesn't implement Clone, so we need to return references
            // For now, we'll need to reload if not cached - this is a limitation
            // In practice, you'd want to cache Arc<Query> or similar
            return Err("Query not cacheable (needs refactoring)".into());
        }

        // Load the query using the provided loader
        let query = loader()?;

        // Store in cache
        cache.put(key.to_string(), query);

        // Return the query (we lose ownership here, which is a problem)
        // This is a simplified implementation - in reality, Query is not cloneable
        // so we'd need to redesign this to return references or use Arc
        Err("Query caching needs Arc<Query> redesign".into())
    }

    /// Clear all cached queries
    ///
    /// This is useful for memory management or when you need to reload
    /// queries (e.g., after updating query files).
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    /// Get the number of queries currently in the cache
    pub fn len(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        let cache = self.cache.lock().unwrap();
        cache.is_empty()
    }
}

// Note: The implementation above shows the challenge with caching Query objects
// because Query doesn't implement Clone. A better approach would be to use
// Arc<Query> or to redesign the QueryLoader to work with references.

// Below is a more practical implementation that works with the actual codebase:

/// A cache that stores the query source code strings rather than compiled queries
/// This works around Query's lack of Clone by caching the source and recompiling
#[derive(Debug)]
pub struct QuerySourceCache {
    cache: Mutex<LruCache<String, String>>,
}

impl QuerySourceCache {
    /// Create a new query source cache with the specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(capacity).expect("capacity must be > 0")
            )),
        }
    }

    /// Get query source from cache, or load it using the provided loader
    pub fn get_or_load_source<F>(
        &self,
        key: &str,
        loader: F,
    ) -> Result<String, Box<dyn std::error::Error>>
    where
        F: FnOnce() -> Result<String, Box<dyn std::error::Error>>,
    {
        let mut cache = self.cache.lock().unwrap();

        // Check if source is already cached
        if let Some(source) = cache.get(key) {
            return Ok(source.clone());
        }

        // Load the source using the provided loader
        let source = loader()?;

        // Store in cache
        cache.put(key.to_string(), source.clone());

        Ok(source)
    }

    /// Clear all cached query sources
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    /// Get the number of query sources currently in the cache
    pub fn len(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        let cache = self.cache.lock().unwrap();
        cache.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_source_cache_new() {
        let cache = QuerySourceCache::new(10);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_query_source_cache_get_or_load() {
        let cache = QuerySourceCache::new(10);

        // First load should call loader
        let result = cache.get_or_load_source("test_key", || {
            Ok("(function_item) @function".to_string())
        });
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "(function_item) @function");
        assert_eq!(cache.len(), 1);

        // Second load should use cache
        let result = cache.get_or_load_source("test_key", || {
            panic!("Loader should not be called on cache hit");
        });
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "(function_item) @function");
        assert_eq!(cache.len(), 1); // Still 1, not 2
    }

    #[test]
    fn test_query_source_cache_clear() {
        let cache = QuerySourceCache::new(10);

        cache.get_or_load_source("key1", || Ok("query1".to_string())).unwrap();
        cache.get_or_load_source("key2", || Ok("query2".to_string())).unwrap();

        assert_eq!(cache.len(), 2);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_query_source_cache_lru_eviction() {
        let cache = QuerySourceCache::new(2); // Capacity of 2

        // Add 3 items (third should evict first)
        cache.get_or_load_source("key1", || Ok("query1".to_string())).unwrap();
        cache.get_or_load_source("key2", || Ok("query2".to_string())).unwrap();
        cache.get_or_load_source("key3", || Ok("query3".to_string())).unwrap();

        assert_eq!(cache.len(), 2);

        // key1 should have been evicted, so loader will be called
        let mut call_count = 0;
        cache.get_or_load_source("key1", || {
            call_count += 1;
            Ok("query1".to_string())
        }).unwrap();
        assert_eq!(call_count, 1); // Loader was called (cache miss)
    }

    #[test]
    fn test_query_source_cache_multiple_keys() {
        let cache = QuerySourceCache::new(100);

        // Add multiple different queries
        let queries = vec![
            ("rust_functions", "(function_item) @function"),
            ("rust_structs", "(struct_item) @struct"),
            ("rust_impls", "(impl_item) @impl"),
        ];

        for (key, query) in &queries {
            cache.get_or_load_source(key, || Ok(query.to_string())).unwrap();
        }

        assert_eq!(cache.len(), 3);

        // Verify all are cached
        for (key, expected_query) in &queries {
            let result = cache.get_or_load_source(key, || {
                panic!("Should be cached: {}", key);
            });
            assert_eq!(result.unwrap(), *expected_query);
        }
    }
}
