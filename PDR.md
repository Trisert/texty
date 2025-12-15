# Vim-like Terminal Text Editor - Project Documentation

## Project Overview

A high-performance, cross-platform terminal text editor written in Rust with Vim keybindings, featuring:
- **Rope-based buffer** for efficient handling of large files (10MB+)
- **Tree-sitter** syntax highlighting and parsing
- **LSP integration** for IDE-like features (diagnostics, completion, go-to-definition)
- **Smart formatting** with external formatters (rustfmt, black, prettier)
- **Beautiful TUI** using ratatui

## Quick Start with Opencode

### Prerequisites

```bash
# Rust toolchain (1.70+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Language servers (optional but recommended)
cargo install rust-analyzer
npm install -g typescript-language-server prettier
uv pip install black pyright

# Formatter tools
cargo install rustfmt
```

### Initial Project Setup

```bash
# Create project
cargo new vim-editor
cd vim-editor

# Initialize git
git init
git add .
git commit -m "Initial commit"
```

### Using Opencode

Start opencode in your project directory:

```bash
cd vim-editor
claude-code
```

## Project Structure

```
vim-editor/
├── Cargo.toml                   # Dependencies and package config
├── README.md                    # Project overview
├── .gitignore                   # Git ignore rules
├── config/
│   ├── default.toml            # Default editor config
│   └── themes/
│       ├── dracula.toml
│       ├── solarized-dark.toml
│       └── github-light.toml
├── queries/                     # Tree-sitter query files
│   ├── rust/
│   │   ├── highlights.scm
│   │   └── indents.scm
│   ├── python/
│   │   ├── highlights.scm
│   │   └── indents.scm
│   ├── javascript/
│   │   ├── highlights.scm
│   │   └── indents.scm
│   └── typescript/
│       ├── highlights.scm
│       └── indents.scm
├── test_files/                  # Test files for development
│   ├── test.rs
│   ├── test.py
│   └── test.js
├── tests/                       # Integration tests
│   └── integration_test.rs
├── benches/                     # Performance benchmarks
│   ├── buffer_ops.rs
│   └── syntax_parse.rs
└── src/
    ├── main.rs                  # Entry point
    ├── lib.rs                   # Library root
    ├── editor.rs                # Core editor coordinator
    ├── buffer.rs                # Rope-based text buffer
    ├── cursor.rs                # Cursor and selection
    ├── viewport.rs              # Viewport management
    ├── mode.rs                  # Vim modes
    ├── command.rs               # Command system
    ├── keymap.rs                # Key bindings
    ├── config.rs                # Configuration
    ├── performance.rs           # Performance monitoring
    ├── syntax/
    │   ├── mod.rs
    │   ├── highlighter.rs       # Tree-sitter integration
    │   ├── language.rs          # Language configurations
    │   └── cache.rs             # Highlight caching
    ├── lsp/
    │   ├── mod.rs
    │   ├── client.rs            # LSP client
    │   ├── manager.rs           # Multi-server management
    │   ├── diagnostics.rs       # Diagnostic handling
    │   ├── completion.rs        # Completion logic
    │   └── actions.rs           # Code actions
    ├── formatter/
    │   ├── mod.rs
    │   ├── external.rs          # External formatters
    │   ├── indent.rs            # Smart indentation
    │   └── language.rs          # Language-specific rules
    ├── ui/
    │   ├── mod.rs
    │   ├── renderer.rs          # Main renderer
    │   ├── theme.rs             # Theme system
    │   └── widgets/
    │       ├── editor_pane.rs   # Main editor widget
    │       ├── gutter.rs        # Line numbers & diagnostics
    │       ├── status_bar.rs    # Status bar
    │       ├── completion.rs    # Completion popup
    │       ├── hover.rs         # Hover window
    │       └── menu.rs          # Code action menu
    └── util/
        ├── mod.rs
        ├── edit.rs              # Edit operations
        └── position.rs          # Position utilities
```

## Development Phases

### Phase 1: MVP Foundation (Weeks 1-4)

**Goal**: Basic working editor with text editing and file I/O

**Opencode Prompts**:

1. **Initial Setup**:
```
Create the basic project structure with Cargo.toml containing these dependencies:
- ropey = "1.6"
- crossterm = "0.27"
- unicode-segmentation = "1.10"

Also create src/lib.rs with module declarations and src/main.rs with a basic
terminal setup using crossterm (raw mode, alternate screen).
```

2. **Buffer Implementation**:
```
Implement src/buffer.rs with a Buffer struct that:
- Wraps ropey::Rope for text storage
- Has insert_char, delete_char, insert_text methods
- Tracks file_path, modified state, and version
- Has line_count, line, line_len helper methods
- Implements load_from_file and save_to_file

Use proper error handling with a custom BufferError enum.
```

3. **Cursor & Viewport**:
```
Create src/cursor.rs with a Cursor struct that tracks line, col, and desired_col.
Create src/viewport.rs with viewport management (offset_line, offset_col, rows, cols)
and a scroll_to_cursor method that keeps the cursor visible.
```

4. **Mode System**:
```
Implement src/mode.rs with Mode enum (Normal, Insert, Visual, Command).
Create src/command.rs with Command enum for all editor operations
(MoveLeft, MoveRight, InsertChar, etc.).
```

5. **Basic Rendering**:
```
Create a simple renderer in src/main.rs that:
- Renders visible lines from the buffer
- Shows a status bar with mode and cursor position
- Handles the main event loop with crossterm
- Processes keyboard input and executes commands
```

**Acceptance Criteria**:
- Can open, edit, and save files
- Cursor movement works (hjkl)
- Mode switching between Normal and Insert
- Basic status line shows current mode

### Phase 2: Syntax Highlighting (Weeks 5-8)

**Goal**: Tree-sitter integration with incremental parsing

**Opencode Prompts**:

1. **Tree-sitter Setup**:
```
Add tree-sitter dependencies to Cargo.toml:
- tree-sitter = "0.20"
- tree-sitter-rust = "0.20"
- tree-sitter-python = "0.20"
- tree-sitter-javascript = "0.20"
- tree-sitter-typescript = "0.20"

Create src/syntax/mod.rs with LanguageId enum and LanguageConfig struct.
```

2. **Highlighter Implementation**:
```
Implement src/syntax/highlighter.rs with SyntaxHighlighter that:
- Parses text using tree-sitter Parser
- Stores Tree and manages incremental updates
- Executes highlight queries to extract tokens
- Caches highlights by line for efficient rendering
- Implements update_parse for incremental re-parsing on edits

Include proper handling of InputEdit for tree-sitter.
```

3. **Query Files**:
```
Create queries/rust/highlights.scm with tree-sitter query patterns for:
- Keywords (fn, let, mut, pub, etc.)
- Function names and calls
- Types (primitive and custom)
- Strings, numbers, comments
- Variables, parameters, properties

Use @keyword, @function, @type, @string, @comment captures.
```

4. **Highlight Cache**:
```
Implement src/syntax/cache.rs with HighlightCache that:
- Stores tokens grouped by line or in chunks
- Provides efficient get_line_highlights lookup
- Supports incremental updates for changed regions
- Manages memory efficiently for large files
```

5. **Renderer Integration**:
```
Update the renderer to:
- Get highlights from SyntaxHighlighter
- Apply colors based on token types
- Map HighlightType to Color using a theme
- Render syntax-highlighted text with ratatui Spans
```

**Acceptance Criteria**:
- Syntax highlighting works for Rust, Python, JS/TS
- Incremental parsing on edits (< 16ms for single char)
- Visible performance with 10MB files
- Proper handling of incomplete/invalid code

### Phase 3: Formatting (Weeks 9-10)

**Goal**: External formatters and smart indentation

**Opencode Prompts**:

1. **Formatter Config**:
```
Create src/formatter/external.rs with:
- FormatterConfig struct defining command, args, stdin_mode
- Formatter::new that validates the formatter is available
- format_text method that runs external formatter via subprocess
- Language-specific configs for rustfmt, black, prettier
```

2. **Smart Indentation**:
```
Implement src/formatter/indent.rs with IndentationEngine that:
- Uses tree-sitter queries to identify indent/dedent nodes
- Calculates proper indentation level for any line
- Handles auto-indent on newline based on context
- Supports language-specific indent widths
```

3. **Cursor Preservation**:
```
Add cursor position mapping after formatting:
- Use similar crate for text diffing
- Implement CursorMapper that tracks line changes
- Map old cursor position to new position after format
- Handle edge cases (deleted lines, inserted lines)
```

4. **Editor Integration**:
```
Add to src/editor.rs:
- format_buffer method that calls external formatter
- format_on_save configuration option
- handle_smart_newline for auto-indentation
- apply_char_formatting for format-on-type
```

**Acceptance Criteria**:
- External formatting works (rustfmt, black, prettier)
- Cursor stays in correct position after format
- Smart indentation on Enter key
- Format-on-save optional feature

### Phase 4: LSP Integration (Weeks 11-14)

**Goal**: Full LSP support for IDE features

**Opencode Prompts**:

1. **LSP Client Setup**:
```
Add dependencies:
- lsp-types = "0.95"
- lsp-server = "0.7"
- tokio = { version = "1", features = ["full"] }
- serde_json = "1.0"

Create src/lsp/client.rs with LspClient that:
- Spawns language server process via Command
- Creates JSON-RPC connection over stdio
- Implements initialize handshake
- Sends requests and receives responses
- Handles notifications asynchronously
```

2. **Multi-Server Management**:
```
Implement src/lsp/manager.rs with LspManager that:
- Stores HashMap of LanguageId -> LspClient
- Has configs for rust-analyzer, pyright, typescript-language-server
- get_or_start_client method that lazily starts servers
- Routes requests to appropriate server based on buffer language
```

3. **Diagnostics**:
```
Create src/lsp/diagnostics.rs with DiagnosticManager that:
- Receives publishDiagnostics notifications
- Stores diagnostics per file URI
- Provides get_diagnostics_at_line queries
- Maps DiagnosticSeverity to colors for rendering
```

4. **Completion**:
```
Implement src/lsp/completion.rs with:
- request_completion that sends CompletionParams to LSP
- Stores CompletionList with items
- Handles navigation (up/down) in completion popup
- accept_completion that applies TextEdit or insert_text
- Trigger characters per language (., ::, ->, etc.)
```

5. **Go To Definition**:
```
Add goto_definition to src/lsp/ that:
- Sends GotoDefinitionParams at cursor position
- Handles GotoDefinitionResponse (single/multiple locations)
- Loads target file if different from current
- Jumps cursor to target position
- Supports both same-file and cross-file jumps
```

6. **Hover Info**:
```
Implement hover information:
- Request hover on cursor position
- Display in floating window above cursor
- Format Markdown content for terminal display
- Auto-dismiss on cursor movement or edit
```

7. **Document Sync**:
```
Add to Buffer:
- notify_lsp_open when file is opened
- notify_lsp_change with TextDocumentContentChangeEvent
- notify_lsp_save when file is saved
- Batch pending changes with debouncing
```

**Acceptance Criteria**:
- LSP servers start automatically per language
- Diagnostics appear in real-time with colored indicators
- Completion popup appears on trigger characters
- Go-to-definition works across files
- Hover shows type information

### Phase 5: Beautiful TUI (Weeks 15-16)

**Goal**: Migrate to ratatui with polished UI

**Opencode Prompts**:

1. **Ratatui Setup**:
```
Add ratatui = "0.26" to Cargo.toml.

Create src/ui/renderer.rs with TuiRenderer that:
- Wraps Terminal<CrosstermBackend>
- Implements draw method that creates layouts
- Splits screen into editor/status/command areas
```

2. **Custom Widgets**:
```
Implement src/ui/widgets/editor_pane.rs with EditorPane widget that:
- Renders visible lines with syntax highlighting
- Shows cursor position
- Applies styles from theme
- Renders diagnostic squiggles under errors

Implement src/ui/widgets/gutter.rs with line numbers and diagnostic icons.
```

3. **Overlays**:
```
Create floating window widgets:
- src/ui/widgets/completion.rs - completion popup with icons
- src/ui/widgets/hover.rs - hover information window
- src/ui/widgets/menu.rs - code action menu

Each should calculate proper positioning relative to cursor.
```

4. **Theme System**:
```
Implement src/ui/theme.rs with Theme struct containing:
- Foreground/background colors
- Syntax highlight colors (keyword, string, comment, etc.)
- UI element colors (status bar, cursor, selection)
- Load themes from TOML files in config/themes/
```

5. **Status Bar**:
```
Create src/ui/widgets/status_bar.rs that shows:
- Current mode with color (Normal=cyan, Insert=green, etc.)
- File name and modified indicator
- Cursor position (line:col)
- File percentage
- Language server status
```

**Acceptance Criteria**:
- Beautiful, modern TUI with consistent styling
- Smooth rendering with proper layouts
- Floating windows for completion/hover
- Theme support (at least 3 themes)
- Git gutter indicators (bonus)

## Opencode Workflow

### Starting a New Feature

```bash
# Example: Starting LSP integration
claude-code
```

Then in opencode:
```
I want to implement LSP client support. Let's start with the basic client 
that can start a language server process and perform the initialize handshake.

Create src/lsp/client.rs with LspClient struct that:
1. Spawns the language server process using std::process::Command
2. Creates stdin/stdout pipes for JSON-RPC communication
3. Implements the initialize request/response flow
4. Stores ServerCapabilities after initialization

Use proper error handling and make it work with rust-analyzer as the first server.
```

### Iterating on Implementation

```
The LspClient works but I need to handle async notifications. Update it to:
1. Spawn a background thread for receiving messages
2. Use mpsc channels to send notifications to the main thread
3. Store pending requests in a HashMap with request IDs
4. Match responses to pending requests
```

### Debugging Issues

```
I'm getting an error when trying to parse the InitializeResult. The error is:
[paste error message]

Can you check the deserialization code and fix the issue? Make sure we're 
properly handling all fields from the LSP spec.
```

### Testing

```
Create integration tests in tests/lsp_test.rs that:
1. Mock a simple LSP server that responds to initialize
2. Test sending completion requests
3. Test receiving publishDiagnostics notifications
4. Verify proper shutdown sequence
```

## Configuration Files

### .gitignore

```gitignore
# Rust
target/
Cargo.lock
**/*.rs.bk
*.pdb

# Editor
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Testing
test_output/
```

### Cargo.toml (Complete)

```toml
[package]
name = "vim-editor"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "A Vim-like terminal text editor with LSP support"
license = "MIT"

[dependencies]
# Core text handling
ropey = "1.6"
unicode-segmentation = "1.10"

# Terminal UI
ratatui = "0.26"
crossterm = "0.27"

# Syntax highlighting
tree-sitter = "0.20"
tree-sitter-rust = "0.20"
tree-sitter-python = "0.20"
tree-sitter-javascript = "0.20"
tree-sitter-typescript = "0.20"

# LSP
lsp-types = "0.95"
lsp-server = "0.7"

# Async runtime
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Utilities
anyhow = "1.0"
thiserror = "1.0"
url = "2.4"
similar = "2.3"

[dev-dependencies]
criterion = "0.5"
proptest = "1.0"
tempfile = "3.8"

[[bench]]
name = "buffer_ops"
harness = false

[[bench]]
name = "syntax_parse"
harness = false

[profile.release]
opt-level = 3
lto = true
codegen-units = 1

[profile.dev]
opt-level = 0
```

### config/default.toml

```toml
# Editor Configuration

[general]
tab_width = 4
use_spaces = true
line_numbers = true
relative_line_numbers = false
theme = "dracula"

[formatting]
format_on_save = false
format_on_type = true
auto_close_pairs = true

[lsp]
enabled = true
debounce_ms = 300
show_diagnostics = true
show_inlay_hints = true

[syntax]
highlighting = true
treesitter_enabled = true

[ui]
show_status_bar = true
show_line_numbers = true
cursor_style = "block"  # block, line, underline

[performance]
lazy_parse_threshold = 1000000  # 1MB
max_completions = 50
async_lsp = true

[languages.rust]
indent_width = 4
formatter = "rustfmt"
lsp_server = "rust-analyzer"

[languages.python]
indent_width = 4
formatter = "black"
lsp_server = "pyright"

[languages.javascript]
indent_width = 2
formatter = "prettier"
lsp_server = "typescript-language-server"

[languages.typescript]
indent_width = 2
formatter = "prettier"
lsp_server = "typescript-language-server"
```

## Common Opencode Commands

### Project Setup
```
Create the complete project structure as defined in the documentation.
Include all necessary directories and placeholder files.
```

### Code Generation
```
Implement [module/feature] according to the architecture document.
Include proper error handling, documentation, and tests.
```

### Refactoring
```
Refactor [module] to improve [aspect]. Keep the public API unchanged
and ensure all existing tests still pass.
```

### Documentation
```
Add comprehensive documentation to [module] including:
- Module-level docs explaining purpose
- Struct/enum docs with examples
- Method docs with parameters and return values
```

### Testing
```
Create unit tests for [module] covering:
- Happy path scenarios
- Edge cases
- Error conditions
Add property-based tests using proptest for [specific functionality].
```

## Performance Targets

| Operation | Target Time | Notes |
|-----------|-------------|-------|
| Single char insert | < 16ms | For 60fps feel |
| Newline with indent | < 16ms | Should be instant |
| Initial file parse (10MB) | < 500ms | Acceptable on load |
| Incremental re-parse | < 50ms | After edit |
| LSP completion request | < 100ms | Perceived as fast |
| Full file format | < 200ms | Acceptable delay |
| Scroll (no re-parse) | < 5ms | Smooth scrolling |

## Troubleshooting

### LSP Server Not Starting
```
Check if language server is installed:
  rust-analyzer --version
  pyright --version
  typescript-language-server --version

Check editor logs for error messages.
Try starting the server manually to test.
```

### Syntax Highlighting Not Working
```
Verify tree-sitter grammar is installed.
Check queries/*.scm files exist and are valid.
Enable debug logging to see parse errors.
```

### Performance Issues
```
Profile with: cargo build --release && perf record ./target/release/vim-editor
Check if lazy parsing threshold needs adjustment.
Verify highlight cache is working properly.
```

## Next Steps

1. **Start with Phase 1**: Get the basic editor working
2. **Test thoroughly**: Each phase should be stable before moving on
3. **Iterate with Opencode**: Use it to implement each module
4. **Profile early**: Use benchmarks to catch performance issues
5. **Document as you go**: Keep docs updated with actual implementation

## Resources

- [Ropey Documentation](https://docs.rs/ropey/)
- [Tree-sitter Documentation](https://tree-sitter.github.io/tree-sitter/)
- [LSP Specification](https://microsoft.github.io/language-server-protocol/)
- [Ratatui Book](https://ratatui.rs/)
- [Vim Documentation](https://vimhelp.org/)

---

**Ready to start building!** Begin with Phase 1 and use Opencode to implement each module step by step.
