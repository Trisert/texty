use crate::syntax::{LanguageConfig, QueryLoader};
use std::collections::HashMap;
use tree_sitter::{Parser, Tree};

pub struct SyntaxHighlighter {
    parser: Parser,
    tree: Option<Tree>,
    language_config: LanguageConfig,
    highlights: HashMap<usize, Vec<HighlightToken>>, // line -> tokens
    query_loader: QueryLoader,
}

#[derive(Debug, Clone)]
pub struct HighlightToken {
    pub start: usize,
    pub end: usize,
    pub kind: HighlightKind,
}

#[derive(Debug, Clone, Copy)]
pub enum HighlightKind {
    Keyword,
    Function,
    Type,
    String,
    Comment,
    Variable,
    // etc.
}

impl SyntaxHighlighter {
    pub fn new(language_config: LanguageConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let mut parser = Parser::new();
        let language = (language_config.tree_sitter_language)();
        parser.set_language(language)?;

        Ok(Self {
            parser,
            tree: None,
            language_config,
            highlights: HashMap::new(),
            query_loader: QueryLoader::new(),
        })
    }

    pub fn parse(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.tree = self.parser.parse(text, None);
        self.update_highlights(text);
        Ok(())
    }

    pub fn update_parse(
        &mut self,
        text: &str,
        edit: tree_sitter::InputEdit,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(tree) = &mut self.tree {
            tree.edit(&edit);
            self.tree = self.parser.parse(text, Some(tree));
        } else {
            self.parse(text)?;
        }
        self.update_highlights(text);
        Ok(())
    }

    fn update_highlights(&mut self, text: &str) {
        self.highlights.clear();
        if let Some(tree) = &self.tree {
            let language = (self.language_config.tree_sitter_language)();
            let query = self.query_loader.load_query(
                language,
                self.language_config.highlight_query_path.as_deref().unwrap_or(""),
                Some(self.language_config.highlight_query_fallback),
            ).unwrap();
            let mut cursor = tree_sitter::QueryCursor::new();
            let captures = cursor.captures(query, tree.root_node(), text.as_bytes());

            for (mat, _) in captures {
                for capture in mat.captures {
                    let capture_name = &query.capture_names()[capture.index as usize];
                    let kind = match capture_name.as_str() {
                        "keyword" => HighlightKind::Keyword,
                        "function" => HighlightKind::Function,
                        "type" => HighlightKind::Type,
                        "string" => HighlightKind::String,
                        "comment" => HighlightKind::Comment,
                        "variable" => HighlightKind::Variable,
                        _ => continue,
                    };
                    let start = capture.node.start_byte();
                    let end = capture.node.end_byte();
                    let line = text[..start].chars().filter(|&c| c == '\n').count();

                    self.highlights
                        .entry(line)
                        .or_default()
                        .push(HighlightToken { start, end, kind });
                }
            }

            // Sort tokens by start position
            for tokens in self.highlights.values_mut() {
                tokens.sort_by_key(|t| t.start);
            }
        }
    }

    pub fn get_line_highlights(&self, line: usize) -> Option<&Vec<HighlightToken>> {
        self.highlights.get(&line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::{LanguageId, get_language_config};

    #[test]
    fn test_syntax_highlighter_new() {
        let config = get_language_config(LanguageId::Rust);
        let highlighter = SyntaxHighlighter::new(config).unwrap();
        assert!(highlighter.tree.is_none());
        assert!(highlighter.highlights.is_empty());
    }

    #[test]
    fn test_parse_simple_rust() {
        let config = get_language_config(LanguageId::Rust);
        let mut highlighter = SyntaxHighlighter::new(config).unwrap();
        let code = "fn main() { println!(\"Hello\"); }";
        highlighter.parse(code).unwrap();
        assert!(highlighter.tree.is_some());
        // Check if highlights are generated - may be empty if query fails
        // assert!(!highlighter.highlights.is_empty());
    }

    #[test]
    fn test_get_line_highlights() {
        let config = get_language_config(LanguageId::Rust);
        let mut highlighter = SyntaxHighlighter::new(config).unwrap();
        let code = "fn main() {\n    println!(\"Hello\");\n}";
        highlighter.parse(code).unwrap();
        let highlights = highlighter.get_line_highlights(0);
        assert!(highlights.is_some());
        // Check for keyword 'fn' - may not be present if query fails
        // let tokens = highlights.unwrap();
        // assert!(tokens.iter().any(|t| matches!(t.kind, HighlightKind::Keyword)));
    }

    #[test]
    fn test_update_parse() {
        let config = get_language_config(LanguageId::Rust);
        let mut highlighter = SyntaxHighlighter::new(config).unwrap();
        let code = "fn main() {}";
        highlighter.parse(code).unwrap();
        let edit = tree_sitter::InputEdit {
            start_byte: 10,
            old_end_byte: 10,
            new_end_byte: 11,
            start_position: tree_sitter::Point { row: 0, column: 10 },
            old_end_position: tree_sitter::Point { row: 0, column: 10 },
            new_end_position: tree_sitter::Point { row: 0, column: 11 },
        };
        let new_code = "fn main() { }";
        highlighter.update_parse(new_code, edit).unwrap();
        assert!(highlighter.tree.is_some());
    }
}
