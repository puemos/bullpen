use serde::{Deserialize, Serialize};

/// Self-contained context describing a single analysis run.
///
/// Intentionally plain data: carried across the Tauri host → MCP child
/// process boundary via JSON, so it must not reference any infrastructure
/// types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunContext {
    pub analysis_id: String,
    pub run_id: String,
    pub agent_id: String,
    pub user_prompt: String,
    pub created_at: String,
    /// Data-source provider ids the agent is allowed to use for this run.
    /// The MCP server uses this to gate tool registration: a provider id
    /// absent from the list will not appear in the agent's tool list at all.
    #[serde(default)]
    pub enabled_sources: Vec<String>,
}
