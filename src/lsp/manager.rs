// src/lsp/manager.rs - Multi-server LSP management

use super::client::LspClient;
use super::client::LspError;
use super::progress::ProgressManager;
use crate::syntax::LanguageId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;

#[derive(Debug, Clone)]
pub struct LspConfig {
    pub command: String,
    pub args: Vec<String>,
    pub trigger_characters: Vec<String>,
}

#[derive(Clone)]
pub struct LspManager {
    clients: Arc<AsyncMutex<HashMap<LanguageId, LspClient>>>,
    configs: HashMap<LanguageId, LspConfig>,
    progress_manager: Arc<ProgressManager>,
}

impl Default for LspManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LspManager {
    pub fn new() -> Self {
        let mut configs = HashMap::new();
        configs.insert(
            LanguageId::Rust,
            LspConfig {
                command: "rust-analyzer".to_string(),
                args: vec![],
                trigger_characters: vec!["::".to_string(), ".".to_string()],
            },
        );
        configs.insert(
            LanguageId::Python,
            LspConfig {
                command: "pyright-langserver".to_string(),
                args: vec!["--stdio".to_string()],
                trigger_characters: vec![".".to_string()],
            },
        );
        configs.insert(
            LanguageId::JavaScript,
            LspConfig {
                command: "typescript-language-server".to_string(),
                args: vec!["--stdio".to_string()],
                trigger_characters: vec![".".to_string()],
            },
        );
        configs.insert(
            LanguageId::TypeScript,
            LspConfig {
                command: "typescript-language-server".to_string(),
                args: vec!["--stdio".to_string()],
                trigger_characters: vec![".".to_string()],
            },
        );

        Self {
            clients: Arc::new(AsyncMutex::new(HashMap::new())),
            configs,
            progress_manager: Arc::new(ProgressManager::new()),
        }
    }

    pub async fn get_or_start_client(&self, language: LanguageId) -> Result<(), LspError> {
        let mut clients: tokio::sync::MutexGuard<'_, HashMap<LanguageId, LspClient>> =
            self.clients.lock().await;
        if let std::collections::hash_map::Entry::Vacant(e) = clients.entry(language) {
            if let Some(config) = self.configs.get(&language) {
                let mut client = LspClient::new(&config.command, &config.args).await?;
                // Initialize the client
                let workspace_folders = None; // Could be made configurable
                let root_uri = None; // Could be set to project root
                client.initialize(workspace_folders, root_uri).await?;
                e.insert(client);
            } else {
                return Err(LspError::Protocol(format!(
                    "No config for language {:?}",
                    language
                )));
            }
        }
        Ok(())
    }

    pub async fn get_client(&self, language: LanguageId) -> Option<LspClient> {
        let clients: tokio::sync::MutexGuard<'_, HashMap<LanguageId, LspClient>> =
            self.clients.lock().await;
        clients.get(&language).cloned()
    }

    pub async fn is_client_initialized(&self, language: LanguageId) -> bool {
        let clients: tokio::sync::MutexGuard<'_, HashMap<LanguageId, LspClient>> =
            self.clients.lock().await;
        clients
            .get(&language)
            .map(|c| c.is_initialized())
            .unwrap_or(false)
    }

    pub async fn shutdown_all(&self) -> Result<(), LspError> {
        let mut clients: tokio::sync::MutexGuard<'_, HashMap<LanguageId, LspClient>> =
            self.clients.lock().await;
        for client in clients.values_mut() {
            client.shutdown().await?;
        }
        Ok(())
    }

    pub fn progress_manager(&self) -> Arc<ProgressManager> {
        Arc::clone(&self.progress_manager)
    }

    pub fn is_trigger_character(&self, language: LanguageId, character: &str) -> bool {
        if let Some(config) = self.configs.get(&language) {
            config
                .trigger_characters
                .iter()
                .any(|trigger| character.ends_with(trigger))
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lsp_manager_new() {
        let manager = LspManager::new();
        {
            let clients = manager.clients.lock().await;
            assert_eq!(clients.len(), 0);
        }

        // Check that configs are initialized
        assert!(
            manager
                .configs
                .contains_key(&crate::syntax::LanguageId::Rust)
        );
        assert!(
            manager
                .configs
                .contains_key(&crate::syntax::LanguageId::Python)
        );
        assert!(
            manager
                .configs
                .contains_key(&crate::syntax::LanguageId::JavaScript)
        );
        assert!(
            manager
                .configs
                .contains_key(&crate::syntax::LanguageId::TypeScript)
        );
    }

    #[test]
    fn test_lsp_config_rust() {
        let manager = LspManager::new();
        let config = manager
            .configs
            .get(&crate::syntax::LanguageId::Rust)
            .unwrap();
        assert_eq!(config.command, "rust-analyzer");
        assert!(config.args.is_empty());
    }

    #[test]
    fn test_lsp_config_python() {
        let manager = LspManager::new();
        let config = manager
            .configs
            .get(&crate::syntax::LanguageId::Python)
            .unwrap();
        assert_eq!(config.command, "pyright-langserver");
        assert_eq!(config.args, vec!["--stdio"]);
    }

    #[test]
    fn test_lsp_config_javascript() {
        let manager = LspManager::new();
        let config = manager
            .configs
            .get(&crate::syntax::LanguageId::JavaScript)
            .unwrap();
        assert_eq!(config.command, "typescript-language-server");
        assert_eq!(config.args, vec!["--stdio"]);
    }

    #[test]
    fn test_lsp_config_typescript() {
        let manager = LspManager::new();
        let config = manager
            .configs
            .get(&crate::syntax::LanguageId::TypeScript)
            .unwrap();
        assert_eq!(config.command, "typescript-language-server");
        assert_eq!(config.args, vec!["--stdio"]);
    }

    #[tokio::test]
    async fn test_has_client() {
        let manager = LspManager::new();
        let clients = manager.clients.lock().await;
        assert!(!clients.contains_key(&crate::syntax::LanguageId::Rust));
        // Note: We can't easily test the true case without mocking
    }

    #[tokio::test]
    async fn test_shutdown_all() {
        let manager = LspManager::new();
        // Should not panic even with no clients
        let result = manager.shutdown_all().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_lsp_config_creation() {
        let config = LspConfig {
            command: "rust-analyzer".to_string(),
            args: vec![],
            trigger_characters: vec![".".to_string(), "::".to_string()],
        };

        assert_eq!(config.command, "rust-analyzer");
        assert_eq!(config.trigger_characters.len(), 2);
        assert!(config.trigger_characters.contains(&".".to_string()));
        assert!(config.trigger_characters.contains(&"::".to_string()));
    }

    #[test]
    fn test_trigger_character_detection() {
        let manager = LspManager::new();

        // Test trigger character detection for Rust
        assert!(manager.is_trigger_character(crate::syntax::LanguageId::Rust, "."));
        assert!(manager.is_trigger_character(crate::syntax::LanguageId::Rust, "::"));
        assert!(!manager.is_trigger_character(crate::syntax::LanguageId::Rust, ","));

        // Test trigger character detection for Python
        assert!(manager.is_trigger_character(crate::syntax::LanguageId::Python, "."));
        assert!(!manager.is_trigger_character(crate::syntax::LanguageId::Python, "::"));

        // Test unknown language
        assert!(!manager.is_trigger_character(crate::syntax::LanguageId::JavaScript, "unknown"));
    }

    #[test]
    fn test_language_config_lookup() {
        let manager = LspManager::new();

        // Check that all supported languages have configs
        let rust_config = manager.configs.get(&crate::syntax::LanguageId::Rust);
        assert!(rust_config.is_some());
        assert_eq!(rust_config.unwrap().command, "rust-analyzer");

        let python_config = manager.configs.get(&crate::syntax::LanguageId::Python);
        assert!(python_config.is_some());
        assert_eq!(python_config.unwrap().command, "pyright-langserver");
    }
}
