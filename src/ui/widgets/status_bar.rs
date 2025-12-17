// ui/widgets/status_bar.rs - Status bar widget

use ratatui::{buffer::Buffer, layout::Rect, style::Style, text::Line, widgets::Widget};

use crate::editor::Editor;
use crate::ui::theme::Theme;

/// Status bar widget showing mode, file info, cursor position, LSP status
pub struct StatusBar<'a> {
    pub editor: &'a Editor,
    pub theme: &'a Theme,
}

impl<'a> StatusBar<'a> {
    pub fn new(editor: &'a Editor, theme: &'a Theme) -> Self {
        Self { editor, theme }
    }
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lsp_status = self.get_lsp_status();
        let progress_info = self.get_progress_info();

        // Format the status line
        let base_status = format!(
            " {} | {}:{} | Modified: {}",
            mode_to_str(&self.editor.mode),
            self.editor.cursor.line,
            self.editor.cursor.col,
            self.editor.buffer.modified
        );

        let status = if let Some(msg) = &self.editor.status_message {
            format!("{} | {}", base_status, msg)
        } else {
            format!(
                "{} | LSP: {}{}",
                base_status,
                lsp_status,
                if progress_info.is_empty() {
                    String::new()
                } else {
                    format!(" | {}", progress_info)
                }
            )
        };

        // Pad the status text to fill the entire width
        let padded_status = if status.len() < area.width as usize {
            format!(
                "{}{}",
                status,
                " ".repeat(area.width as usize - status.len())
            )
        } else {
            status
        };

        let line_widget = Line::from(padded_status).style(
            Style::default()
                .bg(self.theme.ui.status_bar_bg)
                .fg(self.theme.ui.status_bar_fg),
        );

        buf.set_line(area.x, area.y, &line_widget, area.width);
    }
}

impl StatusBar<'_> {
    fn get_lsp_status(&self) -> &'static str {
        if let Some(_language) = self.editor.current_language {
            // Check if we have an LSP client for this language
            // Since we can't do async operations in render, we'll use a simple check
            // TODO: Implement proper LSP status checking
            "ready"
        } else {
            "none"
        }
    }

    fn get_progress_info(&self) -> String {
        let progress_items = self.editor.progress_items.lock().unwrap();
        if progress_items.is_empty() {
            String::new()
        } else {
            // Show the first active progress item
            let item = &progress_items[0];
            let percentage = item
                .percentage
                .map(|p| format!("{}%", p))
                .unwrap_or_default();
            let message = item.message.as_deref().unwrap_or("");
            format!("{} {}{}", item.title, message, percentage)
                .trim()
                .to_string()
        }
    }
}

fn mode_to_str(mode: &crate::mode::Mode) -> &'static str {
    match mode {
        crate::mode::Mode::Normal => "NORMAL",
        crate::mode::Mode::Insert => "INSERT",
        crate::mode::Mode::Visual => "VISUAL",
        crate::mode::Mode::Command => "COMMAND",
        crate::mode::Mode::FuzzySearch => "FUZZY",
    }
}
