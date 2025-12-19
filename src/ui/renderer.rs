// ui/renderer.rs - Ratatui-based renderer for the text editor

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
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
    pub fn draw(&mut self, editor: &mut Editor) -> Result<(), Box<dyn std::error::Error>> {
        self.terminal.draw(|f| {
            let size = f.size();

            // Check if fuzzy search is active
            let fuzzy_search_active = editor.fuzzy_search.is_some();

            let (_fuzzy_area, content_area) = if fuzzy_search_active {
                let show_preview = editor
                    .fuzzy_search
                    .as_ref()
                    .map(|_| false)
                    .unwrap_or(false);

                if show_preview {
                    // When preview is enabled, fuzzy search takes full screen
                    if let Some(fuzzy_state) = &mut editor.fuzzy_search {
                        let fuzzy_widget = FuzzySearchWidget::new(fuzzy_state, &self.theme, None);
                        f.render_widget(fuzzy_widget, size);
                    }
                    (None, Rect::default()) // No content area when preview is full screen
                } else {
                    // Original behavior: split screen when no preview
                    let fuzzy_width = FuzzySearchWidget::calculate_width(size.width, show_preview);
                    let main_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Length(fuzzy_width), // Fuzzy search width
                            Constraint::Min(1),              // Content area (editor)
                        ])
                        .split(size);

                    // Render fuzzy search in left panel
                    if let Some(fuzzy_state) = &mut editor.fuzzy_search {
                        let fuzzy_widget = FuzzySearchWidget::new(fuzzy_state, &self.theme, None);
                        f.render_widget(fuzzy_widget, main_chunks[0]);
                    }

                    (Some(main_chunks[0]), main_chunks[1]) // Return both areas
                }
            } else {
                (None, size) // No fuzzy area, content gets full screen
            };

            // Only render editor if there's a valid content area (not empty when preview is full screen)
            if content_area.width > 0 && content_area.height > 0 {
                // Render editor in content area
                // Create main layout: editor area + status bar
                let vertical_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(1),    // Editor area
                        Constraint::Length(1), // Status bar (1 line)
                    ])
                    .split(content_area);

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

                // Set cursor (only when editor is visible and not in fuzzy search mode)
                if !fuzzy_search_active {
                    let cursor_row = editor
                        .cursor
                        .line
                        .saturating_sub(editor.viewport.offset_line)
                        as u16;
                    let cursor_col =
                        editor.cursor.col.saturating_sub(editor.viewport.offset_col) as u16;
                    if cursor_row < editor_chunks[1].height && cursor_col < editor_chunks[1].width {
                        f.set_cursor(
                            editor_chunks[1].x + cursor_col,
                            editor_chunks[1].y + cursor_row,
                        );
                    }
                }
            }

            // Render status bar at the bottom of the terminal
            let status_bar_area = Rect {
                x: 0,
                y: size.height - 1,
                width: size.width,
                height: 1,
            };

            if editor.mode == crate::mode::Mode::Command {
                // Show command line on status bar line, filling full width
                let command_text = editor.get_command_line_display();
                let padded_command = if command_text.len() < status_bar_area.width as usize {
                    format!(
                        "{}{}",
                        command_text,
                        " ".repeat(status_bar_area.width as usize - command_text.len())
                    )
                } else {
                    command_text
                };
                let command_line = ratatui::text::Line::from(padded_command).style(
                    Style::default()
                        .bg(self.theme.ui.status_bar_bg)
                        .fg(self.theme.ui.status_bar_fg),
                );
                f.buffer_mut()
                    .set_line(0, status_bar_area.y, &command_line, status_bar_area.width);
            } else {
                // Show normal status bar
                f.render_widget(StatusBar::new(editor, &self.theme), status_bar_area);
            }

            // Render overlays (floating windows)
            // Calculate cursor position relative to content area
            let (cursor_x, cursor_y) = if fuzzy_search_active {
                // When fuzzy search is active, cursor is not visible, use center of content area
                (
                    content_area.x + content_area.width / 2,
                    content_area.y + content_area.height / 2,
                )
            } else {
                (
                    content_area.x
                        + 4
                        + editor.cursor.col.saturating_sub(editor.viewport.offset_col) as u16, // +4 for gutter
                    content_area.y
                        + editor
                            .cursor
                            .line
                            .saturating_sub(editor.viewport.offset_line)
                            as u16,
                )
            };

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
        })?;
        Ok(())
    }
}

