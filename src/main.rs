use crossterm::{
    event::{Event, KeyCode, read},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use texty::ui::renderer::TuiRenderer;
use texty::{command::Command, editor::Editor, mode::Mode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Enable raw mode and enter alternate screen
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;

    // Initialize editor
    let mut editor = Editor::new();

    // Initialize renderer
    let mut renderer = TuiRenderer::new()?;

    // Basic event loop
    loop {
        // Render
        renderer.draw(&editor)?;

        // Read event
        match read()? {
            Event::Key(key_event) => {
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
                    }
                    _ => {
                        // Handle normal commands
                        let command = key_to_command(key_event, &editor.mode);
                        if let Some(cmd) = command
                            && editor.execute_command(cmd) {
                                break; // Quit
                            }
                    }
                }
            }
            Event::Resize(rows, cols) => {
                editor.handle_resize(rows, cols);
            }
            _ => {}
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
            KeyCode::Char('h') => Some(Command::MoveLeft),
            KeyCode::Char('j') => Some(Command::MoveDown),
            KeyCode::Char('k') => Some(Command::MoveUp),
            KeyCode::Char('l') => Some(Command::MoveRight),
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
            _ => None,
        },
        Mode::Insert => match key_event.code {
            KeyCode::Esc => Some(Command::NormalMode),
            KeyCode::Char(c) => Some(Command::InsertChar(c)),
            KeyCode::Enter => Some(Command::InsertChar('\n')),
            KeyCode::Backspace => Some(Command::DeleteChar),
            _ => None,
        },
        _ => None,
    }
}
