// src/lsp/diagnostics.rs - LSP diagnostics handling

use crossterm::style::Color;
use lsp_types::{Diagnostic, DiagnosticSeverity, Url};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;

/// Manages LSP diagnostics with thread-safe async access
///
/// This manager stores diagnostics per file URI and provides methods
/// to add, retrieve, and clear diagnostic information. All operations
/// are async to work seamlessly with the LSP async runtime.
#[derive(Debug, Clone)]
pub struct DiagnosticManager {
    diagnostics: Arc<AsyncMutex<HashMap<Url, Vec<Diagnostic>>>>,
}

impl Default for DiagnosticManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticManager {
    pub fn new() -> Self {
        Self {
            diagnostics: Arc::new(AsyncMutex::new(HashMap::new())),
        }
    }

    /// Add diagnostics for a specific file URI
    pub async fn add_diagnostics(&self, uri: Url, diagnostics: Vec<lsp_types::Diagnostic>) {
        let mut diags = self.diagnostics.lock().await;
        diags.insert(uri, diagnostics);
    }

    /// Get all diagnostics for a specific file URI
    pub async fn get_diagnostics(&self, uri: &Url) -> Vec<Diagnostic> {
        let diags = self.diagnostics.lock().await;
        diags.get(uri).cloned().unwrap_or_default()
    }

    /// Clear diagnostics for a specific file URI
    pub async fn clear_diagnostics(&self, uri: &Url) {
        let mut diags = self.diagnostics.lock().await;
        diags.remove(uri);
    }

    /// Clear all diagnostics
    pub async fn clear_all_diagnostics(&self) {
        let mut diags = self.diagnostics.lock().await;
        diags.clear();
    }

    /// Get all diagnostics across all files
    pub async fn get_all_diagnostics(&self) -> Vec<(Url, Vec<Diagnostic>)> {
        let diags = self.diagnostics.lock().await;
        diags.clone().into_iter().collect()
    }

    /// Update diagnostics for a file (alias for add_diagnostics)
    pub async fn update_diagnostics(&self, uri: Url, diagnostics: Vec<Diagnostic>) {
        self.add_diagnostics(uri, diagnostics).await;
    }

    /// Get diagnostics for a specific line in a file
    pub async fn get_diagnostics_at_line(&self, uri: &Url, line: u32) -> Vec<Diagnostic> {
        let diags = self.diagnostics.lock().await;
        if let Some(file_diags) = diags.get(uri) {
            file_diags
                .iter()
                .filter(|d| d.range.start.line == line)
                .cloned()
                .collect()
        } else {
            vec![]
        }
    }

    /// Convert diagnostic severity to a terminal color
    pub fn diagnostic_to_color(severity: DiagnosticSeverity) -> Color {
        match severity {
            DiagnosticSeverity::ERROR => Color::Red,
            DiagnosticSeverity::WARNING => Color::Yellow,
            DiagnosticSeverity::INFORMATION => Color::Blue,
            DiagnosticSeverity::HINT => Color::Cyan,
            _ => Color::White,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{DiagnosticSeverity, Position, Range};

    #[test]
    fn test_diagnostic_manager_empty() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let manager = DiagnosticManager::new();
        let uri = Url::parse("file:///test.rs").unwrap();

        let diagnostics = rt.block_on(manager.get_diagnostics(&uri));
        let all_diagnostics = rt.block_on(manager.get_all_diagnostics());

        assert_eq!(diagnostics.len(), 0);
        assert_eq!(all_diagnostics.len(), 0);
    }

    #[test]
    fn test_diagnostic_manager_add_and_get() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let manager = DiagnosticManager::new();
        let uri = Url::parse("file:///test.rs").unwrap();

        let diagnostic = lsp_types::Diagnostic {
            range: lsp_types::Range {
                start: lsp_types::Position {
                    line: 1,
                    character: 5,
                },
                end: lsp_types::Position {
                    line: 1,
                    character: 15,
                },
            },
            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
            message: "Undefined variable".to_string(),
            source: Some("rustc".to_string()),
            ..Default::default()
        };

        rt.block_on(manager.add_diagnostics(uri.clone(), vec![diagnostic.clone()]));

        let diagnostics = rt.block_on(manager.get_diagnostics(&uri));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "Undefined variable");
        assert_eq!(
            diagnostics[0].severity,
            Some(lsp_types::DiagnosticSeverity::ERROR)
        );
    }

    #[test]
    fn test_diagnostic_manager_clear() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let manager = DiagnosticManager::new();
        let uri1 = Url::parse("file:///test1.rs").unwrap();
        let uri2 = Url::parse("file:///test2.rs").unwrap();

        let diagnostic = lsp_types::Diagnostic {
            message: "Test error".to_string(),
            ..Default::default()
        };

        rt.block_on(manager.add_diagnostics(uri1.clone(), vec![diagnostic.clone()]));
        rt.block_on(manager.add_diagnostics(uri2.clone(), vec![diagnostic.clone()]));

        let all_diags = rt.block_on(manager.get_all_diagnostics());
        assert_eq!(all_diags.len(), 2);

        // Clear specific file
        rt.block_on(manager.clear_diagnostics(&uri1));
        let diags1 = rt.block_on(manager.get_diagnostics(&uri1));
        let diags2 = rt.block_on(manager.get_diagnostics(&uri2));
        assert_eq!(diags1.len(), 0);
        assert_eq!(diags2.len(), 1);

        // Clear all
        rt.block_on(manager.clear_all_diagnostics());
        let all_diags = rt.block_on(manager.get_all_diagnostics());
        assert_eq!(all_diags.len(), 0);
    }

    #[test]
    fn test_diagnostic_manager_multiple_files() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let manager = DiagnosticManager::new();
        let uri1 = Url::parse("file:///file1.rs").unwrap();
        let uri2 = Url::parse("file:///file2.rs").unwrap();

        let diag1 = lsp_types::Diagnostic {
            message: "Error in file1".to_string(),
            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
            ..Default::default()
        };

        let diag2 = lsp_types::Diagnostic {
            message: "Warning in file2".to_string(),
            severity: Some(lsp_types::DiagnosticSeverity::WARNING),
            ..Default::default()
        };

        rt.block_on(manager.add_diagnostics(uri1.clone(), vec![diag1]));
        rt.block_on(manager.add_diagnostics(uri2.clone(), vec![diag2]));

        let all_diags = rt.block_on(manager.get_all_diagnostics());
        assert_eq!(all_diags.len(), 2);

        // Check diagnostics by URI
        let diags1 = rt.block_on(manager.get_diagnostics(&uri1));
        assert_eq!(diags1.len(), 1);
        assert_eq!(diags1[0].message, "Error in file1");

        let diags2 = rt.block_on(manager.get_diagnostics(&uri2));
        assert_eq!(diags2.len(), 1);
        assert_eq!(diags2[0].message, "Warning in file2");
    }

    #[test]
    fn test_diagnostic_manager_operations() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let manager = DiagnosticManager::new();
        let uri = Url::parse("file:///test.rs").unwrap();

        let diagnostic = Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 10,
                },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            message: "Test error".to_string(),
            ..Default::default()
        };

        // Test async add_diagnostics
        rt.block_on(manager.add_diagnostics(uri.clone(), vec![diagnostic.clone()]));

        // Test async get_diagnostics
        let diagnostics = rt.block_on(manager.get_diagnostics(&uri));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "Test error");

        // Test clear diagnostics
        rt.block_on(manager.clear_diagnostics(&uri));
        let diagnostics = rt.block_on(manager.get_diagnostics(&uri));
        assert_eq!(diagnostics.len(), 0);
    }

    #[test]
    fn test_diagnostic_async_line_access() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let manager = DiagnosticManager::new();
        let uri = Url::parse("file:///async_test.rs").unwrap();

        let diagnostic = Diagnostic {
            range: Range {
                start: Position {
                    line: 1,
                    character: 5,
                },
                end: Position {
                    line: 1,
                    character: 15,
                },
            },
            severity: Some(DiagnosticSeverity::WARNING),
            message: "Async warning".to_string(),
            ..Default::default()
        };

        // Add diagnostics
        rt.block_on(manager.add_diagnostics(uri.clone(), vec![diagnostic.clone()]));

        // Test async line access
        let line_diags = rt.block_on(manager.get_diagnostics_at_line(&uri, 1));
        assert_eq!(line_diags.len(), 1);
        assert_eq!(line_diags[0].message, "Async warning");
        assert_eq!(line_diags[0].severity, Some(DiagnosticSeverity::WARNING));

        // Test line filtering
        let no_diags = rt.block_on(manager.get_diagnostics_at_line(&uri, 0));
        assert_eq!(no_diags.len(), 0);
    }

    #[test]
    fn test_diagnostic_severity_colors() {
        assert_eq!(
            DiagnosticManager::diagnostic_to_color(DiagnosticSeverity::ERROR),
            Color::Red
        );
        assert_eq!(
            DiagnosticManager::diagnostic_to_color(DiagnosticSeverity::WARNING),
            Color::Yellow
        );
        assert_eq!(
            DiagnosticManager::diagnostic_to_color(DiagnosticSeverity::INFORMATION),
            Color::Blue
        );
        assert_eq!(
            DiagnosticManager::diagnostic_to_color(DiagnosticSeverity::HINT),
            Color::Cyan
        );
    }

    #[test]
    fn test_diagnostic_range_filtering() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let manager = DiagnosticManager::new();
        let uri = Url::parse("file:///range_test.rs").unwrap();

        let diagnostics = vec![
            Diagnostic {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 5,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: "Line 0 error".to_string(),
                ..Default::default()
            },
            Diagnostic {
                range: Range {
                    start: Position {
                        line: 1,
                        character: 10,
                    },
                    end: Position {
                        line: 1,
                        character: 20,
                    },
                },
                severity: Some(DiagnosticSeverity::WARNING),
                message: "Line 1 warning".to_string(),
                ..Default::default()
            },
            Diagnostic {
                range: Range {
                    start: Position {
                        line: 1,
                        character: 25,
                    },
                    end: Position {
                        line: 1,
                        character: 30,
                    },
                },
                severity: Some(DiagnosticSeverity::INFORMATION),
                message: "Line 1 info".to_string(),
                ..Default::default()
            },
        ];

        rt.block_on(manager.add_diagnostics(uri.clone(), diagnostics));

        // Test line 0 diagnostics (using async method)
        let line_0_diags = rt.block_on(manager.get_diagnostics_at_line(&uri, 0));
        assert_eq!(line_0_diags.len(), 1);
        assert_eq!(line_0_diags[0].message, "Line 0 error");

        // Test line 1 diagnostics (using async method)
        let line_1_diags = rt.block_on(manager.get_diagnostics_at_line(&uri, 1));
        assert_eq!(line_1_diags.len(), 2);
        assert!(line_1_diags.iter().any(|d| d.message == "Line 1 warning"));
        assert!(line_1_diags.iter().any(|d| d.message == "Line 1 info"));
    }
}
