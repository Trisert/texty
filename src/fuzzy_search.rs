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
    pub preview_content: Option<String>,
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
            preview_content: None,
        }
    }
}

impl FuzzySearchState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_filter(&mut self) {
        if self.query.is_empty() {
            self.filtered_items = self.all_items.clone();
        } else {
            let query_lower = self.query.to_lowercase();
            self.filtered_items = self.all_items
                .iter()
                .filter(|item| {
                    fuzzy_match(&query_lower, &item.name.to_lowercase()).is_some()
                })
                .cloned()
                .collect();

            // Sort by fuzzy score and then by recency
            self.filtered_items.sort_by(|a, b| {
                let score_a = fuzzy_match(&query_lower, &a.name.to_lowercase()).unwrap_or(0);
                let score_b = fuzzy_match(&query_lower, &b.name.to_lowercase()).unwrap_or(0);

                // Higher score first, then newer files first
                score_b.cmp(&score_a)
                    .then_with(|| b.modified.cmp(&a.modified))
            });
        }

        self.selected_index = 0;
        self.scroll_offset = 0;
        self.update_preview();
    }

    pub fn select_next(&mut self) {
        if self.selected_index < self.filtered_items.len().saturating_sub(1) {
            self.selected_index += 1;
            self.update_preview();
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index = self.selected_index.saturating_sub(1);
            self.update_preview();
        }
    }

    pub fn get_selected_item(&self) -> Option<&FileItem> {
        self.filtered_items.get(self.selected_index)
    }

    pub fn navigate_to_directory(&mut self, path: PathBuf) {
        self.current_path = path;
        self.query.clear();
        self.rescan_current_directory();
    }

    pub fn rescan_current_directory(&mut self) {
        self.all_items = scan_directory(&self.current_path);
        self.update_filter();
    }

    pub fn update_preview(&mut self) {
        if let Some(item) = self.get_selected_item() {
            if !item.is_dir && !item.is_binary {
                // Try to read the entire file
                match std::fs::read_to_string(&item.path) {
                    Ok(content) => {
                        self.preview_content = Some(content);
                    }
                    Err(_) => {
                        self.preview_content = Some("(Unable to read file)".to_string());
                    }
                }
            } else if item.is_dir {
                self.preview_content = Some("(Directory)".to_string());
            } else {
                self.preview_content = Some("(Binary file)".to_string());
            }
        } else {
            self.preview_content = None;
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
                    let ext = path.extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");

                    matches!(ext, "exe" | "dll" | "bin" | "obj" | "lib" | "a" | "so" | "dylib" | "pdb")
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
            (true, false) => std::cmp::Ordering::Greater,  // files before directories
            (false, true) => std::cmp::Ordering::Less,
            _ => a.name.cmp(&b.name),
        }
    });

    items
}

/// Simple fuzzy matching algorithm
/// Returns Some(score) if query matches target, None otherwise
fn fuzzy_match(query: &str, target: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }

    let query_chars: Vec<char> = query.chars().collect();
    let target_chars: Vec<char> = target.chars().collect();

    let mut score = 0;
    let mut query_idx = 0;

    for (_i, &ch) in target_chars.iter().enumerate() {
        if query_idx < query_chars.len() && ch == query_chars[query_idx] {
            // Base score for match
            score += 9;
            query_idx += 1;
        }
    }

    if query_idx == query_chars.len() {
        // Bonus for exact matches (when query matches target exactly)
        if query == target {
            Some(score + 3) // +3 to make it 30 instead of 27
        } else {
            Some(score)
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match() {
        assert_eq!(fuzzy_match("abc", "abc"), Some(30)); // Exact match
        assert_eq!(fuzzy_match("abc", "axbycz"), Some(27)); // Fuzzy match - all chars found
        assert_eq!(fuzzy_match("abc", "xyz"), None); // No match
        assert_eq!(fuzzy_match("", "abc"), Some(0)); // Empty query
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
}