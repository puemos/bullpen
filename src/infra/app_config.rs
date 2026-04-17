use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub custom_agent_command: Option<String>,
    #[serde(default)]
    pub custom_agent_args: Vec<String>,
    pub timeout_secs: u64,
    pub source_freshness_days: u32,
    pub disclaimer: String,
    #[serde(default)]
    pub model_by_agent: BTreeMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            custom_agent_command: None,
            custom_agent_args: Vec::new(),
            timeout_secs: 1800,
            source_freshness_days: 7,
            disclaimer: "Research only. Not investment advice.".to_string(),
            model_by_agent: BTreeMap::new(),
        }
    }
}

#[must_use]
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
    if let Ok(path) = std::env::var("BULLPEN_CONFIG_PATH") {
        return PathBuf::from(path);
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = home::home_dir() {
            return home
                .join("Library")
                .join("Application Support")
                .join("Bullpen")
                .join("config.json");
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return PathBuf::from(appdata).join("Bullpen").join("config.json");
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
            return PathBuf::from(xdg).join("bullpen").join("config.json");
        }
        if let Some(home) = home::home_dir() {
            return home.join(".config").join("bullpen").join("config.json");
        }
    }

    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".bullpen")
        .join("config.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // BULLPEN_CONFIG_PATH is process-global; serialize tests that touch it.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
        previous: Option<String>,
    }

    impl EnvGuard {
        fn set(path: &std::path::Path) -> Self {
            let lock = ENV_LOCK
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let previous = std::env::var("BULLPEN_CONFIG_PATH").ok();
            unsafe {
                std::env::set_var("BULLPEN_CONFIG_PATH", path);
            }
            Self {
                _lock: lock,
                previous,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            unsafe {
                match &self.previous {
                    Some(value) => std::env::set_var("BULLPEN_CONFIG_PATH", value),
                    None => std::env::remove_var("BULLPEN_CONFIG_PATH"),
                }
            }
        }
    }

    #[test]
    fn load_returns_default_when_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("absent.json");
        let _guard = EnvGuard::set(&path);
        assert_eq!(load_config(), AppConfig::default());
    }

    #[test]
    fn load_returns_default_when_file_is_malformed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, "{ not json").unwrap();
        let _guard = EnvGuard::set(&path);
        assert_eq!(load_config(), AppConfig::default());
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let _guard = EnvGuard::set(&path);

        let mut model_by_agent = BTreeMap::new();
        model_by_agent.insert("codex".into(), "gpt-5-codex".into());
        model_by_agent.insert("claude".into(), "claude-opus-4-7".into());
        let config = AppConfig {
            custom_agent_command: Some("/usr/local/bin/codex".to_string()),
            custom_agent_args: vec!["--mode".into(), "acp".into()],
            timeout_secs: 600,
            source_freshness_days: 14,
            disclaimer: "Custom disclaimer.".into(),
            model_by_agent,
        };

        save_config(&config).unwrap();
        let loaded = load_config();
        assert_eq!(loaded, config);
    }

    #[test]
    fn save_creates_missing_parent_directory() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a").join("b").join("config.json");
        let _guard = EnvGuard::set(&nested);

        save_config(&AppConfig::default()).unwrap();
        assert!(nested.exists());
    }
}
