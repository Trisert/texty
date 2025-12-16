# Agent Instructions for Texty (Terminal Text Editor)

## Build/Lint/Test Commands
- Build: `cargo build`
- Run: `cargo run`
- Test all: `cargo test`
- Test single: `cargo test <module>::<test_name>` (e.g., `cargo test buffer::test_load_and_save`)
- Integration tests: `cargo test --test integration_test`
- Lint: `cargo clippy`
- Format: `cargo fmt`

## Code Style Guidelines
- **Imports**: Group std, external crates, then local modules. Use explicit imports.
- **Formatting**: Run `cargo fmt` before committing. Follows rustfmt defaults.
- **Naming**: snake_case for functions/variables, PascalCase for types/enums, SCREAMING_SNAKE_CASE for constants.
- **Types**: Prefer explicit types. Use &str for string slices, String for owned strings.
- **Error Handling**: Use custom error enums with From implementations. Return Result<T, E>.
- **Documentation**: Add doc comments for public APIs with examples.
- **Testing**: Use proptest for property-based tests. Write unit tests for all public functions.
- **Comments**: Do not add comments unless requested.
- **Libraries**: Check existing usage before adding new dependencies.
- **Security**: Avoid exposing or logging secrets; follow best practices.
- **LSP**: Use async architecture with Arc<Mutex<>> for thread safety.

## Project Structure
- Core modules: buffer, cursor, viewport, mode, command, keymap, lsp
- Syntax highlighting: tree-sitter based in syntax/ module
- LSP: Helix-style async transport layer with multi-server support
- Dependencies: ropey (text), crossterm (terminal), tree-sitter (parsing), tokio (async)

## Agent Behavior
- Be concise, direct, and to the point.
- Minimize output; answer in 1-3 sentences unless detail requested.
- Only use emojis if explicitly requested.
- Refuse malicious code requests.
- Run lint/format after code changes.
- Do not commit unless asked.