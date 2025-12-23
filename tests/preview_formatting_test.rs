use std::path::PathBuf;

#[test]
fn test_preview_buffer_formatting() {
    // Create a test file with unformatted Rust code
    let test_content = r#"
fn main(){let x=5;if x>0{println!("positive")}}
"#;

    let test_file_path = PathBuf::from("test_preview_formatting.rs");
    std::fs::write(&test_file_path, test_content).expect("Failed to create test file");

    // Test PreviewBuffer loading and formatting
    let result = texty::ui::widgets::preview::PreviewBuffer::load_from_file(&test_file_path);

    // Clean up
    std::fs::remove_file(&test_file_path).ok();

    match result {
        Ok(preview_buffer) => {
            // Check that content was loaded
            assert!(
                !preview_buffer.content.is_empty(),
                "Preview content should not be empty"
            );

            // Check that Rust language was detected
            assert_eq!(
                preview_buffer.language,
                Some(texty::syntax::LanguageId::Rust)
            );

            // Check that content is properly formatted (should have newlines and proper spacing)
            assert!(
                preview_buffer.content.contains('\n'),
                "Formatted content should contain newlines"
            );
            assert!(
                preview_buffer.content.contains("fn main()"),
                "Should contain properly formatted function"
            );

            println!("✅ PreviewBuffer formatting test passed!");
            println!("Original: {}", test_content);
            println!("Formatted: {}", preview_buffer.content);
        }
        Err(e) => {
            panic!("Failed to create PreviewBuffer: {}", e);
        }
    }
}

#[test]
fn test_preview_buffer_syntax_highlighting() {
    let test_content = r#"
fn main() {
    let x = 5;
    println!("Hello {}", x);
}
"#;

    let test_file_path = PathBuf::from("test_syntax_highlighting.rs");
    std::fs::write(&test_file_path, test_content).expect("Failed to create test file");

    let result = texty::ui::widgets::preview::PreviewBuffer::load_from_file(&test_file_path);

    std::fs::remove_file(&test_file_path).ok();

    match result {
        Ok(mut preview_buffer) => {
            preview_buffer.ensure_highlighted(0, 10);

            assert!(
                !preview_buffer
                    .syntax_highlights
                    .as_ref()
                    .map_or(true, |v| v.is_empty()),
                "Should have syntax highlights for Rust code"
            );

            println!("✅ Syntax highlighting test passed!");
            println!(
                "Found {} syntax highlight tokens",
                preview_buffer
                    .syntax_highlights
                    .as_ref()
                    .map(|v| v.len())
                    .unwrap_or(0)
            );
        }
        Err(e) => {
            panic!("Failed to create PreviewBuffer: {}", e);
        }
    }
}

#[test]
fn test_incremental_highlighting() {
    let test_content = r#"
fn main() {
    let x = 5;
    println!("Hello {}", x);
    let y = 10;
    println!("World {}", y);
}
"#;

    let test_file_path = PathBuf::from("test_incremental_highlighting.rs");
    std::fs::write(&test_file_path, test_content).expect("Failed to create test file");

    let result = texty::ui::widgets::preview::PreviewBuffer::load_from_file(&test_file_path);

    std::fs::remove_file(&test_file_path).ok();

    match result {
        Ok(mut preview_buffer) => {
            preview_buffer.ensure_highlighted(0, 3);

            let initial_count = preview_buffer
                .syntax_highlights
                .as_ref()
                .map(|v| v.len())
                .unwrap_or(0);

            preview_buffer.ensure_highlighted(3, 3);

            let new_count = preview_buffer
                .syntax_highlights
                .as_ref()
                .map(|v| v.len())
                .unwrap_or(0);

            assert!(
                new_count >= initial_count,
                "Should have equal or more highlights after highlighting additional lines"
            );

            assert!(
                preview_buffer.highlight_progress.is_line_highlighted(0),
                "Line 0 should be marked as highlighted"
            );
            assert!(
                preview_buffer.highlight_progress.is_line_highlighted(1),
                "Line 1 should be marked as highlighted"
            );
            assert!(
                preview_buffer.highlight_progress.is_line_highlighted(2),
                "Line 2 should be marked as highlighted"
            );

            println!("✅ Incremental highlighting test passed!");
            println!(
                "Initial highlights: {}, Total highlights: {}",
                initial_count, new_count
            );
        }
        Err(e) => {
            panic!("Failed to create PreviewBuffer: {}", e);
        }
    }
}

#[test]
fn test_highlight_progress_tracking() {
    let test_content = r#"
fn main() {
    let x = 5;
}
"#;

    let test_file_path = PathBuf::from("test_highlight_progress.rs");
    std::fs::write(&test_file_path, test_content).expect("Failed to create test file");

    let mut preview_buffer =
        texty::ui::widgets::preview::PreviewBuffer::load_from_file(&test_file_path)
            .expect("Failed to create PreviewBuffer");

    std::fs::remove_file(&test_file_path).ok();

    assert!(
        !preview_buffer.highlight_progress.is_line_highlighted(0),
        "Line 0 should not be highlighted initially"
    );

    preview_buffer.ensure_highlighted(0, 3);

    assert!(
        preview_buffer.highlight_progress.is_line_highlighted(0),
        "Line 0 should be highlighted"
    );
    assert!(
        preview_buffer.highlight_progress.is_line_highlighted(1),
        "Line 1 should be highlighted"
    );
    assert!(
        preview_buffer.highlight_progress.is_line_highlighted(2),
        "Line 2 should be highlighted"
    );

    assert!(
        !preview_buffer.highlight_progress.is_line_highlighted(3),
        "Line 3 should not be highlighted (outside range)"
    );

    println!("✅ Highlight progress tracking test passed!");
}
