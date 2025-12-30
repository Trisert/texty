// ui/theme.rs - Theme system for UI styling

use super::system_theme::TerminalPalette;
use ratatui::style::Color;

/// Theme configuration
#[derive(Debug, Clone, Default)]
pub struct Theme {
    pub general: GeneralTheme,
    pub syntax: SyntaxTheme,
    pub ui: UiTheme,
    pub editor: EditorTheme,
    pub popup: PopupTheme,
    pub loaded_syntax_theme: Option<crate::syntax::Theme>,
    pub use_terminal_palette: bool,
    pub terminal_palette: Option<TerminalPalette>,
    pub named_theme: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GeneralTheme {
    pub background: Color,
    pub foreground: Color,
}

#[derive(Debug, Clone)]
pub struct SyntaxTheme {
    pub keyword: Color,
    pub function: Color,
    pub r#type: Color,
    pub string: Color,
    pub comment: Color,
    pub variable: Color,
}

#[derive(Debug, Clone)]
pub struct UiTheme {
    pub status_bar_bg: Color,
    pub status_bar_fg: Color,
    pub gutter_fg: Color,
    pub cursor_bg: Color,
    pub cursor_fg: Color,
    pub diagnostic_error: Color,
    pub diagnostic_warning: Color,
    pub diagnostic_info: Color,
    pub diagnostic_hint: Color,
}

#[derive(Debug, Clone)]
pub struct EditorTheme {
    pub background: Color,
    pub foreground: Color,
    pub line_number_bg: Color,
    pub line_number_fg: Color,
    pub line_number_active_fg: Color,
    pub line_number_current_fg: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub primary_selection_bg: Color,
    pub primary_selection_fg: Color,
    pub indent_guide: Color,
    pub whitespace: Color,
    pub invisible: Color,
}

#[derive(Debug, Clone)]
pub struct PopupTheme {
    pub background: Color,
    pub foreground: Color,
    pub border_color: Color,
    pub highlight_bg: Color,
    pub highlight_fg: Color,
}

impl Default for GeneralTheme {
    /// Default general theme: Monokai-style black background with light-gray foreground (RGB(248, 248, 242)).
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::style::Color;
    /// use texty::ui::theme::GeneralTheme;
    ///
    /// let theme = GeneralTheme::default();
    /// assert_eq!(theme.background, Color::Black);
    /// assert_eq!(theme.foreground, Color::Rgb(248, 248, 242));
    /// ```
    fn default() -> Self {
        Self {
            background: Color::Black,
            foreground: Color::Rgb(248, 248, 242), // Monokai white
        }
    }
}

impl Default for SyntaxTheme {
    /// Creates a `SyntaxTheme` populated with Monokai colors for common syntax categories.
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::style::Color;
    /// use texty::ui::theme::SyntaxTheme;
    ///
    /// let theme = SyntaxTheme::default();
    /// // keyword uses Monokai magenta
    /// assert_eq!(theme.keyword, Color::Rgb(198, 120, 221));
    /// ```
    fn default() -> Self {
        Self {
            keyword: Color::Rgb(198, 120, 221),  // Monokai magenta
            function: Color::Rgb(166, 226, 46),  // Monokai green
            r#type: Color::Rgb(102, 217, 239),   // Monokai cyan
            string: Color::Rgb(230, 219, 116),   // Monokai yellow
            comment: Color::Rgb(73, 81, 98),     // Monokai gray
            variable: Color::Rgb(248, 248, 242), // Monokai white
        }
    }
}

impl Default for UiTheme {
    /// Creates a UiTheme populated with Monokai-inspired default UI colors.
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::style::Color;
    /// use texty::ui::theme::UiTheme;
    ///
    /// let theme = UiTheme::default();
    /// assert_eq!(theme.status_bar_bg, Color::Blue);
    /// assert_eq!(theme.cursor_fg, Color::Black);
    /// ```
    fn default() -> Self {
        Self {
            status_bar_bg: Color::Blue,
            status_bar_fg: Color::White,
            gutter_fg: Color::DarkGray,
            cursor_bg: Color::Gray,
            cursor_fg: Color::Black,
            diagnostic_error: Color::Red,
            diagnostic_warning: Color::Yellow,
            diagnostic_info: Color::Blue,
            diagnostic_hint: Color::Cyan,
        }
    }
}

impl Default for EditorTheme {
    fn default() -> Self {
        Self {
            background: Color::Black,
            foreground: Color::Rgb(248, 248, 242),
            line_number_bg: Color::Rgb(40, 44, 52),
            line_number_fg: Color::Rgb(88, 92, 106),
            line_number_active_fg: Color::Rgb(171, 178, 191),
            line_number_current_fg: Color::Rgb(139, 233, 253),
            selection_bg: Color::Rgb(68, 71, 90),
            selection_fg: Color::Rgb(248, 248, 242),
            primary_selection_bg: Color::Rgb(98, 114, 164),
            primary_selection_fg: Color::Rgb(248, 248, 242),
            indent_guide: Color::Rgb(68, 71, 90),
            whitespace: Color::Rgb(68, 71, 90),
            invisible: Color::Rgb(68, 71, 90),
        }
    }
}

impl Default for PopupTheme {
    fn default() -> Self {
        Self {
            background: Color::Rgb(40, 44, 52),
            foreground: Color::Rgb(248, 248, 242),
            border_color: Color::Rgb(68, 71, 90),
            highlight_bg: Color::Rgb(98, 114, 164),
            highlight_fg: Color::White,
        }
    }
}

impl Theme {
    /// Creates a Theme configured to use the detected terminal palette.
    ///
    /// The returned `Theme` has `use_terminal_palette` set to `true` and `terminal_palette` initialized
    /// from `TerminalPalette::detect()`. All other theme fields are set to their defaults.
    ///
    /// # Examples
    ///
    /// ```
    /// use texty::ui::theme::Theme;
    ///
    /// let theme = Theme::with_terminal_palette();
    /// assert!(theme.use_terminal_palette);
    /// assert!(theme.terminal_palette.is_some());
    /// ```
    pub fn with_terminal_palette() -> Self {
        Self {
            use_terminal_palette: true,
            terminal_palette: Some(TerminalPalette::detect()),
            ..Default::default()
        }
    }

    pub fn with_named_theme(name: String) -> Self {
        Self {
            use_terminal_palette: false,
            terminal_palette: None,
            named_theme: Some(name),
            ..Default::default()
        }
    }

    pub fn load_from_file(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let syntax_theme =
            crate::syntax::Theme::from_file(&format!("runtime/themes/{}.toml", name))?;

        let editor_theme = Self::extract_editor_theme(&syntax_theme);
        let ui_theme = Self::extract_ui_theme(&syntax_theme);
        let popup_theme = Self::extract_popup_theme(&syntax_theme);

        Ok(Self {
            use_terminal_palette: false,
            terminal_palette: None,
            named_theme: Some(name.to_string()),
            loaded_syntax_theme: Some(syntax_theme),
            editor: editor_theme,
            ui: ui_theme,
            popup: popup_theme,
            ..Default::default()
        })
    }

    fn extract_editor_theme(syntax_theme: &crate::syntax::Theme) -> EditorTheme {
        let background_style = syntax_theme.get_editor_style("background");
        EditorTheme {
            background: Self::style_to_bg(&background_style),
            foreground: Self::style_to_fg(&background_style),
            line_number_bg: Self::style_to_bg(&syntax_theme.get_editor_style("line_number")),
            line_number_fg: Self::style_to_fg(&syntax_theme.get_editor_style("line_number")),
            line_number_active_fg: Self::style_to_fg(
                &syntax_theme.get_editor_style("line_number_selected"),
            ),
            line_number_current_fg: Self::style_to_fg(
                &syntax_theme.get_editor_style("line_number_selected"),
            ),
            selection_bg: Self::style_to_bg(&syntax_theme.get_editor_style("selection")),
            selection_fg: Self::style_to_fg(&syntax_theme.get_editor_style("selection")),
            primary_selection_bg: Self::style_to_bg(
                &syntax_theme.get_editor_style("primary_selection"),
            ),
            primary_selection_fg: Self::style_to_fg(
                &syntax_theme.get_editor_style("primary_selection"),
            ),
            indent_guide: Self::style_to_fg(&syntax_theme.get_editor_style("indent_guide")),
            whitespace: Self::style_to_fg(&syntax_theme.get_editor_style("whitespace")),
            invisible: Self::style_to_fg(&syntax_theme.get_editor_style("whitespace")),
        }
    }

    fn extract_ui_theme(syntax_theme: &crate::syntax::Theme) -> UiTheme {
        UiTheme {
            status_bar_bg: Self::style_to_bg(&syntax_theme.get_status_style("normal")),
            status_bar_fg: Self::style_to_fg(&syntax_theme.get_status_style("normal")),
            gutter_fg: Self::style_to_fg(&syntax_theme.get_editor_style("line_number")),
            cursor_bg: Self::style_to_bg(&syntax_theme.get_editor_style("cursor")),
            cursor_fg: Self::style_to_fg(&syntax_theme.get_editor_style("cursor")),
            diagnostic_error: Color::Red,
            diagnostic_warning: Color::Yellow,
            diagnostic_info: Color::Blue,
            diagnostic_hint: Color::Cyan,
        }
    }

    fn extract_popup_theme(syntax_theme: &crate::syntax::Theme) -> PopupTheme {
        PopupTheme {
            background: Self::style_to_bg(&syntax_theme.get_popup_style("background")),
            foreground: Self::style_to_fg(&syntax_theme.get_popup_style("background")),
            border_color: Self::style_to_fg(&syntax_theme.get_popup_style("border")),
            highlight_bg: Self::style_to_bg(&syntax_theme.get_popup_style("menu_selected")),
            highlight_fg: Self::style_to_fg(&syntax_theme.get_popup_style("menu_selected")),
        }
    }

    fn style_to_fg(style: &crate::syntax::ResolvedStyle) -> Color {
        style
            .fg
            .map(|c| Color::Rgb(c.r, c.g, c.b))
            .unwrap_or(Color::White)
    }

    fn style_to_bg(style: &crate::syntax::ResolvedStyle) -> Color {
        style
            .bg
            .map(|c| Color::Rgb(c.r, c.g, c.b))
            .unwrap_or(Color::Black)
    }

    pub fn switch_theme(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let new_theme = Self::load_from_file(name)?;
        self.named_theme = Some(name.to_string());
        self.loaded_syntax_theme = new_theme.loaded_syntax_theme;
        self.editor = new_theme.editor;
        self.ui = new_theme.ui;
        self.popup = new_theme.popup;
        Ok(())
    }

    /// Selects a color for a given syntax capture name based on the active theme sources.
    ///
    /// If a terminal palette is enabled and available, the palette's mapping for the capture name is used. If no terminal palette is active but a loaded syntax theme exists, that theme's foreground for the capture is used. If neither source is available, built-in fallback colors are returned. Unknown or unmapped capture names fall back to the theme's general foreground.
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::style::Color;
    /// use texty::ui::theme::Theme;
    ///
    /// let theme = Theme::default();
    /// let color: Color = theme.syntax_color("keyword");
    /// ```
    pub fn syntax_color(&self, capture_name: &str) -> Color {
        // If terminal palette is enabled, use it
        if self.use_terminal_palette
            && let Some(palette) = &self.terminal_palette
        {
            let colors = palette.get_syntax_colors();
            return match capture_name {
                "keyword" => colors.keyword,
                "function" | "function.macro" | "function.method" => colors.function,
                "type" | "type.builtin" => colors.r#type,
                "string" | "string.escape" => colors.string,
                "comment" => colors.comment,
                "variable" | "variable.member" => colors.variable,
                "constant.builtin" | "constant.numeric.integer" | "constant.numeric.float" => {
                    colors.constant
                }
                "operator" => colors.operator,
                "punctuation.bracket" => colors.punctuation,
                _ => self.general.foreground,
            };
        }

        // If we have a loaded theme, use it
        if let Some(loaded_theme) = &self.loaded_syntax_theme {
            let style = loaded_theme.get_style(capture_name);
            if let Some(color) = style.fg {
                // Convert syntax::theme::Color to ratatui::Color
                Color::Rgb(color.r, color.g, color.b)
            } else {
                self.general.foreground
            }
        } else {
            // Fallback to hardcoded colors with better contrast
            match capture_name {
                "keyword" => self.syntax.keyword,
                "function" | "function.macro" | "function.method" => self.syntax.function,
                "type" | "type.builtin" => self.syntax.r#type,
                "string" | "string.escape" => self.syntax.string,
                "comment" => self.syntax.comment,
                "variable" | "variable.member" => self.syntax.variable,
                "constant.builtin" | "constant.numeric.integer" | "constant.numeric.float" => {
                    Color::Rgb(139, 233, 253) // Cyan
                }
                "operator" => Color::Rgb(255, 121, 198), // Pink
                "punctuation.bracket" => self.syntax.variable,
                _ => self.general.foreground, // default color
            }
        }
    }

    pub fn get_line_number_style(
        &self,
        is_current: bool,
        is_active: bool,
    ) -> ratatui::style::Style {
        use ratatui::style::Style;
        let fg = if is_current {
            self.editor.line_number_current_fg
        } else if is_active {
            self.editor.line_number_active_fg
        } else {
            self.editor.line_number_fg
        };
        Style::default().fg(fg).bg(self.editor.line_number_bg)
    }

    pub fn get_selection_style(&self, is_primary: bool) -> ratatui::style::Style {
        use ratatui::style::Style;
        let (bg, fg) = if is_primary {
            (
                self.editor.primary_selection_bg,
                self.editor.primary_selection_fg,
            )
        } else {
            (self.editor.selection_bg, self.editor.selection_fg)
        };
        Style::default().fg(fg).bg(bg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_color_with_loaded_theme() {
        use crate::syntax::Theme as SyntaxTheme;
        use ratatui::style::Color;

        // Load monokai syntax theme
        let syntax_theme = SyntaxTheme::from_file("runtime/themes/monokai.toml").unwrap();

        // Create UI theme with loaded syntax theme
        let ui_theme = Theme {
            use_terminal_palette: false,
            terminal_palette: None,
            named_theme: Some("monokai".to_string()),
            loaded_syntax_theme: Some(syntax_theme),
            ..Default::default()
        };

        // Test that comment color is from loaded theme
        let comment_color = ui_theme.syntax_color("comment");
        match comment_color {
            Color::Rgb(r, g, b) => {
                assert_eq!(r, 98);
                assert_eq!(g, 114);
                assert_eq!(b, 164);
            }
            _ => panic!("Expected RGB color, got {:?}", comment_color),
        }

        // Test that keyword color is from loaded theme
        let keyword_color = ui_theme.syntax_color("keyword");
        match keyword_color {
            Color::Rgb(r, g, b) => {
                assert_eq!(r, 255);
                assert_eq!(g, 121);
                assert_eq!(b, 198);
            }
            _ => panic!("Expected RGB color, got {:?}", keyword_color),
        }
    }
}
