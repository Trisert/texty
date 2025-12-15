// src/lsp/diagnostics.rs - LSP diagnostics handling

use crossterm::style::Color;
use lsp_types::{Diagnostic, DiagnosticSeverity, Url};
use std::collections::HashMap;

#[derive(Debug)]
pub struct DiagnosticManager {
    diagnostics: HashMap<Url, Vec<Diagnostic>>,
}

impl Default for DiagnosticManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticManager {
    pub fn add_diagnostics(&mut self, uri: Url, diagnostics: Vec<lsp_types::Diagnostic>) {
        self.diagnostics.insert(uri, diagnostics);
    }

    pub fn get_diagnostics(&self, uri: &Url) -> &[lsp_types::Diagnostic] {
        self.diagnostics.get(uri).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn clear_diagnostics(&mut self, uri: &Url) {
        self.diagnostics.remove(uri);
    }

    pub fn clear_all_diagnostics(&mut self) {
        self.diagnostics.clear();
    }

    pub fn get_all_diagnostics(&self) -> impl Iterator<Item = (&Url, &Vec<lsp_types::Diagnostic>)> {
        self.diagnostics.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_manager_empty() {
        let manager = DiagnosticManager::new();
        assert_eq!(manager.get_diagnostics(&Url::parse("file:///test.rs").unwrap()).len(), 0);
        assert_eq!(manager.get_all_diagnostics().count(), 0);
    }

    #[test]
    fn test_diagnostic_manager_add_and_get() {
        let mut manager = DiagnosticManager::new();
        let uri = Url::parse("file:///test.rs").unwrap();

        let diagnostic = lsp_types::Diagnostic {
            range: lsp_types::Range {
                start: lsp_types::Position { line: 1, character: 5 },
                end: lsp_types::Position { line: 1, character: 15 },
            },
            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
            message: "Undefined variable".to_string(),
            source: Some("rustc".to_string()),
            ..Default::default()
        };

        manager.add_diagnostics(uri.clone(), vec![diagnostic.clone()]);

        let diagnostics = manager.get_diagnostics(&uri);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "Undefined variable");
        assert_eq!(diagnostics[0].severity, Some(lsp_types::DiagnosticSeverity::ERROR));
    }

    #[test]
    fn test_diagnostic_manager_clear() {
        let mut manager = DiagnosticManager::new();
        let uri1 = Url::parse("file:///test1.rs").unwrap();
        let uri2 = Url::parse("file:///test2.rs").unwrap();

        let diagnostic = lsp_types::Diagnostic {
            message: "Test error".to_string(),
            ..Default::default()
        };

        manager.add_diagnostics(uri1.clone(), vec![diagnostic.clone()]);
        manager.add_diagnostics(uri2.clone(), vec![diagnostic.clone()]);

        assert_eq!(manager.get_all_diagnostics().count(), 2);

        // Clear specific file
        manager.clear_diagnostics(&uri1);
        assert_eq!(manager.get_diagnostics(&uri1).len(), 0);
        assert_eq!(manager.get_diagnostics(&uri2).len(), 1);

        // Clear all
        manager.clear_all_diagnostics();
        assert_eq!(manager.get_all_diagnostics().count(), 0);
    }

    #[test]
    fn test_diagnostic_manager_multiple_files() {
        let mut manager = DiagnosticManager::new();
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

        manager.add_diagnostics(uri1.clone(), vec![diag1]);
        manager.add_diagnostics(uri2.clone(), vec![diag2]);

        let all_diags: Vec<_> = manager.get_all_diagnostics().collect();
        assert_eq!(all_diags.len(), 2);

        // Check diagnostics by URI
        let diags1 = manager.get_diagnostics(&uri1);
        assert_eq!(diags1.len(), 1);
        assert_eq!(diags1[0].message, "Error in file1");

        let diags2 = manager.get_diagnostics(&uri2);
        assert_eq!(diags2.len(), 1);
        assert_eq!(diags2[0].message, "Warning in file2");
    }
}

impl DiagnosticManager {
    pub fn new() -> Self {
        Self {
            diagnostics: HashMap::new(),
        }
    }

    pub fn update_diagnostics(&mut self, uri: Url, diagnostics: Vec<Diagnostic>) {
        self.diagnostics.insert(uri, diagnostics);
    }

    pub fn get_diagnostics_at_line(&self, uri: &Url, line: u32) -> Vec<&Diagnostic> {
        if let Some(diags) = self.diagnostics.get(uri) {
            diags
                .iter()
                .filter(|d| d.range.start.line == line)
                .collect()
        } else {
            vec![]
        }
    }

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
