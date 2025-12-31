// src/vim_parser.rs - Multi-key command parser for Vim-style key sequences

use crate::command::Command;
use crossterm::event::{KeyEvent, KeyCode};

/// Result of parsing a key event
#[derive(Debug, Clone, PartialEq)]
pub enum ParseResult {
    /// Complete command ready to execute
    Command(Command),
    /// More keys needed to complete the command
    Pending,
    /// Invalid key sequence
    Invalid,
}

/// Operators that can combine with motions
#[derive(Debug, Clone, Copy, PartialEq)]
enum Operator {
    Delete,
    Yank,
    Change,
    Indent,
    Unindent,
    Format,
}

/// Parser state machine
#[derive(Debug, Clone, PartialEq)]
enum ParserState {
    Idle,
    ReadingCount,
    ReadingRegister,
    ReadingOperator,
    ReadingOperatorCount,
    ReadingMotion,
    ReadingTextObject,
    ReadingReplaceChar,
}

/// Parser for Vim-style multi-key commands
#[derive(Debug, Clone)]
pub struct VimParser {
    state: ParserState,
    count: Option<usize>,
    register: Option<char>,
    operator: Option<Operator>,
    operator_count: Option<usize>,
    motion_buffer: Vec<char>,
    _replace_char: Option<char>,
}

impl Default for VimParser {
    fn default() -> Self {
        Self::new()
    }
}

impl VimParser {
    pub fn new() -> Self {
        Self {
            state: ParserState::Idle,
            count: None,
            register: None,
            operator: None,
            operator_count: None,
            motion_buffer: Vec::new(),
            _replace_char: None,
        }
    }

    /// Reset parser to initial state
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Process a key event and return the parse result
    pub fn process_key(&mut self, key: KeyEvent) -> ParseResult {
        use crossterm::event::{KeyCode, KeyModifiers};

        // Handle arrow keys directly
        match key.code {
            KeyCode::Left => {
                self.reset();
                return ParseResult::Command(Command::MoveLeft);
            }
            KeyCode::Down => {
                self.reset();
                return ParseResult::Command(Command::MoveDown);
            }
            KeyCode::Up => {
                self.reset();
                return ParseResult::Command(Command::MoveUp);
            }
            KeyCode::Right => {
                self.reset();
                return ParseResult::Command(Command::MoveRight);
            }
            _ => {}
        }

        // Extract character from key event
        let ch = match key.code {
            KeyCode::Char(c) => Some(c),
            KeyCode::Enter => Some('\n'),
            KeyCode::Tab => Some('\t'),
            KeyCode::Backspace => return ParseResult::Command(Command::DeleteChar),
            KeyCode::Esc => {
                self.reset();
                return ParseResult::Command(Command::NormalMode);
            }
            _ => None,
        };

        // Handle Ctrl key combinations
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return self.process_ctrl_key(key.code);
        }

        match self.state {
            ParserState::Idle => self.process_idle(ch),
            ParserState::ReadingCount => self.process_reading_count(ch),
            ParserState::ReadingRegister => self.process_reading_register(ch),
            ParserState::ReadingOperator => self.process_reading_operator(ch),
            ParserState::ReadingOperatorCount => self.process_reading_operator_count(ch),
            ParserState::ReadingMotion => self.process_reading_motion(ch),
            ParserState::ReadingTextObject => self.process_reading_text_object(ch),
            ParserState::ReadingReplaceChar => self.process_reading_replace_char(ch),
        }
    }

    fn process_ctrl_key(&mut self, code: KeyCode) -> ParseResult {
        match code {
            KeyCode::Char('r') => ParseResult::Command(Command::Redo),
            KeyCode::Char('f') => ParseResult::Command(Command::OpenFuzzySearch),
            _ => ParseResult::Invalid,
        }
    }

    fn process_idle(&mut self, ch: Option<char>) -> ParseResult {
        let ch = match ch {
            Some(c) => c,
            None => return ParseResult::Invalid,
        };

        match ch {
            // Numbers start count parsing
            '1'..='9' => {
                self.count = Some(ch.to_digit(10).unwrap() as usize);
                self.state = ParserState::ReadingCount;
                ParseResult::Pending
            }
            '0' => ParseResult::Command(Command::MoveLineStart),

            // Single-character commands
            'x' => {
                let count = self.count.unwrap_or(1);
                self.reset();
                ParseResult::Command(Command::DeleteCharForward(count))
            }
            'X' => ParseResult::Command(Command::DeleteChar),
            's' => {
                self.reset();
                ParseResult::Command(Command::SubstituteChar)
            }
            'S' => {
                self.reset();
                ParseResult::Command(Command::SubstituteLine)
            }
            'p' => {
                self.reset();
                ParseResult::Command(Command::PasteAfter)
            }
            'P' => {
                self.reset();
                ParseResult::Command(Command::PasteBefore)
            }
            'r' => {
                self.state = ParserState::ReadingReplaceChar;
                ParseResult::Pending
            }
            'J' => {
                let count = self.count.unwrap_or(1);
                self.reset();
                ParseResult::Command(Command::JoinLines(count))
            }
            'u' => {
                self.reset();
                ParseResult::Command(Command::Undo)
            }

            // Motion commands
            'h' => {
                let _count = self.count.unwrap_or(1);
                self.reset();
                ParseResult::Command(Command::MoveLeft)
            }
            'j' => {
                let _count = self.count.unwrap_or(1);
                self.reset();
                ParseResult::Command(Command::MoveDown)
            }
            'k' => {
                let _count = self.count.unwrap_or(1);
                self.reset();
                ParseResult::Command(Command::MoveUp)
            }
            'l' => {
                let _count = self.count.unwrap_or(1);
                self.reset();
                ParseResult::Command(Command::MoveRight)
            }
            'w' => {
                let count = self.count.unwrap_or(1);
                self.reset();
                ParseResult::Command(Command::MoveWordForward(count))
            }
            'b' => {
                let count = self.count.unwrap_or(1);
                self.reset();
                ParseResult::Command(Command::MoveWordBackward(count))
            }
            'e' => {
                let count = self.count.unwrap_or(1);
                self.reset();
                ParseResult::Command(Command::MoveWordEnd(count))
            }
            '$' => {
                let count = self.count.unwrap_or(1);
                self.reset();
                ParseResult::Command(Command::MoveLineEnd(count))
            }
            '^' => {
                self.reset();
                ParseResult::Command(Command::MoveFirstNonBlank)
            }
            'G' => {
                let _line = self.count.unwrap_or(0); // 0 means end of file
                self.reset();
                ParseResult::Command(Command::MoveFileEnd)
            }
            'H' => ParseResult::Command(Command::MoveScreenTop),
            'M' => ParseResult::Command(Command::MoveScreenMiddle),
            'L' => ParseResult::Command(Command::MoveScreenBottom),

            // Operator-pending commands
            'd' | 'y' | 'c' | '>' | '<' | '=' | 'g' | 'f' | 't' | 'T' | 'F' => {
                let op = match ch {
                    'd' => Operator::Delete,
                    'y' => Operator::Yank,
                    'c' => Operator::Change,
                    '>' => Operator::Indent,
                    '<' => Operator::Unindent,
                    '=' => Operator::Format,
                    _ => return ParseResult::Invalid,
                };
                self.operator = Some(op);
                self.state = ParserState::ReadingOperator;
                ParseResult::Pending
            }

            // Register selection
            '"' => {
                self.state = ParserState::ReadingRegister;
                ParseResult::Pending
            }

            // Visual mode
            'v' => {
                self.reset();
                ParseResult::Command(Command::VisualChar)
            }
            'V' => {
                self.reset();
                ParseResult::Command(Command::VisualLine)
            }

            // Mode switching
            'i' => {
                self.reset();
                ParseResult::Command(Command::InsertMode)
            }
            ':' => {
                self.reset();
                ParseResult::Command(Command::EnterCommandMode)
            }

            // Other characters
            _ => {
                self.reset();
                ParseResult::Invalid
            }
        }
    }

    fn process_reading_count(&mut self, ch: Option<char>) -> ParseResult {
        let ch = match ch {
            Some(c) => c,
            None => {
                self.reset();
                return ParseResult::Invalid;
            }
        };

        if ch.is_ascii_digit() {
            // Continue building count
            let current = self.count.unwrap_or(0);
            self.count = Some(current * 10 + ch.to_digit(10).unwrap() as usize);
            ParseResult::Pending
        } else {
            // Count finished, process this character
            self.state = ParserState::Idle;
            self.process_idle(Some(ch))
        }
    }

    fn process_reading_register(&mut self, ch: Option<char>) -> ParseResult {
        let ch = match ch {
            Some(c) => c,
            None => {
                self.reset();
                return ParseResult::Invalid;
            }
        };

        // Valid register names: ", a-z, 0-9, -, *, +, .
        if ch.is_alphanumeric() || matches!(ch, '"' | '-' | '*' | '+' | '.') {
            self.register = Some(ch);
            self.state = ParserState::Idle;
            ParseResult::Pending
        } else {
            self.reset();
            ParseResult::Invalid
        }
    }

    fn process_reading_operator(&mut self, ch: Option<char>) -> ParseResult {
        let ch = match ch {
            Some(c) => c,
            None => {
                self.reset();
                return ParseResult::Invalid;
            }
        };

        if ch.is_ascii_digit() {
            self.operator_count = Some(ch.to_digit(10).unwrap() as usize);
            self.state = ParserState::ReadingOperatorCount;
            ParseResult::Pending
        } else {
            self.motion_buffer.clear();
            self.state = ParserState::ReadingMotion;
            self.process_reading_motion(Some(ch))
        }
    }

    fn process_reading_operator_count(&mut self, ch: Option<char>) -> ParseResult {
        let ch = match ch {
            Some(c) => c,
            None => {
                self.reset();
                return ParseResult::Invalid;
            }
        };

        if ch.is_ascii_digit() {
            let current = self.operator_count.unwrap_or(0);
            self.operator_count = Some(current * 10 + ch.to_digit(10).unwrap() as usize);
            ParseResult::Pending
        } else {
            self.motion_buffer.clear();
            self.state = ParserState::ReadingMotion;
            self.process_reading_motion(Some(ch))
        }
    }

    fn process_reading_motion(&mut self, ch: Option<char>) -> ParseResult {
        let ch = match ch {
            Some(c) => c,
            None => {
                self.reset();
                return ParseResult::Invalid;
            }
        };

        self.motion_buffer.push(ch);

        // Check for complete motion
        let motion_str: String = self.motion_buffer.iter().collect();
        let count = self.operator_count.or(self.count).unwrap_or(1);

        let cmd = match (self.operator, motion_str.as_str()) {
            // Delete motions
            (Some(Operator::Delete), "d") => Command::DeleteLine,
            (Some(Operator::Delete), "w") => Command::DeleteWord(count),
            (Some(Operator::Delete), "e") => Command::DeleteToEndWord(count),
            (Some(Operator::Delete), "b") => Command::DeleteToStartWord(count),
            (Some(Operator::Delete), "$") => Command::DeleteToEnd,
            (Some(Operator::Delete), "0") => Command::DeleteToStart,
            (Some(Operator::Delete), "G") => Command::DeleteToEndOfFile,
            (Some(Operator::Delete), "gg") => Command::DeleteToStartOfFile,
            (Some(Operator::Delete), "i") => {
                self.state = ParserState::ReadingTextObject;
                return ParseResult::Pending;
            }

            // Yank motions
            (Some(Operator::Yank), "y") => Command::YankLine,
            (Some(Operator::Yank), "w") => Command::YankWord(count),
            (Some(Operator::Yank), "$") => Command::YankToEnd,
            (Some(Operator::Yank), "0") => Command::YankToStart,

            // Change motions
            (Some(Operator::Change), "c") => Command::ChangeLine,
            (Some(Operator::Change), "w") => Command::ChangeWord(count),
            (Some(Operator::Change), "$") => Command::ChangeToEnd,
            (Some(Operator::Change), "0") => Command::ChangeToStart,

            // Double operators as linewise operations
            (Some(Operator::Indent), ">") => Command::IndentLine(count),
            (Some(Operator::Unindent), "<") => Command::UnindentLine(count),
            (Some(Operator::Format), "=") => Command::FormatBuffer,

            _ => return ParseResult::Pending,
        };

        self.reset();
        ParseResult::Command(cmd)
    }

    fn process_reading_text_object(&mut self, ch: Option<char>) -> ParseResult {
        let ch = match ch {
            Some(c) => c,
            None => {
                self.reset();
                return ParseResult::Invalid;
            }
        };

        // Text objects: iw, aw, i", a", i), a), etc.
        let inner = self.motion_buffer.contains(&'i');
        let text_obj = ch;

        let count = self.operator_count.or(self.count).unwrap_or(1);
        let cmd = match (self.operator, inner, text_obj) {
            (Some(Operator::Change), true, 'w') => Command::ChangeInnerWord(count),
            (Some(Operator::Change), false, 'w') => Command::ChangeAWord(count),
            (Some(Operator::Delete), true, 'w') => Command::DeleteInnerWord(count),
            (Some(Operator::Delete), false, 'w') => Command::DeleteAWord(count),
            (Some(Operator::Yank), true, 'w') => Command::YankInnerWord(count),
            (Some(Operator::Yank), false, 'w') => Command::YankAWord(count),
            _ => {
                self.reset();
                return ParseResult::Invalid;
            }
        };

        self.reset();
        ParseResult::Command(cmd)
    }

    fn process_reading_replace_char(&mut self, ch: Option<char>) -> ParseResult {
        let ch = match ch {
            Some(c) => c,
            None => {
                self.reset();
                return ParseResult::Invalid;
            }
        };

        if !ch.is_control() && ch != '\n' {
            let cmd = Command::ReplaceChar(ch);
            self.reset();
            ParseResult::Command(cmd)
        } else {
            self.reset();
            ParseResult::Invalid
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyCode};

    fn key_char(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), crossterm::event::KeyModifiers::NONE)
    }

    #[test]
    fn test_simple_motion() {
        let mut parser = VimParser::new();
        assert_eq!(
            parser.process_key(key_char('w')),
            ParseResult::Command(Command::MoveWordForward(1))
        );
    }

    #[test]
    fn test_counted_motion() {
        let mut parser = VimParser::new();
        assert_eq!(parser.process_key(key_char('3')), ParseResult::Pending);
        assert_eq!(
            parser.process_key(key_char('w')),
            ParseResult::Command(Command::MoveWordForward(3))
        );
    }

    #[test]
    fn test_double_key_command() {
        let mut parser = VimParser::new();
        assert_eq!(parser.process_key(key_char('d')), ParseResult::Pending);
        assert_eq!(
            parser.process_key(key_char('d')),
            ParseResult::Command(Command::DeleteLine)
        );
    }

    #[test]
    fn test_operator_with_motion() {
        let mut parser = VimParser::new();
        assert_eq!(parser.process_key(key_char('d')), ParseResult::Pending);
        assert_eq!(parser.process_key(key_char('w')), ParseResult::Command(Command::DeleteWord(1)));
    }

    #[test]
    fn test_counted_operator_with_motion() {
        let mut parser = VimParser::new();
        assert_eq!(parser.process_key(key_char('2')), ParseResult::Pending);
        assert_eq!(parser.process_key(key_char('d')), ParseResult::Pending);
        assert_eq!(parser.process_key(key_char('w')), ParseResult::Command(Command::DeleteWord(2)));
    }

    #[test]
    fn test_operator_count_with_motion() {
        let mut parser = VimParser::new();
        assert_eq!(parser.process_key(key_char('d')), ParseResult::Pending);
        assert_eq!(parser.process_key(key_char('3')), ParseResult::Pending);
        assert_eq!(parser.process_key(key_char('w')), ParseResult::Command(Command::DeleteWord(3)));
    }

    #[test]
    fn test_reset_on_escape() {
        let mut parser = VimParser::new();
        assert_eq!(parser.process_key(key_char('d')), ParseResult::Pending);
        let esc_key = KeyEvent::new(KeyCode::Esc, crossterm::event::KeyModifiers::NONE);
        assert_eq!(parser.process_key(esc_key), ParseResult::Command(Command::NormalMode));
        assert_eq!(parser.state, ParserState::Idle);
    }

    #[test]
    fn test_simple_delete_char() {
        let mut parser = VimParser::new();
        assert_eq!(
            parser.process_key(key_char('x')),
            ParseResult::Command(Command::DeleteCharForward(1))
        );
    }

    #[test]
    fn test_counted_delete_char() {
        let mut parser = VimParser::new();
        assert_eq!(parser.process_key(key_char('5')), ParseResult::Pending);
        assert_eq!(
            parser.process_key(key_char('x')),
            ParseResult::Command(Command::DeleteCharForward(5))
        );
    }
}
