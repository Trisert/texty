// src/motion.rs - Position and motion calculation for Vim commands

use crate::buffer::Buffer;

/// A position in the buffer (line, column)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

impl Position {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

/// A range from start to end (inclusive)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Normalize range so start <= end
    pub fn normalized(self) -> Range {
        if self.start.line < self.end.line
            || (self.start.line == self.end.line && self.start.col <= self.end.col)
        {
            self
        } else {
            Range {
                start: self.end,
                end: self.start,
            }
        }
    }
}

/// Word boundaries for Vim-style word movement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordKind {
    /// Word consists of alphanumeric and underscore
    Alphanumeric,
    /// Word is whitespace
    Whitespace,
    /// Word is other non-whitespace
    Other,
}

/// Determine the kind of character at a position
fn char_kind(c: char) -> WordKind {
    if c.is_alphanumeric() || c == '_' {
        WordKind::Alphanumeric
    } else if c.is_whitespace() {
        WordKind::Whitespace
    } else {
        WordKind::Other
    }
}

/// Move forward by one word (Vim's `w` motion)
///
/// Rules:
/// - If on whitespace, skip to next word
/// - If on a word, go to start of next word
/// - Word boundaries are alphanumeric/underscore vs other vs whitespace
pub fn word_forward(buffer: &Buffer, pos: Position) -> Position {
    let line_count = buffer.line_count();
    if line_count == 0 {
        return pos;
    }

    let mut line = pos.line;
    let mut col = pos.col;

    loop {
        let current_line = buffer.line(line);
        let current_line = match current_line {
            Some(l) => l,
            None => {
                // End of buffer
                return Position::new(line_count.saturating_sub(1), 0);
            }
        };

        let chars: Vec<char> = current_line.chars().collect();

        // Skip to end of current word/whitespace
        let start_kind = if col < chars.len() {
            char_kind(chars[col])
        } else {
            WordKind::Whitespace
        };

        // Move forward to different character kind
        while col < chars.len() && char_kind(chars[col]) == start_kind {
            col += 1;
        }

        // Skip whitespace between words
        while col < chars.len() && chars[col].is_whitespace() {
            col += 1;
        }

        // If we found a non-whitespace character, that's the next word start
        if col < chars.len() {
            return Position::new(line, col);
        }

        // Otherwise, move to next line
        line += 1;
        col = 0;
        if line >= line_count {
            // End of buffer
            return Position::new(line_count.saturating_sub(1), buffer.line_len(line_count - 1));
        }
    }
}

/// Move forward to end of word (Vim's `e` motion)
pub fn word_end(buffer: &Buffer, pos: Position) -> Position {
    let line_count = buffer.line_count();
    if line_count == 0 {
        return pos;
    }

    let mut line = pos.line;
    let mut col = pos.col.min(buffer.line_len(line).saturating_sub(1));

    // If we're at end of line, move to next line
    if col >= buffer.line_len(line).saturating_sub(1) {
        if line + 1 >= line_count {
            return Position::new(line, buffer.line_len(line).saturating_sub(1));
        }
        line += 1;
        col = 0;
    }

    let current_line = match buffer.line(line) {
        Some(l) => l,
        None => return Position::new(line, col),
    };

    let chars: Vec<char> = current_line.chars().collect();
    if chars.is_empty() {
        return Position::new(line, 0);
    }

    // Move to the end of the current word
    let _start_kind = if col < chars.len() {
        char_kind(chars[col])
    } else {
        WordKind::Whitespace
    };

    // If we're on whitespace, skip it
    while col < chars.len() && char_kind(chars[col]) == WordKind::Whitespace {
        col += 1;
    }

    // Find end of current word
    while col < chars.len() && char_kind(chars[col]) == WordKind::Alphanumeric {
        col += 1;
    }

    // Move back to last alphanumeric character
    col = col.saturating_sub(1);

    Position::new(line, col)
}

/// Move backward by one word (Vim's `b` motion)
pub fn word_backward(buffer: &Buffer, pos: Position) -> Position {
    let mut line = pos.line;
    let mut col = pos.col;

    loop {
        if line >= buffer.line_count() {
            line = buffer.line_count().saturating_sub(1);
            col = buffer.line_len(line);
        }

        let current_line = match buffer.line(line) {
            Some(l) => l,
            None => {
                if line > 0 {
                    line -= 1;
                    col = buffer.line_len(line);
                    continue;
                } else {
                    return Position::new(0, 0);
                }
            }
        };

        let chars: Vec<char> = current_line.chars().collect();

        if col == 0 {
            if line == 0 {
                return Position::new(0, 0);
            }
            line -= 1;
            col = buffer.line_len(line);
            continue;
        }

        col = col.min(chars.len());

        // Skip whitespace behind us
        while col > 0 && chars[col - 1].is_whitespace() {
            col -= 1;
        }

        if col == 0 {
            if line == 0 {
                return Position::new(0, 0);
            }
            line -= 1;
            col = buffer.line_len(line);
            continue;
        }

        // Find the word boundary
        let start_kind = char_kind(chars[col - 1]);
        while col > 0 && char_kind(chars[col - 1]) == start_kind {
            col -= 1;
        }

        // Return to the start of this word
        return Position::new(line, col);
    }
}

/// Move to start of line (Vim's `0` motion)
pub fn line_start(_pos: Position) -> Position {
    Position::new(_pos.line, 0)
}

/// Move to first non-blank character of line (Vim's `^` motion)
pub fn first_non_blank(buffer: &Buffer, pos: Position) -> Position {
    if let Some(line) = buffer.line(pos.line) {
        let col = line.find(|c: char| !c.is_whitespace())
            .unwrap_or(0);
        Position::new(pos.line, col)
    } else {
        pos
    }
}

/// Move to end of line (Vim's `$` motion)
pub fn line_end(buffer: &Buffer, pos: Position) -> Position {
    let line_len = buffer.line_len(pos.line);
    Position::new(pos.line, line_len.saturating_sub(1))
}

/// Move to start of file (Vim's `gg` motion)
pub fn file_start(_pos: Position) -> Position {
    Position::new(0, 0)
}

/// Move to end of file (Vim's `G` motion)
pub fn file_end(buffer: &Buffer, _pos: Position) -> Position {
    let last_line = buffer.line_count().saturating_sub(1);
    Position::new(last_line, 0)
}

/// Find matching pair character (parens, braces, brackets)
pub fn find_matching_pair(buffer: &Buffer, pos: Position) -> Option<Position> {
    let line = buffer.line(pos.line)?;
    let chars: Vec<char> = line.chars().collect();

    if pos.col >= chars.len() {
        return None;
    }

    let current = chars[pos.col];
    let (target, direction) = match current {
        '(' => (')', 1),
        '[' => (']', 1),
        '{' => ('}', 1),
        ')' => ('(', -1),
        ']' => ('[', -1),
        '}' => ('{', -1),
        _ => return None,
    };

    let mut depth = 0;
    let mut line = pos.line as isize;
    let mut col = pos.col as isize;

    loop {
        let current_line = buffer.line(line as usize)?;
        let current_chars: Vec<char> = current_line.chars().collect();

        while if direction > 0 { col < current_chars.len() as isize } else { col >= 0 } {
            let c = current_chars[col as usize];
            if c == current {
                depth += 1;
            } else if c == target {
                depth -= 1;
                if depth == 0 {
                    return Some(Position::new(line as usize, col as usize));
                }
            }
            col += direction;
        }

        // Move to next/prev line
        line += direction;
        if line < 0 || line >= buffer.line_count() as isize {
            return None;
        }

        col = if direction > 0 { 0 } else { buffer.line_len(line as usize) as isize - 1 };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_new() {
        let pos = Position::new(5, 10);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.col, 10);
    }

    #[test]
    fn test_range_normalized() {
        let range1 = Range::new(Position::new(5, 10), Position::new(2, 3));
        let norm1 = range1.normalized();
        assert_eq!(norm1.start, Position::new(2, 3));
        assert_eq!(norm1.end, Position::new(5, 10));

        let range2 = Range::new(Position::new(2, 3), Position::new(5, 10));
        let norm2 = range2.normalized();
        assert_eq!(norm2.start, Position::new(2, 3));
        assert_eq!(norm2.end, Position::new(5, 10));
    }

    #[test]
    fn test_line_start() {
        let pos = Position::new(5, 10);
        let result = line_start(pos);
        assert_eq!(result.line, 5);
        assert_eq!(result.col, 0);
    }

    #[test]
    fn test_file_start() {
        let pos = Position::new(100, 50);
        let result = file_start(pos);
        assert_eq!(result.line, 0);
        assert_eq!(result.col, 0);
    }

    #[test]
    fn test_char_kind() {
        assert_eq!(char_kind('a'), WordKind::Alphanumeric);
        assert_eq!(char_kind('Z'), WordKind::Alphanumeric);
        assert_eq!(char_kind('0'), WordKind::Alphanumeric);
        assert_eq!(char_kind('_'), WordKind::Alphanumeric);
        assert_eq!(char_kind(' '), WordKind::Whitespace);
        assert_eq!(char_kind('\t'), WordKind::Whitespace);
        assert_eq!(char_kind('.'), WordKind::Other);
        assert_eq!(char_kind('('), WordKind::Other);
    }
}
