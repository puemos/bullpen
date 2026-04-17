//! Progress event payload exchanged between the ACP worker, the SQLite
//! progress store, and the Tauri frontend channel.
//!
//! This module lives in `infra` rather than `domain` because the variants are
//! shaped to match the UI's expectations of the wire format. It does not depend
//! on any other `infra` submodule, so `db` and `commands` can both depend on it
//! without creating a cycle.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data")]
pub enum ProgressEventPayload {
    Log(String),
    MessageDelta {
        id: String,
        delta: String,
    },
    ThoughtDelta {
        id: String,
        delta: String,
    },
    ToolCallStarted {
        tool_call_id: String,
        title: String,
        kind: String,
    },
    ToolCallComplete {
        tool_call_id: String,
        status: String,
        title: String,
        raw_input: Option<serde_json::Value>,
        raw_output: Option<serde_json::Value>,
    },
    Plan(FrontendPlan),
    PlanSubmitted,
    SourceSubmitted,
    MetricSubmitted,
    ArtifactSubmitted,
    BlockSubmitted,
    StanceSubmitted,
    ProjectionSubmitted,
    Completed,
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendPlan {
    pub entries: Vec<FrontendPlanEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendPlanEntry {
    pub content: String,
    pub priority: String,
    pub status: String,
}
