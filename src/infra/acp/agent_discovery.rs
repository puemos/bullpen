use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentModel {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCandidate {
    pub id: String,
    pub label: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub available: bool,
    #[serde(default)]
    pub models: Vec<AgentModel>,
    #[serde(default)]
    pub model_flag: Option<String>,
    #[serde(default)]
    pub model_value_prefix: Option<String>,
    #[serde(default)]
    pub model_env: Option<String>,
}

impl AgentCandidate {
    /// Build (flag, env) model override tuples from an optional user-selected
    /// model id. Either/both may be None — when None, the agent runs with its
    /// default model.
    #[allow(clippy::type_complexity)]
    #[must_use]
    pub fn resolve_model(
        &self,
        model_id: Option<&str>,
    ) -> (Option<(String, String)>, Option<(String, String)>) {
        let Some(model_id) = model_id.map(str::trim).filter(|m| !m.is_empty()) else {
            return (None, None);
        };

        if self.models.is_empty() || !self.models.iter().any(|m| m.id == model_id) {
            return (None, None);
        }

        let model_flag = self.model_flag.as_ref().and_then(|flag| {
            let flag = flag.trim();
            if flag.is_empty() {
                return None;
            }
            let value = match &self.model_value_prefix {
                Some(prefix) => format!("{prefix}{model_id}"),
                None => model_id.to_string(),
            };
            Some((flag.to_string(), value))
        });
        let model_env = self.model_env.as_ref().and_then(|env| {
            let env = env.trim();
            if env.is_empty() {
                return None;
            }
            Some((env.to_string(), model_id.to_string()))
        });
        (model_flag, model_env)
    }
}

fn model(id: &str, name: &str) -> AgentModel {
    AgentModel {
        id: id.to_string(),
        name: name.to_string(),
    }
}

#[must_use]
pub fn list_agent_candidates() -> Vec<AgentCandidate> {
    let config = crate::infra::app_config::load_config();
    let mut agents = vec![
        codex_candidate(),
        {
            let mut c = npx_candidate(
                "claude",
                "Claude",
                "CLAUDE_ACP_BIN",
                "@zed-industries/claude-code-acp",
            );
            c.model_flag = Some("--model".into());
            c.models = vec![
                model("sonnet", "Sonnet"),
                model("opus", "Opus"),
                model("haiku", "Haiku"),
                model("sonnet[1m]", "Sonnet (1M)"),
                model("opusplan", "Opus Plan"),
            ];
            c
        },
        {
            let mut c = command_candidate(
                "gemini",
                "Gemini",
                "GEMINI_ACP_BIN",
                "gemini",
                &["--experimental-acp"],
            );
            c.model_flag = Some("--model".into());
            c.models = vec![
                model("gemini-3-pro-preview", "Gemini 3 Pro (Preview)"),
                model("gemini-3-flash-preview", "Gemini 3 Flash (Preview)"),
                model("gemini-2.5-pro", "Gemini 2.5 Pro"),
                model("gemini-2.5-flash", "Gemini 2.5 Flash"),
                model("gemini-2.5-flash-lite", "Gemini 2.5 Flash Lite"),
            ];
            c
        },
        {
            let mut c = command_candidate(
                "qwen",
                "Qwen Code",
                "QWEN_ACP_BIN",
                "qwen",
                &["--experimental-acp"],
            );
            c.model_flag = Some("--model".into());
            c.models = vec![
                model("qwen3-coder-plus", "Qwen3 Coder Plus"),
                model("qwen3-coder-flash", "Qwen3 Coder Flash"),
                model("qwen3.5-plus", "Qwen3.5 Plus"),
                model("qwen3-max", "Qwen3 Max"),
            ];
            c
        },
        command_candidate(
            "mistral",
            "Mistral",
            "MISTRAL_ACP_BIN",
            "mistral",
            &["--experimental-acp"],
        ),
        {
            let mut c = command_candidate(
                "kimi",
                "Kimi",
                "KIMI_ACP_BIN",
                "kimi",
                &["--experimental-acp"],
            );
            c.model_flag = Some("--model".into());
            c.models = vec![
                model("kimi-for-coding", "Kimi for Coding"),
                model("kimi-k2.5", "Kimi K2.5"),
                model("kimi-k2-thinking", "Kimi K2 Thinking"),
                model("kimi-k2-thinking-turbo", "Kimi K2 Thinking Turbo"),
            ];
            c
        },
        command_candidate(
            "opencode",
            "OpenCode",
            "OPENCODE_ACP_BIN",
            "opencode",
            &["acp"],
        ),
    ];

    let custom_command = config
        .custom_agent_command
        .filter(|value| !value.trim().is_empty())
        .or_else(|| std::env::var("BULLPEN_CUSTOM_AGENT").ok());

    if let Some(custom) = custom_command {
        let args = if config.custom_agent_args.is_empty() {
            std::env::var("BULLPEN_CUSTOM_AGENT_ARGS")
                .map(|raw| raw.split_whitespace().map(str::to_string).collect())
                .unwrap_or_default()
        } else {
            config.custom_agent_args
        };
        let resolved = crate::infra::shell::find_bin(&custom)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(custom);
        agents.push(AgentCandidate {
            id: "custom".into(),
            label: "Custom".into(),
            command: Some(resolved),
            args,
            available: true,
            models: Vec::new(),
            model_flag: None,
            model_value_prefix: None,
            model_env: None,
        });
    }

    agents
}

fn codex_candidate() -> AgentCandidate {
    let codex_models = vec![
        model("gpt-5.3-codex", "GPT-5.3 Codex"),
        model("gpt-5.3-codex-spark", "GPT-5.3 Codex Spark"),
        model("gpt-5.2-codex", "GPT-5.2 Codex"),
        model("gpt-5.2", "GPT-5.2"),
        model("gpt-5.1-codex-max", "GPT-5.1 Codex Max"),
        model("gpt-5.1-codex", "GPT-5.1 Codex"),
        model("gpt-5-codex", "GPT-5 Codex"),
    ];

    if let Ok(path) = std::env::var("CODEX_ACP_BIN")
        && !path.trim().is_empty()
    {
        let resolved = crate::infra::shell::find_bin(&path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(path);
        return AgentCandidate {
            id: "codex".into(),
            label: "Codex".into(),
            command: Some(resolved),
            args: Vec::new(),
            available: true,
            models: codex_models,
            model_flag: Some("-c".into()),
            model_value_prefix: Some("model=".into()),
            model_env: None,
        };
    }

    let command =
        crate::infra::shell::find_bin("npx").map(|path| path.to_string_lossy().to_string());
    let package = std::env::var("CODEX_ACP_PACKAGE")
        .unwrap_or_else(|_| "@zed-industries/codex-acp@latest".to_string());
    AgentCandidate {
        id: "codex".into(),
        label: "Codex".into(),
        available: command.is_some(),
        command,
        args: vec!["-y".into(), package],
        models: codex_models,
        model_flag: Some("-c".into()),
        model_value_prefix: Some("model=".into()),
        model_env: None,
    }
}

fn npx_candidate(id: &str, label: &str, env_var: &str, package: &str) -> AgentCandidate {
    if let Ok(path) = std::env::var(env_var)
        && !path.trim().is_empty()
    {
        let resolved = crate::infra::shell::find_bin(&path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(path);
        return AgentCandidate {
            id: id.into(),
            label: label.into(),
            command: Some(resolved),
            args: Vec::new(),
            available: true,
            models: Vec::new(),
            model_flag: None,
            model_value_prefix: None,
            model_env: None,
        };
    }

    let command =
        crate::infra::shell::find_bin("npx").map(|path| path.to_string_lossy().to_string());
    AgentCandidate {
        id: id.into(),
        label: label.into(),
        available: command.is_some(),
        command,
        args: vec!["-y".into(), package.into()],
        models: Vec::new(),
        model_flag: None,
        model_value_prefix: None,
        model_env: None,
    }
}

fn command_candidate(
    id: &str,
    label: &str,
    env_var: &str,
    bin: &str,
    args: &[&str],
) -> AgentCandidate {
    if let Ok(path) = std::env::var(env_var)
        && !path.trim().is_empty()
    {
        let resolved = crate::infra::shell::find_bin(&path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(path);
        return AgentCandidate {
            id: id.into(),
            label: label.into(),
            command: Some(resolved),
            args: args.iter().map(|arg| (*arg).to_string()).collect(),
            available: true,
            models: Vec::new(),
            model_flag: None,
            model_value_prefix: None,
            model_env: None,
        };
    }

    let command = crate::infra::shell::find_bin(bin).map(|path| path.to_string_lossy().to_string());

    AgentCandidate {
        id: id.into(),
        label: label.into(),
        available: command.is_some(),
        command,
        args: args.iter().map(|arg| (*arg).to_string()).collect(),
        models: Vec::new(),
        model_flag: None,
        model_value_prefix: None,
        model_env: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(
        models: Vec<(&str, &str)>,
        flag: Option<&str>,
        prefix: Option<&str>,
        env: Option<&str>,
    ) -> AgentCandidate {
        AgentCandidate {
            id: "test".into(),
            label: "Test".into(),
            command: Some("test-bin".into()),
            args: Vec::new(),
            available: true,
            models: models.into_iter().map(|(i, n)| model(i, n)).collect(),
            model_flag: flag.map(str::to_string),
            model_value_prefix: prefix.map(str::to_string),
            model_env: env.map(str::to_string),
        }
    }

    #[test]
    fn resolve_none_when_no_model_requested() {
        let c = build(vec![("sonnet", "Sonnet")], Some("--model"), None, None);
        let (flag, env) = c.resolve_model(None);
        assert!(flag.is_none() && env.is_none());
    }

    #[test]
    fn resolve_none_when_model_not_supported() {
        let c = build(vec![("sonnet", "Sonnet")], Some("--model"), None, None);
        let (flag, env) = c.resolve_model(Some("haiku"));
        assert!(flag.is_none() && env.is_none());
    }

    #[test]
    fn resolve_flag_with_prefix() {
        let c = build(
            vec![("gpt-5.2", "GPT-5.2")],
            Some("-c"),
            Some("model="),
            None,
        );
        let (flag, env) = c.resolve_model(Some("gpt-5.2"));
        assert_eq!(flag, Some(("-c".into(), "model=gpt-5.2".into())));
        assert!(env.is_none());
    }

    #[test]
    fn resolve_plain_flag() {
        let c = build(vec![("sonnet", "Sonnet")], Some("--model"), None, None);
        let (flag, env) = c.resolve_model(Some("sonnet"));
        assert_eq!(flag, Some(("--model".into(), "sonnet".into())));
        assert!(env.is_none());
    }

    #[test]
    fn resolve_trims_empty_model_id() {
        let c = build(vec![("sonnet", "Sonnet")], Some("--model"), None, None);
        let (flag, env) = c.resolve_model(Some("   "));
        assert!(flag.is_none() && env.is_none());
    }

    #[test]
    fn resolve_emits_both_flag_and_env_when_candidate_has_both() {
        let c = build(
            vec![("gpt-5", "GPT-5")],
            Some("--model"),
            None,
            Some("OPENAI_MODEL"),
        );
        let (flag, env) = c.resolve_model(Some("gpt-5"));
        assert_eq!(flag, Some(("--model".into(), "gpt-5".into())));
        assert_eq!(env, Some(("OPENAI_MODEL".into(), "gpt-5".into())));
    }

    #[test]
    fn resolve_skips_blank_flag_and_env_strings() {
        let c = build(vec![("sonnet", "Sonnet")], Some("   "), None, Some("   "));
        let (flag, env) = c.resolve_model(Some("sonnet"));
        assert!(flag.is_none());
        assert!(env.is_none());
    }

    #[test]
    fn candidate_lookup_never_empty_and_has_stable_ids() {
        let candidates = list_agent_candidates();
        assert!(!candidates.is_empty());
        for expected in ["codex", "claude", "gemini", "qwen", "kimi", "opencode"] {
            assert!(
                candidates.iter().any(|c| c.id == expected),
                "missing built-in agent id {expected}",
            );
        }
    }
}
