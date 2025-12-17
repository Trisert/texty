use std::collections::HashMap;
use std::fs;
use tree_sitter::Query;

/// Loads and caches tree-sitter queries from runtime files
#[derive(Debug)]
pub struct QueryLoader {
    cache: HashMap<String, Query>,
}

impl QueryLoader {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Load a query from a file path, with fallback to embedded query if file doesn't exist
    pub fn load_query(
        &mut self,
        language: tree_sitter::Language,
        path: &str,
        fallback_query: Option<&str>,
    ) -> Result<&Query, Box<dyn std::error::Error>> {
        let cache_key = format!("{:?}_{}", language, path);

        if !self.cache.contains_key(&cache_key) {
            let query_source = match fs::read_to_string(path) {
                Ok(content) => content,
                Err(_) => {
                    // Fallback to embedded query if file doesn't exist
                    fallback_query
                        .ok_or_else(|| {
                            format!("Query file not found and no fallback provided: {}", path)
                        })?
                        .to_string()
                }
            };

            let query = Query::new(language, &query_source)?;
            self.cache.insert(cache_key.clone(), query);
        }

        Ok(self.cache.get(&cache_key).unwrap())
    }

    /// Clear the query cache (useful for memory management or reloading)
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

impl Default for QueryLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter_rust::language as rust_language;

    #[test]
    fn test_query_loader_new() {
        let loader = QueryLoader::new();
        assert_eq!(loader.cache_size(), 0);
    }

    #[test]
    fn test_load_query_with_fallback() {
        let mut loader = QueryLoader::new();
        let language = rust_language();
        let fallback_query = "(function_item) @function";

        // This should use the fallback since the path doesn't exist
        let query = loader
            .load_query(language, "nonexistent/path.scm", Some(fallback_query))
            .unwrap();
        assert!(query.capture_names().contains(&"function".to_string()));
    }

    #[test]
    fn test_cache_functionality() {
        let mut loader = QueryLoader::new();
        let language = rust_language();
        let fallback_query = "(function_item) @function";

        // First load
        loader
            .load_query(language, "test/path.scm", Some(fallback_query))
            .unwrap();
        assert_eq!(loader.cache_size(), 1);

        // Second load should use cache
        loader
            .load_query(language, "test/path.scm", Some(fallback_query))
            .unwrap();
        assert_eq!(loader.cache_size(), 1);
    }
}
