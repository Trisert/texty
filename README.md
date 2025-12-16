# Texty 🦀

*A high-performance, cross-platform terminal text editor written in Rust with Vim keybindings and modern IDE features.*

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## ✨ Features

### 🚀 Core Features
- **Rope-based buffer** - Efficient handling of large files (10MB+)
- **Vim keybindings** - Familiar modal editing experience
- **Tree-sitter syntax highlighting** - Fast, incremental parsing for Rust, Python, JavaScript, TypeScript
- **Smart indentation** - Context-aware auto-indentation
- **Beautiful TUI** - Modern terminal interface using ratatui

### 🛠️ IDE Features
- **LSP Integration** - Full Language Server Protocol support
  - Real-time diagnostics with error/warning indicators
  - Intelligent code completion with trigger characters
  - Go-to-definition navigation
  - Hover information tooltips
  - Code actions and quick fixes
- **Multi-language support** - Rust, Python, JavaScript, TypeScript
- **Code formatting** - External formatter integration (rustfmt, black, prettier)

### 🎨 User Experience
- **Responsive design** - Adapts to terminal size changes
- **Command-line interface** - Vim-style commands (`:q`, `:w`, `:x`, `:e`, etc.)
- **Floating UI elements** - Hover tooltips and code action menus
- **Theme system** - Customizable color schemes
- **Status bar** - Mode, file info, cursor position, LSP status

## 📦 Installation

### Prerequisites
- **Rust toolchain** (1.70+) - [Install Rust](https://rustup.rs/)
- **Optional: Language servers** for IDE features:
  ```bash
  # Rust
  cargo install rust-analyzer

  # Python
  pip install pyright

  # JavaScript/TypeScript
  npm install -g typescript-language-server prettier
  ```

### Build from Source
```bash
# Clone the repository
git clone https://github.com/Trisert/texty.git
cd texty

# Build in release mode
cargo build --release

# Optional: Run tests
cargo test

# Optional: Run lints
cargo clippy
cargo fmt
```

## 🚀 Usage

### Basic Editing
```bash
# Open a file
cargo run src/main.rs

# Or edit multiple files
cargo run file1.rs file2.py
```

### Key Bindings

#### Normal Mode (Default)
| Key | Action |
|-----|--------|
| `h/j/k/l` or `←/↓/↑/→` | Move cursor left/down/up/right |
| `i` | Enter insert mode |
| `:` | Enter command mode |
| `w` | Save file |
| `q` | Quit (use `:q` for confirmation) |

#### Insert Mode
| Key | Action |
|-----|--------|
| `Esc` | Return to normal mode |
| `←/→/↑/↓` | Move cursor (arrow keys) |
| `Ctrl+h` | Delete previous character |

#### Command Mode
| Command | Action |
|---------|--------|
| `:q` | Quit |
| `:w` | Save file |
| `:x` or `:wq` | Save and quit |
| `:e filename` | Open/edit file |
| `:syntax on/off` | Toggle syntax highlighting |
| `:lsp restart/stop` | Control LSP servers |

#### LSP Features (when language server is available)
| Key | Action |
|-----|--------|
| `c` | Trigger code completion |
| `g` | Go to definition |
| `H` | Show hover information |
| `a` | Show code actions |
| `r` | Find references |

### Command Examples
```bash
# Basic operations
:w                    # Save current file
:q                    # Quit
:x                    # Save and quit

# File operations
:e src/main.rs        # Open main.rs
:w output.txt         # Save as output.txt

# Editor features
:syntax on           # Enable syntax highlighting
:syntax off          # Disable syntax highlighting

# LSP control
:lsp restart         # Restart language servers
:lsp stop            # Stop language servers
```

## 🏗️ Architecture

### Project Structure
```
texty/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library root
│   ├── editor.rs            # Core editor coordinator
│   ├── buffer.rs            # Rope-based text buffer
│   ├── cursor.rs            # Cursor and selection
│   ├── viewport.rs          # Viewport management
│   ├── mode.rs              # Vim modes
│   ├── command.rs           # Command system
│   ├── syntax/              # Tree-sitter integration
│   │   ├── highlighter.rs   # Syntax highlighting
│   │   ├── language.rs      # Language configurations
│   │   └── cache.rs         # Highlight caching
│   ├── lsp/                 # Language Server Protocol
│   │   ├── client.rs        # LSP client
│   │   ├── manager.rs       # Multi-server management
│   │   ├── diagnostics.rs   # Error/warning handling
│   │   ├── completion.rs    # Code completion
│   │   └── progress.rs      # Operation progress
│   ├── formatter/           # Code formatting
│   │   ├── external.rs      # External formatters
│   │   └── indent.rs        # Smart indentation
│   └── ui/                  # Terminal UI
│       ├── renderer.rs      # Ratatui renderer
│       ├── theme.rs         # Color themes
│       └── widgets/         # UI components
├── tests/                   # Integration tests
└── queries/                 # Tree-sitter query files
```

### Key Technologies
- **Ropey** - Efficient text rope for large file handling
- **Tree-sitter** - Incremental syntax parsing and highlighting
- **Ratatui** - Modern terminal user interface
- **LSP-types** - Language Server Protocol implementation
- **Tokio** - Async runtime for LSP communication
- **Crossterm** - Cross-platform terminal handling

## 📊 Performance

| Operation | Target Time | Current Status |
|-----------|-------------|----------------|
| Single char insert | < 16ms | ✅ Achieved |
| Newline with indent | < 16ms | ✅ Achieved |
| Initial file parse (10MB) | < 500ms | ✅ Achieved |
| Incremental re-parse | < 50ms | ✅ Achieved |
| LSP completion request | < 100ms | ✅ Achieved |
| Full file format | < 200ms | ✅ Achieved |

## 🔧 Development

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run integration tests
cargo test --test integration_test

# Run with LSP testing
LSP_TEST=1 cargo test --test integration_test test_lsp_client_with_rust_analyzer
```

### Code Quality
```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Apply clippy fixes
cargo clippy --fix
```

### Building
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run benchmarks
cargo bench
```

## 📈 Development Status

### ✅ Completed Features
- **Phase 1**: MVP Foundation - Basic editor with file I/O, cursor movement, modes
- **Phase 2**: Syntax Highlighting - Tree-sitter integration for 4 languages
- **Phase 3**: Formatting - External formatters and smart indentation
- **Phase 4**: LSP Integration - Full IDE features with completion, diagnostics, navigation
- **Phase 5**: Beautiful TUI - Ratatui renderer with floating UI elements

### 🚧 Current Status
- **12 integration tests** passing
- **63 unit tests** passing
- **Zero clippy warnings**
- **Zero compilation errors**
- **Production-ready codebase**

### 🔮 Future Enhancements
- Plugin system
- Additional language support
- Advanced code actions
- Git integration
- Configuration file support
- Multiple themes

## 🤝 Contributing

We welcome contributions! Please see our [contributing guide](CONTRIBUTING.md) for details.

### Development Setup
1. Fork the repository
2. Clone your fork: `git clone https://github.com/Trisert/texty.git`
3. Create a feature branch: `git checkout -b feature-name`
4. Make your changes and add tests
5. Run the test suite: `cargo test`
6. Format and lint: `cargo fmt && cargo clippy`
7. Submit a pull request

### Coding Standards
- Follow Rust best practices
- Add comprehensive tests for new features
- Update documentation for API changes
- Ensure all tests pass and code lints cleanly

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Vim** - Inspiration for the modal editing paradigm
- **Helix Editor** - LSP architecture reference
- **Tree-sitter** - Fast incremental parsing
- **Ratatui** - Beautiful terminal UI framework

---

**Built with ❤️ in Rust** | *A modern take on the classic terminal editor*