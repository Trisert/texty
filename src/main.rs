use crossterm::{
    event::{Event, KeyCode, KeyModifiers, read},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::time::{Duration, Instant};
use texty::cli;
use texty::ui::renderer::TuiRenderer;
use texty::{command::Command, editor::Editor, mode::Mode, vim_parser::ParseResult};

// Global state for double space detection
static mut LAST_SPACE_TIME: Option<Instant> = None;

/// Application entry point: parse command-line arguments, initialize the terminal and editor state,
/// open a file or directory if provided, run the main event loop, and restore the terminal on exit.
///
/// The function performs terminal setup (raw mode, alternate screen), constructs the UI renderer
/// according to CLI flags or the `TEXTY_TERMINAL_PALETTE` environment variable, and drives input
/// events until the editor requests shutdown. On exit it leaves the alternate screen and disables
/// raw mode.
///
/// # Returns
///
/// `Ok(())` on normal shutdown, or an error if terminal setup, renderer creation, I/O, or event
/// handling fails.
///
/// # Examples
///
/// ```no_run
/// // Run the application's async main from a synchronous context.
/// tokio::runtime::Runtime::new().unwrap().block_on(crate::main()).unwrap();
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments first (before terminal setup)
    let cli_args = match cli::parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error parsing arguments: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize logger (set RUST_LOG env var to control verbosity)
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    // Enable raw mode and enter alternate screen
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;

    // Initialize editor
    let mut editor = Editor::new();

    // Handle file/directory argument if specified
    if let Some(path) = &cli_args.file {
        if !cli_args.exists() {
            eprintln!("Error: Path '{}' does not exist", path.display());
            // Continue with empty buffer if path doesn't exist
        } else if cli_args.is_directory() {
            // Directory → start in fuzzy search mode
            editor.start_fuzzy_search_in_dir(path);
        } else {
            // File → open normally (using async version to avoid blocking)
            if let Err(e) = editor.open_file_async(&path.to_string_lossy()).await {
                eprintln!("Error opening file '{}': {}", path.display(), e);
                // Continue with empty buffer if file can't be opened
            }
        }
    }

    // Handle --list-themes flag
    if cli_args.list_themes {
        let themes = texty::theme_discovery::list_builtin_themes();
        println!("Available built-in themes:");
        for theme in themes {
            println!("  {}", theme);
        }
        std::process::exit(0);
    }

    // Initialize renderer
    let use_terminal_palette = cli_args.terminal_palette
        || std::env::var("TEXTY_TERMINAL_PALETTE")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);
    let mut renderer = TuiRenderer::new(use_terminal_palette, &cli_args.theme)?;

    // Frame rate limiting constants
    const TARGET_FPS: u64 = 60;
    const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / TARGET_FPS);

    // Event loop with frame rate limiting
    let mut last_frame_time = Instant::now();
    let mut needs_redraw = true;

    loop {
        // Only render if needed and enough time has elapsed since last frame
        if needs_redraw && last_frame_time.elapsed() >= FRAME_DURATION {
            renderer.draw(&mut editor)?;
            last_frame_time = Instant::now();
            needs_redraw = false;
        }

        // Read event (blocking, with timeout for periodic redraws)
        let event = if last_frame_time.elapsed() < FRAME_DURATION {
            // Use poll with timeout to respect frame rate
            let timeout = FRAME_DURATION.saturating_sub(last_frame_time.elapsed());
            if crossterm::event::poll(timeout)? {
                Some(read()?)
            } else {
                None
            }
        } else {
            Some(read()?)
        };

        match event {
            Some(Event::Key(key_event)) => {
                match &editor.mode {
                    Mode::Command => {
                        // Handle command line input
                        let should_quit = match key_event.code {
                            KeyCode::Char(c) => editor.handle_command_input(c)?,
                            KeyCode::Enter => editor.handle_command_input('\n')?,
                            KeyCode::Backspace => editor.handle_command_input('\x08')?,
                            KeyCode::Esc => editor.handle_command_input('\x1b')?,
                            _ => false,
                        };
                        if should_quit {
                            break;
                        }
                        needs_redraw = true;
                    }
                    Mode::Normal | Mode::Visual => {
                        // Special handling for double-space to open fuzzy search
                        if key_event.code == KeyCode::Char(' ') {
                            let now = Instant::now();
                            let is_double_space = unsafe {
                                if let Some(last_time) = LAST_SPACE_TIME {
                                    now.duration_since(last_time) < Duration::from_millis(500)
                                } else {
                                    false
                                }
                            };

                            unsafe {
                                LAST_SPACE_TIME = Some(now);
                            }

                            if is_double_space {
                                if editor.execute_command(Command::OpenFuzzySearch) {
                                    break;
                                }
                                needs_redraw = true;
                            }
                        } else {
                            // Use Vim parser for multi-key command sequences
                            match editor.vim_parser.process_key(key_event) {
                                ParseResult::Command(cmd) => {
                                    if editor.execute_command(cmd) {
                                        break; // Quit
                                    }
                                    needs_redraw = true;
                                }
                                ParseResult::Pending => {
                                    // Continue waiting for more keys (multi-key sequence)
                                    needs_redraw = true;
                                }
                                ParseResult::Invalid => {
                                    // Invalid sequence, reset parser
                                    editor.vim_parser.reset();
                                    editor.status_message = Some("Invalid command".to_string());
                                    needs_redraw = true;
                                }
                            }
                        }
                    }
                    _ => {
                        // Handle other modes with simple key_to_command
                        let command = key_to_command(key_event, &editor.mode);
                        if let Some(cmd) = command {
                            if editor.execute_command(cmd) {
                                break; // Quit
                            }
                            needs_redraw = true;
                        }
                    }
                }
            }
            Some(Event::Resize(rows, cols)) => {
                editor.handle_resize(rows, cols);
                needs_redraw = true;
            }
            None => {
                // Timeout - no event, continue loop
            }
            Some(_) => {}
        }
    }

    // Leave alternate screen and disable raw mode
    crossterm::execute!(stdout, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn key_to_command(key_event: crossterm::event::KeyEvent, mode: &Mode) -> Option<Command> {
    match mode {
        Mode::Normal => match key_event.code {
            // Vim-style movement
            KeyCode::Char('h') => Some(Command::MoveLeft),
            KeyCode::Char('j') => Some(Command::MoveDown),
            KeyCode::Char('k') => Some(Command::MoveUp),
            KeyCode::Char('l') => Some(Command::MoveRight),
            // Arrow key movement (same as hjkl)
            KeyCode::Left => Some(Command::MoveLeft),
            KeyCode::Down => Some(Command::MoveDown),
            KeyCode::Up => Some(Command::MoveUp),
            KeyCode::Right => Some(Command::MoveRight),
            KeyCode::Char('i') => Some(Command::InsertMode),
            KeyCode::Char(':') => Some(Command::EnterCommandMode),
            KeyCode::Char('f') => Some(Command::FormatBuffer),
            KeyCode::Char('c') => Some(Command::Completion),
            KeyCode::Char('n') => Some(Command::CompletionNext),
            KeyCode::Char('p') => Some(Command::CompletionPrev),
            KeyCode::Enter => Some(Command::CompletionAccept),
            KeyCode::Char('g') => Some(Command::GotoDefinition),
            KeyCode::Char('r') => Some(Command::FindReferences),
            KeyCode::Char('H') => Some(Command::Hover),
            KeyCode::Char('s') => Some(Command::WorkspaceSymbols),
            KeyCode::Char('a') => Some(Command::CodeAction),
            KeyCode::Char('w') => Some(Command::SaveFile),
            KeyCode::Char('q') => Some(Command::Quit),
            KeyCode::Char(' ') => {
                // Check for double space
                let now = Instant::now();
                let is_double_space = unsafe {
                    if let Some(last_time) = LAST_SPACE_TIME {
                        now.duration_since(last_time) < Duration::from_millis(500)
                    } else {
                        false
                    }
                };

                unsafe {
                    LAST_SPACE_TIME = Some(now);
                }

                if is_double_space {
                    Some(Command::OpenFuzzySearch)
                } else {
                    None
                }
            }
            _ => None,
        },
        Mode::Insert => match key_event.code {
            KeyCode::Esc => Some(Command::NormalMode),
            KeyCode::Char(c) => Some(Command::InsertChar(c)),
            KeyCode::Enter => Some(Command::InsertChar('\n')),
            KeyCode::Backspace => Some(Command::DeleteChar),
            // Arrow keys for navigation in insert mode
            KeyCode::Left => Some(Command::MoveLeft),
            KeyCode::Right => Some(Command::MoveRight),
            KeyCode::Up => Some(Command::MoveUp),
            KeyCode::Down => Some(Command::MoveDown),
            _ => None,
        },
        Mode::FuzzySearch => match key_event.code {
            KeyCode::Esc => Some(Command::FuzzySearchCancel),
            KeyCode::Enter => Some(Command::FuzzySearchSelect),
            KeyCode::Up | KeyCode::Char('k') => Some(Command::FuzzySearchUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Command::FuzzySearchDown),
            KeyCode::Tab => Some(Command::FuzzySearchLoadMore),
            KeyCode::Char('r') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Command::FuzzySearchToggleRecursive)
            }
            KeyCode::Char('g') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Command::FuzzySearchToggleGitignore)
            }
            KeyCode::Char(c)
                if c.is_alphanumeric() || c == ' ' || c == '.' || c == '_' || c == '-' =>
            {
                // Add character to fuzzy search query
                Some(Command::InsertChar(c))
            }
            KeyCode::Backspace => Some(Command::DeleteChar),
            _ => None,
        },
        _ => None,
    }
}
