// src/lsp/manager.rs - Multi-server LSP management

use super::client::LspClient;
use super::client::LspError;
use crate::syntax::LanguageId;
use std::collections::HashMap;

#[derive(Debug)]
pub struct LspConfig {
    pub command: String,
    pub args: Vec<String>,
}

pub struct LspManager {
    clients: HashMap<LanguageId, LspClient>,
    configs: HashMap<LanguageId, LspConfig>,
}

impl Default for LspManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_manager_new() {
        let manager = LspManager::new();
        assert_eq!(manager.clients.len(), 0);

        // Check that configs are initialized
        assert!(manager.configs.contains_key(&crate::syntax::LanguageId::Rust));
        assert!(manager.configs.contains_key(&crate::syntax::LanguageId::Python));
        assert!(manager.configs.contains_key(&crate::syntax::LanguageId::JavaScript));
        assert!(manager.configs.contains_key(&crate::syntax::LanguageId::TypeScript));
    }

    #[test]
    fn test_lsp_config_rust() {
        let manager = LspManager::new();
        let config = manager.configs.get(&crate::syntax::LanguageId::Rust).unwrap();
        assert_eq!(config.command, "rust-analyzer");
        assert!(config.args.is_empty());
    }

    #[test]
    fn test_lsp_config_python() {
        let manager = LspManager::new();
        let config = manager.configs.get(&crate::syntax::LanguageId::Python).unwrap();
        assert_eq!(config.command, "pyright-langserver");
        assert_eq!(config.args, vec!["--stdio"]);
    }

    #[test]
    fn test_lsp_config_javascript() {
        let manager = LspManager::new();
        let config = manager.configs.get(&crate::syntax::LanguageId::JavaScript).unwrap();
        assert_eq!(config.command, "typescript-language-server");
        assert_eq!(config.args, vec!["--stdio"]);
    }

    #[test]
    fn test_lsp_config_typescript() {
        let manager = LspManager::new();
        let config = manager.configs.get(&crate::syntax::LanguageId::TypeScript).unwrap();
        assert_eq!(config.command, "typescript-language-server");
        assert_eq!(config.args, vec!["--stdio"]);
    }



    #[test]
    fn test_has_client() {
        let manager = LspManager::new();
        assert!(!manager.clients.contains_key(&crate::syntax::LanguageId::Rust));
        // Note: We can't easily test the true case without mocking
    }

    #[test]
    fn test_shutdown_all() {
        let mut manager = LspManager::new();
        // Should not panic even with no clients
        assert!(manager.shutdown_all().is_ok());
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
            },
        );
        configs.insert(
            LanguageId::Python,
            LspConfig {
                command: "pyright-langserver".to_string(),
                args: vec!["--stdio".to_string()],
            },
        );
        configs.insert(
            LanguageId::JavaScript,
            LspConfig {
                command: "typescript-language-server".to_string(),
                args: vec!["--stdio".to_string()],
            },
        );
        configs.insert(
            LanguageId::TypeScript,
            LspConfig {
                command: "typescript-language-server".to_string(),
                args: vec!["--stdio".to_string()],
            },
        );

        Self {
            clients: HashMap::new(),
            configs,
        }
    }

    pub fn get_or_start_client(
        &mut self,
        language: LanguageId,
    ) -> Result<&mut LspClient, LspError> {
        if !self.clients.contains_key(&language) {
            if let Some(config) = self.configs.get(&language) {
                let mut client = LspClient::new(&config.command, &config.args)?;
                // Initialize the client
                let workspace_folders = None; // Could be made configurable
                client.initialize(workspace_folders)?;
                self.clients.insert(language, client);
            } else {
                return Err(LspError::Protocol(format!(
                    "No config for language {:?}",
                    language
                )));
            }
        }
        Ok(self.clients.get_mut(&language).unwrap())
    }

    pub fn is_client_initialized(&self, language: LanguageId) -> bool {
        self.clients
            .get(&language)
            .map(|c| c.is_initialized())
            .unwrap_or(false)
    }

    pub fn shutdown_all(&mut self) -> Result<(), LspError> {
        for client in self.clients.values_mut() {
            client.shutdown()?;
        }
        Ok(())
    }
}
