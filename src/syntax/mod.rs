pub mod cache;
pub mod highlighter;
pub mod language;
pub mod query_loader;

pub use highlighter::{HighlightKind, HighlightToken, SyntaxHighlighter};
pub use language::get_language_config;
pub use query_loader::QueryLoader;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LanguageId {
    Rust,
    Python,
    JavaScript,
    TypeScript,
}

#[derive(Debug)]
pub struct LanguageConfig {
    pub id: LanguageId,
    pub tree_sitter_language: fn() -> tree_sitter::Language,
    pub highlight_query_path: Option<String>,
    pub highlight_query_fallback: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_id_variants() {
        assert_eq!(LanguageId::Rust, LanguageId::Rust);
        assert_ne!(LanguageId::Python, LanguageId::JavaScript);
    }
}
