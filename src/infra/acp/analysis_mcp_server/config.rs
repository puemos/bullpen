use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunContext {
    pub analysis_id: String,
    pub run_id: String,
    pub agent_id: String,
    pub user_prompt: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct ServerConfig {
    pub run_context: Option<PathBuf>,
    pub db_path: Option<PathBuf>,
}

impl ServerConfig {
    pub fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        Self::from_args_iter(args)
    }

    pub fn from_args_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        let mut config = ServerConfig::default();
        if let Ok(path) = std::env::var("BULLPEN_ANALYSIS_CONTEXT") {
            config.run_context = Some(PathBuf::from(path));
        }
        if let Ok(path) = std::env::var("BULLPEN_DB_PATH") {
            config.db_path = Some(PathBuf::from(path));
        }

        let args: Vec<String> = iter.into_iter().collect();
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--analysis-context" => {
                    if i + 1 < args.len() {
                        config.run_context = Some(PathBuf::from(&args[i + 1]));
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--db-path" => {
                    if i + 1 < args.len() {
                        config.db_path = Some(PathBuf::from(&args[i + 1]));
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                _ => i += 1,
            }
        }
        config
    }

    pub fn load_context(&self) -> anyhow::Result<RunContext> {
        let path = self
            .run_context
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing --analysis-context"))?;
        let raw = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&raw)?)
    }
}
