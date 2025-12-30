# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

```bash
# Build
cargo build              # Debug build
cargo build --release    # Optimized build
cargo check              # Quick compile check

# Testing
cargo test                              # All tests
cargo test --test integration_test      # Integration tests only
cargo test --doc                        # Documentation tests
cargo test test_name                    # Specific test

# Code quality
cargo fmt               # Format code
cargo clippy            # Lint

# Run the editor
cargo run --release -- [file]        # Open file
cargo run --release -- --theme dracula [file]  # With theme
cargo run --release -- --list-themes  # List available themes
```

## Architecture Overview

Texty is a **modal terminal text editor** written in Rust with Vim-inspired keybindings. The architecture follows a modular design with clear separation of concerns.

### Core Components

- **`src/main.rs`** - Entry point: terminal setup (raw mode, alternate screen), event loop, CLI parsing, keybinding routing
- **`src/editor.rs`** - Central coordinator: manages all editor state (buffer, cursor, mode, viewport), orchestrates LSP/formatting/UI, command execution hub
- **`src/buffer.rs`** - Text storage: uses **Ropey** for efficient UTF-8 handling, file I/O, syntax highlighter integration, version tracking
- **`src/command.rs`** - Command system: enum-based pattern, all editor actions as first-class clonable values
- **`src/mode.rs`** - Editing modes: Normal, Insert, Visual, Command, FuzzySearch with mode-aware keybinding handling

### Key Subsystems

#### Syntax Highlighting (`src/syntax/`)
- **Tree-sitter** integration with incremental parsing
- **Query-based** highlighting with injection support (embedded languages)
- **Feature-gated** language support: Rust (always), Python/JS/TS (optional features)
- **LRU cache** for query loading performance
- **Helix-inspired theme system** with TOML configuration

#### LSP Integration (`src/lsp/`)
- **Async-first** design using Tokio runtime
- Multi-server support with dedicated managers:
  - `completion.rs` - Auto-completion with popup UI
  - `diagnostics.rs` - Error/warning processing
  - `progress.rs` - Progress reporting
- Graceful degradation when servers unavailable

#### UI System (`src/ui/`)
- **Ratatui**-based terminal interface
- Widget-based architecture:
  - `editor_pane.rs` - Main text editing area with viewport rendering
  - `fuzzy_search.rs` - File search interface
  - `completion.rs` - Auto-completion popup
  - `status_bar.rs` - Mode, file status, LSP info
- **Theme support** for comprehensive UI styling (syntax, UI elements, popups)

#### Fuzzy Search (`src/fuzzy_search.rs`)
- **FZF-style** matching algorithm with optimized character scoring
- **Gitignore filtering** support for file search
- **Parallel directory traversal** with Rayon
- Memory-efficient slab allocator for file entries

### Theme System

The theme system follows a **Helix-inspired design** with multiple discovery locations:

**Priority order** (highest to lowest):
1. CLI flag: `texty --theme <name>` or `--theme /path/to/theme.toml`
2. Config file: `theme = "name"` in `~/.config/texty/config.toml`
3. User theme files: `~/.config/texty/theme.toml`, `~/.texty/theme.toml`, or `./theme.toml`
4. Built-in themes in `runtime/themes/` (falls back to monokai)

**Built-in themes**: monokai (default), dracula, nord, gruvbox, solarized-dark, tokyo-night

**Theme TOML structure**:
- `[palette]` - Color definitions for reuse
- `[ui]` - UI element styling (editor, popup, status, etc.)
- Syntax scopes (e.g., `[syntax "function.builtin"]`) - Hierarchical fallback
- Inheritance via `inherits = "parent-theme.toml"`

**Special mode**: `--terminal-palette` uses terminal colors instead of themes

### Performance Patterns

- **Ropey** for efficient large file editing (O(log n) operations)
- **Tree-sitter** incremental parsing (only re-parses changed regions)
- **LRU caching** for syntax queries
- **Lazy loading** of syntax highlighting (viewport-only)
- **Viewport-aware** rendering (only visible lines)
- **Copy-on-write** patterns throughout
- **Async I/O** for LSP and file operations

### Error Handling

- Custom error types using `thiserror`
- `Result`-based APIs throughout
- User-friendly error messages with fallbacks (especially for theme loading)

## Language Support

### Base Support (always available)
- Rust syntax highlighting

### Optional Features (enabled via Cargo features)
- Python: `--features python`
- JavaScript/TypeScript: `--features javascript`

Add to `Cargo.toml` features or use: `cargo build --features python,javascript`

### Language Servers (optional but recommended)
```bash
# Rust
cargo install rust-analyzer

# Python
pip install pyright

# JavaScript/TypeScript
npm install -g typescript-language-server prettier
```

## Key Binding System

Keybindings are mode-aware. The routing flow:
1. Terminal event → `src/main.rs` event loop
2. Look up in mode-specific keymap (`src/mode.rs`)
3. Convert to `Command` enum (`src/command.rs`)
4. Execute in `src/editor.rs`

**Common bindings**:
- Normal mode: Vim-style (`h/j/k/l`, `i`, `:`, `gg`, `G`, etc.)
- Insert mode: Typing, `Esc` to normal, arrows to move, `Ctrl+h` to delete
- LSP (when available): `c` (completion), `g` (goto), `H` (hover), `a` (actions), `r` (references)

## Testing Strategy

- **Unit tests**: Inline in modules using `#[cfg(test)]`
- **Integration tests**: `tests/integration_test.rs` for end-to-end workflows
- **Property-based testing**: Uses `proptest` for fuzzy search algorithm correctness
- **Temporary files**: Uses `tempfile` crate for safe file I/O testing

When adding features, write tests that follow this pattern: unit tests for individual functions, integration tests for user-facing workflows.
