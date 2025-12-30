// Common test utilities for overflow and bounds testing

use texty::buffer::Buffer;
use texty::editor::Editor;
use texty::motion::Position;
use texty::command::Command;

/// Utilities for creating test buffers with specific characteristics
pub mod boundary {
    use super::*;

    /// Create a buffer with n lines of varying lengths
    /// Line i has (i % 10) + 1 characters
    #[allow(dead_code)]
    pub fn create_multiline_buffer(line_count: usize) -> Buffer {
        let mut buffer = Buffer::new();
        for i in 0..line_count {
            if i > 0 {
                buffer.insert_char('\n', 0, 0).unwrap();
            }
            let line_content = "a".repeat(i % 10 + 1);
            buffer.insert_text(&line_content, 0, 0).unwrap();
        }
        buffer
    }

    /// Create editor with specific buffer state
    #[allow(dead_code)]
    pub fn create_editor_with_text(text: &str) -> Editor {
        let mut editor = Editor::new();
        editor.buffer.insert_text(text, 0, 0).unwrap();
        editor
    }

    /// Get all boundary positions for a buffer
    /// Returns positions at: start, end of each line, and corners
    #[allow(dead_code)]
    pub fn get_boundary_positions(buffer: &Buffer) -> Vec<(usize, usize)> {
        let mut positions = Vec::new();

        // Add (0, 0)
        positions.push((0, 0));

        // Add end of each line
        for line in 0..buffer.line_count() {
            let line_len = buffer.line_len(line);
            positions.push((line, 0)); // Start of line
            if line_len > 0 {
                positions.push((line, line_len - 1)); // Last char
            }
            positions.push((line, line_len)); // One past end (often used)
        }

        positions
    }

    /// Create a buffer that has been tested to have specific overflow risk points
    #[allow(dead_code)]
    pub fn create_overflow_risk_buffer() -> Buffer {
        let mut buffer = Buffer::new();
        // Create buffer that tests various edge cases
        buffer.insert_text("a\nb\nc", 0, 0).unwrap();
        buffer
    }
}

/// Validation utilities for checking invariants
pub mod validation {
    use super::*;

    /// Check if a position is valid for the given buffer
    #[allow(dead_code)]
    pub fn is_valid_position(buffer: &Buffer, pos: Position) -> bool {
        if pos.line >= buffer.line_count() {
            return false;
        }
        // Allow column to be at line_len (one past end is valid for insertions)
        pos.col <= buffer.line_len(pos.line)
    }

    /// Assert cursor is in valid position
    pub fn assert_cursor_valid(editor: &Editor) {
        assert!(
            editor.cursor.line < editor.buffer.line_count(),
            "Cursor line {} out of bounds (line_count: {})",
            editor.cursor.line,
            editor.buffer.line_count()
        );
        assert!(
            editor.cursor.col <= editor.buffer.line_len(editor.cursor.line),
            "Cursor col {} out of bounds for line {} (line_len: {})",
            editor.cursor.col,
            editor.cursor.line,
            editor.buffer.line_len(editor.cursor.line)
        );
    }

    /// Assert buffer invariants (line count >= 1, etc.)
    #[allow(dead_code)]
    pub fn assert_buffer_invariants(buffer: &Buffer) {
        assert!(buffer.line_count() >= 1, "Buffer must have at least 1 line");
        // All lines should be accessible
        for line in 0..buffer.line_count() {
            let _ = buffer.line(line).expect("All lines should be accessible");
        }
    }
}

/// Utilities for testing overflow scenarios
pub mod overflow {
    /// Test that an operation doesn't panic with extreme values
    #[allow(dead_code)]
    pub fn test_no_panic_with_extremes<F>(mut operation: F) -> bool
    where
        F: FnMut(),
    {
        // Run the operation - if it panics, the test will fail
        operation();
        true
    }

    /// Create scenarios that could cause arithmetic overflow
    #[allow(dead_code)]
    pub const EXTREME_COUNTS: &[usize] = &[0, 1, 2, 100, usize::MAX - 1, usize::MAX];

    /// Get all extreme count values for testing
    #[allow(dead_code)]
    pub fn get_extreme_counts() -> Vec<usize> {
        EXTREME_COUNTS.to_vec()
    }
}

/// Utilities for rapid operation testing
pub mod stress {
    use super::*;

    /// Perform many random operations and ensure no panics
    #[allow(dead_code)]
    pub fn rapid_operations(editor: &mut Editor, operations: &[Command], iterations: usize) {
        for _ in 0..iterations {
            for cmd in operations {
                editor.execute_command(cmd.clone());
                validation::assert_cursor_valid(editor);
            }
        }
    }

    /// Test all movement commands in sequence
    #[allow(dead_code)]
    pub fn test_all_movements(editor: &mut Editor) {
        let movements = vec![
            Command::MoveLeft,
            Command::MoveRight,
            Command::MoveUp,
            Command::MoveDown,
            Command::MoveWordForward(1),
            Command::MoveWordBackward(1),
            Command::MoveLineStart,
            Command::MoveLineEnd(1),
            Command::MoveFileStart,
            Command::MoveFileEnd,
        ];

        for cmd in movements {
            editor.execute_command(cmd);
            validation::assert_cursor_valid(editor);
        }
    }
}
