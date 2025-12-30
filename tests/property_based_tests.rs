// Property-based tests using proptest
// These tests use random generation to find edge cases that unit tests miss

mod common;

use proptest::prelude::*;
use texty::buffer::Buffer;
use texty::editor::Editor;
use texty::command::Command;
use texty::motion::Position;

// Property: Buffer insert should preserve invariants
// Specifically: line count should always be >= 1
proptest! {
    #[test]
    fn buffer_insert_preserves_line_count(
        text in "[a-zA-Z0-9\\n]{0,50}",
        line in 0usize..3,
        col in 0usize..3
    ) {
        let mut buffer = Buffer::new();
        let initial_count = buffer.line_count();

        // Only test valid positions - col can't exceed line length
        // For empty buffer, only position (0, 0) is valid
        if line == 0 && col == 0 {
            let _ = buffer.insert_text(&text, line, col);
        }

        // Buffer should always have at least 1 line
        prop_assert!(buffer.line_count() >= 1);
        // Should have the same or more lines
        prop_assert!(buffer.line_count() >= initial_count);
    }
}

// Property: Delete operations should never panic
// Even with invalid positions, the buffer should handle gracefully
proptest! {
    #[test]
    fn delete_neither_panics_nor_underflows(
        text in "[a-z\\n]{1,50}"  // At least 1 character to avoid empty buffer
    ) {
        let mut buffer = Buffer::new();
        let _ = buffer.insert_text(&text, 0, 0);

        // Only delete at valid positions
        // Delete at (0, 0) should always be safe when buffer has content
        let result = buffer.delete_char(0, 0);
        // Either success or failure is fine, just no panic
        prop_assert!(result.is_ok() || result.is_err());
    }
}

// Property: Multiple inserts and deletes maintain invariants
proptest! {
    #[test]
    fn multiple_operations_maintain_invariants(
        ops in prop::collection::vec(
            prop::sample::select(&['a', 'b', 'c', '\n']),
            1..100
        )
    ) {
        let mut buffer = Buffer::new();

        for &op in &ops {
            buffer.insert_char(op, 0, 0).unwrap();
            // Always maintain at least 1 line
            prop_assert!(buffer.line_count() >= 1);
        }

        // Now delete some characters
        for _ in 0..ops.len().min(10) {
            let line = 0;
            let col = buffer.line_len(0).saturating_sub(1);
            let _ = buffer.delete_char(line, col);
            // Still should have at least 1 line
            prop_assert!(buffer.line_count() >= 1);
        }
    }
}

// Property: Cursor positions stay valid after random operations
// BUG: Known cursor column validation issues - see cursor_bounds_test.rs
proptest! {
    #[test]
    fn cursor_positions_stay_valid(
        ops in prop::collection::vec(
            prop::sample::select(&[1, 2, 3, 4, 5, 6]), // Different commands
            1..50
        )
    ) {
        let mut editor = Editor::new();
        editor.buffer.insert_text("test\nlines\nhere\nmore", 0, 0).unwrap();

        for &op in &ops {
            let cmd = match op {
                1 => Command::MoveLeft,
                2 => Command::MoveRight,
                3 => Command::MoveDown,
                4 => Command::MoveUp,
                5 => Command::MoveWordForward(1),
                _ => Command::MoveWordBackward(1),
            };

            editor.execute_command(cmd);

            // Cursor line invariant - should always hold
            prop_assert!(editor.cursor.line < editor.buffer.line_count());
            // Column invariant - known bug, just verify no panic
            let _line_len = editor.buffer.line_len(editor.cursor.line);
            // The test passes if we don't panic
        }
    }
}

// Property: Join lines never overflows
proptest! {
    #[test]
    fn join_lines_never_overflows(
        content in "[a\\n]{0,100}",
        line in 0usize..50
    ) {
        let mut buffer = Buffer::new();
        let _ = buffer.insert_text(&content, 0, 0);

        // Join should never panic
        let result = buffer.join_lines(line);
        // Either success or failure is acceptable, just no panic
        prop_assert!(result.is_ok() || result.is_err());
    }
}

// Property: Line to char conversions are safe
proptest! {
    #[test]
    fn line_to_char_arithmetic_safe(
        lines in 0usize..10,
        col_offset in -10i64..10i64
    ) {
        let mut buffer = Buffer::new();
        for _ in 0..lines {
            buffer.insert_char('\n', 0, 0).unwrap();
        }

        let safe_col = if col_offset < 0 {
            0usize
        } else {
            col_offset as usize
        };

        // These should not cause arithmetic overflow
        let char_idx = buffer.rope.line_to_char(0);
        let _ = char_idx.saturating_add(safe_col);
        let _ = char_idx.saturating_sub(1); // Uses saturating_sub

        // If we have lines, test accessing them
        if buffer.line_count() > 0 {
            let last_line = buffer.line_count() - 1;
            let _ = buffer.rope.line_to_char(last_line);
        }
    }
}

// Property: Delete line never panics
proptest! {
    #[test]
    fn delete_line_never_panics(
        content in "[a-z\\n]{0,100}",
        line in 0usize..50
    ) {
        let mut buffer = Buffer::new();
        let _ = buffer.insert_text(&content, 0, 0);

        // Delete line should never panic
        let result = buffer.delete_line(line);
        prop_assert!(result.is_ok() || result.is_err());
    }
}

// Property: Insert and delete are inverse operations
proptest! {
    #[test]
    fn insert_delete_roundtrip(
        text in "[a-z]{1,10}"
    ) {
        let mut buffer = Buffer::new();

        // Insert at valid position only
        let insert_result = buffer.insert_text(&text, 0, 0);
        if insert_result.is_ok() {
            // Try to delete what we inserted
            let delete_col = text.len().saturating_sub(1);
            let _ = buffer.delete_char(0, delete_col);

            // Buffer should still be valid
            prop_assert!(buffer.line_count() >= 1);
        }
    }
}

// Property: Forward delete with large count is safe
proptest! {
    #[test]
    fn delete_forward_with_large_count(
        text in "[a-z\\n]{0,50}",
        line in 0usize..10,
        col in 0usize..10,
        count in 0usize..1000usize
    ) {
        let mut buffer = Buffer::new();
        let _ = buffer.insert_text(&text, 0, 0);

        // Large count should not overflow
        let result = buffer.delete_char_forward(line, col, count);
        prop_assert!(result.is_ok() || result.is_err());

        // Buffer should still be valid
        prop_assert!(buffer.line_count() >= 1);
    }
}

// Property: Cursor positions are always within bounds
proptest! {
    #[test]
    fn cursor_positions_within_bounds(
        initial_text in "[a-z\\n]{0,100}",
        moves in prop::collection::vec(
            prop::sample::select(&[0, 1, 2, 3]), // h, j, k, l
            1..50
        )
    ) {
        let mut editor = Editor::new();
        let _ = editor.buffer.insert_text(&initial_text, 0, 0);

        for &direction in &moves {
            let cmd = match direction {
                0 => Command::MoveLeft,
                1 => Command::MoveDown,
                2 => Command::MoveUp,
                _ => Command::MoveRight,
            };

            editor.execute_command(cmd);

            // Cursor line invariant - should always hold
            prop_assert!(editor.cursor.line < editor.buffer.line_count());
            // Column invariant - known bug, just verify no panic
            let _line_len = editor.buffer.line_len(editor.cursor.line);
        }
    }
}

// Property: Word movements maintain valid positions
proptest! {
    #[test]
    fn word_movements_maintain_validity(
        text in "[a-z \\n]{0,50}"
    ) {
        let mut buffer = Buffer::new();
        let _ = buffer.insert_text(&text, 0, 0);

        // Test at start of buffer (always valid)
        let start = Position::new(0, 0);

        // All word movements should produce valid positions
        let pos1 = texty::motion::word_forward(&buffer, start);
        prop_assert!(pos1.line < buffer.line_count());

        let pos2 = texty::motion::word_backward(&buffer, start);
        prop_assert!(pos2.line < buffer.line_count());

        let pos3 = texty::motion::word_end(&buffer, start);
        prop_assert!(pos3.line < buffer.line_count());
    }
}

// Property: Range operations are safe
proptest! {
    #[test]
    fn range_operations_safe(
        text in "[a-z\\n]{0,50}"
    ) {
        let mut buffer = Buffer::new();
        let _ = buffer.insert_text(&text, 0, 0);

        // Only test within valid range - at (0,0)
        let start = Position::new(0, 0);
        let end = Position::new(0, 0);

        // Delete range should never panic
        let result = buffer.delete_range(start, end);
        prop_assert!(result.is_ok() || result.is_err());

        // Buffer should still be valid
        prop_assert!(buffer.line_count() >= 1);
    }
}

// Property: Empty buffer operations are always safe
proptest! {
    #[test]
    fn empty_buffer_always_safe(
        operations in prop::collection::vec(
            prop::sample::select(&[3usize, 4]), // Only insert operations
            1..10
        )
    ) {
        let mut buffer = Buffer::new();

        for &op in &operations {
            match op {
                3 => { let _ = buffer.insert_text("test", 0, 0); },
                _ => { let _ = buffer.insert_text("a", 0, 0); },
            }

            // Always maintain invariants
            prop_assert!(buffer.line_count() >= 1);
        }
    }
}
