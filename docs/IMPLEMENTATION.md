# Texty Implementation Guide

This document describes the architecture and implementation details of Texty, the terminal text editor.

## Architecture Overview

Texty follows a modular architecture with clear separation of concerns:

```
src/
├── main.rs          # Application entry point
├── lib.rs           # Library exports
├── buffer.rs        # Text buffer management
├── editor.rs        # Core editor logic
├── cursor.rs        # Cursor positioning
├── mode.rs          # Editing modes
├── command.rs       # Command system
├── keymap.rs        # Keybinding management
├── syntax/          # Syntax highlighting
├── lsp/             # Language Server Protocol
├── ui/              # Terminal UI
└── viewport.rs      # View management
```

## Core Components

### Buffer (`buffer.rs`)
Manages text content using Ropey for efficient text manipulation.

**Key Features:**
- UTF-8 aware text storage
- Line-based operations
- File I/O with error handling
- Syntax highlighter integration

**API:**
```rust
pub struct Buffer {
    pub rope: Rope,
    pub file_path: Option<String>,
    pub modified: bool,
    pub highlighter: Option<SyntaxHighlighter>,
}
```

### Editor (`editor.rs`)
Orchestrates all editor components and manages application state.

**Responsibilities:**
- Buffer management
- Mode switching
- Command execution
- LSP coordination
- UI rendering

### Syntax Highlighting (`syntax/`)
Tree-sitter based syntax analysis with theme support.

**Modules:**
- `highlighter.rs` - Core highlighting logic
- `query_loader.rs` - Query caching and loading
- `theme.rs` - TOML theme parsing
- `language.rs` - Language configuration
- `config.rs` - Runtime configuration

**Key Design Decisions:**
- Capture names instead of enums for flexibility
- Theme-based styling with TOML configuration
- Query caching for performance
- Support for injections and locals queries

### LSP Integration (`lsp/`)
Asynchronous language server communication.

**Components:**
- `client.rs` - LSP message handling
- `manager.rs` - Server lifecycle management
- `diagnostics.rs` - Error/warning processing
- `completion.rs` - Auto-completion
- `progress.rs` - Progress reporting

### UI System (`ui/`)
Ratatui-based terminal interface.

**Structure:**
```
ui/
├── renderer.rs       # Main rendering loop
├── theme.rs          # UI theming
└── widgets/          # UI components
    ├── editor_pane.rs
    ├── status_bar.rs
    └── completion.rs
```

## Design Decisions

### Text Storage
- **Ropey** for efficient large file handling
- Immutable operations with copy-on-write
- UTF-8 native support

### Modal Editing
- Vim-inspired modes for efficiency
- Clear mode indicators
- Consistent keybindings

### Syntax Highlighting Architecture
- Tree-sitter for parsing accuracy
- Capture-based highlighting for extensibility
- TOML themes for easy customization
- Query injection support for embedded languages

### LSP Architecture
- Async-first design with Tokio
- Multi-server support
- Graceful degradation when servers unavailable

### Error Handling
- Custom error types with `thiserror`
- Result-based APIs
- User-friendly error messages

## Performance Considerations

### Syntax Highlighting
- Incremental parsing with tree-sitter
- Query result caching
- Lazy highlighting updates

### Rendering
- Efficient span-based rendering
- Viewport-aware updates
- Minimal redraws

### Memory Management
- Rope-based text storage
- Query caching with LRU
- Streaming file I/O

## Extension Points

### Adding Languages
1. Add tree-sitter grammar dependency
2. Create syntax queries in `runtime/queries/`
3. Update language configuration in `runtime/languages.toml`
4. Add language support in `syntax/language.rs`

### Custom Themes
- Modify `runtime/themes/default.toml`
- Add new color schemes
- Support for different terminal capabilities

### LSP Servers
- Implement server-specific handlers
- Add configuration options
- Extend capability negotiation

## Testing Strategy

### Unit Tests
- Component-level testing
- Mock LSP servers
- Syntax query validation

### Integration Tests
- End-to-end editing workflows
- File operations
- UI rendering verification

### Benchmarks
- Large file performance
- Syntax highlighting speed
- Memory usage analysis

## Development Workflow

### Building
```bash
cargo build          # Debug build
cargo build --release # Optimized build
```

### Testing
```bash
cargo test           # All tests
cargo test --doc     # Documentation tests
```

### Linting
```bash
cargo clippy         # Code quality checks
cargo fmt            # Code formatting
```

### Debugging
- Use `RUST_BACKTRACE=1` for stack traces
- Enable debug logging with `env_logger`
- Profile with `cargo flamegraph`

## Future Enhancements

### Planned Features
- Multiple cursors
- Tree-sitter based refactoring
- Plugin system
- Collaborative editing
- Integrated terminal

### Architecture Improvements
- Actor-based concurrency
- Plugin API
- Configuration system
- Theme marketplace

## Contributing

When adding features:
1. Follow existing code conventions
2. Add comprehensive tests
3. Update documentation
4. Maintain performance benchmarks
5. Ensure cross-platform compatibility

For questions, see the main README or open an issue.</content>
<parameter name="filePath">docs/IMPLEMENTATION.md