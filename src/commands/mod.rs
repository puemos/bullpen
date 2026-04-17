pub mod error;
pub mod update;

pub use error::CommandError;

use crate::domain::{
    Analysis, AnalysisIntent, AnalysisReport, AnalysisRun, AnalysisStatus, AnalysisSummary,
};
use crate::infra::acp::analysis_generator::{
    AcpCancelled, GenerateAnalysisInput, ProgressEvent, generate_with_acp,
};
use crate::infra::acp::analysis_mcp_server::RunContext;
use crate::infra::acp::{AgentCandidate, list_agent_candidates};
use crate::infra::app_config::{AppConfig, load_config, save_config};
use crate::state::AppState;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State, ipc::Channel};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// Standalone viewer HTML, built by `pnpm build:viewer` and embedded at compile time.
/// A sentinel string inside the template is replaced at export time with the
/// report JSON for the analysis being exported.
const STANDALONE_VIEWER_HTML: &str = include_str!("../../frontend/dist-viewer/viewer.html");
const REPORT_PLACEHOLDER: &str = "\"__BULLPEN_REPORT_JSON__\"";

pub use crate::infra::progress::{FrontendPlan, FrontendPlanEntry, ProgressEventPayload};

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
pub async fn get_agents() -> Result<Vec<AgentCandidate>, CommandError> {
    Ok(list_agent_candidates())
}

#[tauri::command]
pub async fn get_settings() -> Result<AppConfig, CommandError> {
    Ok(load_config())
}

#[tauri::command]
pub async fn update_settings(config: AppConfig) -> Result<AppConfig, CommandError> {
    save_config(&config)?;
    Ok(config)
}

#[tauri::command]
pub async fn get_all_analyses(
    state: State<'_, AppState>,
) -> Result<Vec<AnalysisSummary>, CommandError> {
    let db = &state.db;
    Ok(db.list_analyses()?)
}

#[tauri::command]
pub async fn get_analysis_report(
    state: State<'_, AppState>,
    analysis_id: String,
    run_id: Option<String>,
) -> Result<Option<AnalysisReport>, CommandError> {
    let db = &state.db;
    Ok(db.get_report(&analysis_id, run_id.as_deref())?)
}

#[tauri::command]
pub async fn delete_analysis(
    state: State<'_, AppState>,
    analysis_id: String,
) -> Result<(), CommandError> {
    let db = &state.db;
    Ok(db.delete_analysis(&analysis_id)?)
}

#[tauri::command]
pub async fn stop_analysis(state: State<'_, AppState>, run_id: String) -> Result<(), CommandError> {
    let token = {
        let active = state.active_runs.lock()?;
        active.get(&run_id).cloned()
    };
    if let Some(token) = token {
        token.cancel();
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportedHtml {
    pub path: String,
    pub size_bytes: usize,
}

fn build_standalone_html(report: &AnalysisReport) -> Result<String, CommandError> {
    if !STANDALONE_VIEWER_HTML.contains(REPORT_PLACEHOLDER) {
        return Err(
            "viewer template is missing the report placeholder; run `pnpm build:viewer`".into(),
        );
    }
    let json = serde_json::to_string(report)?;
    // Escape sequences that would close the host <script> tag when the JSON
    // is embedded inline. Standard precaution for JSON-in-HTML.
    let safe = json.replace("</", "<\\/").replace("<!--", "<\\!--");
    Ok(STANDALONE_VIEWER_HTML.replace(REPORT_PLACEHOLDER, &safe))
}

fn safe_filename(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else if c.is_whitespace() {
                '-'
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = cleaned.trim_matches(|c: char| c == '-' || c == '_');
    let base = if trimmed.is_empty() {
        "bullpen-report"
    } else {
        trimmed
    };
    let base: String = base.chars().take(80).collect();
    format!("{base}.html")
}

#[tauri::command]
pub async fn export_analysis_html(
    app: AppHandle,
    state: State<'_, AppState>,
    analysis_id: String,
) -> Result<Option<ExportedHtml>, CommandError> {
    use tauri_plugin_dialog::DialogExt;

    let report = {
        let db = &state.db;
        db.get_report(&analysis_id, None)?
    };
    let Some(report) = report else {
        return Err("analysis not found".into());
    };

    let html = build_standalone_html(&report)?;
    let default_name = safe_filename(&report.analysis.title);

    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title("Export report as HTML")
        .set_file_name(&default_name)
        .add_filter("HTML", &["html"])
        .save_file(move |path| {
            let _ = tx.send(path);
        });

    let Some(path) = rx.await? else {
        return Ok(None);
    };
    let path_buf = path
        .into_path()
        .map_err(|err| CommandError::new(format!("invalid save path: {err}")))?;

    let size_bytes = html.len();
    std::fs::write(&path_buf, html.as_bytes())
        .map_err(|err| CommandError::new(format!("write failed: {err}")))?;

    Ok(Some(ExportedHtml {
        path: path_buf.display().to_string(),
        size_bytes,
    }))
}

const DEFAULT_PUBLISH_URL: &str = "https://pagedrop.dev/api/v1/sites";

#[derive(Debug, Clone, Serialize)]
pub struct PublishedReport {
    pub url: String,
    pub delete_token: String,
    pub site_id: String,
    pub provider: String,
}

#[tauri::command]
pub async fn publish_analysis_html(
    state: State<'_, AppState>,
    analysis_id: String,
) -> Result<PublishedReport, CommandError> {
    let report = {
        let db = &state.db;
        db.get_report(&analysis_id, None)?
    };
    let Some(report) = report else {
        return Err("analysis not found".into());
    };

    let html = build_standalone_html(&report)?;
    let title = report.analysis.title.clone();

    let endpoint =
        std::env::var("BULLPEN_PUBLISH_URL").unwrap_or_else(|_| DEFAULT_PUBLISH_URL.to_string());

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|err| CommandError::new(format!("http client: {err}")))?;

    let body = serde_json::json!({
        "html": html,
        "title": title,
    });

    let response = client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|err| CommandError::new(format!("publish failed: {err}")))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|err| CommandError::new(format!("publish failed: read body: {err}")))?;

    if !status.is_success() {
        return Err(CommandError::new(format!(
            "publish failed: {status}: {text}"
        )));
    }

    let envelope: serde_json::Value = serde_json::from_str(&text)
        .map_err(|err| CommandError::new(format!("publish failed: parse: {err}: {text}")))?;

    let data = envelope.get("data").unwrap_or(&envelope);
    let url = data
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            CommandError::new(format!("publish failed: missing url in response: {text}"))
        })?
        .to_string();
    let site_id = data
        .get("siteId")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let delete_token = data
        .get("deleteToken")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    Ok(PublishedReport {
        url,
        delete_token,
        site_id,
        provider: "PageDrop.io".to_string(),
    })
}

#[tauri::command]
pub async fn export_analysis_markdown(
    state: State<'_, AppState>,
    analysis_id: String,
) -> Result<String, CommandError> {
    let report = {
        let db = &state.db;
        db.get_report(&analysis_id, None)?
    };
    let Some(report) = report else {
        return Err("analysis not found".into());
    };
    Ok(render_markdown(&report))
}

#[tauri::command]
pub async fn create_analysis(
    state: State<'_, AppState>,
    user_prompt: String,
) -> Result<String, CommandError> {
    let trimmed = user_prompt.trim();
    if trimmed.is_empty() {
        return Err("Enter a research request before starting analysis.".into());
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
    let db = &state.db;
    db.save_analysis(&analysis)?;
    Ok(id)
}

#[tauri::command]
pub async fn set_active_run(
    state: State<'_, AppState>,
    analysis_id: String,
    run_id: String,
) -> Result<(), CommandError> {
    let db = &state.db;
    let analysis = db
        .get_report(&analysis_id, None)?
        .ok_or_else(|| CommandError::new("Analysis not found"))?
        .analysis;
    db.save_analysis(&Analysis {
        active_run_id: Some(run_id),
        updated_at: chrono::Utc::now().to_rfc3339(),
        ..analysis
    })?;
    Ok(())
}

#[tauri::command]
pub async fn get_run_progress(
    state: State<'_, AppState>,
    run_id: String,
) -> Result<Vec<ProgressEventPayload>, CommandError> {
    let db = &state.db;
    Ok(db.get_run_progress(&run_id)?)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn generate_analysis(
    app: AppHandle,
    state: State<'_, AppState>,
    user_prompt: String,
    agent_id: String,
    model_id: Option<String>,
    analysis_id: String,
    run_id: Option<String>,
    on_progress: Channel<ProgressEventPayload>,
) -> Result<GenerateAnalysisResult, CommandError> {
    let trimmed_prompt = user_prompt.trim();
    if trimmed_prompt.is_empty() {
        return Err("Enter a research request before starting analysis.".into());
    }

    let candidates = list_agent_candidates();
    let candidate = candidates
        .iter()
        .find(|candidate| candidate.id == agent_id)
        .or_else(|| candidates.iter().find(|candidate| candidate.available))
        .ok_or_else(|| CommandError::new("No ACP agent is configured. Add one in Settings."))?;

    let command = candidate.command.clone().ok_or_else(|| {
        CommandError::new(format!(
            "Agent '{}' is not available. Configure the binary in Settings or environment.",
            candidate.label
        ))
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
        let db = &state.db;
        db.save_run(&run)?;
        db.set_active_run_if_empty(&analysis_id, &run_id)?;
        db.path().clone()
    };

    let cancel_token = CancellationToken::new();
    {
        let mut active = state.active_runs.lock()?;
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
            let _ = db_clone.append_progress_event(&run_id_clone, &payload);
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
        let mut active = state.active_runs.lock()?;
        active.remove(&run_id);
    }

    match generation_result {
        Ok(_) => {
            let db = &state.db;
            db.update_run_status(&run_id, AnalysisStatus::Completed, None)?;
            db.recompute_analysis_status(&analysis_id)?;
            let _ = on_progress.send(ProgressEventPayload::Completed);
        }
        Err(err) => {
            let is_cancelled = err.downcast_ref::<AcpCancelled>().is_some();
            let message = err.to_string();
            let status = if is_cancelled {
                AnalysisStatus::Cancelled
            } else {
                AnalysisStatus::Failed
            };
            let db = &state.db;
            db.update_run_status(&run_id, status, Some(&message))?;
            db.recompute_analysis_status(&analysis_id)?;
            let _ = on_progress.send(ProgressEventPayload::Error {
                message: message.clone(),
            });
            return Err(message.into());
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
    use std::fmt::Write as _;
    let mut out = String::new();
    let _ = writeln!(out, "# {}\n", report.analysis.title);
    out.push_str("> Research only. Not investment advice.\n\n");
    let _ = writeln!(out, "**Request:** {}\n", report.analysis.user_prompt);
    if let Some(stance) = &report.final_stance {
        let _ = writeln!(
            out,
            "## Final Stance\n\n**{}**, horizon: {}, confidence: {:.0}%\n\n{}\n",
            stance.stance,
            stance.horizon,
            stance.confidence * 100.0,
            stance.summary
        );
    }
    if !report.projections.is_empty() {
        out.push_str("## Projections\n\n");
        for projection in &report.projections {
            let _ = writeln!(
                out,
                "### {} ({}) · horizon {} · current {}\n",
                projection.metric,
                projection.entity_id,
                projection.horizon,
                projection.current_value_label
            );
            let _ = writeln!(out, "**Methodology:** {}\n", projection.methodology);
            for scenario in &projection.scenarios {
                let _ = writeln!(
                    out,
                    "- **{}** → {} ({:+.1}%, probability {:.0}%) — {}",
                    scenario.label,
                    scenario.target_label,
                    scenario.upside_pct * 100.0,
                    scenario.probability * 100.0,
                    scenario.rationale
                );
            }
            if !projection.key_assumptions.is_empty() {
                out.push_str("\n**Key assumptions:**\n");
                for assumption in &projection.key_assumptions {
                    let _ = writeln!(out, "- {assumption}");
                }
            }
            if !projection.evidence_ids.is_empty() {
                let _ = writeln!(out, "\nSources: `{}`", projection.evidence_ids.join("`, `"));
            }
            out.push('\n');
        }
    }
    if !report.artifacts.is_empty() {
        out.push_str("## Structured Evidence\n\n");
        for artifact in &report.artifacts {
            let _ = writeln!(out, "### {}\n\n{}\n", artifact.title, artifact.summary);
            if !artifact.evidence_ids.is_empty() {
                let _ = writeln!(out, "Sources: `{}`\n", artifact.evidence_ids.join("`, `"));
            }
        }
    }
    for block in &report.blocks {
        let _ = writeln!(out, "## {}\n\n{}\n", block.title, block.body);
        if !block.evidence_ids.is_empty() {
            let _ = writeln!(out, "Sources: `{}`\n", block.evidence_ids.join("`, `"));
        }
    }
    if !report.sources.is_empty() {
        out.push_str("## Sources\n\n");
        for source in &report.sources {
            let link = source.url.clone().unwrap_or_default();
            let _ = writeln!(
                out,
                "- `{}` {} ({}, retrieved {})",
                source.id, source.title, link, source.retrieved_at
            );
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_report(title: &str) -> AnalysisReport {
        AnalysisReport {
            analysis: Analysis {
                id: "analysis-1".to_string(),
                title: title.to_string(),
                user_prompt: "Is AAPL a buy?".to_string(),
                intent: AnalysisIntent::SingleEquity,
                status: AnalysisStatus::Completed,
                active_run_id: Some("run-1".to_string()),
                created_at: "2026-04-17T00:00:00Z".to_string(),
                updated_at: "2026-04-17T00:00:00Z".to_string(),
            },
            runs: vec![],
            research_plan: None,
            entities: vec![],
            sources: vec![],
            metrics: vec![],
            artifacts: vec![],
            blocks: vec![],
            final_stance: None,
            projections: vec![],
            counter_theses: vec![],
            uncertainty_entries: vec![],
            methodology_note: None,
            decision_criterion_answers: vec![],
        }
    }

    #[test]
    fn standalone_html_substitutes_report() {
        let report = sample_report("Sample");
        let html = build_standalone_html(&report).expect("build");
        assert!(!html.contains(REPORT_PLACEHOLDER));
        assert!(html.contains("\"analysis-1\""));
    }

    #[test]
    fn safe_filename_sanitises_title() {
        assert_eq!(
            safe_filename("NVDA vs AMD: margins"),
            "NVDA-vs-AMD_-margins.html"
        );
        assert_eq!(safe_filename(""), "bullpen-report.html");
        assert_eq!(safe_filename("  "), "bullpen-report.html");
    }

    #[test]
    fn safe_filename_handles_pure_punctuation_and_unicode() {
        // Pure punctuation: every character maps to '_', then trim_matches
        // strips them all, leaving empty → fallback name.
        assert_eq!(safe_filename("!!!???"), "bullpen-report.html");

        // Non-ASCII letters are not ASCII-alphanumeric, so they become
        // underscores. Trailing underscores are trimmed but interior ones
        // remain.
        assert_eq!(safe_filename("néon"), "n_on.html");
    }

    #[test]
    fn safe_filename_caps_long_inputs_to_eighty_chars_plus_extension() {
        let long = "a".repeat(200);
        let out = safe_filename(&long);
        assert!(
            std::path::Path::new(&out)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("html"))
        );
        // 80 base chars + ".html" = 85
        assert_eq!(out.len(), 85);
        assert!(out.chars().take(80).all(|c| c == 'a'));
    }

    #[test]
    fn standalone_html_escapes_script_close_and_html_comment() {
        let mut report = sample_report("Sample");
        report.analysis.user_prompt =
            "BULLPENPAYLOAD </script><!-- and </Script> noise".to_string();
        let html = build_standalone_html(&report).expect("build");

        // The viewer template legitimately contains </script> tags and HTML
        // comments. Confirm the *injected JSON payload* never carries raw
        // </ or <!-- after escaping: the marker token only appears inside the
        // injected JSON, so the escaped forms must surround it and the
        // originals must be absent from the JSON segment.
        assert!(
            html.contains("BULLPENPAYLOAD <\\/script><\\!--"),
            "expected escaped sequence in html"
        );
        assert!(!html.contains("BULLPENPAYLOAD </script>"));
        assert!(!html.contains("BULLPENPAYLOAD <!--"));
    }

    #[test]
    fn derive_title_falls_back_for_empty_or_whitespace_lines() {
        assert_eq!(derive_title(""), "Stock analysis");
        assert_eq!(derive_title("\n\n"), "Stock analysis");
        assert_eq!(derive_title("   \nrest"), "Stock analysis");
    }

    #[test]
    fn derive_title_returns_short_prompts_unchanged() {
        assert_eq!(derive_title("Buy AAPL?"), "Buy AAPL?");
    }

    #[test]
    fn derive_title_truncates_long_prompts_at_seventy_two_chars() {
        let prompt = "a".repeat(80);
        let out = derive_title(&prompt);
        assert!(out.ends_with("..."));
        assert_eq!(out.chars().count(), 75); // 72 chars + "..."
        assert!(out.starts_with(&"a".repeat(72)));
    }

    #[test]
    fn derive_title_uses_only_first_line() {
        assert_eq!(
            derive_title("Headline goes here\nbody continues below"),
            "Headline goes here"
        );
    }

    #[test]
    fn render_markdown_includes_title_stance_and_disclaimer() {
        use crate::infra::db::Database;
        use crate::infra::db::tests::seed_full_single_equity;
        use std::path::PathBuf;

        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let (run_id, _) = seed_full_single_equity(&db);
        let report = db.get_report("a", Some(&run_id)).unwrap().unwrap();

        let md = render_markdown(&report);
        assert!(md.starts_with(&format!("# {}", report.analysis.title)));
        assert!(md.contains("> Research only. Not investment advice."));
        assert!(md.contains("## Final Stance"));

        // confidence rendered as a whole percent (0.7 → 70%)
        let stance = report.final_stance.as_ref().unwrap();
        let percent = format!("confidence: {:.0}%", stance.confidence * 100.0);
        assert!(md.contains(&percent), "expected {percent:?} in markdown");
    }
}
