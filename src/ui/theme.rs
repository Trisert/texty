// ui/theme.rs - Theme system for UI styling

use ratatui::style::Color;

/// Theme configuration
#[derive(Debug, Clone, Default)]
pub struct Theme {
    pub general: GeneralTheme,
    pub syntax: SyntaxTheme,
    pub ui: UiTheme,
    pub loaded_syntax_theme: Option<crate::syntax::Theme>,
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

impl Default for GeneralTheme {
    fn default() -> Self {
        Self {
            background: Color::Black,
            foreground: Color::Rgb(248, 248, 242), // Light gray for better contrast
        }
    }
}

impl Default for SyntaxTheme {
    fn default() -> Self {
        Self {
            keyword: Color::Rgb(255, 121, 198), // Pink/cyan
            function: Color::Rgb(80, 250, 123), // Green
            r#type: Color::Rgb(139, 233, 253), // Cyan
            string: Color::Rgb(241, 250, 140), // Yellow
            comment: Color::Rgb(98, 114, 164), // Dark blue
            variable: Color::Rgb(248, 248, 242), // Light gray
        }
    }
}

impl Default for UiTheme {
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

impl Theme {
    /// Get syntax color for a capture name
    pub fn syntax_color(&self, capture_name: &str) -> Color {
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
                "function" | "function.macro" => self.syntax.function,
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
}
