// ui/theme_manager.rs - Central theme management and switching

use super::Theme;
use super::theme_loader::ThemeLoader;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub enum ThemeChangeResult {
    Switched(String),
    Error(String),
}

pub struct ThemeManager {
    loader: Arc<RwLock<ThemeLoader>>,
    current_theme: Arc<RwLock<Theme>>,
}

impl ThemeManager {
    pub fn new() -> Self {
        let mut loader = ThemeLoader::new();
        loader.discover_themes();

        let current_theme = Arc::new(RwLock::new(Theme::default()));

        Self {
            loader: Arc::new(RwLock::new(loader)),
            current_theme,
        }
    }

    pub fn get_current_theme(&self) -> Theme {
        self.current_theme.read().unwrap().clone()
    }

    pub fn set_theme(&self, theme: Theme) {
        *self.current_theme.write().unwrap() = theme;
    }

    pub fn list_available_themes(&self) -> Vec<String> {
        let mut loader = self.loader.write().unwrap();
        loader.list_themes()
    }

    pub fn switch_theme(&self, name: &str) -> ThemeChangeResult {
        let loader = self.loader.read().unwrap();

        match loader.load_theme(name) {
            Ok(syntax_theme) => {
                let mut current = self.current_theme.write().unwrap();
                *current = Theme {
                    use_terminal_palette: false,
                    terminal_palette: None,
                    named_theme: Some(name.to_string()),
                    loaded_syntax_theme: Some(syntax_theme),
                    ..Default::default()
                };
                ThemeChangeResult::Switched(name.to_string())
            }
            Err(e) => ThemeChangeResult::Error(e.to_string()),
        }
    }

    pub fn use_terminal_palette(&self) {
        let mut current = self.current_theme.write().unwrap();
        *current = Theme::with_terminal_palette();
    }

    pub fn reload_themes(&self) {
        let mut loader = self.loader.write().unwrap();
        loader.discover_themes();
    }

    pub fn theme_exists(&self, name: &str) -> bool {
        self.loader.read().unwrap().theme_exists(name)
    }

    pub fn get_theme_info(&self, name: &str) -> Option<String> {
        self.loader
            .read()
            .unwrap()
            .get_theme_info(name)
            .map(|info| {
                format!(
                    "{}{}{}",
                    info.name,
                    info.inherits
                        .as_ref()
                        .map(|p| format!(" (inherits: {})", p))
                        .unwrap_or_default(),
                    info.description
                        .as_ref()
                        .map(|d| format!(" - {}", d))
                        .unwrap_or_default()
                )
            })
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_manager_creation() {
        let manager = ThemeManager::new();
        let themes = manager.list_available_themes();
        assert!(!themes.is_empty());
    }

    #[test]
    fn test_theme_switching() {
        let manager = ThemeManager::new();

        if manager.theme_exists("monokai") {
            let result = manager.switch_theme("monokai");
            assert!(matches!(result, ThemeChangeResult::Switched(_)));

            let current = manager.get_current_theme();
            assert_eq!(current.named_theme, Some("monokai".to_string()));
        }
    }

    #[test]
    fn test_terminal_palette_switching() {
        let manager = ThemeManager::new();

        manager.use_terminal_palette();
        let current = manager.get_current_theme();
        assert!(current.use_terminal_palette);
        assert!(current.terminal_palette.is_some());
    }

    #[test]
    fn test_theme_reload() {
        let manager = ThemeManager::new();
        manager.reload_themes();

        let themes = manager.list_available_themes();
        assert!(!themes.is_empty());
    }

    #[test]
    fn test_theme_exists() {
        let manager = ThemeManager::new();
        assert!(manager.theme_exists("monokai"));
        assert!(!manager.theme_exists("nonexistent_theme"));
    }

    #[test]
    fn test_invalid_theme_switch() {
        let manager = ThemeManager::new();

        let result = manager.switch_theme("nonexistent_theme");
        assert!(matches!(result, ThemeChangeResult::Error(_)));
    }

    #[test]
    fn test_get_current_theme() {
        let manager = ThemeManager::new();

        let theme = manager.get_current_theme();
        assert_eq!(theme.named_theme, None);
        assert!(!theme.use_terminal_palette);
    }

    #[test]
    fn test_set_theme() {
        let manager = ThemeManager::new();

        let new_theme = Theme::with_terminal_palette();
        manager.set_theme(new_theme.clone());

        let current = manager.get_current_theme();
        assert_eq!(current.use_terminal_palette, new_theme.use_terminal_palette);
    }
}
