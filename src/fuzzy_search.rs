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

/// Cached preview content types
#[derive(Debug, Clone)]
pub enum PreviewCache {
    PlainContent(String), // Plain text content for files (fallback)
    HighlightedContent(Vec<ratatui::text::Line<'static>>), // Syntax highlighted lines
    FormattedContent(String), // Formatted plain content
    FormattedHighlighted(Vec<ratatui::text::Line<'static>>), // Formatted + syntax highlighted
    Directory(Vec<String>), // Directory listing
    Binary,               // Binary file marker
    LargeFile,            // File too large marker
    Error(String),        // Error message
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
    pub show_preview: bool,
    pub show_formatted_preview: bool, // NEW: Toggle formatted preview
    pub preview_cache: HashMap<PathBuf, PreviewCache>,
    pub formatted_preview_cache: HashMap<PathBuf, crate::ui::widgets::preview::PreviewBuffer>, // NEW: Cache for formatted previews
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
            show_preview: true,
            show_formatted_preview: true, // Default: formatted preview enabled
            preview_cache: HashMap::new(),
            formatted_preview_cache: HashMap::new(),
        }
    }
}

impl FuzzySearchState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_filter(&mut self) {
        self.query = self.query.trim().to_string();
        self.selected_index = 0;
        self.scroll_offset = 0;

        // Filter items based on query
        if self.query.is_empty() {
            self.filtered_items = self.all_items.clone();
        } else {
            self.filtered_items = self
                .all_items
                .iter()
                .filter(|item| fuzzy_match(&self.query, &item.name).is_some())
                .cloned()
                .collect();
        }
    }

    pub fn select_next(&mut self) {
        if self.selected_index < self.filtered_items.len().saturating_sub(1) {
            self.selected_index += 1;
            self.update_preview(None);
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index = self.selected_index.saturating_sub(1);
            self.update_preview(None);
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
        self.update_preview(None);
    }

    pub fn update_preview(&mut self, _formatter: Option<&crate::formatter::external::Formatter>) {
        // Clear cache when selection changes to ensure fresh content
        self.preview_cache.clear();

        if let Some(item) = self.get_selected_item() {
            let item = item.clone(); // Clone to avoid borrowing issues

            // Check if we already have this item cached
            if self.preview_cache.contains_key(&item.path) {
                return; // Already cached
            }

            // Following Helix's pattern: check file type and cache appropriately
            if item.is_dir {
                // For directories, scan and cache the contents
                self.preview_cache.insert(
                    item.path.clone(),
                    PreviewCache::Directory(self.scan_directory_preview(&item.path)),
                );
            } else if item.is_binary {
                self.preview_cache
                    .insert(item.path.clone(), PreviewCache::Binary);
            } else {
                // For text files, read content and format if needed
                self.load_file_preview(&item, self.show_formatted_preview);
            }
        }
    }

    fn scan_directory_preview(&self, path: &PathBuf) -> Vec<String> {
        let mut entries = Vec::new();

        if let Ok(read_dir) = fs::read_dir(path) {
            for entry in read_dir.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if metadata.is_dir() {
                        entries.push(format!("{}/", name));
                    } else {
                        entries.push(name);
                    }
                }
            }
        }

        // Sort: directories first, then files, alphabetically
        entries.sort_by(|a, b| {
            let a_is_dir = a.ends_with('/');
            let b_is_dir = b.ends_with('/');
            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less, // directories first
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.cmp(b),
            }
        });

        entries
    }

    fn load_file_preview(&mut self, item: &FileItem, format_content: bool) {
        // Check file size - don't preview files larger than 10MB (following Helix)
        const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;
        if item.size.unwrap_or(0) > MAX_FILE_SIZE {
            self.preview_cache
                .insert(item.path.clone(), PreviewCache::LargeFile);
            return;
        }

        // Try to read the file
        match fs::read(&item.path) {
            Ok(bytes) => {
                // Check if file contains binary data
                let is_binary = bytes.contains(&0)
                    || bytes
                        .iter()
                        .any(|&b| b.is_ascii_control() && !matches!(b, b'\n' | b'\r' | b'\t'));

                if is_binary {
                    self.preview_cache
                        .insert(item.path.clone(), PreviewCache::Binary);
                    return;
                }

                // Convert to string
                match String::from_utf8(bytes) {
                    Ok(content) => {
                        // Limit preview to first 1000 lines to prevent performance issues
                        let lines: Vec<&str> = content.lines().take(1000).collect();
                        let truncated_content = if lines.len() >= 1000 {
                            format!("{}\n... (truncated)", lines.join("\n"))
                        } else {
                            lines.join("\n")
                        };

                        if format_content {
                            // TODO: Dynamic formatting will be handled by FuzzySearchWidget
                            // For now, store plain content - widget will format on demand
                            self.preview_cache.insert(
                                item.path.clone(),
                                PreviewCache::PlainContent(truncated_content),
                            );
                        } else {
                            // Store as plain content
                            self.preview_cache.insert(
                                item.path.clone(),
                                PreviewCache::PlainContent(truncated_content),
                            );
                        }
                    }
                    Err(_) => {
                        self.preview_cache
                            .insert(item.path.clone(), PreviewCache::Binary);
                    }
                }
            }
            Err(e) => {
                self.preview_cache.insert(
                    item.path.clone(),
                    PreviewCache::Error(format!("Unable to read file: {}", e)),
                );
            }
        }
    }

    pub fn get_preview(&self, path: &PathBuf) -> Option<&PreviewCache> {
        self.preview_cache.get(path)
    }

    pub fn toggle_preview(&mut self) {
        self.show_preview = !self.show_preview;
        // Clear cache when toggling to ensure fresh content when re-enabled
        if !self.show_preview {
            self.preview_cache.clear();
        }
    }

    pub fn toggle_formatted_preview(&mut self) {
        self.show_formatted_preview = !self.show_formatted_preview;
        // Clear cache when toggling to ensure fresh content with new formatting
        self.preview_cache.clear();
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

    for &ch in &target_chars {
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
