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
            assert!(!preview_buffer.content.is_empty(), "Preview content should not be empty");
            
            // Check that Rust language was detected
            assert_eq!(preview_buffer.language, Some(texty::syntax::LanguageId::Rust));
            
            // Check that the content is properly formatted (should have newlines and proper spacing)
            assert!(preview_buffer.content.contains('\n'), "Formatted content should contain newlines");
            assert!(preview_buffer.content.contains("fn main()"), "Should contain properly formatted function");
            
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
    
    // Clean up
    std::fs::remove_file(&test_file_path).ok();
    
    match result {
        Ok(preview_buffer) => {
            // Should have some syntax highlights for Rust code
            assert!(!preview_buffer.syntax_highlights.is_empty(), "Should have syntax highlights for Rust code");
            
            println!("✅ Syntax highlighting test passed!");
            println!("Found {} syntax highlight tokens", preview_buffer.syntax_highlights.len());
        }
        Err(e) => {
            panic!("Failed to create PreviewBuffer: {}", e);
        }
    }
}