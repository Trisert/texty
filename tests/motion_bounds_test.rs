// Motion operation boundary tests
// Tests for P2 motion boundary issues in motion.rs

mod common;
use common::validation;

use texty::buffer::Buffer;
use texty::motion::{Position, word_forward, word_backward, word_end, file_end, file_start};

/// MOT-001: Test word_forward on empty buffer
/// Risk: motion.rs:121 has `line_count - 1` underflow
#[test]
fn test_word_forward_empty_buffer() {
    let buffer = Buffer::new();
    let pos = Position::new(0, 0);

    // Line 121: line_count - 1 could underflow if empty
    let result = word_forward(&buffer, pos);

    assert_eq!(result.line, 0);
    assert_eq!(result.col, 0);
    validation::assert_buffer_invariants(&buffer);
}

/// Test word_forward on single line
#[test]
fn test_word_forward_single_line() {
    let mut buffer = Buffer::new();
    buffer.insert_text("hello world", 0, 0).unwrap();

    let pos = Position::new(0, 5);
    let result = word_forward(&buffer, pos);

    // Should handle end of buffer gracefully
    assert!(result.line <= buffer.line_count().saturating_sub(1));
    validation::assert_buffer_invariants(&buffer);
}

/// Test word_forward at end of buffer
#[test]
fn test_word_forward_at_end() {
    let mut buffer = Buffer::new();
    buffer.insert_text("test", 0, 0).unwrap();

    let pos = Position::new(0, 3); // At 't'
    let result = word_forward(&buffer, pos);

    // Should stay at safe position
    assert!(result.line < buffer.line_count());
    assert!(result.col <= buffer.line_len(result.line));
    validation::assert_buffer_invariants(&buffer);
}

/// Test word_backward on empty buffer
#[test]
fn test_word_backward_empty_buffer() {
    let buffer = Buffer::new();
    let pos = Position::new(0, 0);

    let result = word_backward(&buffer, pos);

    assert_eq!(result.line, 0);
    assert_eq!(result.col, 0);
    validation::assert_buffer_invariants(&buffer);
}

/// Test word_backward at start of buffer
#[test]
fn test_word_backward_at_start() {
    let mut buffer = Buffer::new();
    buffer.insert_text("hello world", 0, 0).unwrap();

    let pos = Position::new(0, 0);
    let result = word_backward(&buffer, pos);

    // Should not go beyond start
    assert_eq!(result.line, 0);
    assert_eq!(result.col, 0);
    validation::assert_buffer_invariants(&buffer);
}

/// Test word_end at boundary
/// Risk: motion.rs:134 uses saturating operations
#[test]
fn test_word_end_at_boundary() {
    let mut buffer = Buffer::new();
    buffer.insert_text("a", 0, 0).unwrap();

    // Line 134: col.min(line_len(line).saturating_sub(1))
    let pos = Position::new(0, 0);
    let result = word_end(&buffer, pos);

    assert!(result.line < buffer.line_count());
    assert!(result.col <= buffer.line_len(result.line));
    validation::assert_buffer_invariants(&buffer);
}

/// Test word_end on empty buffer
#[test]
fn test_word_end_empty_buffer() {
    let buffer = Buffer::new();
    let pos = Position::new(0, 0);

    let result = word_end(&buffer, pos);

    assert_eq!(result.line, 0);
    assert_eq!(result.col, 0);
    validation::assert_buffer_invariants(&buffer);
}

/// Test word_end on single character
#[test]
fn test_word_end_single_character() {
    let mut buffer = Buffer::new();
    buffer.insert_text("a", 0, 0).unwrap();

    let pos = Position::new(0, 0);
    let result = word_end(&buffer, pos);

    assert_eq!(result.line, 0);
    assert_eq!(result.col, 0);
    validation::assert_buffer_invariants(&buffer);
}

/// Test file_end on empty buffer
#[test]
fn test_file_end_empty_buffer() {
    let buffer = Buffer::new();
    let pos = Position::new(0, 0);

    // file_end uses line_count().saturating_sub(1)
    let result = file_end(&buffer, pos);

    assert_eq!(result.line, 0);
    assert_eq!(result.col, 0);
    validation::assert_buffer_invariants(&buffer);
}

/// Test file_end on single line buffer
#[test]
fn test_file_end_single_line() {
    let mut buffer = Buffer::new();
    buffer.insert_text("test", 0, 0).unwrap();

    let pos = Position::new(0, 2);
    let result = file_end(&buffer, pos);

    assert_eq!(result.line, 0);
    // file_end moves to column 0 of last line
    assert_eq!(result.col, 0);
    validation::assert_buffer_invariants(&buffer);
}

/// Test file_start always returns (0, 0)
#[test]
fn test_file_start_always_origin() {
    let _buffer = Buffer::new();
    let pos = Position::new(100, 50);

    let result = file_start(pos);

    assert_eq!(result.line, 0);
    assert_eq!(result.col, 0);
}

/// Test all motions on single character buffer
#[test]
fn test_all_motions_on_single_char() {
    let mut buffer = Buffer::new();
    buffer.insert_text("a", 0, 0).unwrap();

    let pos = Position::new(0, 0);

    // All motions should handle single character gracefully
    let result1 = word_forward(&buffer, pos);
    assert!(validation::is_valid_position(&buffer, result1));

    let result2 = word_backward(&buffer, pos);
    assert!(validation::is_valid_position(&buffer, result2));

    let result3 = word_end(&buffer, pos);
    assert!(validation::is_valid_position(&buffer, result3));

    let result4 = file_end(&buffer, pos);
    assert!(validation::is_valid_position(&buffer, result4));

    let result5 = file_start(pos);
    assert!(validation::is_valid_position(&buffer, result5));
}

/// Test word movements on multiline buffer
#[test]
fn test_word_movements_multiline() {
    let mut buffer = Buffer::new();
    buffer.insert_text("hello world\nfoo bar", 0, 0).unwrap();

    let pos = Position::new(0, 0);

    // Forward through buffer
    let pos1 = word_forward(&buffer, pos);
    assert!(validation::is_valid_position(&buffer, pos1));

    let pos2 = word_forward(&buffer, pos1);
    assert!(validation::is_valid_position(&buffer, pos2));

    // Backward from end
    let pos3 = word_backward(&buffer, pos2);
    assert!(validation::is_valid_position(&buffer, pos3));
}

/// Test motion at line boundaries
#[test]
fn test_motion_at_line_boundaries() {
    let mut buffer = Buffer::new();
    buffer.insert_text("line1\nline2\nline3", 0, 0).unwrap();

    // Test motion at line 1 end
    let pos = Position::new(0, 4);
    let result = word_forward(&buffer, pos);
    assert!(validation::is_valid_position(&buffer, result));

    // Test motion at line 2 start
    let pos = Position::new(1, 0);
    let result = word_backward(&buffer, pos);
    assert!(validation::is_valid_position(&buffer, result));
}

/// Test motions don't create invalid positions
#[test]
fn test_motions_never_invalid() {
    let mut buffer = Buffer::new();
    buffer.insert_text("test hello world foo bar", 0, 0).unwrap();

    // Test multiple word motions in sequence
    let mut pos = Position::new(0, 0);
    for _ in 0..10 {
        pos = word_forward(&buffer, pos);
        assert!(validation::is_valid_position(&buffer, pos));
    }

    // Now go back
    for _ in 0..10 {
        pos = word_backward(&buffer, pos);
        assert!(validation::is_valid_position(&buffer, pos));
    }
}

/// Test motion on buffer with varying line lengths
#[test]
fn test_motion_varying_line_lengths() {
    let mut buffer = Buffer::new();
    buffer.insert_text("a\nbbbbbbbb\ncc", 0, 0).unwrap();

    let pos = Position::new(1, 5);
    let result = word_forward(&buffer, pos);

    // Should handle transition from long to short line
    assert!(validation::is_valid_position(&buffer, result));
}
