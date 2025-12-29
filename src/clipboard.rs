// clipboard.rs - Clipboard operations for texty editor
use anyhow::Result;
use arboard::Clipboard as ArboardClipboard;
use std::fmt;

#[derive(Debug)]
pub enum ClipboardError {
    AccessDenied,
    EmptyClipboard,
    UnsupportedPlatform,
    SystemError(String),
}

impl fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClipboardError::AccessDenied => write!(f, "Clipboard access denied"),
            ClipboardError::EmptyClipboard => write!(f, "Clipboard is empty"),
            ClipboardError::UnsupportedPlatform => {
                write!(f, "Platform does not support clipboard operations")
            }
            ClipboardError::SystemError(msg) => write!(f, "System error: {}", msg),
        }
    }
}

impl std::error::Error for ClipboardError {}

/// Clipboard operations interface
pub struct Clipboard {
    inner: ArboardClipboard,
}

impl Clipboard {
    /// Create new clipboard instance
    pub fn new() -> Result<Self> {
        match ArboardClipboard::new() {
            Ok(clipboard) => Ok(Self { inner: clipboard }),
            Err(e) => Err(anyhow::anyhow!("Failed to initialize clipboard: {}", e)),
        }
    }

    /// Get text content from clipboard
    pub fn get_text(&mut self) -> Result<String> {
        self.inner
            .get_text()
            .map_err(|e| anyhow::anyhow!("Failed to get clipboard text: {}", e))
    }

    /// Set text content to clipboard
    pub fn set_text(&mut self, text: &str) -> Result<()> {
        self.inner
            .set_text(text)
            .map_err(|e| anyhow::anyhow!("Failed to set clipboard text: {}", e))
    }

    /// Check if clipboard has content
    pub fn has_content(&mut self) -> Result<bool> {
        self.get_text().map(|content| !content.is_empty())
    }

    /// Clear clipboard content
    pub fn clear(&mut self) -> Result<()> {
        self.set_text("")
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            inner: ArboardClipboard::new()
                .unwrap_or_else(|_| panic!("Failed to create default clipboard")),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_creation() {
        let _clipboard = Clipboard::default();
        assert!(true); // Basic creation test
    }

    #[test]
    fn test_clipboard_operations() {
        // Note: These tests may fail in headless environments
        // In real usage, clipboard operations should handle gracefully

        // Just test that the struct can be created without panicking
        let _clipboard = Clipboard::default();
    }
}
