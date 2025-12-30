use std::path::PathBuf;

pub fn get_config_dir() -> PathBuf {
    let config_home = dirs::config_dir().unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config")
    });
    config_home.join("texty")
}

pub fn find_config_file() -> Option<PathBuf> {
    let config_dir = get_config_dir();
    let paths = vec![
        config_dir.join("config.toml"),
        dirs::home_dir()?.join(".texty").join("config.toml"),
    ];

    paths.into_iter().find(|p| p.exists())
}

pub fn find_user_theme() -> Option<PathBuf> {
    let config_dir = get_config_dir();
    let paths = vec![
        config_dir.join("theme.toml"),
        dirs::home_dir()?.join(".texty").join("theme.toml"),
        PathBuf::from("./theme.toml"),
    ];

    paths.into_iter().find(|p| p.exists())
}

pub fn list_builtin_themes() -> Vec<String> {
    vec![
        "monokai".to_string(),
        "dracula".to_string(),
        "nord".to_string(),
        "gruvbox".to_string(),
        "solarized-dark".to_string(),
    ]
}
