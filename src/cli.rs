use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Default, Parser)]
#[command(name = "texty")]
#[command(version = "0.1.0")]
#[command(about = "A terminal text editor with LSP support")]
pub struct CliArgs {
    /// File or directory to open
    pub file: Option<PathBuf>,

    /// Use terminal color palette instead of theme colors
    #[arg(long, short = 't')]
    pub terminal_palette: bool,

    /// Syntax theme to use (default or monokai)
    #[arg(long, short = 'T', default_value_t = String::from("default"))]
    pub theme: String,
}

impl CliArgs {
    /// Check if the provided path is a directory (following symlinks)
    pub fn is_directory(&self) -> bool {
        if let Some(path) = &self.file {
            std::fs::metadata(path).map(|m| m.is_dir()).unwrap_or(false)
        } else {
            false
        }
    }

    /// Check if the provided path exists (following symlinks)
    pub fn exists(&self) -> bool {
        if let Some(path) = &self.file {
            std::fs::metadata(path).is_ok()
        } else {
            false
        }
    }
}

pub fn parse_args() -> Result<CliArgs, Box<dyn std::error::Error>> {
    Ok(CliArgs::parse())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_default_cli_args() {
        let args = CliArgs::default();
        assert!(args.file.is_none());
        assert!(!args.is_directory());
        assert!(!args.exists());
    }

    #[test]
    fn test_parse_no_args() {
        let args = CliArgs::parse_from(&["texty"]);
        assert!(args.file.is_none());
        assert_eq!(args.theme, "default");
    }

    #[test]
    fn test_parse_with_theme() {
        let args = CliArgs::parse_from(&["texty", "--theme", "monokai"]);
        assert_eq!(args.theme, "monokai");
    }

    #[test]
    fn test_directory_detection() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        let file_path = dir_path.join("test_file.txt");
        fs::write(&file_path, "test content").unwrap();

        let file_args = CliArgs {
            file: Some(file_path.clone()),
            terminal_palette: false,
            theme: "default".to_string(),
        };

        let dir_args = CliArgs {
            file: Some(dir_path.to_path_buf()),
            terminal_palette: false,
            theme: "default".to_string(),
        };

        let nonexistent_args = CliArgs {
            file: Some(PathBuf::from("/nonexistent/path")),
            terminal_palette: false,
            theme: "default".to_string(),
        };

        assert!(file_args.exists());
        assert!(!file_args.is_directory());
        assert!(dir_args.exists());
        assert!(dir_args.is_directory());
        assert!(!nonexistent_args.exists());
        assert!(!nonexistent_args.is_directory());
    }

    #[test]
    fn test_none_path() {
        let args = CliArgs::default();
        assert!(!args.exists());
        assert!(!args.is_directory());
    }
}
