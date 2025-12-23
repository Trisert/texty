// ui/theme.rs - Theme system for UI styling

use super::system_theme::TerminalPalette;
use ratatui::style::Color;

/// Theme configuration
#[derive(Debug, Clone, Default)]
pub struct Theme {
    pub general: GeneralTheme,
    pub syntax: SyntaxTheme,
    pub ui: UiTheme,
    pub loaded_syntax_theme: Option<crate::syntax::Theme>,
    pub use_terminal_palette: bool,
    pub terminal_palette: Option<TerminalPalette>,
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
    /// Default general theme: black background with a light-gray foreground (RGB(248, 248, 242)) for better contrast.
    ///
    /// # Examples
    ///
    /// ```
    /// let theme = GeneralTheme::default();
    /// assert_eq!(theme.background, Color::Black);
    /// assert_eq!(theme.foreground, Color::Rgb(248, 248, 242));
    /// ```
    fn default() -> Self {
        Self {
            background: Color::Black,
            foreground: Color::Rgb(248, 248, 242), // Light gray for better contrast
        }
    }
}

impl Default for SyntaxTheme {
    /// Creates a `SyntaxTheme` populated with the default colors for common syntax categories.
    ///
    /// # Examples
    ///
    /// ```
    /// let theme = crate::ui::theme::SyntaxTheme::default();
    /// // keyword uses a pink/cyan hue
    /// assert_eq!(theme.keyword, ratatui::style::Color::Rgb(255, 121, 198));
    /// ```
    fn default() -> Self {
        Self {
            keyword: Color::Rgb(255, 121, 198),  // Pink/cyan
            function: Color::Rgb(80, 250, 123),  // Green
            r#type: Color::Rgb(139, 233, 253),   // Cyan
            string: Color::Rgb(241, 250, 140),   // Yellow
            comment: Color::Rgb(98, 114, 164),   // Dark blue
            variable: Color::Rgb(248, 248, 242), // Light gray
        }
    }
}

impl Default for UiTheme {
    /// Creates a UiTheme populated with the module's standard default UI colors.
    ///
    /// # Examples
    ///
    /// ```
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

impl Theme {
    /// Creates a Theme configured to use the detected terminal palette.
    ///
    /// The returned `Theme` has `use_terminal_palette` set to `true` and `terminal_palette` initialized
    /// from `TerminalPalette::detect()`. All other theme fields are set to their defaults.
    ///
    /// # Examples
    ///
    /// ```
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

    /// Selects a color for a given syntax capture name based on the active theme sources.
    ///
    /// If a terminal palette is enabled and available, the palette's mapping for the capture name is used. If no terminal palette is active but a loaded syntax theme exists, that theme's foreground for the capture is used. If neither source is available, built-in fallback colors are returned. Unknown or unmapped capture names fall back to the theme's general foreground.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::ui::theme::Theme;
    /// use ratatui::style::Color;
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
                "function" | "function.macro" => colors.function,
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