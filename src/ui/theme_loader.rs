// ui/theme_loader.rs - Theme discovery, loading, and management system

use crate::syntax::Theme as SyntaxTheme;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ThemeLoaderError {
    #[error("Theme not found: {0}")]
    NotFound(String),
    #[error("Failed to read theme file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse theme TOML: {0}")]
    ParseError(String),
    #[error("Invalid theme: {0}")]
    InvalidTheme(String),
}

#[derive(Debug, Clone)]
pub struct ThemeInfo {
    pub name: String,
    pub path: PathBuf,
    pub inherits: Option<String>,
    pub description: Option<String>,
}

pub struct ThemeLoader {
    theme_directories: Vec<PathBuf>,
    theme_cache: HashMap<String, ThemeInfo>,
}

impl ThemeLoader {
    pub fn new() -> Self {
        let mut loader = Self {
            theme_directories: Vec::new(),
            theme_cache: HashMap::new(),
        };

        loader.add_default_theme_directories();
        loader
    }

    fn add_default_theme_directories(&mut self) {
        let mut dirs = vec![PathBuf::from("runtime/themes"), PathBuf::from("themes")];

        if let Ok(home) = std::env::var("HOME") {
            dirs.push(PathBuf::from(home).join(".config/texty/themes"));
        }

        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            dirs.push(PathBuf::from(xdg_config).join("texty/themes"));
        }

        for dir in dirs {
            if dir.exists() {
                self.theme_directories.push(dir);
            }
        }
    }

    pub fn add_theme_directory(&mut self, path: PathBuf) {
        if path.exists() {
            self.theme_directories.push(path);
        }
    }

    pub fn discover_themes(&mut self) -> Vec<ThemeInfo> {
        let mut themes = Vec::new();

        for dir in &self.theme_directories {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Some(ext) = entry.path().extension()
                        && ext == "toml"
                        && let Ok(info) = self.parse_theme_info(&entry.path())
                    {
                        themes.push(info);
                    }
                }
            }
        }

        self.theme_cache.clear();
        for theme in &themes {
            self.theme_cache.insert(theme.name.clone(), theme.clone());
        }

        themes.sort_by(|a, b| a.name.cmp(&b.name));
        themes
    }

    fn parse_theme_info(&self, path: &Path) -> Result<ThemeInfo, ThemeLoaderError> {
        let content = fs::read_to_string(path)?;
        let value: toml::Value =
            toml::from_str(&content).map_err(|e| ThemeLoaderError::ParseError(e.to_string()))?;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let inherits = value
            .get("inherits")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let description = value
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(ThemeInfo {
            name,
            path: path.to_path_buf(),
            inherits,
            description,
        })
    }

    pub fn load_theme(&self, name: &str) -> Result<SyntaxTheme, ThemeLoaderError> {
        if let Some(info) = self.theme_cache.get(name) {
            return self.load_theme_from_info(info);
        }

        for dir in &self.theme_directories {
            let path = dir.join(format!("{}.toml", name));
            if path.exists() {
                let info = self.parse_theme_info(&path)?;
                return self.load_theme_from_info(&info);
            }
        }

        Err(ThemeLoaderError::NotFound(name.to_string()))
    }

    fn load_theme_from_info(&self, info: &ThemeInfo) -> Result<SyntaxTheme, ThemeLoaderError> {
        SyntaxTheme::from_file(info.path.to_str().unwrap())
            .map_err(|e| ThemeLoaderError::ParseError(e.to_string()))
    }

    pub fn list_themes(&mut self) -> Vec<String> {
        self.discover_themes();
        self.theme_cache.keys().cloned().collect()
    }

    pub fn get_available_themes(&self) -> Vec<String> {
        self.theme_cache.keys().cloned().collect()
    }

    pub fn get_theme_info(&self, name: &str) -> Option<&ThemeInfo> {
        self.theme_cache.get(name)
    }

    pub fn theme_exists(&self, name: &str) -> bool {
        self.theme_cache.contains_key(name)
    }
}

impl Default for ThemeLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_loader_creation() {
        let loader = ThemeLoader::new();
        assert!(!loader.theme_directories.is_empty());
    }

    #[test]
    fn test_theme_discovery() {
        let mut loader = ThemeLoader::new();
        let themes = loader.discover_themes();
        assert!(!themes.is_empty());
        assert!(themes.iter().any(|t| t.name == "monokai"));
    }

    #[test]
    fn test_theme_loading() {
        let mut loader = ThemeLoader::new();
        loader.discover_themes();

        let theme = loader.load_theme("monokai");
        assert!(theme.is_ok());
    }

    #[test]
    fn test_theme_exists() {
        let mut loader = ThemeLoader::new();
        loader.discover_themes();

        assert!(loader.theme_exists("monokai"));
        assert!(!loader.theme_exists("nonexistent"));
    }

    #[test]
    fn test_list_themes() {
        let mut loader = ThemeLoader::new();
        let themes = loader.list_themes();
        assert!(!themes.is_empty());
        assert!(themes.contains(&"monokai".to_string()));
    }
}
