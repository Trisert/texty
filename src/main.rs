use crossterm::{
    cursor::MoveTo,
    event::{Event, KeyCode, read},
    execute, queue,
    style::{Color, Print, PrintStyledContent, Stylize},
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use std::io::{Write, stdout};
use texty::syntax::HighlightKind;
use texty::{command::Command, editor::Editor, mode::Mode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Enable raw mode and enter alternate screen
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    // Initialize editor
    let mut editor = Editor::new();

    // Basic event loop
    loop {
        // Render
        render(&mut stdout, &editor)?;

        // Read event
        match read()? {
            Event::Key(key_event) => {
                let command = key_to_command(key_event, &editor.mode);
                if let Some(cmd) = command {
                    if matches!(cmd, Command::Quit) {
                        break;
                    }
                    editor.execute_command(cmd);
                }
            }
            Event::Resize(rows, cols) => {
                editor.handle_resize(rows, cols);
            }
            _ => {}
        }
    }

    // Leave alternate screen and disable raw mode
    execute!(stdout, LeaveAlternateScreen)?;
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
            KeyCode::Char('f') => Some(Command::FormatBuffer),
            KeyCode::Char('c') => Some(Command::Completion),
            KeyCode::Char('g') => Some(Command::GotoDefinition),
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

fn render<W: Write>(w: &mut W, editor: &Editor) -> Result<(), std::io::Error> {
    // Clear screen
    queue!(w, Clear(ClearType::All), MoveTo(0, 0))?;

    // Render visible lines
    for i in 0..editor.viewport.rows {
        let line_idx = editor.viewport.offset_line + i;
        if let Some(line) = editor.buffer.line(line_idx) {
            let visible_line = &line[editor.viewport.offset_col..];
            if let Some(highlights) = editor
                .buffer
                .highlighter
                .as_ref()
                .and_then(|h| h.get_line_highlights(line_idx))
            {
                // Render with highlights
                let mut pos = 0;
                for token in highlights {
                    if token.start >= editor.viewport.offset_col
                        && token.start < editor.viewport.offset_col + visible_line.len()
                    {
                        let start = token.start.saturating_sub(editor.viewport.offset_col);
                        let end = token
                            .end
                            .min(editor.viewport.offset_col + visible_line.len())
                            .saturating_sub(editor.viewport.offset_col);
                        if start > pos {
                            queue!(w, Print(&visible_line[pos..start]))?;
                        }
                        let styled = visible_line[start..end].with(kind_to_color(token.kind));
                        queue!(w, PrintStyledContent(styled))?;
                        pos = end;
                    }
                }
                if pos < visible_line.len() {
                    queue!(w, Print(&visible_line[pos..]))?;
                }
            } else {
                queue!(w, Print(visible_line))?;
            }
            queue!(w, Print("\r\n"))?;
        } else {
            queue!(w, Print("~\r\n"))?;
        }
    }

    // Status bar
    queue!(w, MoveTo(0, editor.viewport.rows as u16))?;
    let status = format!(
        " {} | {}:{} | Modified: {} ",
        mode_to_str(&editor.mode),
        editor.cursor.line,
        editor.cursor.col,
        editor.buffer.modified
    );
    queue!(w, Print(status))?;

    // Move cursor to position
    let screen_row = editor
        .cursor
        .line
        .saturating_sub(editor.viewport.offset_line) as u16;
    let screen_col = editor.cursor.col.saturating_sub(editor.viewport.offset_col) as u16;
    queue!(w, MoveTo(screen_col, screen_row))?;

    w.flush()
}

fn mode_to_str(mode: &Mode) -> &'static str {
    match mode {
        Mode::Normal => "NORMAL",
        Mode::Insert => "INSERT",
        Mode::Visual => "VISUAL",
        Mode::Command => "COMMAND",
    }
}

fn kind_to_color(kind: HighlightKind) -> Color {
    match kind {
        HighlightKind::Keyword => Color::Cyan,
        HighlightKind::Function => Color::Green,
        HighlightKind::Type => Color::Yellow,
        HighlightKind::String => Color::Red,
        HighlightKind::Comment => Color::Blue,
        HighlightKind::Variable => Color::White,
    }
}
