use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
pub struct TextyConfig {
    pub theme: Option<String>,
}

impl TextyConfig {
    pub fn from_file(path: &PathBuf) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: TextyConfig =
            toml::from_str(&content).map_err(|e| format!("Invalid config format: {}", e))?;

        Ok(config)
    }
}
