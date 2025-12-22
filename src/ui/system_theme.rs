// ui/system_theme.rs - System theme detection

use ratatui::style::Color;
use std::io::{self, Read, Write};

/// Detected system theme
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemTheme {
    Light,
    Dark,
    Unknown,
}

/// Terminal color capability
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalCapability {
    NoColor,
    Basic16,
    Color256,
    TrueColor,
}

/// Terminal color palette
#[derive(Debug, Clone)]
pub struct TerminalPalette {
    pub capability: TerminalCapability,
    pub black: Color,
    pub red: Color,
    pub green: Color,
    pub yellow: Color,
    pub blue: Color,
    pub magenta: Color,
    pub cyan: Color,
    pub white: Color,
    pub bright_black: Color,
    pub bright_red: Color,
    pub bright_green: Color,
    pub bright_yellow: Color,
    pub bright_blue: Color,
    pub bright_magenta: Color,
    pub bright_cyan: Color,
    pub bright_white: Color,
    pub background: Option<Color>,
    pub foreground: Option<Color>,
}

impl Default for TerminalPalette {
    fn default() -> Self {
        Self::new(TerminalCapability::Basic16)
    }
}

/// Syntax colors derived from terminal palette
#[derive(Debug, Clone)]
pub struct SyntaxPaletteColors {
    pub keyword: Color,
    pub function: Color,
    pub r#type: Color,
    pub string: Color,
    pub comment: Color,
    pub variable: Color,
    pub constant: Color,
    pub operator: Color,
    pub punctuation: Color,
}

impl TerminalPalette {
    pub fn new(capability: TerminalCapability) -> Self {
        Self {
            capability,
            black: Color::Black,
            red: Color::Red,
            green: Color::Green,
            yellow: Color::Yellow,
            blue: Color::Blue,
            magenta: Color::Magenta,
            cyan: Color::Cyan,
            white: Color::White,
            bright_black: Color::DarkGray,
            bright_red: Color::LightRed,
            bright_green: Color::LightGreen,
            bright_yellow: Color::LightYellow,
            bright_blue: Color::LightBlue,
            bright_magenta: Color::LightMagenta,
            bright_cyan: Color::LightCyan,
            bright_white: Color::Gray,
            background: None,
            foreground: None,
        }
    }

    pub fn get_syntax_colors(&self) -> SyntaxPaletteColors {
        SyntaxPaletteColors {
            keyword: self.bright_magenta,
            function: self.bright_green,
            r#type: self.bright_cyan,
            string: self.bright_yellow,
            comment: self.bright_blue,
            variable: self.foreground.unwrap_or(self.white),
            constant: self.bright_cyan,
            operator: self.bright_red,
            punctuation: self.foreground.unwrap_or(self.white),
        }
    }

    pub fn detect() -> Self {
        let capability = detect_terminal_capability();

        if capability == TerminalCapability::NoColor {
            return Self::new(capability);
        }

        if capability == TerminalCapability::TrueColor
            && let Some(palette) = query_terminal_palette()
        {
            return palette;
        }

        Self::new(capability)
    }
}

/// Get system theme preference
pub fn detect_system_theme() -> SystemTheme {
    // Try environment variables first
    if let Ok(theme) = std::env::var("COLORFGBG") {
        return if theme.contains("dark") {
            SystemTheme::Dark
        } else if theme.contains("light") {
            SystemTheme::Light
        } else {
            SystemTheme::Unknown
        };
    }

    // Check for common indicators of dark mode
    if std::env::var("DARK_MODE").is_ok() {
        return SystemTheme::Dark;
    }

    // Try to detect from terminal capabilities
    if let Ok(term) = std::env::var("TERM")
        && (term.contains("dark") || term.contains("night"))
    {
        return SystemTheme::Dark;
    }

    // Use common light/dark color schemes as fallback
    // Most modern terminals default to dark, but we'll check some common patterns
    SystemTheme::Dark
}

/// Detect terminal color capability
pub fn detect_terminal_capability() -> TerminalCapability {
    if let Ok(colorterm) = std::env::var("COLORTERM") {
        if colorterm.contains("truecolor") || colorterm.contains("24bit") {
            return TerminalCapability::TrueColor;
        }
        if colorterm == "256color" {
            return TerminalCapability::Color256;
        }
    }

    if let Ok(term) = std::env::var("TERM") {
        if term.contains("256color") {
            return TerminalCapability::Color256;
        }
        if term.contains("direct") || term.contains("truecolor") || term.contains("24bit") {
            return TerminalCapability::TrueColor;
        }
        if term == "xterm" || term == "xterm-16color" {
            return TerminalCapability::Basic16;
        }
    }

    if std::env::var("NO_COLOR").is_ok() {
        return TerminalCapability::NoColor;
    }

    TerminalCapability::Color256
}

/// Query terminal for color palette using OSC sequences
fn query_terminal_palette() -> Option<TerminalPalette> {
    let mut palette = TerminalPalette::new(TerminalCapability::TrueColor);

    let palette_ansi_colors = [
        "black",
        "red",
        "green",
        "yellow",
        "blue",
        "magenta",
        "cyan",
        "white",
        "bright-black",
        "bright-red",
        "bright-green",
        "bright-yellow",
        "bright-blue",
        "bright-magenta",
        "bright-cyan",
        "bright-white",
    ];

    for (idx, &color_name) in palette_ansi_colors.iter().enumerate() {
        if let Some(rgb) = query_terminal_color(idx) {
            let color = Color::Rgb(rgb.0, rgb.1, rgb.2);

            match color_name {
                "black" => palette.black = color,
                "red" => palette.red = color,
                "green" => palette.green = color,
                "yellow" => palette.yellow = color,
                "blue" => palette.blue = color,
                "magenta" => palette.magenta = color,
                "cyan" => palette.cyan = color,
                "white" => palette.white = color,
                "bright-black" => palette.bright_black = color,
                "bright-red" => palette.bright_red = color,
                "bright-green" => palette.bright_green = color,
                "bright-yellow" => palette.bright_yellow = color,
                "bright-blue" => palette.bright_blue = color,
                "bright-magenta" => palette.bright_magenta = color,
                "bright-cyan" => palette.bright_cyan = color,
                "bright-white" => palette.bright_white = color,
                _ => {}
            }
        }
    }

    if let Some(bg_rgb) = query_terminal_special_color(10) {
        palette.background = Some(Color::Rgb(bg_rgb.0, bg_rgb.1, bg_rgb.2));
    }

    if let Some(fg_rgb) = query_terminal_special_color(11) {
        palette.foreground = Some(Color::Rgb(fg_rgb.0, fg_rgb.1, fg_rgb.2));
    }

    Some(palette)
}

/// Query terminal for a specific color index using OSC 4
fn query_terminal_color(color_index: usize) -> Option<(u8, u8, u8)> {
    let mut query = Vec::new();
    query.extend_from_slice(&[0x1b, b']']);
    query.extend_from_slice(format!("4;{};?", color_index).as_bytes());
    query.extend_from_slice(&[0x1b, b'\\']);

    if io::stdout().write_all(&query).is_err() {
        return None;
    }

    if io::stdout().flush().is_err() {
        return None;
    }

    read_osc_response()
}

/// Query terminal for background (10) or foreground (11) color
fn query_terminal_special_color(color_id: usize) -> Option<(u8, u8, u8)> {
    let mut query = Vec::new();
    query.extend_from_slice(&[0x1b, b']']);
    query.extend_from_slice(format!("{};?", color_id).as_bytes());
    query.extend_from_slice(&[0x1b, b'\\']);

    if io::stdout().write_all(&query).is_err() {
        return None;
    }

    if io::stdout().flush().is_err() {
        return None;
    }

    read_osc_response()
}

/// Read OSC response from terminal
fn read_osc_response() -> Option<(u8, u8, u8)> {
    let stdin = io::stdin();
    let mut stdin_handle = stdin.lock();

    let mut buffer = [0u8; 1024];

    let bytes_read = stdin_handle.read(&mut buffer).ok()?;
    let response = String::from_utf8_lossy(&buffer[..bytes_read]);

    parse_osc_color_response(&response)
}

/// Parse OSC color response
fn parse_osc_color_response(response: &str) -> Option<(u8, u8, u8)> {
    let response = response.trim();

    let esc = std::char::from_u32(0x1b)?;
    let st = std::char::from_u32(0x9d)?;

    let prefix1 = format!("{}]", esc);
    let prefix2 = format!("{}]", st);

    if response.starts_with(&prefix1) || response.starts_with(&prefix2) {
        let response = response
            .strip_prefix(&prefix1)
            .or_else(|| response.strip_prefix(&prefix2))?;

        let parts: Vec<&str> = response.split(';').collect();

        if parts.len() >= 3
            && let (Some(r_str), Some(g_str), Some(b_str)) =
                (parts.get(2), parts.get(3), parts.get(4))
        {
            let r = r_str
                .trim_start_matches("rgb:")
                .trim_end_matches('/')
                .parse::<u8>()
                .ok()?;
            let g = g_str.trim_end_matches('/').parse::<u8>().ok()?;
            let b = b_str.trim_end_matches('/').parse::<u8>().ok()?;
            return Some((r, g, b));
        }
    }

    None
}

/// Get appropriate colors based on system theme
pub fn get_system_theme_colors() -> ThemeColors {
    match detect_system_theme() {
        SystemTheme::Light => ThemeColors::light(),
        SystemTheme::Dark => ThemeColors::dark(),
        SystemTheme::Unknown => ThemeColors::default(),
    }
}

/// Theme colors for different modes
#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub background: Color,
    pub foreground: Color,
    pub keyword: Color,
    pub function: Color,
    pub r#type: Color,
    pub string: Color,
    pub comment: Color,
    pub variable: Color,
    pub cursor_bg: Color,
    pub cursor_fg: Color,
    pub status_bar_bg: Color,
    pub status_bar_fg: Color,
    pub gutter_fg: Color,
    pub diagnostic_error: Color,
    pub diagnostic_warning: Color,
    pub diagnostic_info: Color,
    pub diagnostic_hint: Color,
}

impl Default for ThemeColors {
    fn default() -> Self {
        // Default to dark theme for better compatibility
        Self::dark()
    }
}

impl ThemeColors {
    /// Dark theme colors (high contrast)
    pub fn dark() -> Self {
        Self {
            background: Color::Black,
            foreground: Color::Rgb(248, 248, 242), // Light gray
            keyword: Color::Rgb(255, 121, 198),    // Pink/cyan
            function: Color::Rgb(80, 250, 123),    // Green
            r#type: Color::Rgb(139, 233, 253),     // Cyan
            string: Color::Rgb(241, 250, 140),     // Yellow
            comment: Color::Rgb(98, 114, 164),     // Dark blue
            variable: Color::Rgb(248, 248, 242),   // Light gray
            cursor_bg: Color::Gray,
            cursor_fg: Color::Black,
            status_bar_bg: Color::Rgb(40, 44, 52), // Dark blue-gray
            status_bar_fg: Color::Rgb(248, 248, 242), // Light gray
            gutter_fg: Color::DarkGray,
            diagnostic_error: Color::Rgb(255, 85, 85), // Light red
            diagnostic_warning: Color::Rgb(255, 184, 0), // Yellow
            diagnostic_info: Color::Rgb(139, 233, 253), // Cyan
            diagnostic_hint: Color::Rgb(255, 121, 198), // Pink
        }
    }

    /// Light theme colors (high contrast)
    pub fn light() -> Self {
        Self {
            background: Color::White,
            foreground: Color::Black,
            keyword: Color::Rgb(0, 100, 200),   // Blue
            function: Color::Rgb(0, 128, 0),    // Dark green
            r#type: Color::Rgb(0, 32, 128),     // Dark blue
            string: Color::Rgb(163, 21, 21),    // Dark red
            comment: Color::Rgb(128, 128, 128), // Gray
            variable: Color::Black,
            cursor_bg: Color::Black,
            cursor_fg: Color::White,
            status_bar_bg: Color::Rgb(200, 200, 200), // Light gray
            status_bar_fg: Color::Black,
            gutter_fg: Color::Gray,
            diagnostic_error: Color::Red,
            diagnostic_warning: Color::Rgb(200, 150, 0), // Dark yellow
            diagnostic_info: Color::Blue,
            diagnostic_hint: Color::Rgb(0, 128, 128), // Dark gray
        }
    }
}
