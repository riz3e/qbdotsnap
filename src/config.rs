use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct GitConfig {
    pub remote: String,
    #[serde(default = "default_branch")]
    pub branch: String,
    #[serde(default)]
    pub auto_push: bool,
}

fn default_branch() -> String { "main".into() }

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub track: Vec<String>,
    #[serde(default)]
    pub skip_patterns: Vec<String>,
    pub snapshot_dir: Option<String>,
    #[serde(default = "default_true")]
    pub notifications: bool,
    pub git: Option<GitConfig>,
}

fn default_true() -> bool { true }

impl Config {
    pub fn snapshot_dir(&self) -> PathBuf {
        match &self.snapshot_dir {
            Some(p) => expand_tilde(p),
            None => home_dir().join(".qbdotsnap"),
        }
    }

    pub fn tracked_paths(&self) -> Vec<PathBuf> {
        self.track.iter().map(|p| expand_tilde(p)).collect()
    }

    pub fn should_skip(&self, path: &std::path::Path) -> bool {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        self.skip_patterns.iter().any(|pat| {
            if pat.starts_with('*') { name.ends_with(&pat[1..]) } else { name == pat }
        })
    }
}

pub fn load() -> Result<Config> {
    let config_path = home_dir().join(".qbdotsnap.toml");

    if !config_path.exists() {
        let default = Config {
            track: vec![
                "~/.zshrc".into(),
                "~/.bashrc".into(),
                "~/.gitconfig".into(),
                "~/.config/hypr".into(),
                "~/.config/quickshell".into(),
                "~/.config/ags".into(),
                "~/.config/nvim".into(),
                "~/.config/wofi".into(),
                "~/.config/gtk-3.0".into(),
            ],
            skip_patterns: vec![
                "*.log".into(), "*.sock".into(), "*.pid".into(),
                ".cache".into(), "hyprland.log".into(), "hyprpaper.log".into(),
            ],
            snapshot_dir: None,
            notifications: true,
            git: None,
        };
        let toml_str = toml::to_string_pretty(&default)?;
        std::fs::write(&config_path, toml_str)?;
        eprintln!("Created default config at {}", config_path.display());
        return Ok(default);
    }

    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Could not read {}", config_path.display()))?;
    toml::from_str(&content)
        .with_context(|| format!("Invalid TOML in {}", config_path.display()))
}

pub fn save(cfg: &Config) -> Result<()> {
    let config_path = home_dir().join(".qbdotsnap.toml");
    let toml_str = toml::to_string_pretty(cfg)?;
    std::fs::write(config_path, toml_str)?;
    Ok(())
}

pub fn home_dir() -> PathBuf {
    std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/root"))
}

pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        home_dir().join(rest)
    } else if path == "~" {
        home_dir()
    } else {
        PathBuf::from(path)
    }
}