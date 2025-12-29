# Agent Instructions for Texty (Terminal Text Editor)

## Build/Lint/Test Commands
- Build: `cargo build` | Run: `cargo run`
- Test all: `cargo test` | Test single: `cargo test <module>::<test_name>`
- Lint: `cargo clippy` | Format: `cargo fmt`

## Code Style Guidelines
- **Imports**: Group std, external crates, then local modules. Use explicit imports.
- **Naming**: snake_case for functions/variables, PascalCase for types/enums.
- **Types**: Prefer explicit types; use &str for string slices, String for owned strings.
- **Error Handling**: Use custom error enums with From implementations. Return Result<T, E>.
- **Testing**: Use proptest for property-based tests. Write unit tests for public functions.
- **Comments**: Do not add comments unless requested.

## AST-Grep
- Use `ast-grep` for structural code search and refactoring.
- Example: `sg -p 'fn $NAME() -> $TYPE' --lang rust`

## Agent Behavior
- Be concise and direct (1-3 sentences unless detail requested).
- Run cargo clippy and cargo fmt after code changes.
- Check existing dependencies before adding new ones.
- Do not commit unless explicitly asked.
