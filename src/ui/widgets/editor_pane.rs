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
        // Clear entire editor area to prevent character artifacts during file switching
        for y in 0..area.height {
            for x in 0..area.width {
                buf.get_mut(area.x + x, area.y + y)
                    .set_char(' ')
                    .set_style(Style::default().bg(self.theme.general.background));
            }
        }

        for i in 0..area.height as usize {
            let line_idx = self.editor.viewport.offset_line + i;
            if let Some(line) = self.editor.buffer.line(line_idx) {
                let visible_line = line
                    .chars()
                    .skip(self.editor.viewport.offset_col)
                    .collect::<String>();

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

                        // Calculate character positions with bounds checking
                        let char_start = line_text
                            .char_indices()
                            .find(|(byte_idx, _)| *byte_idx == rel_byte_start)
                            .map(|(idx, _)| idx)
                            .unwrap_or(line_text.len());
                        let char_end = line_text
                            .char_indices()
                            .find(|(byte_idx, _)| *byte_idx == rel_byte_end)
                            .map(|(idx, _)| idx)
                            .unwrap_or(line_text.len());

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
                            let start_idx = start.min(visible_line.len());
                            spans.push(Span::styled(
                                visible_line[pos..start_idx].to_string(),
                                Style::default().fg(self.theme.general.foreground),
                            ));
                        }
                        let end_idx = end.min(visible_line.len());
                        let clamped_start = start.min(end_idx);
                        spans.push(Span::styled(
                            visible_line[clamped_start..end_idx].to_string(),
                            Style::default().fg(color),
                        ));
                        pos = end;
                    }

                    if pos < visible_line.len() {
                        spans.push(Span::styled(
                            visible_line[pos..].to_string(),
                            Style::default().fg(self.theme.general.foreground),
                        ));
                    }

                    let line_widget = Line::from(spans);
                    buf.set_line(area.x, area.y + i as u16, &line_widget, area.width);
                } else {
                    let line_widget = Line::from(vec![Span::styled(
                        visible_line,
                        Style::default().fg(self.theme.general.foreground),
                    )]);
                    buf.set_line(area.x, area.y + i as u16, &line_widget, area.width);
                }
            } else {
                let line_widget = Line::from(vec![Span::styled(
                    "~",
                    Style::default().fg(self.theme.general.foreground),
                )]);
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
