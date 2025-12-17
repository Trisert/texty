use crate::syntax::{LanguageConfig, LanguageId};

pub fn get_language_config(id: LanguageId) -> LanguageConfig {
    match id {
        LanguageId::Rust => LanguageConfig {
            id,
            tree_sitter_language: || tree_sitter_rust::language(),
            highlight_query_path: Some("runtime/queries/rust/highlights.scm".to_string()),
            highlight_query_fallback: include_str!("../../queries/rust/highlights.scm"),
        },
        LanguageId::Python => LanguageConfig {
            id,
            tree_sitter_language: || tree_sitter_python::language(),
            highlight_query_path: Some("runtime/queries/python/highlights.scm".to_string()),
            highlight_query_fallback: include_str!("../../queries/python/highlights.scm"),
        },
        LanguageId::JavaScript => LanguageConfig {
            id,
            tree_sitter_language: || tree_sitter_javascript::language(),
            highlight_query_path: Some("runtime/queries/javascript/highlights.scm".to_string()),
            highlight_query_fallback: include_str!("../../queries/javascript/highlights.scm"),
        },
        LanguageId::TypeScript => LanguageConfig {
            id,
            tree_sitter_language: || tree_sitter_typescript::language_typescript(),
            highlight_query_path: Some("runtime/queries/typescript/highlights.scm".to_string()),
            highlight_query_fallback: include_str!("../../queries/typescript/highlights.scm"),
        },
    }
}

pub fn get_language_config_by_extension(ext: &str) -> Option<LanguageConfig> {
    match ext {
        "rs" => Some(get_language_config(LanguageId::Rust)),
        "py" => Some(get_language_config(LanguageId::Python)),
        "js" => Some(get_language_config(LanguageId::JavaScript)),
        "ts" => Some(get_language_config(LanguageId::TypeScript)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_language_config_rust() {
        let config = get_language_config(LanguageId::Rust);
        assert_eq!(config.id, LanguageId::Rust);
        assert!(config.highlight_query_path.is_some());
        assert!(!config.highlight_query_fallback.is_empty());
    }

    #[test]
    fn test_get_language_config_python() {
        let config = get_language_config(LanguageId::Python);
        assert_eq!(config.id, LanguageId::Python);
        assert!(config.highlight_query_path.is_some());
        assert!(!config.highlight_query_fallback.is_empty());
    }

    #[test]
    fn test_get_language_config_javascript() {
        let config = get_language_config(LanguageId::JavaScript);
        assert_eq!(config.id, LanguageId::JavaScript);
        assert!(config.highlight_query_path.is_some());
        assert!(!config.highlight_query_fallback.is_empty());
    }

    #[test]
    fn test_get_language_config_typescript() {
        let config = get_language_config(LanguageId::TypeScript);
        assert_eq!(config.id, LanguageId::TypeScript);
        assert!(config.highlight_query_path.is_some());
        assert!(!config.highlight_query_fallback.is_empty());
    }

    #[test]
    fn test_get_language_config_by_extension() {
        assert!(get_language_config_by_extension("rs").is_some());
        assert!(get_language_config_by_extension("py").is_some());
        assert!(get_language_config_by_extension("js").is_some());
        assert!(get_language_config_by_extension("ts").is_some());
        assert!(get_language_config_by_extension("txt").is_none());
    }
}
