// src/formatter/external.rs - External formatter integration

use crate::syntax::LanguageId;
use std::io::Write;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct FormatterConfig {
    pub command: String,
    pub args: Vec<String>,
    pub stdin_mode: bool,
}

pub struct Formatter {
    config: FormatterConfig,
}

impl Formatter {
    pub fn new(config: FormatterConfig) -> Result<Self, std::io::Error> {
        // Validate formatter is available
        let output = Command::new(&config.command).arg("--version").output()?;
        if !output.status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Formatter {} not found", config.command),
            ));
        }
        Ok(Self { config })
    }

    pub fn format_text(&self, text: &str) -> Result<String, std::io::Error> {
        let mut child = Command::new(&self.config.command)
            .args(&self.config.args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(std::io::Error::other(String::from_utf8_lossy(
                &output.stderr,
            )))
        }
    }
}

pub fn get_formatter_config(language: LanguageId) -> Option<FormatterConfig> {
    match language {
        LanguageId::Rust => Some(FormatterConfig {
            command: "rustfmt".to_string(),
            args: vec!["--emit".to_string(), "stdout".to_string()],
            stdin_mode: true,
        }),
        LanguageId::Python => Some(FormatterConfig {
            command: "black".to_string(),
            args: vec!["-".to_string()],
            stdin_mode: true,
        }),
        LanguageId::JavaScript | LanguageId::TypeScript => Some(FormatterConfig {
            command: "prettier".to_string(),
            args: vec!["--stdin-filepath".to_string(), "file.js".to_string()], // TODO: pass actual filepath
            stdin_mode: true,
        }),
    }
}
