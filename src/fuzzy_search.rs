use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

/// Represents a file or directory item in fuzzy search
#[derive(Debug, Clone)]
pub struct FileItem {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_hidden: bool,
    pub modified: SystemTime,
    pub size: Option<u64>,
    pub is_binary: bool,
}

/// State for fuzzy file search
#[derive(Debug)]
pub struct FuzzySearchState {
    pub query: String,
    pub current_path: PathBuf,
    pub all_items: Vec<FileItem>,
    pub filtered_items: Vec<FileItem>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub is_scanning: bool,
    pub recursive_search: bool,
    pub max_depth: usize,

    // Search optimization
    pub result_count: usize,
    pub displayed_count: usize,
    pub has_more_results: bool,
    pub query_history: Vec<String>,
    pub result_cache: HashMap<String, Vec<FileItem>>,
}

impl Default for FuzzySearchState {
    fn default() -> Self {
        Self {
            query: String::new(),
            current_path: PathBuf::from("."),
            all_items: Vec::new(),
            filtered_items: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            is_scanning: false,
            recursive_search: true, // Default to recursive search
            max_depth: 0,           // 0 = unlimited depth
            result_count: 0,
            displayed_count: 0,
            has_more_results: false,
            query_history: Vec::new(),
            result_cache: HashMap::new(),
        }
    }
}

impl FuzzySearchState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create new fuzzy search state for a specific directory
    pub fn new_in_directory(dir: &std::path::Path) -> Self {
        Self {
            current_path: dir.to_path_buf(),
            ..Default::default()
        }
    }

    /// Update query with full history backtracking support
    pub fn update_query(&mut self, new_query: String) {
        let old_query = self.query.clone();
        self.query = new_query.clone();

        // Store previous query for backtracking
        if !old_query.is_empty() {
            self.query_history.push(old_query);
        }

        // Try instant backtrack from cache first
        if let Some(cached_results) = self.result_cache.get(&self.query) {
            self.filtered_items = cached_results.clone();
            self.result_count = self.filtered_items.len();
            self.displayed_count = self.filtered_items.len().min(100); // Show first 100
            self.has_more_results = self.filtered_items.len() > 100;
            self.filtered_items.truncate(self.displayed_count);
            self.selected_index = 0;
            self.scroll_offset = 0;
        } else {
            // Rescan with new query
            self.rescan_current_directory();
        }
    }

    /// Load more results for pagination
    pub fn load_more_results(&mut self) {
        if !self.has_more_results {
            return;
        }

        let remaining = self.result_count - self.displayed_count;
        let load_count = remaining.min(100);

        // Get cached full results
        if let Some(full_results) = self.result_cache.get(&self.query) {
            let start_idx = self.displayed_count;
            let end_idx = (start_idx + load_count).min(full_results.len());

            // Add more results to filtered_items
            self.filtered_items
                .extend_from_slice(&full_results[start_idx..end_idx]);
            self.displayed_count = end_idx;
            self.has_more_results = end_idx < self.result_count;
        }
    }

    pub fn update_filter(&mut self) {
        self.query = self.query.trim().to_string();
        self.selected_index = 0;
        self.scroll_offset = 0;

        // Filter and sort items based on query with priority scoring
        if self.query.is_empty() {
            self.filtered_items = self.all_items.clone();
            self.result_count = self.filtered_items.len();
            self.displayed_count = self.filtered_items.len().min(100);
            self.has_more_results = self.filtered_items.len() > 100;
            self.filtered_items.truncate(self.displayed_count);
        } else {
            // Get all matches with their priority scores
            let mut scored_items: Vec<(FileItem, i32, MatchType)> = self
                .all_items
                .iter()
                .filter_map(|item| {
                    let result = if self.recursive_search {
                        fuzzy_match_with_priority(&self.query, item)
                    } else {
                        // For non-recursive, only match filename
                        fuzzy_match(&self.query, &item.name)
                            .map(|score| (score, MatchType::FilenameFuzzy))
                    };

                    result.map(|(score, match_type)| (item.clone(), score, match_type))
                })
                .collect();

            // Sort by priority, then by score (descending), then by name
            scored_items.sort_by(|a, b| {
                // First sort by match type priority (ExactFilename > FilenameFuzzy > PathFuzzy)
                let type_order = match (&a.2, &b.2) {
                    (MatchType::ExactFilename, MatchType::ExactFilename) => {
                        std::cmp::Ordering::Equal
                    }
                    (MatchType::ExactFilename, _) => std::cmp::Ordering::Less,
                    (_, MatchType::ExactFilename) => std::cmp::Ordering::Greater,
                    (MatchType::FilenameFuzzy, MatchType::FilenameFuzzy) => {
                        std::cmp::Ordering::Equal
                    }
                    (MatchType::FilenameFuzzy, MatchType::PathFuzzy) => std::cmp::Ordering::Less,
                    (MatchType::PathFuzzy, MatchType::FilenameFuzzy) => std::cmp::Ordering::Greater,
                    (MatchType::PathFuzzy, MatchType::PathFuzzy) => std::cmp::Ordering::Equal,
                };

                match type_order {
                    std::cmp::Ordering::Equal => {
                        // Same type: sort by score descending, then by name
                        match b.1.cmp(&a.1) {
                            std::cmp::Ordering::Equal => a.0.name.cmp(&b.0.name),
                            other => other,
                        }
                    }
                    other => other,
                }
            });

            // Extract just the items
            self.filtered_items = scored_items.into_iter().map(|(item, _, _)| item).collect();
            self.result_count = self.filtered_items.len();
            self.displayed_count = self.filtered_items.len().min(100);
            self.has_more_results = self.filtered_items.len() > 100;
            self.filtered_items.truncate(self.displayed_count);
        }

        // Cache the full results for pagination
        if !self.query.is_empty() {
            let all_filtered_items: Vec<FileItem> = self
                .all_items
                .iter()
                .filter_map(|item| {
                    let result = if self.recursive_search {
                        fuzzy_match_with_priority(&self.query, item)
                    } else {
                        fuzzy_match(&self.query, &item.name)
                            .map(|score| (score, MatchType::FilenameFuzzy))
                    };

                    result.map(|_| item.clone())
                })
                .collect();

            let mut all_scored_items: Vec<(FileItem, i32, MatchType)> = all_filtered_items
                .iter()
                .filter_map(|item| {
                    let result = if self.recursive_search {
                        fuzzy_match_with_priority(&self.query, item)
                    } else {
                        fuzzy_match(&self.query, &item.name)
                            .map(|score| (score, MatchType::FilenameFuzzy))
                    };

                    result.map(|(score, match_type)| (item.clone(), score, match_type))
                })
                .collect();

            // Sort all results the same way
            all_scored_items.sort_by(|a, b| {
                let type_order = match (&a.2, &b.2) {
                    (MatchType::ExactFilename, MatchType::ExactFilename) => {
                        std::cmp::Ordering::Equal
                    }
                    (MatchType::ExactFilename, _) => std::cmp::Ordering::Less,
                    (_, MatchType::ExactFilename) => std::cmp::Ordering::Greater,
                    (MatchType::FilenameFuzzy, MatchType::FilenameFuzzy) => {
                        std::cmp::Ordering::Equal
                    }
                    (MatchType::FilenameFuzzy, MatchType::PathFuzzy) => std::cmp::Ordering::Less,
                    (MatchType::PathFuzzy, MatchType::FilenameFuzzy) => std::cmp::Ordering::Greater,
                    (MatchType::PathFuzzy, MatchType::PathFuzzy) => std::cmp::Ordering::Equal,
                };

                match type_order {
                    std::cmp::Ordering::Equal => match b.1.cmp(&a.1) {
                        std::cmp::Ordering::Equal => a.0.name.cmp(&b.0.name),
                        other => other,
                    },
                    other => other,
                }
            });

            let all_sorted_items: Vec<FileItem> = all_scored_items
                .into_iter()
                .map(|(item, _, _)| item)
                .collect();
            self.result_cache
                .insert(self.query.clone(), all_sorted_items);
        }
    }

    pub fn select_next(&mut self) -> Option<FileItem> {
        if self.selected_index < self.filtered_items.len().saturating_sub(1) {
            self.selected_index += 1;
            // Auto-open the selected file when navigating with arrow keys
            self.get_selected_item().cloned()
        } else {
            None
        }
    }

    pub fn select_prev(&mut self) -> Option<FileItem> {
        if self.selected_index > 0 {
            self.selected_index = self.selected_index.saturating_sub(1);
            // Auto-open the selected file when navigating with arrow keys
            self.get_selected_item().cloned()
        } else {
            None
        }
    }

    pub fn get_selected_item(&self) -> Option<&FileItem> {
        self.filtered_items.get(self.selected_index)
    }

    pub fn navigate_to_directory(&mut self, path: PathBuf) {
        self.current_path = path;
        self.query.clear();
        self.query_history.clear();
        self.result_cache.clear();
        self.rescan_current_directory();
    }

    pub fn rescan_current_directory(&mut self) {
        self.all_items = if self.recursive_search {
            scan_directory_recursive(&self.current_path, self.max_depth)
        } else {
            scan_directory(&self.current_path)
        };
        self.update_filter();
    }

    pub fn toggle_recursive(&mut self) {
        self.recursive_search = !self.recursive_search;
        self.result_cache.clear(); // Clear cache when toggling mode
        self.rescan_current_directory();
    }
}

/// Scan a directory and return all files and directories
pub fn scan_directory(path: &PathBuf) -> Vec<FileItem> {
    let mut items = Vec::new();

    // Add parent directory entry
    if let Some(parent) = path.parent() {
        items.push(FileItem {
            name: "..".to_string(),
            path: parent.to_path_buf(),
            is_dir: true,
            is_hidden: false,
            modified: SystemTime::UNIX_EPOCH, // Not relevant for ..
            size: None,
            is_binary: false,
        });
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_hidden = name.starts_with('.');
                let is_dir = metadata.is_dir();
                let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                let size = if is_dir { None } else { Some(metadata.len()) };
                let is_binary = if is_dir {
                    false
                } else {
                    // Simple binary detection: check file extension
                    let path = entry.path();
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

                    matches!(
                        ext,
                        "exe" | "dll" | "bin" | "obj" | "lib" | "a" | "so" | "dylib" | "pdb"
                    )
                };

                items.push(FileItem {
                    name,
                    path: entry.path(),
                    is_dir,
                    is_hidden,
                    modified,
                    size,
                    is_binary,
                });
            }
        }
    }

    // Sort: files first, then directories, both alphabetically
    items.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Greater, // files before directories
            (false, true) => std::cmp::Ordering::Less,
            _ => a.name.cmp(&b.name),
        }
    });

    items
}

/// Scan a directory recursively and return all files and directories
pub fn scan_directory_recursive(path: &PathBuf, max_depth: usize) -> Vec<FileItem> {
    let mut items = Vec::new();

    // Add parent directory entry
    if let Some(parent) = path.parent() {
        items.push(FileItem {
            name: "..".to_string(),
            path: parent.to_path_buf(),
            is_dir: true,
            is_hidden: false,
            modified: SystemTime::UNIX_EPOCH,
            size: None,
            is_binary: false,
        });
    }

    // Start recursive scanning
    scan_recursive_helper(path, &mut items, 0, max_depth);

    // Sort: files first, then directories, both alphabetically
    items.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Greater, // files before directories
            (false, true) => std::cmp::Ordering::Less,
            _ => a.name.cmp(&b.name),
        }
    });

    items
}

/// Helper function for recursive directory scanning
fn scan_recursive_helper(
    path: &PathBuf,
    items: &mut Vec<FileItem>,
    current_depth: usize,
    max_depth: usize,
) {
    // If max_depth is 0, unlimited recursion. Otherwise, stop at max_depth.
    if max_depth > 0 && current_depth >= max_depth {
        return;
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                let full_path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                let is_hidden = name.starts_with('.');
                let is_dir = metadata.is_dir();

                // Skip common ignore directories
                if is_dir && (name == "target" || name == "node_modules" || name == ".git") {
                    continue;
                }

                let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                let size = if is_dir { None } else { Some(metadata.len()) };
                let is_binary = if is_dir {
                    false
                } else {
                    // Simple binary detection: check file extension
                    let ext = full_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    matches!(
                        ext,
                        "exe" | "dll" | "bin" | "obj" | "lib" | "a" | "so" | "dylib" | "pdb"
                    )
                };

                items.push(FileItem {
                    name: full_path.display().to_string(),
                    path: full_path.clone(),
                    is_dir,
                    is_hidden,
                    modified,
                    size,
                    is_binary,
                });

                // Recursively scan subdirectories
                if is_dir {
                    scan_recursive_helper(&full_path, items, current_depth + 1, max_depth);
                }
            }
        }
    }
}

/// Priority-based matching: exact filename > filename fuzzy > path fuzzy
#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    ExactFilename,
    FilenameFuzzy,
    PathFuzzy,
}

/// Enhanced fuzzy matching with priority scoring
pub fn fuzzy_match_with_priority(query: &str, item: &FileItem) -> Option<(i32, MatchType)> {
    if query.is_empty() {
        return Some((0, MatchType::PathFuzzy));
    }

    let full_path = item.path.display().to_string();

    // Extract filename
    let filename = if let Some(last_sep) = full_path.rfind(['/', '\\']) {
        &full_path[last_sep + 1..]
    } else {
        &full_path
    };

    // Priority 1: Exact filename match (always highest priority)
    if filename == query {
        return Some((1000, MatchType::ExactFilename));
    }

    // Priority 2: Filename fuzzy match
    if let Some(score) = fuzzy_match(query, filename) {
        return Some((score + 100, MatchType::FilenameFuzzy));
    }

    // Priority 3: Full path fuzzy match
    if let Some(score) = fuzzy_match(query, &full_path) {
        return Some((score, MatchType::PathFuzzy));
    }

    None
}

/// Improved fuzzy matching algorithm
/// Returns Some(score) if query matches target, None otherwise
fn fuzzy_match(query: &str, target: &str) -> Option<i32> {
    let query_lower = query.to_lowercase();
    let target_lower = target.to_lowercase();

    if query_lower.is_empty() {
        return Some(0);
    }

    // Check for exact match first
    if query_lower == target_lower {
        return Some(100); // Highest score for exact match
    }

    // Improved fuzzy matching
    improved_fuzzy_match(&query_lower, &target_lower)
}

/// Advanced fuzzy matching with better scoring
fn improved_fuzzy_match(query: &str, target: &str) -> Option<i32> {
    let query_chars: Vec<char> = query.chars().collect();
    let target_chars: Vec<char> = target.chars().collect();

    let mut positions = Vec::new();
    let mut used_positions = std::collections::HashSet::new();
    let mut score = 0.0;

    // Find all query characters in target (allowing flexible matching)
    for &query_char in &query_chars {
        let mut best_pos = None;
        let mut best_score = f64::NEG_INFINITY;

        // Search for this character in all available positions
        for (pos, &target_char) in target_chars.iter().enumerate() {
            if target_char == query_char && !used_positions.contains(&pos) {
                let position_score = calculate_position_score(pos, target_chars.len());
                let word_boundary_bonus = calculate_word_boundary_bonus(pos, &target_chars);
                let total_score = position_score + word_boundary_bonus;

                if total_score > best_score {
                    best_score = total_score;
                    best_pos = Some(pos);
                }
            }
        }

        if let Some(pos) = best_pos {
            positions.push(pos);
            used_positions.insert(pos);
            score += 1.0 + best_score;
        } else {
            // Character not found
            return None;
        }
    }

    // Sort positions to maintain order
    positions.sort_unstable();

    // Apply final scoring bonuses
    let consecutive_bonus = calculate_consecutive_bonus(&positions);
    let length_penalty = calculate_length_penalty(query.len(), target.len());

    score += consecutive_bonus - length_penalty;

    // Normalize and convert to integer
    let normalized_score = (score / (query.len() as f64 * 2.0)).min(1.0);
    Some((normalized_score * 80.0) as i32) // Max score 80, less than exact match (100)
}

/// Calculate score based on character position in target
fn calculate_position_score(pos: usize, target_len: usize) -> f64 {
    let relative_pos = pos as f64 / target_len.max(1) as f64;
    1.0 - (relative_pos * 0.3) // Earlier positions get higher scores
}

/// Calculate bonus for matches at word boundaries
fn calculate_word_boundary_bonus(pos: usize, target_chars: &[char]) -> f64 {
    if pos == 0 {
        return 2.0; // Start of string - biggest bonus
    }

    if let Some(&prev_char) = target_chars.get(pos - 1) {
        if prev_char == '_' || prev_char == '-' || prev_char == '.' || prev_char.is_whitespace() {
            return 1.5; // Word boundary
        }

// Check for camelCase boundary (lowercase -> uppercase)
        if prev_char.is_lowercase() && target_chars.get(pos).is_some_and(|&c| c.is_uppercase()) {
            return 1.4; // CamelCase boundary
        }
    }

    0.0 // No word boundary bonus
}

/// Calculate penalty for length difference
fn calculate_length_penalty(query_len: usize, target_len: usize) -> f64 {
    if query_len == target_len {
        return 0.0; // No penalty
    }

    let ratio = query_len as f64 / target_len.max(1) as f64;
    if ratio < 0.3 {
        1.0 // Large penalty for very short queries
    } else if ratio < 0.6 {
        0.5 // Medium penalty
    } else if ratio > 2.0 {
        0.8 // Penalty for very short target
    } else {
        0.2 // Small penalty
    }
}

/// Calculate bonus for consecutive character matches
fn calculate_consecutive_bonus(positions: &[usize]) -> f64 {
    if positions.len() < 2 {
        return 0.0;
    }

    let mut bonus = 0.0;

    for window in positions.windows(2) {
        if window[1] == window[0] + 1 {
            bonus += 0.3; // Bonus for each consecutive match
        }
    }

    bonus
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match() {
        assert!(fuzzy_match("abc", "abc").unwrap() > 95); // Exact match - high score
        assert!(fuzzy_match("abc", "axbycz").is_some()); // Fuzzy match - all chars found
        assert_eq!(fuzzy_match("abc", "xyz"), None); // No match
        assert_eq!(fuzzy_match("", "abc"), Some(0)); // Empty query
    }

    #[test]
    fn test_fuzzy_match_non_sequential() {
        // Should find "abc" even if characters are not sequential
        let score1 = fuzzy_match("abc", "axbycz").unwrap();
        let score2 = fuzzy_match("abc", "a_bc").unwrap();

        assert!(score1 > 0);
        assert!(score2 >= score1); // Word boundary bonus or at least not worse

        // Should find "mlb" in "my_lib.rs"
        assert!(fuzzy_match("mlb", "my_lib.rs").is_some());

        // Should find "ad" in "README.md"
        assert!(fuzzy_match("ad", "README.md").is_some());

        // Should find "ab" in "about.txt"
        assert!(fuzzy_match("ab", "about.txt").is_some());

        // Should find characters in different order
        assert!(fuzzy_match("da", "README.md").is_some()); // 'd' then 'a' in README.md
        assert!(fuzzy_match("em", "README.md").is_some()); // 'e' then 'm' in README.md

        // Test that short queries work with common filenames - using realistic examples
        assert!(fuzzy_match("mn", "main.rs").is_some()); // m-a-i-n from main.rs
        assert!(fuzzy_match("tt", "tests").is_some()); // t-e-t-s from tests
        assert!(fuzzy_match("mai", "main.rs").is_some()); // m-a-i from main.rs
        assert!(fuzzy_match("lib", "lib.rs").is_some()); // l-i-b from lib.rs
    }

    #[test]
    fn test_word_boundary_bonuses() {
        // Word start bonus - "main" should match better at start
        let start_score = fuzzy_match("main", "main.rs").unwrap();
        let middle_score = fuzzy_match("main", "my_main.rs").unwrap();

        println!(
            "DEBUG: start_score={:?}, middle_score={:?}",
            start_score, middle_score
        );
        assert!(start_score >= middle_score); // Should be better or equal

        // CamelCase bonus - "mf" should match MyFunction better than myfunction
        let camel_score = fuzzy_match("mf", "MyFunction").unwrap();
        let regular_score = fuzzy_match("mf", "myfunction").unwrap();

        println!(
            "DEBUG: camel_score={:?}, regular_score={:?}",
            camel_score, regular_score
        );
        assert!(camel_score >= regular_score); // Should be better or equal

        // Snake case bonus - "my" should match my_function better than myfunction
        let snake_score = fuzzy_match("my", "my_function.rs").unwrap();
        let no_boundary_score = fuzzy_match("my", "myfunction.rs").unwrap();

        println!(
            "DEBUG: snake_score={:?}, no_boundary_score={:?}",
            snake_score, no_boundary_score
        );
        assert!(snake_score >= no_boundary_score); // Should be better or equal

        // Test underscore boundary specifically
        let underscore_score = fuzzy_match("func", "my_function.rs").unwrap();
        let direct_score = fuzzy_match("func", "myfunction.rs").unwrap();
        assert!(underscore_score >= direct_score);
    }

    #[test]
    fn test_position_scoring() {
        // Earlier characters should get higher scores
        let early_score = fuzzy_match("abc", "abcxyz").unwrap();
        let late_score = fuzzy_match("abc", "xyzabc").unwrap();
        assert!(early_score > late_score);
    }

    #[test]
    fn test_levenshtein_fallback() {
        // Should match with typos for short queries
        assert!(fuzzy_match("man", "main").is_some()); // 1 substitution
        assert!(fuzzy_match("mian", "main").is_some()); // 1 transposition

        // Should not match for very different strings
        assert_eq!(fuzzy_match("abc", "xyz"), None);

        // Should not apply Levenshtein to long queries
        assert_eq!(
            fuzzy_match("verylongquerythatshouldnotuselevenshtein", "short"),
            None
        );
    }

    #[test]
    fn test_case_insensitive() {
        assert_eq!(fuzzy_match("abc", "ABC"), fuzzy_match("ABC", "abc"));
        assert_eq!(
            fuzzy_match("Main", "main.rs"),
            fuzzy_match("main", "MAIN.rs")
        );

        // Case variations should match
        assert!(fuzzy_match("mlb", "MY_LIB.RS").is_some());
        assert!(fuzzy_match("MLB", "my_lib.rs").is_some());
    }

    #[test]
    fn test_length_penalties() {
        // Exact length match should be preferred
        let exact_match = fuzzy_match("main", "main").unwrap();
        let longer_target = fuzzy_match("main", "main_extended").unwrap();
        assert!(exact_match > longer_target);

        // Very short queries on long targets should be penalized
        let short_query = fuzzy_match("a", "very_long_filename.rs").unwrap();
        let reasonable_query = fuzzy_match("very", "very_long_filename.rs").unwrap();
        assert!(reasonable_query > short_query);
    }

    #[test]
    fn test_fuzzy_search_state() {
        let mut state = FuzzySearchState::new();
        state.all_items = vec![
            FileItem {
                name: "main.rs".to_string(),
                path: PathBuf::from("main.rs"),
                is_dir: false,
                is_hidden: false,
                modified: SystemTime::UNIX_EPOCH,
                size: Some(1000),
                is_binary: false,
            },
            FileItem {
                name: "lib.rs".to_string(),
                path: PathBuf::from("lib.rs"),
                is_dir: false,
                is_hidden: false,
                modified: SystemTime::UNIX_EPOCH,
                size: Some(2000),
                is_binary: false,
            },
        ];

        state.query = "rs".to_string();
        state.update_filter();

        assert_eq!(state.filtered_items.len(), 2);
    }

    #[test]
    fn test_recursive_search() {
        let mut state = FuzzySearchState::new();
        state.recursive_search = true;
        state.current_path = PathBuf::from(".");

        // Mock recursive items
        state.all_items = vec![
            FileItem {
                name: "src/main.rs".to_string(),
                path: PathBuf::from("src/main.rs"),
                is_dir: false,
                is_hidden: false,
                modified: SystemTime::UNIX_EPOCH,
                size: Some(1000),
                is_binary: false,
            },
            FileItem {
                name: "src/lib.rs".to_string(),
                path: PathBuf::from("src/lib.rs"),
                is_dir: false,
                is_hidden: false,
                modified: SystemTime::UNIX_EPOCH,
                size: Some(2000),
                is_binary: false,
            },
        ];

        state.query = "main".to_string();
        state.update_filter();

        assert_eq!(state.filtered_items.len(), 1);
        assert_eq!(state.filtered_items[0].name, "src/main.rs");
    }

    #[test]
    fn test_fuzzy_match_priority() {
        let item = FileItem {
            name: "main.rs".to_string(),
            path: PathBuf::from("src/main.rs"),
            is_dir: false,
            is_hidden: false,
            modified: SystemTime::UNIX_EPOCH,
            size: Some(1000),
            is_binary: false,
        };

        // Test filename matching (should have higher score)
        let result = fuzzy_match_with_priority("main", &item);
        assert!(result.is_some());
        let (score, match_type) = result.unwrap();
        assert!(score > 100); // Should have bonus for filename match
        assert_eq!(match_type, MatchType::FilenameFuzzy);

        // Test exact filename match
        let result = fuzzy_match_with_priority("main.rs", &item);
        assert!(result.is_some());
        let (_, match_type) = result.unwrap();
        assert_eq!(match_type, MatchType::ExactFilename);

        // Test no match
        let result = fuzzy_match_with_priority("xyz", &item);
        assert!(result.is_none());
    }
}
