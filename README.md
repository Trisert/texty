# Texty

A terminal text editor written in Rust with Vim keybindings and LSP support.

## Features

- Rope-based text buffer for efficient handling of large files
- Vim-style modal editing (normal, insert, command modes)
- Tree-sitter syntax highlighting (Rust, Python, JavaScript, TypeScript)
- LSP integration for diagnostics, completion, and navigation
- External formatter support (rustfmt, black, prettier)
- Context-aware auto-indentation
- Command-line interface (`:q`, `:w`, `:x`, `:e`, etc.)

## Themes

Texty ships with 5 built-in themes and allows custom themes.

### Built-in Themes

- **monokai** (default) - Classic dark theme
- **dracula** - Popular dark purple theme
- **nord** - Arctic, bluish dark theme
- **gruvbox** - Retro groovy color scheme
- **solarized-dark** - Precision colors for solarized

### Theme Priority (highest to lowest)

1. **CLI flag**: `texty --theme dracula`
2. **Config file**: `theme = "nord"` in `~/.config/texty/config.toml`
3. **User theme file**:
   - `~/.config/texty/theme.toml` (XDG config directory)
   - `~/.texty/theme.toml` (home directory)
   - `./theme.toml` (current working directory)
4. **Built-in themes**: Falls back to monokai if not found

### Listing Themes

```bash
texty --list-themes
```

### Using a Theme

```bash
texty --theme monokai
texty --theme dracula
texty --theme /path/to/custom.toml
```

### Configuration File

Create `~/.config/texty/config.toml`:

```toml
theme = "gruvbox"
```

### Creating Custom Themes

Create a `theme.toml` file in any of the supported locations. Use `runtime/themes/monokai.toml` as a reference.

### Error Messages

Theme errors appear in both:
- **Terminal stderr** - For debugging and logging
- **Status bar** - User-friendly error messages during editor use

### Using Terminal Colors

To use your terminal's color palette instead of a theme:

```bash
texty --terminal-palette
```

## Installation

Requires Rust 1.70+. Optional language servers for IDE features:

```bash
# Rust
cargo install rust-analyzer

# Python
pip install pyright

# JavaScript/TypeScript
npm install -g typescript-language-server prettier
```

Build from source:

```bash
git clone https://github.com/Trisert/texty.git
cd texty
cargo build --release
```

## Usage

Open a file:

```bash
cargo run --release -- src/main.rs
```

### Key Bindings

**Normal Mode:**
- `h/j/k/l` or arrow keys - Move cursor
- `i` - Enter insert mode
- `:` - Enter command mode

**Insert Mode:**
- `Esc` - Return to normal mode
- Arrow keys - Move cursor
- `Ctrl+h` - Delete previous character

**Command Mode:**
- `:q` - Quit
- `:w` - Save
- `:x` - Save and quit
- `:e <file>` - Open file
- `:syntax on/off` - Toggle syntax highlighting
- `:lsp restart/stop` - Control LSP servers

**LSP (when available):**
- `c` - Code completion
- `g` - Go to definition
- `H` - Hover information
- `a` - Code actions
- `r` - Find references

## Development

```bash
# Run tests
cargo test

# Run integration tests
cargo test --test integration_test

# Format code
cargo fmt

# Lint
cargo clippy

# Build release
cargo build --release
```

## License

MIT