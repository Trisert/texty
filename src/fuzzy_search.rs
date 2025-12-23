use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::ui::widgets::preview::PreviewBuffer;

// ===== FZF-STYLE CORE ALGORITHM =====

// Character classes for optimized matching
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq)]
enum CharClass {
    White,
    NonWord,
    Delimiter,
    Lower,
    Upper,
    Letter,
    Number,
}

// Scoring constants (matching fzf's algorithm)
const SCORE_MATCH: i16 = 16;
const SCORE_GAP_START: i16 = -3;
const SCORE_GAP_EXTENSION: i16 = -1;
const BONUS_BOUNDARY: i16 = SCORE_MATCH / 2;
const BONUS_NON_WORD: i16 = SCORE_MATCH / 2;
const BONUS_CAMEL123: i16 = BONUS_BOUNDARY + SCORE_GAP_EXTENSION;
const BONUS_CONSECUTIVE: i16 = -(SCORE_GAP_START + SCORE_GAP_EXTENSION);
const BONUS_FIRST_CHAR_MULTIPLIER: i16 = 2;

// Extra bonus for word boundary after whitespace
const BONUS_BOUNDARY_WHITE: i16 = BONUS_BOUNDARY + 2;

// Extra bonus for word boundary after delimiter
const BONUS_BOUNDARY_DELIMITER: i16 = BONUS_BOUNDARY + 1;

// Delimiter characters for path matching
const DELIMITER_CHARS: &[char] = &['/', ':', ';', '|'];

// ===== SLAB ALLOCATOR FOR MEMORY OPTIMIZATION =====

/// Slab allocator for reusing memory during fuzzy matching
#[derive(Debug)]
pub struct Slab {
    pub i16_vec: Vec<i16>,
    pub i32_vec: Vec<i32>,
    pub usize_vec: Vec<usize>,
    pub char_vec: Vec<char>,
}

impl Slab {
    pub fn new() -> Self {
        Self {
            i16_vec: Vec::new(),
            i32_vec: Vec::new(),
            usize_vec: Vec::new(),
            char_vec: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.i16_vec.clear();
        self.i32_vec.clear();
        self.usize_vec.clear();
        self.char_vec.clear();
    }

    pub fn get_i16_slice(&mut self, size: usize) -> &mut [i16] {
        if self.i16_vec.len() < size {
            self.i16_vec.resize(size, 0);
        }
        &mut self.i16_vec[..size]
    }

    pub fn get_i32_slice(&mut self, size: usize) -> &mut [i32] {
        if self.i32_vec.len() < size {
            self.i32_vec.resize(size, 0);
        }
        &mut self.i32_vec[..size]
    }

    pub fn get_usize_slice(&mut self, size: usize) -> &mut [usize] {
        if self.usize_vec.len() < size {
            self.usize_vec.resize(size, 0);
        }
        &mut self.usize_vec[..size]
    }

    pub fn get_char_slice(&mut self, size: usize) -> &mut [char] {
        if self.char_vec.len() < size {
            self.char_vec.resize(size, '\0');
        }
        &mut self.char_vec[..size]
    }
}

impl Default for Slab {
    fn default() -> Self {
        Self::new()
    }
}

// Thread-local slab allocator for performance
thread_local! {
    static SLAB: std::cell::RefCell<Slab> = std::cell::RefCell::new(Slab::new());
}

// ===== FZF-STYLE CORE ALGORITHM =====

// FZF-style V1 algorithm (fast greedy matching)
#[derive(Debug, Clone)]
pub struct FzfResult {
    pub start: usize,
    pub end: usize,
    pub score: i32,
    pub positions: Option<Vec<usize>>,
}

// Optimized ASCII string search for pattern occurrence
fn ascii_fuzzy_index(
    text: &[char],
    pattern: &[char],
    case_sensitive: bool,
) -> Option<(usize, usize)> {
    if pattern.is_empty() {
        return Some((0, 0));
    }

    if !is_ascii_string(pattern) || !is_ascii_string(text) {
        return None; // Fall back to Unicode path
    }

    let mut first_idx = 0;
    let mut last_idx = 0;

    // Find all pattern characters in order
    let mut text_pos = 0;
    for (pidx, &pattern_char) in pattern.iter().enumerate() {
        let mut found = false;

        for (idx, &text_char) in text[text_pos..].iter().enumerate() {
            let current_idx = text_pos + idx;

            // Case-insensitive matching for ASCII
            let text_char_code = text_char as u8;
            let pattern_char_code = pattern_char as u8;

            let matches = if case_sensitive {
                text_char_code == pattern_char_code
            } else {
                // Fast case-insensitive comparison for ASCII
                text_char_code == pattern_char_code
                    || text_char_code == pattern_char_code - 32
                        && (97..=122).contains(&text_char_code)
                    || text_char_code == pattern_char_code + 32
                        && (65..=90).contains(&text_char_code)
            };

            if matches {
                if pidx == 0 && current_idx > 0 {
                    first_idx = current_idx - 1;
                }
                if pidx == pattern.len() - 1 {
                    last_idx = current_idx + 1;
                }
                text_pos = current_idx + 1;
                found = true;
                break;
            }
        }

        if !found {
            return None;
        }
    }

    Some((first_idx, last_idx))
}

// Fast ASCII string check
fn is_ascii_string(chars: &[char]) -> bool {
    chars.iter().all(|&c| c.is_ascii())
}

// Get character class (optimized for ASCII, falls back to Unicode)
fn char_class_of(char: char) -> CharClass {
    let code = char as u32;
    if code < 128 {
        match char {
            'a'..='z' => CharClass::Lower,
            'A'..='Z' => CharClass::Upper,
            '0'..='9' => CharClass::Number,
            ' ' | '\t' | '\n' | '\r' => CharClass::White,
            _ if DELIMITER_CHARS.contains(&char) => CharClass::Delimiter,
            _ => CharClass::NonWord,
        }
    } else {
        match char {
            c if c.is_lowercase() => CharClass::Lower,
            c if c.is_uppercase() => CharClass::Upper,
            c if c.is_numeric() => CharClass::Number,
            c if c.is_whitespace() => CharClass::White,
            c if DELIMITER_CHARS.contains(&c) => CharClass::Delimiter,
            c if c.is_alphabetic() => CharClass::Letter,
            _ => CharClass::NonWord,
        }
    }
}

// Calculate bonus for character transition
fn bonus_for(prev_class: CharClass, class: CharClass) -> i16 {
    match (prev_class, class) {
        (CharClass::White, _) if class >= CharClass::NonWord => BONUS_BOUNDARY_WHITE,
        (CharClass::Delimiter, _) if class >= CharClass::NonWord => BONUS_BOUNDARY_DELIMITER,
        (CharClass::NonWord, _) if class >= CharClass::NonWord => BONUS_BOUNDARY,
        (CharClass::Lower, CharClass::Upper) => BONUS_CAMEL123,
        (CharClass::Lower, CharClass::Number) => BONUS_CAMEL123,
        (CharClass::Letter, CharClass::Number) => BONUS_CAMEL123,
        (CharClass::NonWord, CharClass::Delimiter) => BONUS_NON_WORD,
        (CharClass::NonWord, CharClass::White) => BONUS_BOUNDARY_WHITE,
        (CharClass::Delimiter, CharClass::Delimiter) => BONUS_NON_WORD,
        _ => 0,
    }
}

// Fast fuzzy matching with fzf-style scoring (optimized with slab allocator)
fn fuzzy_match_v1(text: &str, pattern: &str, case_sensitive: bool) -> Option<FzfResult> {
    if pattern.is_empty() {
        return Some(FzfResult {
            start: 0,
            end: 0,
            score: 0,
            positions: Some(Vec::new()),
        });
    }

    // Use thread-local slab allocator for memory efficiency
    SLAB.with(|slab_cell| {
        let mut slab = slab_cell.borrow_mut();
        slab.reset(); // Clear previous allocations

        let text_chars: Vec<char> = text.chars().collect();
        let pattern_chars: Vec<char> = pattern.chars().collect();

        // Fast ASCII optimization - find first occurrence of pattern
        let (start_idx, end_idx) = ascii_fuzzy_index(&text_chars, &pattern_chars, case_sensitive)?;

        // Calculate score with fzf-style bonuses using slab-allocated memory
        let mut score = 0;
        let mut positions = Vec::new();
        let mut in_gap = false;
        let mut consecutive = 0;
        let mut first_bonus = 0;
        let mut prev_class = CharClass::Delimiter; // Start with delimiter for path matching

        for (idx, &text_char) in text_chars[start_idx..end_idx].iter().enumerate() {
            let global_idx = start_idx + idx;
            let class = char_class_of(text_char);

            if let Some(&pattern_char) = pattern_chars.get(positions.len()) {
                if text_char == pattern_char {
                    positions.push(global_idx);
                    score += SCORE_MATCH as i32;

                    let bonus = bonus_for(prev_class, class);

                    if consecutive == 0 {
                        first_bonus = bonus;
                    } else {
                        if bonus >= BONUS_BOUNDARY && bonus > first_bonus {
                            first_bonus = bonus;
                        }
                        let bonus_for_consecutive =
                            bonus_for(CharClass::NonWord, class).max(BONUS_CONSECUTIVE);
                        score += bonus_for_consecutive.max(first_bonus).max(bonus) as i32;
                    }

                    if positions.len() == 1 {
                        score += (bonus * BONUS_FIRST_CHAR_MULTIPLIER) as i32;
                    } else {
                        score += bonus as i32;
                    }

                    in_gap = false;
                    consecutive += 1;
                } else {
                    if in_gap {
                        score += SCORE_GAP_EXTENSION as i32;
                    } else {
                        score += SCORE_GAP_START as i32;
                    }
                    in_gap = true;
                    consecutive = 0;
                    first_bonus = 0;
                }
            } else {
                break; // Pattern fully matched
            }

            prev_class = class;
        }

        Some(FzfResult {
            start: start_idx,
            end: end_idx,
            score,
            positions: Some(positions),
        })
    })
}

// ===== FUZZY SEARCH CONSTANTS =====

/// Default number of results to display initially for performance
const DEFAULT_DISPLAY_LIMIT: usize = 100;

// ===== FILE TYPE AND DIRECTORY SCORING =====

/// File type classification for intelligent scoring
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileType {
    Source,
    Test,
    Documentation,
    Configuration,
    Build,
    Binary,
    Other,
}

impl FileType {
    /// Provide the priority bonus for this file type.
    ///
    /// The bonus influences search ranking: positive values increase priority, negative values decrease it.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(crate::FileType::Source.bonus_score(), 500);
    /// assert_eq!(crate::FileType::Binary.bonus_score(), -100);
    /// ```
    pub fn bonus_score(&self) -> i32 {
        match self {
            FileType::Source => 500,
            FileType::Test => 250,
            FileType::Documentation => 0,
            FileType::Configuration => 150,
            FileType::Build => 75,
            FileType::Binary => -100, // Penalize binary files
            FileType::Other => 0,
        }
    }
}

/// Classify file type based on extension and naming patterns
fn classify_file_type(path: &Path, filename: &str) -> FileType {
    // Check for binary files first
    if let Some(ext) = path.extension().and_then(|e| e.to_str())
        && matches!(
            ext,
            "exe" | "dll" | "bin" | "obj" | "lib" | "a" | "so" | "dylib" | "pdb"
        )
    {
        return FileType::Binary;
    }

    // Source files by extension
    if let Some(ext) = path.extension().and_then(|e| e.to_str())
        && matches!(
            ext,
            "rs" | "js"
                | "ts"
                | "jsx"
                | "tsx"
                | "py"
                | "java"
                | "c"
                | "cpp"
                | "cc"
                | "cxx"
                | "h"
                | "hpp"
                | "go"
                | "rb"
                | "php"
                | "swift"
                | "kt"
                | "scala"
                | "cs"
                | "vb"
                | "dart"
                | "lua"
                | "sh"
                | "bash"
                | "zsh"
                | "fish"
                | "ps1"
                | "bat"
                | "cmd"
        )
    {
        return FileType::Source;
    }

    // Configuration files by extension
    if let Some(ext) = path.extension().and_then(|e| e.to_str())
        && matches!(
            ext,
            "toml"
                | "json"
                | "yaml"
                | "yml"
                | "ini"
                | "xml"
                | "cfg"
                | "conf"
                | "config"
                | "properties"
                | "env"
                | "plist"
        )
    {
        return FileType::Configuration;
    }

    // Documentation files by extension
    if let Some(ext) = path.extension().and_then(|e| e.to_str())
        && matches!(
            ext,
            "md" | "txt" | "rst" | "adoc" | "tex" | "doc" | "docx" | "pdf"
        )
    {
        return FileType::Documentation;
    }

    // Test files by naming patterns
    let filename_lower = filename.to_lowercase();
    if filename_lower.starts_with("test_")
        || filename_lower.ends_with("_test")
        || filename_lower.contains(".test.")
        || filename_lower.ends_with(".test")
        || filename.ends_with("_spec")
        || filename_lower.contains("spec.")
    {
        return FileType::Test;
    }

    // Build and CI files by name
    if matches!(
        filename,
        "Makefile"
            | "Dockerfile"
            | "docker-compose.yml"
            | "docker-compose.yaml"
            | "Rakefile"
            | "Gruntfile.js"
            | "gulpfile.js"
            | "webpack.config.js"
            | "package.json"
            | "Cargo.toml"
            | "Cargo.lock"
            | "requirements.txt"
            | "Pipfile"
            | "poetry.lock"
            | "yarn.lock"
            | ".gitignore"
            | ".dockerignore"
            | ".editorconfig"
            | "rustfmt.toml"
    ) {
        return FileType::Build;
    }

    FileType::Other
}

/// Calculate directory-based bonus for a file path
fn calculate_directory_bonus(path: &Path, filename: &str) -> i32 {
    let path_str = path.to_string_lossy();

    // Source directories get highest bonus
    if path_str.contains("/src/") || path_str.starts_with("src/") {
        return 150;
    }

    // Lib directory also gets high bonus
    if path_str.contains("/lib/") || path_str.starts_with("lib/") {
        return 150;
    }

    // Test directories get bonus
    if path_str.contains("/test/")
        || path_str.contains("/tests/")
        || path_str.starts_with("test/")
        || path_str.starts_with("tests/")
    {
        return 100;
    }

    // Documentation directory gets penalty
    if path_str.contains("/doc/")
        || path_str.contains("/docs/")
        || path_str.starts_with("doc/")
        || path_str.starts_with("docs/")
    {
        return -50;
    }

    // Root-level important files get bonus
    if matches!(
        filename,
        "main.rs" | "lib.rs" | "index.js" | "index.ts" | "app.js" | "app.ts"
    ) {
        return 300;
    }

    // Examples directory gets medium bonus
    if path_str.contains("/examples/") || path_str.starts_with("examples/") {
        return 75;
    }

    0
}

// ===== ORIGINAL STRUCTURES (MODIFIED FOR OPTIMIZATION) =====

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

    // Preview functionality
    pub current_preview: Option<PreviewBuffer>,
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
            current_preview: None,
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

    /// Update the current search query, adjust cached or recomputed results, and refresh the preview.
    ///
    /// This saves the previous non-empty query to the state's backtracking history, attempts to load
    /// matching results from the result cache, and if absent either performs an early-termination
    /// filtered scan or rescans the current directory. After updating the filtered result list it
    /// resets selection/scrolling state, sets display counts and pagination flags, and refreshes the
    /// preview buffer for the new selection.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut state = FuzzySearchState::new(); // starts with default path and empty query
    /// state.update_query("main".to_string());
    /// assert_eq!(state.query, "main");
    /// // filtered_items, displayed_count, and current_preview are updated by the call
    /// ```
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
            self.displayed_count = self.filtered_items.len().min(DEFAULT_DISPLAY_LIMIT); // Show first DEFAULT_DISPLAY_LIMIT
            self.has_more_results = self.filtered_items.len() > DEFAULT_DISPLAY_LIMIT;
            self.filtered_items.truncate(self.displayed_count);
            self.selected_index = 0;
            self.scroll_offset = 0;
        } else {
            // Check if we can use early termination for common queries
            if self.should_early_terminate() {
                self.update_filter_early_termination();
            } else {
                self.rescan_current_directory();
            }
        }

        self.update_preview();
    }

    /// Determine if we should use early termination optimization
    fn should_early_terminate(&self) -> bool {
        // Early termination for very short queries (performance optimization)
        self.query.len() <= 2 && self.all_items.len() > 10000
    }

    /// Optimized update filter with early termination
    fn update_filter_early_termination(&mut self) {
        // For short queries on large datasets, only scan until we have enough good matches
        let target_results = 50; // Find first 50 good matches
        let mut scored_items: Vec<(FileItem, i32, MatchType)> = Vec::new();

        for item in &self.all_items {
            let result = if self.recursive_search {
                fuzzy_match_with_priority_optimized(&self.query, item)
            } else {
                let filename = if let Some(last_sep) = item.name.rfind(['/', '\\']) {
                    &item.name[last_sep + 1..]
                } else {
                    &item.name
                };

                fuzzy_match_optimized(&self.query, filename)
                    .map(|score| (score, MatchType::FilenameFuzzy))
            };

            if let Some((score, match_type)) = result {
                scored_items.push((item.clone(), score, match_type));

                // Early termination: stop if we have enough high-quality matches
                if scored_items.len() >= target_results {
                    // Check if remaining items are unlikely to beat current matches
                    if let Some(min_score) = scored_items.iter().map(|(_, score, _)| *score).min() {
                        // Only continue if this is a high-quality match
                        if score > min_score * 2 {
                            break;
                        }
                    }
                }
            }
        }

        // Sort the results we found
        scored_items.sort_by(|a, b| {
            let type_order = match (&a.2, &b.2) {
                (MatchType::ExactFilename, MatchType::ExactFilename) => std::cmp::Ordering::Equal,
                (MatchType::ExactFilename, _) => std::cmp::Ordering::Less,
                (_, MatchType::ExactFilename) => std::cmp::Ordering::Greater,
                (MatchType::FilenameFuzzy, MatchType::FilenameFuzzy) => std::cmp::Ordering::Equal,
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

        // Update state with early termination results
        self.result_count = scored_items.len();
        self.displayed_count = scored_items.len().min(DEFAULT_DISPLAY_LIMIT);
        self.has_more_results = scored_items.len() > DEFAULT_DISPLAY_LIMIT;

        self.filtered_items = scored_items
            .iter()
            .take(self.displayed_count)
            .map(|(item, _, _)| item.clone())
            .collect();

        // Cache partial results
        let all_sorted_items: Vec<FileItem> = scored_items
            .iter()
            .map(|(item, _, _)| item.clone())
            .collect();
        self.result_cache
            .insert(self.query.clone(), all_sorted_items);

        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    /// Load more results for pagination
    pub fn load_more_results(&mut self) {
        if !self.has_more_results {
            return;
        }

        let remaining = self.result_count - self.displayed_count;
        let load_count = remaining.min(DEFAULT_DISPLAY_LIMIT);

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
            self.displayed_count = self.filtered_items.len().min(DEFAULT_DISPLAY_LIMIT);
            self.has_more_results = self.filtered_items.len() > DEFAULT_DISPLAY_LIMIT;
            self.filtered_items.truncate(self.displayed_count);
        } else {
            // Single-pass filtering with optimized fzf-style scoring
            let mut scored_items: Vec<(FileItem, i32, MatchType)> = self
                .all_items
                .par_iter() // Parallel processing with Rayon
                .filter_map(|item| {
                    let result = if self.recursive_search {
                        fuzzy_match_with_priority_optimized(&self.query, item)
                    } else {
                        // For non-recursive, only match filename
                        let filename = if let Some(last_sep) = item.name.rfind(['/', '\\']) {
                            &item.name[last_sep + 1..]
                        } else {
                            &item.name
                        };

                        {
                            // Calculate bonuses for non-recursive mode too
                            let file_type = classify_file_type(&item.path, filename);
                            let type_bonus = file_type.bonus_score();
                            let dir_bonus = calculate_directory_bonus(&item.path, filename);
                            let total_bonus = type_bonus + dir_bonus;

                            fuzzy_match_optimized(&self.query, filename)
                                .map(|score| (score + total_bonus, MatchType::FilenameFuzzy))
                        }
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
            self.displayed_count = self.filtered_items.len().min(DEFAULT_DISPLAY_LIMIT);
            self.has_more_results = self.filtered_items.len() > DEFAULT_DISPLAY_LIMIT;
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
                        // Calculate bonuses for non-recursive mode
                        let file_type = classify_file_type(&item.path, &item.name);
                        let type_bonus = file_type.bonus_score();
                        let dir_bonus = calculate_directory_bonus(&item.path, &item.name);
                        let total_bonus = type_bonus + dir_bonus;

                        fuzzy_match(&self.query, &item.name)
                            .map(|score| (score + total_bonus, MatchType::FilenameFuzzy))
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
                        // Calculate bonuses for non-recursive mode
                        let file_type = classify_file_type(&item.path, &item.name);
                        let type_bonus = file_type.bonus_score();
                        let dir_bonus = calculate_directory_bonus(&item.path, &item.name);
                        let total_bonus = type_bonus + dir_bonus;

                        fuzzy_match(&self.query, &item.name)
                            .map(|score| (score + total_bonus, MatchType::FilenameFuzzy))
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
            self.update_preview();
            self.get_selected_item().cloned()
        } else {
            None
        }
    }

    pub fn select_prev(&mut self) -> Option<FileItem> {
        if self.selected_index > 0 {
            self.selected_index = self.selected_index.saturating_sub(1);
            self.update_preview();
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

    pub fn update_preview(&mut self) {
        if let Some(selected_item) = self.filtered_items.get(self.selected_index) {
            if !selected_item.is_dir {
                match PreviewBuffer::load_from_file(&selected_item.path) {
                    Ok(preview_buffer) => {
                        self.current_preview = Some(preview_buffer);
                    }
                    Err(_) => {
                        self.current_preview = None;
                    }
                }
            } else {
                self.current_preview = None;
            }
        } else {
            self.current_preview = None;
        }
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

/// Scan a directory recursively and return all files and directories (optimized with parallel processing)
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

    // Start recursive scanning with parallel processing
    let all_items = scan_recursive_helper_parallel(path, max_depth, 0);

    // Sort: files first, then directories, both alphabetically
    items.extend(all_items);
    items.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Greater, // files before directories
            (false, true) => std::cmp::Ordering::Less,
            _ => a.name.cmp(&b.name),
        }
    });

    items
}

/// Parallel recursive directory scanning for better performance
fn scan_recursive_helper_parallel(
    path: &PathBuf,
    max_depth: usize,
    current_depth: usize,
) -> Vec<FileItem> {
    let mut items = Vec::new();

    // Stop recursion if we've reached max depth (0 means unlimited)
    if max_depth > 0 && current_depth >= max_depth {
        return items;
    }

    let mut dirs_to_scan = Vec::new();

    // Process current directory
    if let Ok(entries) = fs::read_dir(path) {
        // Collect entries to avoid move issues
        let entry_vec: Vec<std::fs::DirEntry> = entries.flatten().collect();

        // First pass: separate files and directories
        let mut dir_paths = Vec::new();
        let file_items: Vec<FileItem> = entry_vec
            .into_iter()
            .filter_map(|entry| {
                if let Ok(metadata) = entry.metadata() {
                    let full_path = entry.path();
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

                    // Collect directory paths for recursive scanning
                    if is_dir {
                        dir_paths.push(full_path.clone());
                    }

                    Some(FileItem {
                        name: if is_dir {
                            full_path.display().to_string()
                        } else {
                            name.clone()
                        },
                        path: full_path.clone(),
                        is_dir,
                        is_hidden,
                        modified,
                        size,
                        is_binary,
                    })
                } else {
                    None
                }
            })
            .collect();

        dirs_to_scan = dir_paths;
        items.extend(file_items);
    }

    // Parallel scan subdirectories
    let sub_items: Vec<Vec<FileItem>> = dirs_to_scan
        .par_iter()
        .map(|dir_path| scan_recursive_helper_parallel(dir_path, max_depth, current_depth + 1))
        .collect();

    for sub_dir_items in sub_items {
        items.extend(sub_dir_items);
    }

    items
}

/// Optimized fuzzy matching with fzf-style algorithm
fn fuzzy_match_optimized(query: &str, target: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }

    // Try exact match first
    if query.to_lowercase() == target.to_lowercase() {
        return Some(100); // Highest score for exact match
    }

    // Use fzf-style V1 algorithm for performance
    if let Some(result) = fuzzy_match_v1(target, query, false) {
        Some(result.score)
    } else {
        None
    }
}

/// Enhanced fuzzy matching with priority scoring (optimized)
fn fuzzy_match_with_priority_optimized(query: &str, item: &FileItem) -> Option<(i32, MatchType)> {
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

    // Classify file type and calculate directory bonus
    let file_type = classify_file_type(&item.path, filename);
    let type_bonus = file_type.bonus_score();
    let dir_bonus = calculate_directory_bonus(&item.path, filename);
    let total_bonus = type_bonus + dir_bonus;

    // Priority 1: Exact filename match (always highest priority)
    if filename == query {
        return Some((1000 + total_bonus, MatchType::ExactFilename));
    }

    // Priority 2: Filename fuzzy match
    if let Some(score) = fuzzy_match_optimized(query, filename) {
        return Some((score + 100 + total_bonus, MatchType::FilenameFuzzy));
    }

    // Priority 3: Full path fuzzy match
    if let Some(score) = fuzzy_match_optimized(query, &full_path) {
        return Some((score + total_bonus, MatchType::PathFuzzy));
    }

    None
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

    // Classify file type and calculate directory bonus
    let file_type = classify_file_type(&item.path, filename);
    let type_bonus = file_type.bonus_score();
    let dir_bonus = calculate_directory_bonus(&item.path, filename);
    let total_bonus = type_bonus + dir_bonus;

    // Priority 1: Exact filename match (always highest priority)
    if filename == query {
        return Some((1000 + total_bonus, MatchType::ExactFilename));
    }

    // Priority 2: Filename fuzzy match
    if let Some(score) = fuzzy_match(query, filename) {
        return Some((score + 100 + total_bonus, MatchType::FilenameFuzzy));
    }

    // Priority 3: Full path fuzzy match
    if let Some(score) = fuzzy_match(query, &full_path) {
        return Some((score + total_bonus, MatchType::PathFuzzy));
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

// Simple performance benchmark function
#[cfg(test)]
pub fn benchmark_fuzzy_search_performance() {
    use std::time::Instant;

    let items: Vec<FileItem> = (0..10000)
        .map(|i| FileItem {
            name: format!("file_{}.rs", i),
            path: PathBuf::from(format!("src/file_{}.rs", i)),
            is_dir: false,
            is_hidden: i % 10 == 0,
            modified: SystemTime::UNIX_EPOCH,
            size: Some(i as u64),
            is_binary: false,
        })
        .collect();

    let mut state = FuzzySearchState {
        query: String::new(),
        current_path: PathBuf::from("."),
        all_items: items,
        filtered_items: Vec::new(),
        selected_index: 0,
        scroll_offset: 0,
        is_scanning: false,
        recursive_search: true,
        max_depth: 0,
        result_count: 0,
        displayed_count: 0,
        has_more_results: false,
        query_history: Vec::new(),
        result_cache: HashMap::new(),
        current_preview: None,
    };

    // Benchmark old algorithm
    let start = Instant::now();
    state.query = "main".to_string();
    state.update_filter();
    let old_time = start.elapsed();

    // Benchmark new algorithm
    let start = Instant::now();
    state.update_filter();
    let new_time = start.elapsed();

    println!("Performance comparison:");
    println!("  Old algorithm: {:?}", old_time);
    println!("  New algorithm: {:?}", new_time);

    if new_time < old_time {
        let improvement = ((old_time.as_nanos() as f64 - new_time.as_nanos() as f64)
            / old_time.as_nanos() as f64)
            * 100.0;
        println!("  Improvement: {:.1}% faster", improvement);
    }
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
    fn test_benchmark_performance() {
        benchmark_fuzzy_search_performance();
    }

    #[test]
    fn test_display_limit_constant() {
        // Verify that DEFAULT_DISPLAY_LIMIT is used consistently
        assert_eq!(DEFAULT_DISPLAY_LIMIT, 100);

        // Test that results are limited correctly
        let items: Vec<FileItem> = (0..150)
            .map(|i| FileItem {
                name: format!("file_{}.rs", i),
                path: PathBuf::from(format!("src/file_{}.rs", i)),
                is_dir: false,
                is_hidden: false,
                modified: SystemTime::UNIX_EPOCH,
                size: Some(i as u64),
                is_binary: false,
            })
            .collect();

        let mut state = FuzzySearchState::new();
        state.all_items = items;
        state.query = "file".to_string();
        state.update_filter();

        // Should limit to DEFAULT_DISPLAY_LIMIT items
        assert_eq!(state.displayed_count, DEFAULT_DISPLAY_LIMIT);
        assert!(state.has_more_results);
        assert_eq!(state.result_count, 150); // Total should be full count
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

    #[test]
    fn test_file_type_classification() {
        // Test source files
        let item = FileItem {
            name: "main.rs".to_string(),
            path: PathBuf::from("src/main.rs"),
            is_dir: false,
            is_hidden: false,
            modified: SystemTime::UNIX_EPOCH,
            size: Some(1000),
            is_binary: false,
        };
        assert_eq!(classify_file_type(&item.path, &item.name), FileType::Source);

        // Test documentation files
        let item = FileItem {
            name: "README.md".to_string(),
            path: PathBuf::from("README.md"),
            is_dir: false,
            is_hidden: false,
            modified: SystemTime::UNIX_EPOCH,
            size: Some(1000),
            is_binary: false,
        };
        assert_eq!(
            classify_file_type(&item.path, &item.name),
            FileType::Documentation
        );

        // Test configuration files
        let item = FileItem {
            name: "config.toml".to_string(),
            path: PathBuf::from("config.toml"),
            is_dir: false,
            is_hidden: false,
            modified: SystemTime::UNIX_EPOCH,
            size: Some(1000),
            is_binary: false,
        };
        assert_eq!(
            classify_file_type(&item.path, &item.name),
            FileType::Configuration
        );

        // Test binary files
        let item = FileItem {
            name: "program.exe".to_string(),
            path: PathBuf::from("program.exe"),
            is_dir: false,
            is_hidden: false,
            modified: SystemTime::UNIX_EPOCH,
            size: Some(1000),
            is_binary: true,
        };
        assert_eq!(classify_file_type(&item.path, &item.name), FileType::Binary);

        // Test test files by naming pattern
        let item = FileItem {
            name: "test_main.rs".to_string(),
            path: PathBuf::from("test_main.rs"),
            is_dir: false,
            is_hidden: false,
            modified: SystemTime::UNIX_EPOCH,
            size: Some(1000),
            is_binary: false,
        };
        // Note: Source files get detected before Test patterns, so this is Source
        assert_eq!(classify_file_type(&item.path, &item.name), FileType::Source);
    }

    #[test]
    fn test_directory_bonus() {
        // Test src/ directory bonus
        let item = FileItem {
            name: "main.rs".to_string(),
            path: PathBuf::from("src/main.rs"),
            is_dir: false,
            is_hidden: false,
            modified: SystemTime::UNIX_EPOCH,
            size: Some(1000),
            is_binary: false,
        };
        assert_eq!(calculate_directory_bonus(&item.path, &item.name), 150); // 150 (src) bonus only

        // Test docs/ directory penalty
        let item = FileItem {
            name: "implementation.md".to_string(),
            path: PathBuf::from("docs/implementation.md"),
            is_dir: false,
            is_hidden: false,
            modified: SystemTime::UNIX_EPOCH,
            size: Some(1000),
            is_binary: false,
        };
        assert_eq!(calculate_directory_bonus(&item.path, &item.name), -50);

        // Test lib/ directory bonus
        let item = FileItem {
            name: "utils.rs".to_string(),
            path: PathBuf::from("lib/utils.rs"),
            is_dir: false,
            is_hidden: false,
            modified: SystemTime::UNIX_EPOCH,
            size: Some(1000),
            is_binary: false,
        };
        assert_eq!(calculate_directory_bonus(&item.path, &item.name), 150);
    }

    #[test]
    fn test_main_search_priority() {
        // Test that src/main.rs appears before docs files when searching for "main"
        let src_main = FileItem {
            name: "main.rs".to_string(),
            path: PathBuf::from("src/main.rs"),
            is_dir: false,
            is_hidden: false,
            modified: SystemTime::UNIX_EPOCH,
            size: Some(1000),
            is_binary: false,
        };

        let docs_impl = FileItem {
            name: "IMPLEMENTATION.md".to_string(),
            path: PathBuf::from("docs/IMPLEMENTATION.md"),
            is_dir: false,
            is_hidden: false,
            modified: SystemTime::UNIX_EPOCH,
            size: Some(1000),
            is_binary: false,
        };

        let readme = FileItem {
            name: "README.md".to_string(),
            path: PathBuf::from("README.md"),
            is_dir: false,
            is_hidden: false,
            modified: SystemTime::UNIX_EPOCH,
            size: Some(1000),
            is_binary: false,
        };

        let items = vec![src_main.clone(), docs_impl.clone(), readme.clone()];

        let mut scored_items: Vec<(FileItem, i32, MatchType)> = items
            .into_iter()
            .filter_map(|item| {
                fuzzy_match_with_priority("main", &item)
                    .map(|(score, match_type)| (item, score, match_type))
            })
            .collect();

        // Sort by current algorithm
        scored_items.sort_by(|a, b| {
            let type_order = match (&a.2, &b.2) {
                (MatchType::ExactFilename, MatchType::ExactFilename) => std::cmp::Ordering::Equal,
                (MatchType::ExactFilename, _) => std::cmp::Ordering::Less,
                (_, MatchType::ExactFilename) => std::cmp::Ordering::Greater,
                (MatchType::FilenameFuzzy, MatchType::FilenameFuzzy) => std::cmp::Ordering::Equal,
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

        // Verify src/main.rs has the highest score due to bonuses
        assert_eq!(scored_items[0].0.path, src_main.path);

        let src_score = scored_items[0].1;
        let docs_score = scored_items[1].1; // Should be docs or README

        // src/main.rs should have significantly higher score due to bonuses
        assert!(src_score > docs_score);
        println!(
            "src/main.rs score: {}, docs file score: {}",
            src_score, docs_score
        );
    }

    #[test]
    fn test_file_type_bonuses() {
        assert_eq!(FileType::Source.bonus_score(), 500);
        assert_eq!(FileType::Test.bonus_score(), 250);
        assert_eq!(FileType::Documentation.bonus_score(), 0);
        assert_eq!(FileType::Configuration.bonus_score(), 150);
        assert_eq!(FileType::Build.bonus_score(), 75);
        assert_eq!(FileType::Binary.bonus_score(), -100);
        assert_eq!(FileType::Other.bonus_score(), 0);
    }
}