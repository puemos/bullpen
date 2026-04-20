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
    pub supports_model_override: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentLaunch {
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}

trait AgentDefinition {
    fn candidate(&self) -> &AgentCandidate;

    fn build_launch(&self, model_id: Option<&str>) -> Result<AgentLaunch, String> {
        let model_id = validate_model_selection(self.candidate(), model_id)?;
        self.build_launch_for_model(model_id.as_deref())
    }

    fn build_launch_for_model(&self, model_id: Option<&str>) -> Result<AgentLaunch, String>;
}

#[must_use]
pub fn list_agent_candidates() -> Vec<AgentCandidate> {
    agent_definitions()
        .into_iter()
        .map(|agent| agent.candidate().clone())
        .collect()
}

pub fn resolve_agent_launch(
    agent_id: &str,
    model_id: Option<&str>,
) -> Result<(AgentCandidate, AgentLaunch), String> {
    let agents = agent_definitions();
    let Some(agent) = agents
        .iter()
        .find(|agent| agent.candidate().id == agent_id)
        .or_else(|| agents.iter().find(|agent| agent.candidate().available))
    else {
        return Err("No ACP agent is configured. Add one in Settings.".to_string());
    };

    let candidate = agent.candidate().clone();
    let launch = agent.build_launch(model_id)?;
    Ok((candidate, launch))
}

fn agent_definitions() -> Vec<Box<dyn AgentDefinition>> {
    let config = crate::infra::app_config::load_config();
    let mut agents: Vec<Box<dyn AgentDefinition>> = vec![
        Box::new(CodexAgent::new()),
        Box::new(ClaudeAgent::new()),
        Box::new(GeminiAgent::new()),
        Box::new(QwenAgent::new()),
        Box::new(MistralVibeAgent::new()),
        Box::new(KimiAgent::new()),
        Box::new(OpenCodeAgent::new()),
    ];

    let custom_command = config
        .custom_agent_command
        .filter(|value| !value.trim().is_empty())
        .or_else(|| std::env::var("BULLPEN_CUSTOM_AGENT").ok());

    if let Some(command) = custom_command {
        let args = if config.custom_agent_args.is_empty() {
            std::env::var("BULLPEN_CUSTOM_AGENT_ARGS")
                .map(|raw| raw.split_whitespace().map(str::to_string).collect())
                .unwrap_or_default()
        } else {
            config.custom_agent_args
        };
        agents.push(Box::new(CustomAgent::new(command, args)));
    }

    agents
}

struct CodexAgent {
    candidate: AgentCandidate,
}

impl CodexAgent {
    fn new() -> Self {
        let models = vec![
            model("gpt-5.4", "GPT-5.4"),
            model("gpt-5.4-mini", "GPT-5.4 Mini"),
            model("gpt-5.3-codex", "GPT-5.3 Codex"),
            model("gpt-5.3-codex-spark", "GPT-5.3 Codex Spark"),
            model("gpt-5.2", "GPT-5.2"),
        ];

        if let Ok(path) = std::env::var("CODEX_ACP_BIN")
            && !path.trim().is_empty()
        {
            let resolved = resolve_bin_or_literal(path);
            return Self {
                candidate: AgentCandidate {
                    id: "codex".into(),
                    label: "Codex".into(),
                    command: Some(resolved),
                    args: Vec::new(),
                    available: true,
                    models,
                    supports_model_override: true,
                },
            };
        }

        let package = std::env::var("CODEX_ACP_PACKAGE")
            .unwrap_or_else(|_| "@zed-industries/codex-acp@latest".to_string());
        Self {
            candidate: AgentCandidate {
                id: "codex".into(),
                label: "Codex".into(),
                command: find_bin("npx"),
                args: vec!["-y".into(), package],
                available: crate::infra::shell::find_bin("codex").is_some(),
                models,
                supports_model_override: true,
            },
        }
    }
}

impl AgentDefinition for CodexAgent {
    fn candidate(&self) -> &AgentCandidate {
        &self.candidate
    }

    fn build_launch_for_model(&self, model_id: Option<&str>) -> Result<AgentLaunch, String> {
        let mut launch = launch_from_candidate(&self.candidate)?;
        if let Some(model_id) = model_id {
            push_arg_pair(&mut launch.args, "-c", format!("model={model_id}"));
        }
        Ok(launch)
    }
}

struct ClaudeAgent {
    candidate: AgentCandidate,
}

impl ClaudeAgent {
    fn new() -> Self {
        let mut candidate = npx_candidate(
            "claude",
            "Claude",
            "CLAUDE_ACP_BIN",
            "claude",
            "@zed-industries/claude-code-acp",
        );
        candidate.models = vec![
            model("default", "Default"),
            model("best", "Best"),
            model("sonnet", "Sonnet"),
            model("opus", "Opus"),
            model("haiku", "Haiku"),
            model("sonnet[1m]", "Sonnet (1M)"),
            model("opus[1m]", "Opus (1M)"),
            model("opusplan", "Opus Plan"),
            model("claude-sonnet-4-6", "Claude Sonnet 4.6"),
            model("claude-opus-4-6", "Claude Opus 4.6"),
        ];
        candidate.supports_model_override = true;
        Self { candidate }
    }
}

impl AgentDefinition for ClaudeAgent {
    fn candidate(&self) -> &AgentCandidate {
        &self.candidate
    }

    fn build_launch_for_model(&self, model_id: Option<&str>) -> Result<AgentLaunch, String> {
        let mut launch = launch_from_candidate(&self.candidate)?;
        if let Some(model_id) = model_id {
            push_arg_pair(&mut launch.args, "--model", model_id);
        }
        Ok(launch)
    }
}

struct GeminiAgent {
    candidate: AgentCandidate,
}

impl GeminiAgent {
    fn new() -> Self {
        let mut candidate =
            command_candidate("gemini", "Gemini", "GEMINI_ACP_BIN", "gemini", &["--acp"]);
        candidate.models = vec![
            model("auto", "Auto"),
            model("pro", "Pro"),
            model("flash", "Flash"),
            model("flash-lite", "Flash Lite"),
            model("gemini-3-pro-preview", "Gemini 3 Pro (Preview)"),
            model("gemini-3-flash-preview", "Gemini 3 Flash (Preview)"),
            model("gemini-2.5-pro", "Gemini 2.5 Pro"),
            model("gemini-2.5-flash", "Gemini 2.5 Flash"),
            model("gemini-2.5-flash-lite", "Gemini 2.5 Flash Lite"),
        ];
        candidate.supports_model_override = true;
        Self { candidate }
    }
}

impl AgentDefinition for GeminiAgent {
    fn candidate(&self) -> &AgentCandidate {
        &self.candidate
    }

    fn build_launch_for_model(&self, model_id: Option<&str>) -> Result<AgentLaunch, String> {
        let mut launch = launch_from_candidate(&self.candidate)?;
        if let Some(model_id) = model_id {
            push_arg_pair(&mut launch.args, "--model", model_id);
        }
        Ok(launch)
    }
}

struct QwenAgent {
    candidate: AgentCandidate,
}

impl QwenAgent {
    fn new() -> Self {
        let mut candidate =
            command_candidate("qwen", "Qwen Code", "QWEN_ACP_BIN", "qwen", &["--acp"]);
        candidate.models = vec![
            model("qwen3-coder-plus", "Qwen3 Coder Plus"),
            model("qwen3.5-plus", "Qwen3.5 Plus"),
            model("qwen3-max-2026-01-23", "Qwen3 Max"),
        ];
        candidate.supports_model_override = true;
        Self { candidate }
    }
}

impl AgentDefinition for QwenAgent {
    fn candidate(&self) -> &AgentCandidate {
        &self.candidate
    }

    fn build_launch_for_model(&self, model_id: Option<&str>) -> Result<AgentLaunch, String> {
        let mut launch = launch_from_candidate(&self.candidate)?;
        if let Some(model_id) = model_id {
            push_arg_pair(&mut launch.args, "--model", model_id);
        }
        Ok(launch)
    }
}

struct MistralVibeAgent {
    candidate: AgentCandidate,
}

impl MistralVibeAgent {
    fn new() -> Self {
        Self {
            candidate: command_candidate(
                "mistral",
                "Mistral Vibe",
                "MISTRAL_ACP_BIN",
                "vibe-acp",
                &[],
            ),
        }
    }
}

impl AgentDefinition for MistralVibeAgent {
    fn candidate(&self) -> &AgentCandidate {
        &self.candidate
    }

    fn build_launch_for_model(&self, model_id: Option<&str>) -> Result<AgentLaunch, String> {
        if model_id.is_some() {
            return Err(format!(
                "{} does not expose a documented startup model override. Pick Default.",
                self.candidate.label
            ));
        }
        launch_from_candidate(&self.candidate)
    }
}

struct KimiAgent {
    candidate: AgentCandidate,
}

impl KimiAgent {
    fn new() -> Self {
        let mut candidate = command_candidate("kimi", "Kimi", "KIMI_ACP_BIN", "kimi", &["--acp"]);
        candidate.models = vec![
            model("kimi-k2.5", "Kimi K2.5"),
            model("kimi-k2-0905-preview", "Kimi K2 0905 Preview"),
            model("kimi-k2-0711-preview", "Kimi K2 0711 Preview"),
            model("kimi-k2-turbo-preview", "Kimi K2 Turbo Preview"),
            model("kimi-k2-thinking", "Kimi K2 Thinking"),
            model("kimi-k2-thinking-turbo", "Kimi K2 Thinking Turbo"),
        ];
        candidate.supports_model_override = true;
        Self { candidate }
    }
}

impl AgentDefinition for KimiAgent {
    fn candidate(&self) -> &AgentCandidate {
        &self.candidate
    }

    fn build_launch_for_model(&self, model_id: Option<&str>) -> Result<AgentLaunch, String> {
        let mut launch = launch_from_candidate(&self.candidate)?;
        if let Some(model_id) = model_id {
            push_arg_pair(&mut launch.args, "--model", model_id);
        }
        Ok(launch)
    }
}

struct OpenCodeAgent {
    candidate: AgentCandidate,
}

impl OpenCodeAgent {
    fn new() -> Self {
        let mut candidate = command_candidate(
            "opencode",
            "OpenCode",
            "OPENCODE_ACP_BIN",
            "opencode",
            &["acp"],
        );
        candidate.supports_model_override = true;
        Self { candidate }
    }
}

impl AgentDefinition for OpenCodeAgent {
    fn candidate(&self) -> &AgentCandidate {
        &self.candidate
    }

    fn build_launch_for_model(&self, model_id: Option<&str>) -> Result<AgentLaunch, String> {
        let mut launch = launch_from_candidate(&self.candidate)?;
        if let Some(model_id) = model_id {
            let mut args = Vec::with_capacity(launch.args.len() + 2);
            push_arg_pair(&mut args, "--model", model_id);
            args.extend(launch.args);
            launch.args = args;
        }
        Ok(launch)
    }
}

struct CustomAgent {
    candidate: AgentCandidate,
}

impl CustomAgent {
    fn new(command: String, args: Vec<String>) -> Self {
        Self {
            candidate: AgentCandidate {
                id: "custom".into(),
                label: "Custom".into(),
                command: Some(resolve_bin_or_literal(command)),
                args,
                available: true,
                models: Vec::new(),
                supports_model_override: false,
            },
        }
    }
}

impl AgentDefinition for CustomAgent {
    fn candidate(&self) -> &AgentCandidate {
        &self.candidate
    }

    fn build_launch_for_model(&self, model_id: Option<&str>) -> Result<AgentLaunch, String> {
        if model_id.is_some() {
            return Err("Custom agents do not support Bullpen model overrides.".to_string());
        }
        launch_from_candidate(&self.candidate)
    }
}

fn validate_model_selection(
    candidate: &AgentCandidate,
    model_id: Option<&str>,
) -> Result<Option<String>, String> {
    let Some(model_id) = model_id.map(str::trim).filter(|model| !model.is_empty()) else {
        return Ok(None);
    };

    if !candidate.supports_model_override {
        return Err(format!(
            "{} does not expose a model override. Pick Default.",
            candidate.label
        ));
    }

    Ok(Some(model_id.to_string()))
}

fn launch_from_candidate(candidate: &AgentCandidate) -> Result<AgentLaunch, String> {
    let command = candidate.command.clone().ok_or_else(|| {
        format!(
            "Agent '{}' is not available. Configure the binary in Settings or environment.",
            candidate.label
        )
    })?;
    Ok(AgentLaunch {
        command,
        args: candidate.args.clone(),
        env: Vec::new(),
    })
}

fn npx_candidate(
    id: &str,
    label: &str,
    env_var: &str,
    raw_bin: &str,
    package: &str,
) -> AgentCandidate {
    if let Ok(path) = std::env::var(env_var)
        && !path.trim().is_empty()
    {
        return AgentCandidate {
            id: id.into(),
            label: label.into(),
            command: Some(resolve_bin_or_literal(path)),
            args: Vec::new(),
            available: true,
            models: Vec::new(),
            supports_model_override: false,
        };
    }

    AgentCandidate {
        id: id.into(),
        label: label.into(),
        available: crate::infra::shell::find_bin(raw_bin).is_some(),
        command: find_bin("npx"),
        args: vec!["-y".into(), package.into()],
        models: Vec::new(),
        supports_model_override: false,
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
        return AgentCandidate {
            id: id.into(),
            label: label.into(),
            command: Some(resolve_bin_or_literal(path)),
            args: args.iter().map(|arg| (*arg).to_string()).collect(),
            available: true,
            models: Vec::new(),
            supports_model_override: false,
        };
    }

    AgentCandidate {
        id: id.into(),
        label: label.into(),
        command: find_bin(bin),
        available: crate::infra::shell::find_bin(bin).is_some(),
        args: args.iter().map(|arg| (*arg).to_string()).collect(),
        models: Vec::new(),
        supports_model_override: false,
    }
}

fn model(id: &str, name: &str) -> AgentModel {
    AgentModel {
        id: id.to_string(),
        name: name.to_string(),
    }
}

fn push_arg_pair(args: &mut Vec<String>, flag: &str, value: impl Into<String>) {
    args.push(flag.to_string());
    args.push(value.into());
}

fn resolve_bin_or_literal(bin: String) -> String {
    crate::infra::shell::find_bin(&bin)
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or(bin)
}

fn find_bin(bin: &str) -> Option<String> {
    crate::infra::shell::find_bin(bin).map(|path| path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(id: &str, args: &[&str], supports_model_override: bool) -> AgentCandidate {
        AgentCandidate {
            id: id.into(),
            label: id.into(),
            command: Some(format!("{id}-bin")),
            args: args.iter().map(|arg| (*arg).to_string()).collect(),
            available: true,
            models: vec![model("listed", "Listed")],
            supports_model_override,
        }
    }

    #[test]
    fn codex_launch_appends_config_model_after_adapter_args() {
        let agent = CodexAgent {
            candidate: candidate("codex", &["-y", "@zed-industries/codex-acp@latest"], true),
        };

        let launch = agent.build_launch(Some("gpt-5.4")).unwrap();

        assert_eq!(
            launch.args,
            vec![
                "-y",
                "@zed-industries/codex-acp@latest",
                "-c",
                "model=gpt-5.4"
            ]
        );
    }

    #[test]
    fn claude_launch_appends_selected_model_after_adapter_args() {
        let agent = ClaudeAgent {
            candidate: candidate("claude", &["-y", "@zed-industries/claude-code-acp"], true),
        };

        let launch = agent.build_launch(Some("sonnet")).unwrap();

        assert_eq!(
            launch.args,
            vec!["-y", "@zed-industries/claude-code-acp", "--model", "sonnet"]
        );
    }

    #[test]
    fn opencode_launch_places_model_before_subcommand() {
        let agent = OpenCodeAgent {
            candidate: candidate("opencode", &["acp"], true),
        };

        let launch = agent
            .build_launch(Some("anthropic/claude-sonnet-4-5"))
            .unwrap();

        assert_eq!(
            launch.args,
            vec!["--model", "anthropic/claude-sonnet-4-5", "acp"]
        );
    }

    #[test]
    fn no_model_uses_agent_default_args() {
        let agent = GeminiAgent {
            candidate: candidate("gemini", &["--acp"], true),
        };

        let launch = agent.build_launch(None).unwrap();

        assert_eq!(launch.args, vec!["--acp"]);
    }

    #[test]
    fn trims_empty_model_to_agent_default() {
        let agent = QwenAgent {
            candidate: candidate("qwen", &["--acp"], true),
        };

        let launch = agent.build_launch(Some("  ")).unwrap();

        assert_eq!(launch.args, vec!["--acp"]);
    }

    #[test]
    fn custom_model_is_allowed_for_model_override_agents() {
        let agent = KimiAgent {
            candidate: candidate("kimi", &["--acp"], true),
        };

        let launch = agent.build_launch(Some("custom-kimi-model")).unwrap();

        assert_eq!(launch.args, vec!["--acp", "--model", "custom-kimi-model"]);
    }

    #[test]
    fn model_override_is_rejected_when_agent_does_not_support_it() {
        let agent = MistralVibeAgent {
            candidate: candidate("mistral", &[], false),
        };

        let err = agent.build_launch(Some("devstral-small-2")).unwrap_err();

        assert!(err.contains("does not expose a model override"));
    }

    #[test]
    fn candidate_lookup_never_empty_and_has_stable_ids() {
        let candidates = list_agent_candidates();
        assert!(!candidates.is_empty());
        for expected in ["codex", "claude", "gemini", "qwen", "kimi", "opencode"] {
            assert!(
                candidates.iter().any(|candidate| candidate.id == expected),
                "missing built-in agent id {expected}",
            );
        }
    }

    #[test]
    fn codex_candidate_uses_current_documented_model_ids() {
        let agent = CodexAgent::new();
        let ids: Vec<_> = agent
            .candidate()
            .models
            .iter()
            .map(|model| model.id.as_str())
            .collect();

        assert_eq!(
            ids,
            vec![
                "gpt-5.4",
                "gpt-5.4-mini",
                "gpt-5.3-codex",
                "gpt-5.3-codex-spark",
                "gpt-5.2",
            ]
        );
    }
}
