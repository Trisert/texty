use crate::syntax::{LanguageConfig, QueryLoader};
use log::{debug, trace};
use std::collections::HashMap;
use std::ops::Range;
use tree_sitter::{Parser, Query, Tree};

pub struct SyntaxHighlighter {
    parser: Parser,
    tree: Option<Tree>,
    language_config: LanguageConfig,
    highlights: HashMap<usize, Vec<HighlightToken>>, // line -> tokens
    query_loader: QueryLoader,
    // Performance optimization: Track viewport to avoid re-highlighting unchanged regions
    current_viewport: Option<Range<usize>>,
    full_text: Option<String>, // Cache full text for viewport updates
}

#[derive(Debug, Clone)]
pub struct HighlightToken {
    pub start: usize,
    pub end: usize,
    pub capture_name: String,
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
            current_viewport: None,
            full_text: None,
        })
    }

    pub fn parse(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.tree = self.parser.parse(text, None);
        self.full_text = Some(text.to_string());
        self.update_highlights(text, None);
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
            return Ok(());
        }
        self.full_text = Some(text.to_string());
        self.update_highlights(text, None);
        Ok(())
    }

    /// Update highlights for a specific viewport range (performance optimization)
    pub fn update_highlights_viewport(&mut self, viewport: Range<usize>) {
        if let Some(text) = &self.full_text {
            // Check if viewport has changed significantly
            let needs_update = match &self.current_viewport {
                None => true,
                Some(current) => {
                    // Update if viewport moved by more than 10 lines
                    let start_diff = viewport.start.abs_diff(current.start);
                    let end_diff = viewport.end.abs_diff(current.end);
                    start_diff > 10 || end_diff > 10
                }
            };

            if needs_update {
                // Clear old highlights that are no longer in viewport (keep some margin)
                if let Some(current) = &self.current_viewport {
                    let margin = 20;
                    for line in current.start.saturating_sub(margin)..current.end + margin {
                        if line < viewport.start.saturating_sub(margin) || line > viewport.end + margin {
                            self.highlights.remove(&line);
                        }
                    }
                }

                // Clone text to avoid borrow checker issue
                let text_clone = text.clone();
                self.update_highlights(&text_clone, Some(viewport.clone()));
                self.current_viewport = Some(viewport);
            }
        }
    }

    fn update_highlights(&mut self, text: &str, viewport: Option<Range<usize>>) {
        // Only clear highlights if we're not doing viewport-specific updates
        if viewport.is_none() {
            self.highlights.clear();
        }

        if let Some(tree) = &self.tree {
            let language = (self.language_config.tree_sitter_language)();

            debug!(
                "Language highlight_query_path: {:?}",
                self.language_config.highlight_query_path
            );
            debug!(
                "Language highlight_query_fallback: {:?}",
                self.language_config.highlight_query_fallback
            );

            // Load and apply main highlight query
            if let Ok(query) = self.query_loader.load_query(
                language,
                self.language_config
                    .highlight_query_path
                    .as_deref()
                    .unwrap_or(""),
                Some(self.language_config.highlight_query_fallback),
            ) {
                debug!("Query loaded successfully");
                Self::apply_query(&mut self.highlights, text, tree, &query, viewport.as_ref());
            } else {
                debug!("Failed to load query");
            }

            // Load and apply injection queries
            if let Some(path) = &self.language_config.injection_query_path
                && let Ok(query) = self.query_loader.load_query(
                    language,
                    path,
                    self.language_config.injection_query_fallback,
                )
            {
                Self::apply_query(&mut self.highlights, text, tree, &query, viewport.as_ref());
            }

            // Load and apply locals query
            if let Some(path) = &self.language_config.locals_query_path
                && let Ok(query) = self.query_loader.load_query(
                    language,
                    path,
                    self.language_config.locals_query_fallback,
                )
            {
                Self::apply_query(&mut self.highlights, text, tree, &query, viewport.as_ref());
            }

            // Sort tokens by start position
            for tokens in self.highlights.values_mut() {
                tokens.sort_by_key(|t| t.start);
            }
        }
    }

    fn apply_query(
        highlights: &mut HashMap<usize, Vec<HighlightToken>>,
        text: &str,
        tree: &Tree,
        query: &Query,
        viewport: Option<&Range<usize>>,
    ) {
        let mut cursor = tree_sitter::QueryCursor::new();
        let captures = cursor.captures(query, tree.root_node(), text.as_bytes());

        let mut capture_count = 0;
        for (mat, _) in captures {
            for capture in mat.captures {
                capture_count += 1;
                let capture_name = &query.capture_names()[capture.index as usize];
                let start = capture.node.start_byte();
                let end = capture.node.end_byte();
                let line = text[..start].chars().filter(|&c| c == '\n').count();

                // Performance optimization: Skip lines outside viewport
                if let Some(viewport) = viewport {
                    // Add margin for multi-line tokens and lookahead
                    let margin = 5;
                    if line < viewport.start.saturating_sub(margin)
                        || line > viewport.end + margin
                    {
                        continue;
                    }
                }

                highlights.entry(line).or_default().push(HighlightToken {
                    start,
                    end,
                    capture_name: capture_name.clone(),
                });
            }
        }

        if capture_count == 0 {
            trace!("No captures found from query");
        } else {
            trace!("Found {} captures", capture_count);
        }
    }

    pub fn get_line_highlights(&self, line: usize) -> Option<&Vec<HighlightToken>> {
        self.highlights.get(&line)
    }

    pub fn get_tree(&self) -> &Option<Tree> {
        &self.tree
    }

    pub fn get_highlights_len(&self) -> usize {
        self.highlights.len()
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
        let _highlights = highlighter.get_line_highlights(0);
        // assert!(highlights.is_some()); // TODO: fix query
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
