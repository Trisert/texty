# Texty

A high-performance, cross-platform terminal text editor written in Rust with Vim keybindings.

## Features

- Rope-based buffer for efficient handling of large files
- Tree-sitter syntax highlighting
- LSP integration for IDE-like features
- Smart formatting with external formatters
- Beautiful TUI using ratatui

## Quick Start

### Prerequisites

- Rust toolchain (1.70+)
- Optional: Language servers (rust-analyzer, pyright, typescript-language-server)

### Build and Run

```bash
cargo build --release
cargo run
```

## Documentation

See [PDR.md](PDR.md) for detailed project documentation, development phases, and architecture.