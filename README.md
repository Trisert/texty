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