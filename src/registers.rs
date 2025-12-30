// src/registers.rs - Vim-style yank/paste register system

use std::collections::HashMap;

const NUMBERED_REGISTERS: usize = 9;
const _MAX_UNDO_STACK: usize = 100;

/// Vim-style register system for yank/paste operations
#[derive(Debug, Clone, Default)]
pub struct Registers {
    /// Unnamed register (")
    pub unnamed: String,
    /// Numbered registers 0-9 for deletes
    /// 0 = last yank, 1-9 = deletes (1 = most recent)
    pub numbered: [String; NUMBERED_REGISTERS],
    /// Named registers a-z
    pub named: HashMap<char, String>,
    /// Small delete register (for small deletes < 1 line)
    pub small_delete: String,
    /// Clipboard registers (* and +)
    pub clipboard: String,
    /// Last inserted text
    pub last_inserted: String,
    /// Index for next numbered delete (1-9, wraps around)
    _next_delete_slot: usize,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            unnamed: String::new(),
            numbered: Default::default(),
            named: HashMap::new(),
            small_delete: String::new(),
            clipboard: String::new(),
            last_inserted: String::new(),
            _next_delete_slot: 1,
        }
    }

    /// Yank text to a register
    /// reg = '"' for unnamed, '0'-'9' for numbered, 'a'-'z' for named
    /// '*' and '+' for clipboard, '-' for small delete
    pub fn yank(&mut self, text: String, reg: char) {
        match reg {
            '"' => {
                // Unnamed register
                self.unnamed = text;
            }
            '0' => {
                // Register 0 is for the most recent yank
                self.unnamed = text.clone();
                self.numbered[0] = text;
            }
            '1'..='9' => {
                // Numbered delete registers
                let idx = reg as usize - '0' as usize;
                self.unnamed = text.clone();
                self.numbered[idx] = text;
            }
            '-' => {
                // Small delete register
                self.small_delete = text.clone();
                self.unnamed = text;
            }
            '*' | '+' => {
                // Clipboard register
                self.clipboard = text.clone();
                self.unnamed = text;
            }
            'a'..='z' | 'A'..='Z' => {
                // Named registers
                let key = reg.to_ascii_lowercase();
                self.unnamed = text.clone();
                if reg.is_uppercase() {
                    // Uppercase means append
                    if let Some(existing) = self.named.get_mut(&key) {
                        existing.push_str(&text);
                    } else {
                        self.named.insert(key, text);
                    }
                } else {
                    self.named.insert(key, text);
                }
            }
            _ => {
                // Unknown register, just use unnamed
                self.unnamed = text;
            }
        }
    }

    /// Get text from a register
    pub fn get(&self, reg: char) -> Option<&str> {
        match reg {
            '"' => Some(&self.unnamed),
            '0'..='9' => {
                let idx = reg as usize - '0' as usize;
                if self.numbered[idx].is_empty() {
                    Some(&self.unnamed)
                } else {
                    Some(&self.numbered[idx])
                }
            }
            '-' => {
                if self.small_delete.is_empty() {
                    Some(&self.unnamed)
                } else {
                    Some(&self.small_delete)
                }
            }
            '*' | '+' => {
                if self.clipboard.is_empty() {
                    Some(&self.unnamed)
                } else {
                    Some(&self.clipboard)
                }
            }
            'a'..='z' | 'A'..='Z' => {
                let key = reg.to_ascii_lowercase();
                self.named.get(&key).map(|s| s.as_str()).or(Some(&self.unnamed))
            }
            _ => Some(&self.unnamed),
        }
    }

    /// Add delete to numbered registers
    /// This shifts 1->2, 2->3, etc. and puts new content in 1
    pub fn add_delete(&mut self, text: String) {
        // Shift numbered registers down
        for i in (2..NUMBERED_REGISTERS).rev() {
            self.numbered[i] = std::mem::take(&mut self.numbered[i - 1]);
        }
        // Put new delete in register 1
        self.numbered[1] = text.clone();
        // Update unnamed register
        self.unnamed = text;
    }

    /// Store last inserted text (for repeat with .)
    pub fn store_inserted(&mut self, text: String) {
        self.last_inserted = text;
    }

    /// Get last inserted text
    pub fn get_inserted(&self) -> &str {
        &self.last_inserted
    }

    /// Clear all registers
    pub fn clear(&mut self) {
        self.unnamed.clear();
        for reg in &mut self.numbered {
            reg.clear();
        }
        self.named.clear();
        self.small_delete.clear();
        self.clipboard.clear();
        self.last_inserted.clear();
    }

    /// Check if register exists and has content
    pub fn has_content(&self, reg: char) -> bool {
        self.get(reg).map(|s| !s.is_empty()).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unnamed_register() {
        let mut regs = Registers::new();
        regs.yank("hello".to_string(), '"');
        assert_eq!(regs.get('"'), Some("hello"));
    }

    #[test]
    fn test_numbered_register_zero() {
        let mut regs = Registers::new();
        regs.yank("yanked".to_string(), '0');
        assert_eq!(regs.get('0'), Some("yanked"));
        assert_eq!(regs.get('"'), Some("yanked")); // Unnamed also updated
    }

    #[test]
    fn test_named_registers() {
        let mut regs = Registers::new();
        regs.yank("text_a".to_string(), 'a');
        regs.yank("text_b".to_string(), 'b');

        assert_eq!(regs.get('a'), Some("text_a"));
        assert_eq!(regs.get('b'), Some("text_b"));
    }

    #[test]
    fn test_append_to_named_register() {
        let mut regs = Registers::new();
        regs.yank("hello".to_string(), 'a');
        regs.yank(" world".to_string(), 'A');

        assert_eq!(regs.get('a'), Some("hello world"));
    }

    #[test]
    fn test_add_delete_shifts_registers() {
        let mut regs = Registers::new();
        regs.numbered[1] = "first".to_string();
        regs.numbered[2] = "second".to_string();

        regs.add_delete("new_delete".to_string());

        assert_eq!(regs.numbered[1], "new_delete");
        assert_eq!(regs.numbered[2], "first");
        assert_eq!(regs.get('"'), Some("new_delete"));
    }

    #[test]
    fn test_last_inserted() {
        let mut regs = Registers::new();
        regs.store_inserted("inserted_text".to_string());
        assert_eq!(regs.get_inserted(), "inserted_text");
    }

    #[test]
    fn test_clear() {
        let mut regs = Registers::new();
        regs.yank("hello".to_string(), '"');
        regs.yank("world".to_string(), 'a');

        regs.clear();

        assert!(!regs.has_content('"'));
        assert!(!regs.has_content('a'));
    }

    #[test]
    fn test_has_content() {
        let mut regs = Registers::new();
        assert!(!regs.has_content('"'));

        regs.yank("hello".to_string(), '"');
        assert!(regs.has_content('"'));
    }
}
