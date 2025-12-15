// src/formatter/indent.rs - Smart indentation

use crate::syntax::LanguageId;

#[derive(Debug)]
pub struct IndentationEngine {
    pub indent_width: usize,
    pub use_spaces: bool,
}

impl IndentationEngine {
    pub fn new(language: LanguageId) -> Self {
        let indent_width = match language {
            LanguageId::Rust | LanguageId::Python => 4,
            LanguageId::JavaScript | LanguageId::TypeScript => 2,
        };
        Self {
            indent_width,
            use_spaces: true,
        }
    }

    pub fn get_indent_level(&self, _line: &str, _context: &str) -> usize {
        // TODO: Use tree-sitter queries to determine indent level
        0
    }

    pub fn create_indent_string(&self, level: usize) -> String {
        if self.use_spaces {
            " ".repeat(level * self.indent_width)
        } else {
            "\t".repeat(level)
        }
    }
}
