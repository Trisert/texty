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

    /// Syntax theme to use (monokai, dracula, nord, gruvbox, solarized-dark, or path to custom theme.toml)
    #[arg(long, short = 'T', default_value_t = String::from("monokai"))]
    pub theme: String,

    /// List all available built-in themes and exit
    #[arg(long = "list-themes", short = 'L', action = clap::ArgAction::SetTrue)]
    pub list_themes: bool,
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
        assert_eq!(args.theme, "monokai");
    }

    /// Confirms that the `--theme` CLI option sets the `theme` field to the provided value.
    ///
    /// Parses a simulated command-line containing `--theme monokai` and asserts that the
    /// resulting `CliArgs.theme` equals `"monokai"`.
    ///
    /// # Examples
    ///
    /// ```
    /// let args = CliArgs::parse_from(&["texty", "--theme", "monokai"]);
    /// assert_eq!(args.theme, "monokai");
    /// ```
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
            theme: "monokai".to_string(),
            list_themes: false,
        };

        let dir_args = CliArgs {
            file: Some(dir_path.to_path_buf()),
            terminal_palette: false,
            theme: "monokai".to_string(),
            list_themes: false,
        };

        let nonexistent_args = CliArgs {
            file: Some(PathBuf::from("/nonexistent/path")),
            terminal_palette: false,
            theme: "monokai".to_string(),
            list_themes: false,
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
