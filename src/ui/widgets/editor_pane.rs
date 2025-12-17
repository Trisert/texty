// ui/widgets/editor_pane.rs - Editor pane widget

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

/// Editor pane widget that renders the text editor content
pub struct EditorPane<'a> {
    pub editor: &'a Editor,
    pub theme: &'a Theme,
}

impl<'a> EditorPane<'a> {
    pub fn new(editor: &'a Editor, theme: &'a Theme) -> Self {
        Self { editor, theme }
    }
}

impl EditorPane<'_> {
    fn get_line_diagnostics(&self, line_idx: usize) -> Vec<lsp_types::Diagnostic> {
        if let Some(uri) = self.editor.get_buffer_uri() {
            let diags = self.editor.diagnostics.lock().unwrap();
            if let Some(file_diags) = diags.get(&uri) {
                file_diags
                    .iter()
                    .filter(|d| d.range.start.line as usize == line_idx)
                    .cloned()
                    .collect::<Vec<lsp_types::Diagnostic>>()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    fn diagnostic_color(&self, severity: &Option<DiagnosticSeverity>) -> ratatui::style::Color {
        match severity {
            Some(DiagnosticSeverity::ERROR) => self.theme.ui.diagnostic_error,
            Some(DiagnosticSeverity::WARNING) => self.theme.ui.diagnostic_warning,
            Some(DiagnosticSeverity::INFORMATION) => self.theme.ui.diagnostic_info,
            Some(DiagnosticSeverity::HINT) => self.theme.ui.diagnostic_hint,
            _ => self.theme.ui.diagnostic_error,
        }
    }
}

impl Widget for EditorPane<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for i in 0..area.height as usize {
            let line_idx = self.editor.viewport.offset_line + i;
            if let Some(line) = self.editor.buffer.line(line_idx) {
                let visible_line = line[self.editor.viewport.offset_col..].to_string();

                // Get diagnostics for this line
                let line_diagnostics = self.get_line_diagnostics(line_idx);

                if let Some(highlights) = self
                    .editor
                    .buffer
                    .highlighter
                    .as_ref()
                    .and_then(|h| h.get_line_highlights(line_idx))
                {
                    // Build spans with highlights and diagnostic overlays
                    let mut spans = Vec::new();
                    let mut pos = 0;

                    // Combine syntax highlights with diagnostic highlights
                    let mut highlight_ranges = Vec::new();

                    // Add syntax highlights
                    let line_text = self.editor.buffer.line(line_idx).unwrap();
                    let line_start_byte = self.editor.buffer.rope.line_to_byte(line_idx);

                    for token in highlights {
                        let rel_byte_start = token.start - line_start_byte;
                        let rel_byte_end = token.end - line_start_byte;
                        let char_start = line_text[0..rel_byte_start].chars().count();
                        let char_end = line_text[0..rel_byte_end].chars().count();

                        if char_start < self.editor.viewport.offset_col + visible_line.len()
                            && char_end > self.editor.viewport.offset_col
                        {
                            let start = char_start.saturating_sub(self.editor.viewport.offset_col);
                            let end = char_end
                                .min(self.editor.viewport.offset_col + visible_line.len())
                                .saturating_sub(self.editor.viewport.offset_col);

                            highlight_ranges.push((
                                start,
                                end,
                                self.theme.syntax_color(&token.capture_name),
                            ));
                        }
                    }

                    // Add diagnostic highlights (these take precedence)
                    for diag in &line_diagnostics {
                        let start_char = diag.range.start.character as usize;
                        let end_char = diag.range.end.character as usize;

                        if start_char >= self.editor.viewport.offset_col
                            && start_char < self.editor.viewport.offset_col + visible_line.len()
                        {
                            let start = start_char.saturating_sub(self.editor.viewport.offset_col);
                            let end = end_char
                                .min(self.editor.viewport.offset_col + visible_line.len())
                                .saturating_sub(self.editor.viewport.offset_col);

                            let diag_color = self.diagnostic_color(&diag.severity);
                            highlight_ranges.push((start, end, diag_color));
                        }
                    }

                    // Sort ranges by start position and merge overlapping
                    highlight_ranges.sort_by_key(|(start, _, _)| *start);
                    let mut merged_ranges: Vec<(usize, usize, ratatui::style::Color)> = Vec::new();
                    for (start, end, color) in highlight_ranges {
                        if let Some((_, last_end, _)) = merged_ranges.last_mut()
                            && *last_end >= start
                        {
                            *last_end = (*last_end).max(end);
                            continue;
                        }
                        merged_ranges.push((start, end, color));
                    }

                    // Build spans from merged ranges
                    for (start, end, color) in merged_ranges {
                        if start > pos {
                            spans.push(Span::raw(visible_line[pos..start].to_string()));
                        }
                        spans.push(Span::styled(
                            visible_line[start..end].to_string(),
                            Style::default().fg(color),
                        ));
                        pos = end;
                    }

                    if pos < visible_line.len() {
                        spans.push(Span::raw(visible_line[pos..].to_string()));
                    }

                    let line_widget = Line::from(spans);
                    buf.set_line(area.x, area.y + i as u16, &line_widget, area.width);
                } else {
                    let line_widget = Line::from(visible_line);
                    buf.set_line(area.x, area.y + i as u16, &line_widget, area.width);
                }
            } else {
                let line_widget = Line::from("~");
                buf.set_line(area.x, area.y + i as u16, &line_widget, area.width);
            }
        }

        // Render cursor
        let cursor_row = self
            .editor
            .cursor
            .line
            .saturating_sub(self.editor.viewport.offset_line) as u16;
        let cursor_col = self
            .editor
            .cursor
            .col
            .saturating_sub(self.editor.viewport.offset_col) as u16;

        if cursor_row < area.height && cursor_col < area.width {
            buf.get_mut(area.x + cursor_col, area.y + cursor_row)
                .set_style(
                    Style::default()
                        .bg(self.theme.ui.cursor_bg)
                        .fg(self.theme.ui.cursor_fg),
                );
        }
    }
}
