use crate::syntax::{LanguageId, SyntaxHighlighter, get_language_config};
use ropey::Rope;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum BufferError {
    Io(std::io::Error),
    Rope(ropey::Error),
}

impl From<std::io::Error> for BufferError {
    fn from(err: std::io::Error) -> Self {
        BufferError::Io(err)
    }
}

impl From<ropey::Error> for BufferError {
    fn from(err: ropey::Error) -> Self {
        BufferError::Rope(err)
    }
}

impl std::fmt::Display for BufferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BufferError::Io(err) => write!(f, "IO error: {}", err),
            BufferError::Rope(err) => write!(f, "Rope error: {}", err),
        }
    }
}

impl std::error::Error for BufferError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BufferError::Io(err) => Some(err),
            BufferError::Rope(err) => Some(err),
        }
    }
}

pub struct Buffer {
    pub rope: Rope,
    pub file_path: Option<String>,
    pub modified: bool,
    pub version: usize,
    pub highlighter: Option<SyntaxHighlighter>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            rope: Rope::from(""),
            file_path: None,
            modified: false,
            version: 0,
            highlighter: None,
        }
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Buffer {
    pub fn insert_char(&mut self, char: char, line: usize, col: usize) -> Result<(), BufferError> {
        let char_idx = self.rope.line_to_char(line) + col;
        self.rope.insert_char(char_idx, char);
        self.modified = true;
        self.version += 1;
        self.update_highlighter().ok();
        Ok(())
    }

    pub fn delete_char(&mut self, line: usize, col: usize) -> Result<(), BufferError> {
        if col == 0 && line > 0 {
            // Delete newline
            let char_idx = self.rope.line_to_char(line);
            self.rope.remove(char_idx - 1..char_idx);
        } else if col > 0 {
            let char_idx = self.rope.line_to_char(line) + col;
            self.rope.remove(char_idx - 1..char_idx);
        }
        self.modified = true;
        self.version += 1;
        self.update_highlighter().ok();
        Ok(())
    }

    pub fn insert_text(&mut self, text: &str, line: usize, col: usize) -> Result<(), BufferError> {
        let char_idx = self.rope.line_to_char(line) + col;
        self.rope.insert(char_idx, text);
        self.modified = true;
        self.version += 1;
        self.update_highlighter().ok();
        Ok(())
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn line(&self, line_idx: usize) -> Option<String> {
        if line_idx < self.rope.len_lines() {
            let line = self.rope.line(line_idx).to_string();
            if line.ends_with('\n') {
                Some(line.trim_end_matches('\n').to_string())
            } else {
                Some(line)
            }
        } else {
            None
        }
    }

    pub fn line_len(&self, line_idx: usize) -> usize {
        if line_idx < self.rope.len_lines() {
            self.rope.line(line_idx).len_chars()
        } else {
            0
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), BufferError> {
        let content = fs::read_to_string(path.as_ref())?;
        self.rope = Rope::from_str(&content);
        self.file_path = Some(path.as_ref().to_string_lossy().to_string());
        self.modified = false;
        self.version = 0;

        // Detect language and set highlighter
        if let Some(extension) = path.as_ref().extension() {
            let lang_id = match extension.to_str() {
                Some("rs") => Some(LanguageId::Rust),
                Some("py") => Some(LanguageId::Python),
                Some("js") => Some(LanguageId::JavaScript),
                Some("ts") => Some(LanguageId::TypeScript),
                _ => None,
            };
            if let Some(id) = lang_id {
                let config = get_language_config(id);
                self.highlighter = Some(SyntaxHighlighter::new(config).map_err(|_| {
                    BufferError::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Syntax error",
                    ))
                })?);
                self.highlighter
                    .as_mut()
                    .unwrap()
                    .parse(&content)
                    .map_err(|_| {
                        BufferError::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Parse error",
                        ))
                    })?;
            }
        }

        Ok(())
    }

    pub fn save_to_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), BufferError> {
        fs::write(path.as_ref(), self.rope.to_string())?;
        self.file_path = Some(path.as_ref().to_string_lossy().to_string());
        self.modified = false;
        Ok(())
    }

    pub fn format_buffer(
        &mut self,
        formatter: &crate::formatter::external::Formatter,
        cursor_line: usize,
        cursor_col: usize,
    ) -> Result<(usize, usize), BufferError> {
        let original_text = self.rope.to_string();
        let formatted_text = formatter.format_text(&original_text)?;

        // Simple cursor mapping: keep same line, clamp column
        let new_line_count = formatted_text.lines().count();
        let new_line = cursor_line.min(new_line_count.saturating_sub(1));
        let new_col = if let Some(line) = formatted_text.lines().nth(new_line) {
            cursor_col.min(line.len())
        } else {
            0
        };

        self.rope = Rope::from_str(&formatted_text);
        self.modified = true;
        self.version += 1;
        // TODO: Update highlighter
        Ok((new_line, new_col))
    }

    pub fn update_highlighter(&mut self) -> Result<(), BufferError> {
        if let Some(highlighter) = &mut self.highlighter {
            let text = self.rope.to_string();
            highlighter.parse(&text).map_err(|_| {
                BufferError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Parse error",
                ))
            })?;
        }
        Ok(())
    }
}

#[test]
fn test_insert_char() {
    let mut buffer = Buffer::new();
    buffer.insert_char('a', 0, 0).unwrap();
    assert_eq!(buffer.line(0).unwrap(), "a");
    assert!(buffer.modified);
}

#[test]
fn test_edit_and_save() {
    use tempfile::NamedTempFile;
    let mut buffer = Buffer::new();
    buffer.insert_char('h', 0, 0).unwrap();
    buffer.insert_char('e', 0, 1).unwrap();
    buffer.insert_char('l', 0, 2).unwrap();
    buffer.insert_char('l', 0, 3).unwrap();
    buffer.insert_char('o', 0, 4).unwrap();
    let temp_file = NamedTempFile::new().unwrap();
    buffer.save_to_file(temp_file.path()).unwrap();
    let mut loaded_buffer = Buffer::new();
    loaded_buffer.load_from_file(temp_file.path()).unwrap();
    assert_eq!(loaded_buffer.rope.to_string(), "hello");
}

#[test]
fn test_load_and_save() {
    use tempfile::NamedTempFile;
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), "hello\nworld").unwrap();

    let mut buffer = Buffer::new();
    buffer.load_from_file(temp_file.path()).unwrap();
    assert_eq!(buffer.line_count(), 2);
    assert_eq!(buffer.line(0).unwrap(), "hello");
    assert_eq!(buffer.line(1).unwrap(), "world");

    let save_file = NamedTempFile::new().unwrap();
    buffer.save_to_file(save_file.path()).unwrap();
    let content = fs::read_to_string(save_file.path()).unwrap();
    assert_eq!(content, "hello\nworld");
}

// proptest! {
//     #[test]
//     fn buffer_operations_preserve_invariants(ops in prop::collection::vec((any::<char>(), 0..10usize, 0..100usize), 1..50)) {
//         let mut buffer = Buffer::new();
//         for (ch, line, col) in ops {
//             let _ = buffer.insert_char(ch, line, col); // Ignore errors for proptest
//         }
//         // Invariant: buffer should have at least one line
//         prop_assert!(buffer.line_count() >= 1);
//     }
// }
