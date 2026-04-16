use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCandidate {
    pub id: String,
    pub label: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub available: bool,
}

pub fn list_agent_candidates() -> Vec<AgentCandidate> {
    let config = crate::infra::app_config::load_config();
    let mut agents = vec![
        codex_candidate(),
        npx_candidate(
            "claude",
            "Claude",
            "CLAUDE_ACP_BIN",
            "@zed-industries/claude-code-acp",
        ),
        command_candidate(
            "gemini",
            "Gemini",
            "GEMINI_ACP_BIN",
            "gemini",
            &["--experimental-acp"],
        ),
        command_candidate(
            "qwen",
            "Qwen Code",
            "QWEN_ACP_BIN",
            "qwen",
            &["--experimental-acp"],
        ),
        command_candidate(
            "mistral",
            "Mistral",
            "MISTRAL_ACP_BIN",
            "mistral",
            &["--experimental-acp"],
        ),
        command_candidate(
            "kimi",
            "Kimi",
            "KIMI_ACP_BIN",
            "kimi",
            &["--experimental-acp"],
        ),
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
        .or_else(|| std::env::var("CRAZYLINES_CUSTOM_AGENT").ok());

    if let Some(custom) = custom_command {
        let args = if config.custom_agent_args.is_empty() {
            std::env::var("CRAZYLINES_CUSTOM_AGENT_ARGS")
                .map(|raw| raw.split_whitespace().map(str::to_string).collect())
                .unwrap_or_default()
        } else {
            config.custom_agent_args
        };
        agents.push(AgentCandidate {
            id: "custom".into(),
            label: "Custom".into(),
            command: Some(custom),
            args,
            available: true,
        });
    }

    agents
}

fn codex_candidate() -> AgentCandidate {
    if let Ok(path) = std::env::var("CODEX_ACP_BIN")
        && !path.trim().is_empty()
    {
        return AgentCandidate {
            id: "codex".into(),
            label: "Codex".into(),
            command: Some(path),
            args: Vec::new(),
            available: true,
        };
    }

    let command = which::which("npx")
        .ok()
        .map(|path| path.to_string_lossy().to_string());
    let package = std::env::var("CODEX_ACP_PACKAGE")
        .unwrap_or_else(|_| "@zed-industries/codex-acp@latest".to_string());
    AgentCandidate {
        id: "codex".into(),
        label: "Codex".into(),
        available: command.is_some(),
        command,
        args: vec![
            "-y".into(),
            package,
            "-c".into(),
            "model=\"gpt-5.2\"".into(),
        ],
    }
}

fn npx_candidate(id: &str, label: &str, env_var: &str, package: &str) -> AgentCandidate {
    if let Ok(path) = std::env::var(env_var)
        && !path.trim().is_empty()
    {
        return AgentCandidate {
            id: id.into(),
            label: label.into(),
            command: Some(path),
            args: Vec::new(),
            available: true,
        };
    }

    let command = which::which("npx")
        .ok()
        .map(|path| path.to_string_lossy().to_string());
    AgentCandidate {
        id: id.into(),
        label: label.into(),
        available: command.is_some(),
        command,
        args: vec!["-y".into(), package.into()],
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
            command: Some(path),
            args: args.iter().map(|arg| (*arg).to_string()).collect(),
            available: true,
        };
    }

    let command = which::which(bin)
        .ok()
        .map(|path| path.to_string_lossy().to_string());

    AgentCandidate {
        id: id.into(),
        label: label.into(),
        available: command.is_some(),
        command,
        args: args.iter().map(|arg| (*arg).to_string()).collect(),
    }
}
