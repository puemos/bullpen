use crate::domain::*;
use crate::infra::acp::analysis_generator::{
    GenerateAnalysisInput, ProgressEvent, generate_with_acp,
};
use crate::infra::acp::analysis_mcp_server::RunContext;
use crate::infra::acp::{AgentCandidate, list_agent_candidates};
use crate::infra::app_config::{AppConfig, load_config, save_config};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State, ipc::Channel};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize)]
pub struct DataChangedPayload {
    pub analysis_id: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerateAnalysisResult {
    pub analysis_id: String,
    pub run_id: String,
}

#[tauri::command]
pub async fn get_agents() -> Result<Vec<AgentCandidate>, String> {
    Ok(list_agent_candidates())
}

#[tauri::command]
pub async fn get_settings() -> Result<AppConfig, String> {
    Ok(load_config())
}

#[tauri::command]
pub async fn update_settings(config: AppConfig) -> Result<AppConfig, String> {
    save_config(&config).map_err(|err| err.to_string())?;
    Ok(config)
}

#[tauri::command]
pub async fn get_all_analyses(state: State<'_, AppState>) -> Result<Vec<AnalysisSummary>, String> {
    let db = state.db.lock().map_err(|err| err.to_string())?;
    db.list_analyses().map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn get_analysis_report(
    state: State<'_, AppState>,
    analysis_id: String,
    run_id: Option<String>,
) -> Result<Option<AnalysisReport>, String> {
    let db = state.db.lock().map_err(|err| err.to_string())?;
    db.get_report(&analysis_id, run_id.as_deref())
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn delete_analysis(
    state: State<'_, AppState>,
    analysis_id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|err| err.to_string())?;
    db.delete_analysis(&analysis_id)
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn stop_analysis(state: State<'_, AppState>, run_id: String) -> Result<(), String> {
    let token = {
        let active = state.active_runs.lock().map_err(|err| err.to_string())?;
        active.get(&run_id).cloned()
    };
    if let Some(token) = token {
        token.cancel();
    }
    Ok(())
}

#[tauri::command]
pub async fn export_analysis_markdown(
    state: State<'_, AppState>,
    analysis_id: String,
) -> Result<String, String> {
    let report = {
        let db = state.db.lock().map_err(|err| err.to_string())?;
        db.get_report(&analysis_id, None)
            .map_err(|err| err.to_string())?
    };
    let Some(report) = report else {
        return Err("analysis not found".to_string());
    };
    Ok(render_markdown(&report))
}

#[tauri::command]
pub async fn create_analysis(
    state: State<'_, AppState>,
    user_prompt: String,
) -> Result<String, String> {
    let trimmed = user_prompt.trim();
    if trimmed.is_empty() {
        return Err("Enter a research request before starting analysis.".to_string());
    }
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let analysis = Analysis {
        id: id.clone(),
        title: derive_title(trimmed),
        user_prompt: trimmed.to_string(),
        intent: AnalysisIntent::GeneralResearch,
        status: AnalysisStatus::Running,
        active_run_id: None,
        created_at: now.clone(),
        updated_at: now,
    };
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.save_analysis(&analysis).map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn set_active_run(
    state: State<'_, AppState>,
    analysis_id: String,
    run_id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.save_analysis(&{
        let analysis = db
            .get_report(&analysis_id, None)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Analysis not found".to_string())?
            .analysis;
        Analysis {
            active_run_id: Some(run_id),
            updated_at: chrono::Utc::now().to_rfc3339(),
            ..analysis
        }
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_run_progress(
    state: State<'_, AppState>,
    run_id: String,
) -> Result<Vec<ProgressEventPayload>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_run_progress(&run_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_analysis(
    app: AppHandle,
    state: State<'_, AppState>,
    user_prompt: String,
    agent_id: String,
    model_id: Option<String>,
    analysis_id: String,
    run_id: Option<String>,
    on_progress: Channel<ProgressEventPayload>,
) -> Result<GenerateAnalysisResult, String> {
    let trimmed_prompt = user_prompt.trim();
    if trimmed_prompt.is_empty() {
        return Err("Enter a research request before starting analysis.".to_string());
    }

    let candidates = list_agent_candidates();
    let candidate = candidates
        .iter()
        .find(|candidate| candidate.id == agent_id)
        .or_else(|| candidates.iter().find(|candidate| candidate.available))
        .ok_or_else(|| "No ACP agent is configured. Add one in Settings.".to_string())?;

    let command = candidate.command.clone().ok_or_else(|| {
        format!(
            "Agent '{}' is not available. Configure the binary in Settings or environment.",
            candidate.label
        )
    })?;
    let agent_args = candidate.args.clone();
    let (model_flag, model_env) = candidate.resolve_model(model_id.as_deref());

    let now = chrono::Utc::now().to_rfc3339();
    let run_id = run_id.unwrap_or_else(|| Uuid::new_v4().to_string());

    let run = AnalysisRun {
        id: run_id.clone(),
        analysis_id: analysis_id.clone(),
        agent_id: candidate.id.clone(),
        model_id: model_id.clone(),
        prompt_text: trimmed_prompt.to_string(),
        status: AnalysisStatus::Running,
        started_at: now.clone(),
        completed_at: None,
        error: None,
    };

    let db_path = {
        let db = state.db.lock().map_err(|err| err.to_string())?;
        db.save_run(&run).map_err(|err| err.to_string())?;
        db.set_active_run_if_empty(&analysis_id, &run_id)
            .map_err(|err| err.to_string())?;
        db.path().clone()
    };

    let cancel_token = CancellationToken::new();
    {
        let mut active = state.active_runs.lock().map_err(|err| err.to_string())?;
        active.insert(run_id.clone(), cancel_token.clone());
    }

    let context = RunContext {
        analysis_id: analysis_id.clone(),
        run_id: run_id.clone(),
        agent_id: candidate.id.clone(),
        user_prompt: trimmed_prompt.to_string(),
        created_at: now,
    };

    let (progress_tx, mut progress_rx) = mpsc::unbounded_channel::<ProgressEvent>();
    let progress_channel = on_progress.clone();
    let run_id_clone = run_id.clone();
    let db_clone = state.db.clone();

    let (coalesce_tx, mut coalesce_rx) = mpsc::unbounded_channel::<()>();
    let coalesce_app = app.clone();
    let coalesce_aid = analysis_id.clone();
    tauri::async_runtime::spawn(async move {
        while coalesce_rx.recv().await.is_some() {
            while coalesce_rx.try_recv().is_ok() {}
            let _ = coalesce_app.emit(
                "analysis-data-changed",
                DataChangedPayload {
                    analysis_id: coalesce_aid.clone(),
                    kind: "batch".to_string(),
                },
            );
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    });

    tauri::async_runtime::spawn(async move {
        while let Some(event) = progress_rx.recv().await {
            let payload = match event {
                ProgressEvent::MessageDelta { id, delta } => {
                    ProgressEventPayload::MessageDelta { id, delta }
                }
                ProgressEvent::ThoughtDelta { id, delta } => {
                    ProgressEventPayload::ThoughtDelta { id, delta }
                }
                ProgressEvent::ToolCallStarted {
                    tool_call_id,
                    title,
                    kind,
                } => ProgressEventPayload::ToolCallStarted {
                    tool_call_id,
                    title,
                    kind,
                },
                ProgressEvent::ToolCallComplete {
                    tool_call_id,
                    status,
                    title,
                    raw_input,
                    raw_output,
                } => ProgressEventPayload::ToolCallComplete {
                    tool_call_id,
                    status,
                    title,
                    raw_input,
                    raw_output,
                },
                ProgressEvent::Plan(plan) => ProgressEventPayload::Plan(FrontendPlan {
                    entries: plan
                        .entries
                        .into_iter()
                        .map(|entry| FrontendPlanEntry {
                            content: entry.content,
                            priority: format!("{:?}", entry.priority),
                            status: format!("{:?}", entry.status),
                        })
                        .collect(),
                }),
                ProgressEvent::LocalLog(msg) => ProgressEventPayload::Log(msg),
                ProgressEvent::PlanSubmitted => ProgressEventPayload::PlanSubmitted,
                ProgressEvent::SourceSubmitted => ProgressEventPayload::SourceSubmitted,
                ProgressEvent::MetricSubmitted => ProgressEventPayload::MetricSubmitted,
                ProgressEvent::ArtifactSubmitted => ProgressEventPayload::ArtifactSubmitted,
                ProgressEvent::BlockSubmitted => ProgressEventPayload::BlockSubmitted,
                ProgressEvent::StanceSubmitted => ProgressEventPayload::StanceSubmitted,
                ProgressEvent::ProjectionSubmitted => ProgressEventPayload::ProjectionSubmitted,
                ProgressEvent::Finalized => ProgressEventPayload::Completed,
            };
            let data_kind = match &payload {
                ProgressEventPayload::PlanSubmitted => Some("plan"),
                ProgressEventPayload::SourceSubmitted => Some("source"),
                ProgressEventPayload::MetricSubmitted => Some("metric"),
                ProgressEventPayload::ArtifactSubmitted => Some("artifact"),
                ProgressEventPayload::BlockSubmitted => Some("block"),
                ProgressEventPayload::StanceSubmitted => Some("stance"),
                ProgressEventPayload::ProjectionSubmitted => Some("projection"),
                ProgressEventPayload::Completed => Some("completed"),
                _ => None,
            };
            if let Ok(db) = db_clone.lock() {
                let _ = db.append_progress_event(&run_id_clone, &payload);
            }
            if data_kind.is_some() {
                let _ = coalesce_tx.send(());
            }
            if progress_channel.send(payload).is_err() {
                break;
            }
        }
    });

    let _ = on_progress.send(ProgressEventPayload::Log(format!(
        "Starting analysis with {}...",
        candidate.label
    )));

    let config = load_config();
    let generation_result = generate_with_acp(GenerateAnalysisInput {
        run_context: context,
        agent_command: command,
        agent_args,
        model_flag,
        model_env,
        progress_tx: Some(progress_tx),
        mcp_server_binary: None,
        db_path,
        timeout_secs: Some(config.timeout_secs),
        cancel_token: Some(cancel_token),
    })
    .await;

    {
        let mut active = state.active_runs.lock().map_err(|err| err.to_string())?;
        active.remove(&run_id);
    }

    match generation_result {
        Ok(_) => {
            let db = state.db.lock().map_err(|err| err.to_string())?;
            db.update_run_status(&run_id, AnalysisStatus::Completed, None)
                .map_err(|err| err.to_string())?;
            db.recompute_analysis_status(&analysis_id)
                .map_err(|err| err.to_string())?;
            let _ = on_progress.send(ProgressEventPayload::Completed);
        }
        Err(err) => {
            let message = err.to_string();
            let is_cancelled = message.contains("cancelled by user");
            let status = if is_cancelled {
                AnalysisStatus::Cancelled
            } else {
                AnalysisStatus::Failed
            };
            let db = state.db.lock().map_err(|err| err.to_string())?;
            db.update_run_status(&run_id, status, Some(&message))
                .map_err(|err| err.to_string())?;
            db.recompute_analysis_status(&analysis_id)
                .map_err(|err| err.to_string())?;
            let _ = on_progress.send(ProgressEventPayload::Error {
                message: message.clone(),
            });
            return Err(message);
        }
    }

    Ok(GenerateAnalysisResult {
        analysis_id,
        run_id,
    })
}

fn derive_title(prompt: &str) -> String {
    let first_line = prompt.lines().next().unwrap_or("Stock analysis").trim();
    let mut title = first_line.chars().take(72).collect::<String>();
    if first_line.chars().count() > 72 {
        title.push_str("...");
    }
    if title.is_empty() {
        "Stock analysis".to_string()
    } else {
        title
    }
}

fn render_markdown(report: &AnalysisReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", report.analysis.title));
    out.push_str("> Research only. Not investment advice.\n\n");
    out.push_str(&format!("**Request:** {}\n\n", report.analysis.user_prompt));
    if let Some(stance) = &report.final_stance {
        out.push_str(&format!(
            "## Final Stance\n\n**{}**, horizon: {}, confidence: {:.0}%\n\n{}\n\n",
            stance.stance,
            stance.horizon,
            stance.confidence * 100.0,
            stance.summary
        ));
    }
    if !report.projections.is_empty() {
        out.push_str("## Projections\n\n");
        for projection in &report.projections {
            out.push_str(&format!(
                "### {} ({}) · horizon {} · current {}\n\n",
                projection.metric,
                projection.entity_id,
                projection.horizon,
                projection.current_value_label
            ));
            out.push_str(&format!("**Methodology:** {}\n\n", projection.methodology));
            for scenario in &projection.scenarios {
                out.push_str(&format!(
                    "- **{}** → {} ({:+.1}%, probability {:.0}%) — {}\n",
                    scenario.label,
                    scenario.target_label,
                    scenario.upside_pct * 100.0,
                    scenario.probability * 100.0,
                    scenario.rationale
                ));
            }
            if !projection.key_assumptions.is_empty() {
                out.push_str("\n**Key assumptions:**\n");
                for assumption in &projection.key_assumptions {
                    out.push_str(&format!("- {assumption}\n"));
                }
            }
            if !projection.evidence_ids.is_empty() {
                out.push_str(&format!(
                    "\nSources: `{}`\n",
                    projection.evidence_ids.join("`, `")
                ));
            }
            out.push('\n');
        }
    }
    if !report.artifacts.is_empty() {
        out.push_str("## Structured Evidence\n\n");
        for artifact in &report.artifacts {
            out.push_str(&format!(
                "### {}\n\n{}\n\n",
                artifact.title, artifact.summary
            ));
            if !artifact.evidence_ids.is_empty() {
                out.push_str(&format!(
                    "Sources: `{}`\n\n",
                    artifact.evidence_ids.join("`, `")
                ));
            }
        }
    }
    for block in &report.blocks {
        out.push_str(&format!("## {}\n\n{}\n\n", block.title, block.body));
        if !block.evidence_ids.is_empty() {
            out.push_str(&format!(
                "Sources: `{}`\n\n",
                block.evidence_ids.join("`, `")
            ));
        }
    }
    if !report.sources.is_empty() {
        out.push_str("## Sources\n\n");
        for source in &report.sources {
            let link = source.url.clone().unwrap_or_default();
            out.push_str(&format!(
                "- `{}` {} ({}, retrieved {})\n",
                source.id, source.title, link, source.retrieved_at
            ));
        }
    }
    out
}
