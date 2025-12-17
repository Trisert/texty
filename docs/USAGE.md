# Texty - Terminal Text Editor

Texty is a modern terminal-based text editor built with Rust, inspired by Helix. It features syntax highlighting, LSP integration, and a modal editing experience.

## Installation

### Prerequisites
- Rust 1.70+
- Cargo

### Building
```bash
git clone <repository-url>
cd texty
cargo build --release
```

The binary will be available at `target/release/texty`.

## Usage

### Starting Texty
```bash
# Open a file
texty src/main.rs

# Start with empty buffer
texty
```

### Basic Editing
Texty uses a modal interface similar to Vim:

- **Normal Mode** (default): Navigation and commands
- **Insert Mode**: Text editing
- **Command Mode**: Execute commands

### Keybindings

#### Navigation
- `h/j/k/l` - Move left/down/up/right
- `w/b` - Next/previous word
- `0/$` - Start/end of line
- `gg/G` - Start/end of file

#### Editing
- `i` - Enter insert mode
- `a` - Append after cursor
- `o` - Open new line below
- `x` - Delete character
- `dd` - Delete line
- `yy` - Yank line
- `p` - Paste

#### Files
- `:w` - Write file
- `:q` - Quit
- `:wq` - Write and quit
- `:e <file>` - Open file

#### Search
- `/` - Search forward
- `?` - Search backward
- `n` - Next match
- `N` - Previous match

## Features

### Syntax Highlighting
Texty supports syntax highlighting for:
- Rust
- Python
- JavaScript
- TypeScript

Colors are configurable via themes.

### LSP Integration
Language Server Protocol support provides:
- Code completion
- Diagnostics (errors/warnings)
- Hover information
- Go to definition

### Theming
Customize appearance with TOML theme files in `runtime/themes/`.

Example theme:
```toml
[keyword]
fg = "Blue"

[function]
fg = "Green"

[string]
fg = "Red"

[comment]
fg = "Gray"
```

### Configuration
- Language configurations in `runtime/languages.toml`
- Syntax queries in `runtime/queries/`
- Themes in `runtime/themes/`

## Troubleshooting

### No Colors
Ensure your terminal supports 24-bit color:
```bash
echo $TERM  # Should be something like xterm-256color
```

### LSP Not Working
Check that language servers are installed:
- Rust: `rust-analyzer`
- Python: `pylsp` or `pyright`
- JavaScript/TypeScript: `typescript-language-server`

## Contributing
See `docs/IMPLEMENTATION.md` for development information.</content>
<parameter name="filePath">docs/USAGE.md