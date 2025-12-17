use serde::Deserialize;
use std::collections::HashMap;

/// Configuration loaded from languages.toml
#[derive(Debug, Deserialize)]
pub struct LanguagesConfig {
    pub language: Vec<LanguageEntry>,
}

#[derive(Debug, Deserialize)]
pub struct LanguageEntry {
    pub name: String,
    pub scope: Option<String>,
    #[serde(rename = "file-types")]
    pub file_types: Vec<String>,
    pub grammar: Option<String>,
    #[serde(rename = "highlight-query")]
    pub highlight_query: Option<String>,
    #[serde(rename = "injection-query")]
    pub injection_query: Option<String>,
}

/// Load language configuration from runtime/languages.toml
pub fn load_languages_config() -> Result<LanguagesConfig, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("runtime/languages.toml")?;
    let config: LanguagesConfig = toml::from_str(&content)?;
    Ok(config)
}

/// Language registry for runtime language detection
#[derive(Debug)]
pub struct LanguageRegistry {
    languages: HashMap<String, LanguageEntry>,
    extension_map: HashMap<String, String>, // extension -> language name
}

impl LanguageRegistry {
    pub fn new(config: LanguagesConfig) -> Self {
        let mut languages = HashMap::new();
        let mut extension_map = HashMap::new();

        for lang in config.language {
            let name = lang.name.clone();
            languages.insert(name.clone(), lang);

            // Map extensions to language names
            for ext in &languages[&name].file_types {
                extension_map.insert(ext.clone(), name.clone());
            }
        }

        Self {
            languages,
            extension_map,
        }
    }

    pub fn get_language_by_extension(&self, ext: &str) -> Option<&LanguageEntry> {
        self.extension_map.get(ext)
            .and_then(|name| self.languages.get(name))
    }

    pub fn get_language_by_name(&self, name: &str) -> Option<&LanguageEntry> {
        self.languages.get(name)
    }

    pub fn languages(&self) -> impl Iterator<Item = (&String, &LanguageEntry)> {
        self.languages.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_registry() {
        // This would need a test languages.toml file
        // For now, just test the structure
        let config = LanguagesConfig {
            language: vec![
                LanguageEntry {
                    name: "rust".to_string(),
                    scope: Some("source.rust".to_string()),
                    file_types: vec!["rs".to_string()],
                    grammar: Some("rust".to_string()),
                    highlight_query: Some("runtime/queries/rust/highlights.scm".to_string()),
                    injection_query: None,
                }
            ]
        };

        let registry = LanguageRegistry::new(config);
        let rust_lang = registry.get_language_by_extension("rs").unwrap();
        assert_eq!(rust_lang.name, "rust");
    }
}