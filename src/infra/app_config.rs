use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub custom_agent_command: Option<String>,
    #[serde(default)]
    pub custom_agent_args: Vec<String>,
    pub timeout_secs: u64,
    pub source_freshness_days: u32,
    pub disclaimer: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            custom_agent_command: None,
            custom_agent_args: Vec::new(),
            timeout_secs: 1800,
            source_freshness_days: 7,
            disclaimer: "Research only. Not investment advice.".to_string(),
        }
    }
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    let Ok(raw) = std::fs::read_to_string(path) else {
        return AppConfig::default();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create config parent {}", parent.display()))?;
    }
    std::fs::write(path, serde_json::to_string_pretty(config)?)?;
    Ok(())
}

fn config_path() -> PathBuf {
    if let Ok(path) = std::env::var("CRAZYLINES_CONFIG_PATH") {
        return PathBuf::from(path);
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = home::home_dir() {
            return home
                .join("Library")
                .join("Application Support")
                .join("CrazyLines")
                .join("config.json");
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return PathBuf::from(appdata)
                .join("CrazyLines")
                .join("config.json");
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
            return PathBuf::from(xdg).join("crazylines").join("config.json");
        }
        if let Some(home) = home::home_dir() {
            return home.join(".config").join("crazylines").join("config.json");
        }
    }

    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".crazylines")
        .join("config.json")
}
