# Theme System Implementation

This document describes the Helix-inspired theme system for Texty.

## Overview

The theme system provides comprehensive theming support for both syntax highlighting and UI elements, with the following features:

- **Theme Discovery**: Automatic scanning of theme directories
- **Theme Loading**: Load themes from TOML files with inheritance support
- **Theme Switching**: Dynamic runtime theme switching
- **Syntax Highlighting**: Full syntax scope support with fallback
- **UI Theming**: Comprehensive UI element styling
- **Terminal Palette**: Support for terminal-native colors

## Architecture

### Core Components

#### `ThemeLoader` (`src/ui/theme_loader.rs`)

Handles theme discovery and loading from multiple sources:

```rust
let mut loader = ThemeLoader::new();
loader.discover_themes();

let theme = loader.load_theme("monokai")?;
```

Features:
- Scans multiple theme directories (runtime/themes, ~/.config/texty/themes, etc.)
- Parses TOML theme files
- Validates theme structure
- Supports theme inheritance

#### `Theme` (`src/ui/theme.rs`)

Main theme struct containing all color information:

```rust
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
```

Theme Sub-components:

- **GeneralTheme**: Overall background/foreground colors
- **SyntaxTheme**: Basic syntax colors (fallback when no theme loaded)
- **UiTheme**: Status bar, diagnostics, cursor colors
- **EditorTheme**: Line numbers, selections, indent guides, whitespace
- **PopupTheme**: Popup windows, menus, hover windows

#### `ThemeManager` (`src/ui/theme_manager.rs`)

Central theme management with thread-safe operations:

```rust
let manager = ThemeManager::new();
let themes = manager.list_available_themes();

manager.switch_theme("tokyo-night")?;
let current = manager.get_current_theme();
```

Features:
- Atomic theme switching
- Theme discovery and caching
- Terminal palette mode
- Thread-safe with Arc<RwLock>

## Theme File Format

Theme files use TOML format with sections for different UI elements:

```toml
[palette]
red = "#f7768e"
blue = "#7aa2f7"
# ... more colors

[editor.background]
fg = "fg"
bg = "bg"

[syntax]
keyword = "red"
function = "blue"
# ... more syntax scopes

[ui.status-bar]
bg = "blue"
fg = "white"
```

### Supported Sections

#### Palette
Defines named colors for reuse:

```toml
[palette]
bg = "#1a1b26"
fg = "#a9b1d6"
red = "#f7768e"
green = "#9ece6a"
# ... more colors
```

#### Editor
Editor-specific colors:

```toml
[editor.background]
fg = "fg"
bg = "bg"

[editor.whitespace]
fg = "gray"

[editor.cursor]
fg = "bg"
bg = "blue"
modifiers = ["bold"]

[editor.line-number]
fg = "gray"
bg = "bg"

[editor.selection]
fg = "fg"
bg = "#2f3549"

[editor.primary-selection]
fg = "bg"
bg = "blue"

[editor.indent-guide]
fg = "gray"

[editor.current-line]
bg = "#24283b"
```

#### Popup
Popup window colors:

```toml
[popup.background]
fg = "fg"
bg = "#16161e"

[popup.border]
fg = "gray"
modifiers = ["bold"]

[popup.menu]
fg = "fg"
bg = "bg"

[popup.menu.selected]
fg = "bg"
bg = "blue"
modifiers = ["bold"]
```

#### UI
General UI elements:

```toml
[ui.status-bar]
bg = "blue"
fg = "white"

[ui.gutter]
fg = "gray"

[ui.cursorline]
bg = "#24283b"
```

#### Syntax Styles
Syntax highlighting colors (compatible with Helix scopes):

```toml
[attribute]
fg = "yellow"

[comment]
fg = "gray"
modifiers = ["italic"]

[constant]
fg = "orange"

"constant.builtin"
fg = "orange"
modifiers = ["bold"]

[function]
fg = "blue"

"function.method"
fg = "blue"

"function.macro"
fg = "magenta"

[keyword]
fg = "magenta"
modifiers = ["bold"]

[operator]
fg = "pink"

[punctuation]
fg = "gray"

[string]
fg = "green"

[type]
fg = "cyan"

[variable]
fg = "fg"

"variable.builtin"
fg = "cyan"

# Markup
[markup.heading]
fg = "magenta"
modifiers = ["bold"]

# Diagnostics
[diagnostic.error]
fg = "red"
underline = { color = "red", style = "line" }

# Diff
[diff.plus]
fg = "green"
bg = "#293a3d"
```

### Modifiers

Supported text modifiers:
- `bold` - Bold text
- `dim` - Dimmed text
- `italic` - Italic text
- `underlined` - Underlined text
- `reversed` - Reversed colors
- `crossed_out` - Strikethrough text
- `slow_blink` - Slow blinking
- `rapid_blink` - Fast blinking
- `hidden` - Hidden text

### Underline Styles

Custom underline styles:
- `line` - Simple underline
- `curl` - Curly underline
- `dashed` - Dashed underline
- `dotted` - Dotted underline
- `double_line` - Double underline

### Theme Inheritance

Themes can inherit from other themes:

```toml
inherits = "monokai"

[palette]
# Override specific colors
red = "#ff0000"
```

Child themes override parent theme values.

## Usage Examples

### Basic Theme Switching

```rust
use texty::ui::ThemeManager;

let manager = ThemeManager::new();

// List available themes
let themes = manager.list_available_themes();
for theme in themes {
    println!("{}", theme);
}

// Switch to a theme
manager.switch_theme("tokyo-night")?;

// Get current theme
let theme = manager.get_current_theme();
```

### Using Terminal Palette

```rust
use texty::ui::ThemeManager;

let manager = ThemeManager::new();

// Switch to terminal-native colors
manager.use_terminal_palette();

let theme = manager.get_current_theme();
assert!(theme.use_terminal_palette);
```

### Loading Theme from File

```rust
use texty::ui::Theme;

let theme = Theme::load_from_file("tokyo-night")?;

let keyword_color = theme.syntax_color("keyword");
let line_number_style = theme.get_line_number_style(true, false);
```

### Custom Theme

```rust
use texty::ui::Theme;
use ratatui::style::Color;

let mut theme = Theme::default();
theme.general.background = Color::Rgb(0, 0, 0);
theme.general.foreground = Color::Rgb(255, 255, 255);

// Use custom theme
theme_manager.set_theme(theme);
```

### Syntax Color Lookup

```rust
let theme = Theme::default();

// Get color for syntax capture
let keyword_color = theme.syntax_color("keyword");
let function_color = theme.syntax_color("function");
let method_color = theme.syntax_color("function.method");

// Fallback hierarchy:
// 1. Loaded syntax theme (if available)
// 2. Hardcoded fallback colors
// 3. General foreground
```

### UI Element Styling

```rust
let theme = Theme::default();

// Line number style
let style = theme.get_line_number_style(is_current_line, is_active_in_selection);

// Selection style
let selection_style = theme.get_selection_style(is_primary_selection);

// Editor background
let bg_color = theme.editor.background;
```

## Theme Directories

Themes are searched in the following order:

1. `runtime/themes/` (project directory)
2. `themes/` (project directory)
3. `~/.config/texty/themes/`
4. `$XDG_CONFIG_HOME/texty/themes/`

## Creating Custom Themes

### Step 1: Create Theme File

```bash
touch runtime/themes/my-theme.toml
```

### Step 2: Define Theme

```toml
# my-theme.toml

[palette]
bg = "#1e1e2e"
fg = "#c0caf5"
# ... more palette colors

[editor.background]
fg = "fg"
bg = "bg"

[attribute]
fg = "#ff9e64"

[comment]
fg = "#565f89"
modifiers = ["italic"]
```

### Step 3: Test Theme

```rust
use texty::ui::{Theme, ThemeManager};

let manager = ThemeManager::new();
manager.reload_themes();

assert!(manager.theme_exists("my-theme"));
manager.switch_theme("my-theme")?;
```

## Best Practices

1. **Use Palette Colors**: Define colors once in `[palette]` and reference by name
2. **Provide Good Contrast**: Ensure text is readable against backgrounds
3. **Handle All Scopes**: Provide colors for common syntax scopes
4. **Support Modifiers**: Use modifiers for emphasis (bold, italic)
5. **Test in Light/Dark**: Consider both light and dark modes
6. **Document Your Theme**: Add `description` field with theme info

## Available Themes

Included themes:
- `monokai` - Classic Monokai color scheme
- `dracula` - Dracula theme
- `gruvbox` - Gruvbox colors
- `nord` - Nord color scheme
- `solarized-dark` - Solarized Dark
- `tokyo-night` - Tokyo Night theme
- `default` - Default fallback theme

## API Reference

### ThemeLoader

```rust
impl ThemeLoader {
    pub fn new() -> Self;
    pub fn add_theme_directory(&mut self, path: PathBuf);
    pub fn discover_themes(&mut self) -> Vec<ThemeInfo>;
    pub fn load_theme(&self, name: &str) -> Result<SyntaxTheme, ThemeLoaderError>;
    pub fn list_themes(&mut self) -> Vec<String>;
    pub fn get_theme_info(&self, name: &str) -> Option<&ThemeInfo>;
    pub fn theme_exists(&self, name: &str) -> bool;
}
```

### ThemeManager

```rust
impl ThemeManager {
    pub fn new() -> Self;
    pub fn get_current_theme(&self) -> Theme;
    pub fn set_theme(&self, theme: Theme);
    pub fn list_available_themes(&self) -> Vec<String>;
    pub fn switch_theme(&self, name: &str) -> ThemeChangeResult;
    pub fn use_terminal_palette(&self);
    pub fn reload_themes(&self);
    pub fn theme_exists(&self, name: &str) -> bool;
    pub fn get_theme_info(&self, name: &str) -> Option<String>;
}
```

### Theme

```rust
impl Theme {
    pub fn default() -> Self;
    pub fn with_terminal_palette() -> Self;
    pub fn with_named_theme(name: String) -> Self;
    pub fn load_from_file(name: &str) -> Result<Self, Error>;
    pub fn switch_theme(&mut self, name: &str) -> Result<(), Error>;
    pub fn syntax_color(&self, capture_name: &str) -> Color;
    pub fn get_editor_background(&self) -> Color;
    pub fn get_line_number_style(&self, is_current: bool, is_active: bool) -> Style;
    pub fn get_selection_style(&self, is_primary: bool) -> Style;
}
```

## Future Enhancements

Potential improvements:
- [ ] Theme hot-reloading on file changes
- [ ] Theme validation and linting
- [ ] Color scheme generation from palette
- [ ] Import themes from Helix directly
- [ ] Theme export/import
- [ ] Preview themes before switching
- [ ] User-specific theme overrides
- [ ] Light/dark mode variants
