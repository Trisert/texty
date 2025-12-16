// ui/widgets/gutter.rs - Gutter widget for line numbers and diagnostics

use lsp_types::DiagnosticSeverity;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Widget,
};

use crate::editor::Editor;
use crate::ui::theme::Theme;

/// Gutter widget that shows line numbers and diagnostic indicators
pub struct Gutter<'a> {
    pub editor: &'a Editor,
    pub theme: &'a Theme,
}

impl<'a> Gutter<'a> {
    pub fn new(editor: &'a Editor, theme: &'a Theme) -> Self {
        Self { editor, theme }
    }
}

impl Widget for Gutter<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for i in 0..area.height as usize {
            let line_idx = self.editor.viewport.offset_line + i;
            let line_number = line_idx + 1; // 1-based

            // Get diagnostics for this line
            let diagnostic_symbol = self.get_diagnostic_symbol(line_idx as u32);

            let text = if self.editor.buffer.line(line_idx).is_some() {
                format!("{:>3}{}{}", line_number, diagnostic_symbol, " ")
            } else {
                format!("    {}", diagnostic_symbol)
            };

            let line_widget = Line::from(Span::styled(
                text,
                Style::default().fg(self.theme.ui.gutter_fg),
            ));

            buf.set_line(area.x, area.y + i as u16, &line_widget, area.width);
        }
    }
}

impl Gutter<'_> {
    fn get_diagnostic_symbol(&self, line: u32) -> &'static str {
        if let Some(uri) = self.editor.get_buffer_uri() {
            let diagnostics = {
                let diags = self.editor.diagnostics.lock().unwrap();
                if let Some(file_diags) = diags.get(&uri) {
                    file_diags
                        .iter()
                        .filter(|d| d.range.start.line == line)
                        .cloned()
                        .collect::<Vec<lsp_types::Diagnostic>>()
                } else {
                    vec![]
                }
            };

            // Find the most severe diagnostic
            let mut most_severe = None;
            for diag in diagnostics {
                match diag.severity {
                    Some(DiagnosticSeverity::ERROR) => return "●",
                    Some(DiagnosticSeverity::WARNING) => most_severe = Some("▲"),
                    Some(DiagnosticSeverity::INFORMATION) => {
                        if most_severe.is_none() {
                            most_severe = Some("◆");
                        }
                    }
                    Some(DiagnosticSeverity::HINT) => {
                        if most_severe.is_none() {
                            most_severe = Some("◇");
                        }
                    }
                    _ => {}
                }
            }

            most_severe.unwrap_or("")
        } else {
            ""
        }
    }
}
