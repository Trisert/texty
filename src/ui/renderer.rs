// ui/renderer.rs - Ratatui-based renderer for the text editor

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::Style,
};
use std::io::Stdout;

use crate::editor::Editor;
use crate::ui::theme::Theme;
use crate::ui::widgets::editor_pane::EditorPane;
use crate::ui::widgets::fuzzy_search::FuzzySearchWidget;
use crate::ui::widgets::gutter::Gutter;
use crate::ui::widgets::hover::HoverWindow;
use crate::ui::widgets::menu::CodeActionMenu;
use crate::ui::widgets::status_bar::StatusBar;

/// Ratatui-based renderer for the text editor
pub struct TuiRenderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    theme: Theme,
}

impl TuiRenderer {
    /// Create a new TuiRenderer
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend)?;
        let mut theme = Theme::default();

        // Try to load syntax theme
        if let Ok(loaded_theme) = crate::syntax::Theme::from_file("runtime/themes/default.toml") {
            theme.loaded_syntax_theme = Some(loaded_theme);
        }

        Ok(Self { terminal, theme })
    }

    /// Draw the editor UI
    pub fn draw(&mut self, editor: &Editor) -> Result<(), Box<dyn std::error::Error>> {
        self.terminal.draw(|f| {
            let size = f.size();

            // Create main layout: editor area + status bar
            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),    // Editor area
                    Constraint::Length(1), // Status bar (1 line)
                ])
                .split(size);

            // Split editor area: gutter + text
            let editor_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(4), // Gutter
                    Constraint::Min(1),    // Text area
                ])
                .split(vertical_chunks[0]);

            // Render gutter
            f.render_widget(Gutter::new(editor, &self.theme), editor_chunks[0]);

            // Render editor pane
            f.render_widget(EditorPane::new(editor, &self.theme), editor_chunks[1]);

            // Set cursor
            let cursor_row = editor
                .cursor
                .line
                .saturating_sub(editor.viewport.offset_line) as u16;
            let cursor_col = editor.cursor.col.saturating_sub(editor.viewport.offset_col) as u16;
            if cursor_row < editor_chunks[1].height && cursor_col < editor_chunks[1].width {
                f.set_cursor(
                    editor_chunks[1].x + cursor_col,
                    editor_chunks[1].y + cursor_row,
                );
            }

            // Render status bar or command line
            if editor.mode == crate::mode::Mode::Command {
                // Show command line on status bar line, filling full width
                let command_text = editor.get_command_line_display();
                let padded_command = if command_text.len() < vertical_chunks[1].width as usize {
                    format!(
                        "{}{}",
                        command_text,
                        " ".repeat(vertical_chunks[1].width as usize - command_text.len())
                    )
                } else {
                    command_text
                };
                let command_line = ratatui::text::Line::from(padded_command).style(
                    Style::default()
                        .bg(self.theme.ui.status_bar_bg)
                        .fg(self.theme.ui.status_bar_fg),
                );
                f.buffer_mut().set_line(
                    0,
                    vertical_chunks[1].y,
                    &command_line,
                    vertical_chunks[1].width,
                );
            } else {
                // Show normal status bar
                f.render_widget(StatusBar::new(editor, &self.theme), vertical_chunks[1]);
            }

            // Render overlays (floating windows)
            let cursor_x = editor_chunks[1].x
                + editor.cursor.col.saturating_sub(editor.viewport.offset_col) as u16;
            let cursor_y = editor_chunks[1].y
                + editor
                    .cursor
                    .line
                    .saturating_sub(editor.viewport.offset_line) as u16;

            // Render hover window if active
            if let Some(content) = &editor.hover_content {
                let hover_window = HoverWindow::new(content.clone(), &self.theme);
                let hover_area = hover_window.calculate_position(cursor_x, cursor_y, size);
                f.render_widget(hover_window, hover_area);
            }

            // Render code action menu if active
            if let Some(actions) = &editor.code_actions {
                let mut menu = CodeActionMenu::new(actions.clone(), &self.theme);
                menu.selected_index = editor.code_action_selected;
                let menu_area = menu.calculate_position(cursor_x, cursor_y, size);
                f.render_widget(menu, menu_area);
            }

            // Render fuzzy search if active
            if let Some(fuzzy_state) = &editor.fuzzy_search {
                let fuzzy_widget = FuzzySearchWidget::new(fuzzy_state);
                let fuzzy_area = FuzzySearchWidget::calculate_position(size.width, size.height);
                f.render_widget(fuzzy_widget, fuzzy_area);
            }
        })?;
        Ok(())
    }
}
