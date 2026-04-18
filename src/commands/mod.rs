pub mod error;
pub mod update;

pub use error::CommandError;

use crate::domain::{
    Analysis, AnalysisIntent, AnalysisReport, AnalysisRun, AnalysisStatus, AnalysisSummary,
    Portfolio, PortfolioCsvImportInput, PortfolioDetail, PortfolioImportResult, PortfolioSummary,
    RunContext, stance_stale_metric_names,
};
use crate::infra::acp::analysis_generator::{
    AcpCancelled, GenerateAnalysisInput, generate_with_acp,
};
use crate::infra::acp::{AgentCandidate, list_agent_candidates};
use crate::infra::app_config::{AppConfig, load_config, save_config};
use crate::infra::keystore;
use crate::infra::sources::{self, ProviderDescriptor};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

/// Names of metrics whose source is cited by the stance's evidence graph
/// and whose `as_of` is older than the configured cap. The UI banner in the
/// report viewer consumes this directly — the domain logic (evidence walk
/// + threshold) lives in `domain::freshness`, not in TS.
#[tauri::command]
pub async fn get_stance_stale_metrics(
    state: State<'_, AppState>,
    analysis_id: String,
    run_id: Option<String>,
) -> Result<Vec<String>, CommandError> {
    let db = &state.db;
    let Some(report) = db.get_report(&analysis_id, run_id.as_deref())? else {
        return Ok(Vec::new());
    };
    Ok(stance_stale_metric_names(&report, chrono::Utc::now()))
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
pub async fn create_portfolio(
    state: State<'_, AppState>,
    name: String,
    base_currency: String,
) -> Result<Portfolio, CommandError> {
    let db = &state.db;
    Ok(db.create_portfolio(&name, &base_currency)?)
}

#[tauri::command]
pub async fn get_portfolios(
    state: State<'_, AppState>,
) -> Result<Vec<PortfolioSummary>, CommandError> {
    let db = &state.db;
    Ok(db.list_portfolios()?)
}

#[tauri::command]
pub async fn get_portfolio_detail(
    state: State<'_, AppState>,
    portfolio_id: String,
) -> Result<Option<PortfolioDetail>, CommandError> {
    let db = &state.db;
    Ok(db.get_portfolio_detail(&portfolio_id)?)
}

#[tauri::command]
pub async fn import_portfolio_csv(
    state: State<'_, AppState>,
    mut input: PortfolioCsvImportInput,
) -> Result<PortfolioImportResult, CommandError> {
    for row in &mut input.rows {
        if row.name.is_none() {
            if let Some(ref symbol) = row.symbol.clone() {
                if let Ok(Some(name)) =
                    crate::infra::price_history::fetch_symbol_name(symbol, row.market.as_deref())
                        .await
                {
                    row.name = Some(name);
                }
            }
        }
    }
    let db = &state.db;
    Ok(db.import_portfolio_csv(&input)?)
}

#[tauri::command]
pub async fn delete_portfolio(
    state: State<'_, AppState>,
    portfolio_id: String,
) -> Result<(), CommandError> {
    let db = &state.db;
    Ok(db.delete_portfolio(&portfolio_id)?)
}

#[tauri::command]
pub async fn rename_portfolio(
    state: State<'_, AppState>,
    portfolio_id: String,
    name: String,
) -> Result<Portfolio, CommandError> {
    let db = &state.db;
    Ok(db.rename_portfolio(&portfolio_id, &name)?)
}

#[tauri::command]
pub async fn get_price_history(
    symbol: String,
    market: Option<String>,
) -> Result<Vec<f64>, CommandError> {
    use crate::infra::price_history::fetch_price_history;
    // Swallow transient network/API errors — the sparkline is decorative, we
    // don't want a toast every time Yahoo blinks.
    match fetch_price_history(&symbol, market.as_deref()).await {
        Ok(series) => Ok(series),
        Err(err) => {
            log::debug!("price_history fetch failed for {symbol}: {err:#}");
            Ok(Vec::new())
        }
    }
}

#[tauri::command]
pub async fn stop_analysis(state: State<'_, AppState>, run_id: String) -> Result<(), CommandError> {
    let token = {
        let active = state.active_runs.lock()?;
        active.get(&run_id).cloned()
    };
    if let Some(token) = token {
        token.cancel();
    } else {
        log::warn!("stop_analysis: no active run ({run_id})");
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
const PUBLISH_RESPONSE_MAX_BYTES: usize = 1024 * 1024;

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

    let mut response = client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|err| CommandError::new(format!("publish failed: {err}")))?;

    let status = response.status();

    if let Some(declared) = response.content_length()
        && declared > PUBLISH_RESPONSE_MAX_BYTES as u64
    {
        return Err(CommandError::new(format!(
            "publish failed: response too large ({declared} bytes, max {PUBLISH_RESPONSE_MAX_BYTES})"
        )));
    }

    let mut buf = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|err| CommandError::new(format!("publish failed: read body: {err}")))?
    {
        if buf.len() + chunk.len() > PUBLISH_RESPONSE_MAX_BYTES {
            return Err(CommandError::new(format!(
                "publish failed: response exceeded {PUBLISH_RESPONSE_MAX_BYTES} bytes"
            )));
        }
        buf.extend_from_slice(&chunk);
    }

    let text = String::from_utf8(buf)
        .map_err(|err| CommandError::new(format!("publish failed: non-utf8 body: {err}")))?;

    if !status.is_success() {
        return Err(CommandError::new(format!(
            "publish failed: {status}: {text}"
        )));
    }

    parse_publish_envelope(&text).map_err(CommandError::new)
}

fn parse_publish_envelope(text: &str) -> Result<PublishedReport, String> {
    let envelope: serde_json::Value = serde_json::from_str(text)
        .map_err(|err| format!("publish failed: parse: {err}: {text}"))?;

    let data = envelope.get("data").unwrap_or(&envelope);
    let url = data
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("publish failed: missing url in response: {text}"))?
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
    portfolio_id: Option<String>,
) -> Result<String, CommandError> {
    let db = &state.db;

    let (intent, title, effective_prompt, persisted_portfolio_id) = if let Some(portfolio_id) =
        portfolio_id
    {
        let detail = db
            .get_portfolio_detail(&portfolio_id)?
            .ok_or_else(|| CommandError::new("Portfolio not found"))?;
        let trimmed = user_prompt.trim();
        let title = if trimmed.is_empty() {
            format!("Portfolio review — {}", detail.portfolio.name)
        } else {
            derive_title(trimmed)
        };
        let effective_prompt = if trimmed.is_empty() {
            format!(
                "Review the current snapshot of portfolio \"{}\" ({}): concentration, allocation, risk, scenario/stress outcomes, expected-return model, and non-prescriptive rebalancing scenarios.",
                detail.portfolio.name, detail.portfolio.base_currency
            )
        } else {
            trimmed.to_string()
        };
        (
            AnalysisIntent::Portfolio,
            title,
            effective_prompt,
            Some(portfolio_id),
        )
    } else {
        let trimmed = user_prompt.trim();
        if trimmed.is_empty() {
            return Err("Enter a research request before starting analysis.".into());
        }
        (
            AnalysisIntent::GeneralResearch,
            derive_title(trimmed),
            trimmed.to_string(),
            None,
        )
    };

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let analysis = Analysis {
        id: id.clone(),
        title,
        user_prompt: effective_prompt,
        intent,
        status: AnalysisStatus::Running,
        active_run_id: None,
        portfolio_id: persisted_portfolio_id,
        created_at: now.clone(),
        updated_at: now,
    };
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
    enabled_sources: Option<Vec<String>>,
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

    let config = load_config();
    let resolved_sources: Vec<String> = match enabled_sources {
        Some(list) => list
            .into_iter()
            .filter(|id| sources::get(id).is_some())
            .collect(),
        None => config.enabled_sources.iter().cloned().collect(),
    };
    let enabled_sources_json =
        serde_json::to_string(&resolved_sources).unwrap_or_else(|_| "[]".to_string());

    let db_path = {
        let db = &state.db;
        db.with_tx(|tx| {
            tx.execute(
                "INSERT OR REPLACE INTO analysis_runs
                (id, analysis_id, agent_id, model_id, prompt_text, status, started_at, completed_at, error, enabled_sources)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    run.id,
                    run.analysis_id,
                    run.agent_id,
                    run.model_id,
                    run.prompt_text,
                    run.status.to_string(),
                    run.started_at,
                    run.completed_at,
                    run.error,
                    enabled_sources_json,
                ],
            )?;
            tx.execute(
                "UPDATE analyses SET active_run_id = ?1 WHERE id = ?2 AND active_run_id IS NULL",
                rusqlite::params![run_id, analysis_id],
            )?;
            Ok(())
        })?;
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
        enabled_sources: resolved_sources.clone(),
    };

    let mut source_keys: HashMap<String, String> = HashMap::new();
    for id in &resolved_sources {
        let Some(provider) = sources::get(id) else {
            continue;
        };
        if !provider.descriptor().requires_key {
            continue;
        }
        match keystore::get_key(&sources::key_account(id)) {
            Ok(Some(value)) => {
                source_keys.insert(id.clone(), value);
            }
            Ok(None) => {
                let _ = on_progress.send(ProgressEventPayload::Log(format!(
                    "source '{id}' enabled but no API key stored; its tool will be hidden"
                )));
            }
            Err(err) => {
                let _ = on_progress.send(ProgressEventPayload::Log(format!(
                    "source '{id}' keychain error: {err}; its tool will be hidden"
                )));
            }
        }
    }

    let (progress_tx, mut progress_rx) = mpsc::unbounded_channel::<ProgressEventPayload>();
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
        while let Some(payload) = progress_rx.recv().await {
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
            if let Err(err) = db_clone.append_progress_event(&run_id_clone, &payload) {
                log::warn!("progress persist dropped ({run_id_clone}): {err:#}");
            }
            if data_kind.is_some() && coalesce_tx.send(()).is_err() {
                log::warn!(
                    "progress coalesce channel closed; UI will miss data-changed pulse ({run_id_clone})"
                );
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

    let prompt_text = {
        let db = &state.db;
        let analysis = db
            .get_report(&analysis_id, None)?
            .ok_or_else(|| CommandError::new("Analysis not found"))?
            .analysis;
        crate::prompts::build_prompt_for(&analysis, &context, db)?
    };
    let generation_result = generate_with_acp(GenerateAnalysisInput {
        run_context: context,
        prompt_text,
        agent_command: command,
        agent_args,
        model_flag,
        model_env,
        progress_tx: Some(progress_tx),
        mcp_server_binary: None,
        db_path,
        timeout_secs: Some(config.timeout_secs),
        cancel_token: Some(cancel_token),
        source_keys,
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
            use crate::commands::error::CommandErrorKind;
            use crate::infra::acp::analysis_generator::AcpTimeout;

            let kind = if err.downcast_ref::<AcpCancelled>().is_some() {
                CommandErrorKind::Cancelled
            } else if err.downcast_ref::<AcpTimeout>().is_some() {
                CommandErrorKind::Timeout
            } else {
                CommandErrorKind::Internal
            };
            let message = err.to_string();
            let status = match kind {
                CommandErrorKind::Cancelled => AnalysisStatus::Cancelled,
                _ => AnalysisStatus::Failed,
            };
            let db = &state.db;
            db.update_run_status(&run_id, status, Some(&message))?;
            db.recompute_analysis_status(&analysis_id)?;
            let _ = on_progress.send(ProgressEventPayload::Error {
                message: message.clone(),
            });
            return Err(CommandError::with_kind(message, kind));
        }
    }

    Ok(GenerateAnalysisResult {
        analysis_id,
        run_id,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceDescriptor {
    pub id: String,
    pub display_name: String,
    pub category: String,
    pub requires_key: bool,
    pub default_enabled: bool,
    pub docs_url: String,
    pub key_acquisition_url: Option<String>,
    pub rate_limit_hint: Option<String>,
    pub description: String,
    pub has_key: bool,
    pub enabled: bool,
}

fn category_str(c: crate::infra::sources::SourceCategory) -> &'static str {
    use crate::infra::sources::SourceCategory;
    match c {
        SourceCategory::WebSearch => "web_search",
        SourceCategory::Filings => "filings",
        SourceCategory::Fundamentals => "fundamentals",
        SourceCategory::MarketData => "market_data",
        SourceCategory::News => "news",
        SourceCategory::Forums => "forums",
        SourceCategory::Screener => "screener",
    }
}

fn describe(d: &ProviderDescriptor, has_key: bool, enabled: bool) -> SourceDescriptor {
    SourceDescriptor {
        id: d.id.to_string(),
        display_name: d.display_name.to_string(),
        category: category_str(d.category).to_string(),
        requires_key: d.requires_key,
        default_enabled: d.default_enabled,
        docs_url: d.docs_url.to_string(),
        key_acquisition_url: d.key_acquisition_url.map(str::to_string),
        rate_limit_hint: d.rate_limit_hint.map(str::to_string),
        description: d.description.to_string(),
        has_key,
        enabled,
    }
}

#[tauri::command]
pub async fn list_sources() -> Result<Vec<SourceDescriptor>, CommandError> {
    let config = load_config();
    Ok(sources::all()
        .iter()
        .map(|p| {
            let d = p.descriptor();
            let has_key = config.sources_with_keys.contains(d.id);
            let enabled = config.enabled_sources.contains(d.id);
            describe(&d, has_key, enabled)
        })
        .collect())
}

#[tauri::command]
pub async fn refresh_source_key_status() -> Result<Vec<SourceDescriptor>, CommandError> {
    let mut config = load_config();
    let mut result = Vec::new();
    for p in sources::all() {
        let d = p.descriptor();
        let id = d.id;
        let has_key = if d.requires_key {
            keystore::has_key(&sources::key_account(id)).unwrap_or(false)
        } else {
            false
        };
        if has_key {
            config.sources_with_keys.insert(id.to_string());
        } else {
            config.sources_with_keys.remove(id);
        }
        let enabled = config.enabled_sources.contains(id);
        result.push(describe(&d, has_key, enabled));
    }
    save_config(&config)?;
    Ok(result)
}

#[derive(Debug, Deserialize)]
pub struct SetSourceKeyArgs {
    pub provider_id: String,
    pub key: String,
}

#[tauri::command]
pub async fn set_source_key(args: SetSourceKeyArgs) -> Result<(), CommandError> {
    let provider = sources::get(&args.provider_id)
        .ok_or_else(|| CommandError::new(format!("unknown provider '{}'", args.provider_id)))?;
    let d = provider.descriptor();
    if !d.requires_key {
        return Err(CommandError::new(format!(
            "provider '{}' does not accept an api key",
            d.id
        )));
    }
    let trimmed = args.key.trim();
    if trimmed.is_empty() {
        return Err(CommandError::new("api key is empty"));
    }
    keystore::set_key(&sources::key_account(d.id), trimmed)
        .map_err(|err| CommandError::new(format!("keychain error: {err}")))?;
    let mut config = load_config();
    config.sources_with_keys.insert(d.id.to_string());
    save_config(&config)?;
    Ok(())
}

#[tauri::command]
pub async fn clear_source_key(provider_id: String) -> Result<(), CommandError> {
    let provider = sources::get(&provider_id)
        .ok_or_else(|| CommandError::new(format!("unknown provider '{provider_id}'")))?;
    let d = provider.descriptor();
    keystore::delete_key(&sources::key_account(d.id))
        .map_err(|err| CommandError::new(format!("keychain error: {err}")))?;
    let mut config = load_config();
    config.sources_with_keys.remove(d.id);
    save_config(&config)?;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceKeyTestResult {
    pub status: String,
    pub message: String,
}

#[tauri::command]
pub async fn test_source_key(provider_id: String) -> Result<SourceKeyTestResult, CommandError> {
    use crate::infra::sources::{ProviderCallContext, SourceError};

    let provider = sources::get(&provider_id)
        .ok_or_else(|| CommandError::new(format!("unknown provider '{provider_id}'")))?;
    let d = provider.descriptor();

    let key = if d.requires_key {
        let stored = keystore::get_key(&sources::key_account(d.id))
            .map_err(|err| CommandError::new(format!("keychain error: {err}")))?;
        match stored {
            Some(value) => Some(value),
            None => {
                return Ok(SourceKeyTestResult {
                    status: "missing".into(),
                    message: "no key stored".into(),
                });
            }
        }
    } else {
        None
    };

    let args = test_probe_args(d.id);
    let ctx = ProviderCallContext {
        api_key: key.as_deref(),
    };
    match provider.query(ctx, args).await {
        Ok(_) => Ok(SourceKeyTestResult {
            status: "ok".into(),
            message: "reached provider".into(),
        }),
        Err(SourceError::Upstream { status, .. }) => Ok(SourceKeyTestResult {
            status: status.to_string(),
            message: match status {
                401 | 403 => "auth failed".into(),
                429 => "rate limited".into(),
                _ => format!("http {status}"),
            },
        }),
        Err(SourceError::RateLimited(_)) => Ok(SourceKeyTestResult {
            status: "429".into(),
            message: "rate limited".into(),
        }),
        Err(SourceError::MissingKey(_)) => Ok(SourceKeyTestResult {
            status: "missing".into(),
            message: "no key stored".into(),
        }),
        Err(err) => Ok(SourceKeyTestResult {
            status: "error".into(),
            message: err.to_string(),
        }),
    }
}

/// A minimal, low-cost probe args per provider. Kept here rather than on the
/// trait because test args are a UX-layer concern (we want the cheapest
/// possible call, not a demonstration of capabilities).
fn test_probe_args(id: &str) -> serde_json::Value {
    use serde_json::json;
    match id {
        "tavily" => json!({ "query": "ping", "max_results": 1 }),
        "brave_search" => json!({ "q": "ping", "count": 1 }),
        "sec_edgar" => json!({ "endpoint": "submissions", "cik": "320193" }),
        "alpha_vantage" => json!({ "function": "GLOBAL_QUOTE", "symbol": "IBM" }),
        "fmp" => json!({ "endpoint": "profile", "symbol": "AAPL" }),
        "finnhub" => json!({ "endpoint": "quote", "symbol": "AAPL" }),
        "polygon" => json!({ "endpoint": "ticker_details", "ticker": "AAPL" }),
        "newsapi" => json!({ "q": "markets", "page_size": 1 }),
        "finviz" => json!({ "symbol": "AAPL" }),
        "stocktwits" => json!({ "endpoint": "trending" }),
        "hacker_news" => json!({ "endpoint": "topstories" }),
        "yahoo_finance" => json!({ "endpoint": "chart", "symbol": "AAPL", "range": "5d" }),
        _ => json!({}),
    }
}

#[tauri::command]
pub async fn set_enabled_sources(ids: Vec<String>) -> Result<Vec<String>, CommandError> {
    let valid: std::collections::BTreeSet<String> = ids
        .into_iter()
        .filter(|id| sources::get(id).is_some())
        .collect();
    let mut config = load_config();
    config.enabled_sources.clone_from(&valid);
    save_config(&config)?;
    Ok(valid.into_iter().collect())
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
    if !report.portfolio_scenario_analyses.is_empty()
        || !report.portfolio_expected_return_models.is_empty()
    {
        out.push_str("## Portfolio Outcomes\n\n");
        for analysis in &report.portfolio_scenario_analyses {
            let _ = writeln!(
                out,
                "### Scenario Analysis · {} · confidence {:.0}%\n\n{}\n",
                analysis.horizon,
                analysis.confidence * 100.0,
                analysis.methodology
            );
            for scenario in &analysis.scenarios {
                let _ = writeln!(
                    out,
                    "- **{}**: {:+.1}% return, probability {:.0}% — {}",
                    scenario.label,
                    scenario.portfolio_return_pct * 100.0,
                    scenario.probability * 100.0,
                    scenario.rationale
                );
            }
            if !analysis.stress_cases.is_empty() {
                out.push_str("\n**Stress cases:**\n");
                for stress in &analysis.stress_cases {
                    let _ = writeln!(
                        out,
                        "- **{}**: {:+.1}% — {}",
                        stress.name,
                        stress.estimated_return_pct * 100.0,
                        stress.rationale
                    );
                }
            }
            out.push('\n');
        }
        for model in &report.portfolio_expected_return_models {
            let _ = writeln!(
                out,
                "### Expected-Return Model · {} · {:+.1}%\n\n{}\n",
                model.horizon,
                model.expected_return_pct * 100.0,
                model.summary
            );
            for input in &model.inputs {
                let _ = writeln!(
                    out,
                    "- {} ({}) weight {:.1}% → expected return {:+.1}% — {}",
                    input.name,
                    input.input_type,
                    input.weight * 100.0,
                    input.expected_return_pct * 100.0,
                    input.rationale
                );
            }
            if !model.limitations.is_empty() {
                out.push_str("\n**Limitations:**\n");
                for limitation in &model.limitations {
                    let _ = writeln!(out, "- {limitation}");
                }
            }
            out.push('\n');
        }
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
                // Derive upside from target/current instead of the stored
                // `upside_pct` field — historical rows mixed fraction and
                // percent conventions, so computing fresh is the only way
                // to guarantee the markdown export matches what the viewer
                // shows.
                let upside_fraction = if projection.current_value.abs() > f64::EPSILON {
                    (scenario.target_value - projection.current_value) / projection.current_value
                } else {
                    0.0
                };
                let _ = writeln!(
                    out,
                    "- **{}** → {} ({:+.1}%, probability {:.0}%) — {}",
                    scenario.label,
                    scenario.target_label,
                    upside_fraction * 100.0,
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
                portfolio_id: None,
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
            holding_reviews: vec![],
            allocation_reviews: vec![],
            portfolio_risks: vec![],
            rebalancing_suggestions: vec![],
            portfolio_scenario_analyses: vec![],
            portfolio_expected_return_models: vec![],
        }
    }

    #[test]
    fn parse_publish_envelope_reads_nested_data_form() {
        let body = r#"{"data":{"url":"https://x/a","siteId":"s","deleteToken":"d"}}"#;
        let report = parse_publish_envelope(body).unwrap();
        assert_eq!(report.url, "https://x/a");
        assert_eq!(report.site_id, "s");
        assert_eq!(report.delete_token, "d");
        assert_eq!(report.provider, "PageDrop.io");
    }

    #[test]
    fn parse_publish_envelope_reads_flat_form() {
        let body = r#"{"url":"https://x/a"}"#;
        let report = parse_publish_envelope(body).unwrap();
        assert_eq!(report.url, "https://x/a");
        assert!(report.site_id.is_empty());
        assert!(report.delete_token.is_empty());
    }

    #[test]
    fn parse_publish_envelope_rejects_missing_url() {
        let body = r#"{"data":{"siteId":"s"}}"#;
        let err = parse_publish_envelope(body).unwrap_err();
        assert!(err.contains("missing url"));
    }

    #[test]
    fn parse_publish_envelope_rejects_unrelated_json() {
        let body = r#"{"status":"ok"}"#;
        let err = parse_publish_envelope(body).unwrap_err();
        assert!(err.contains("missing url"));
    }

    #[test]
    fn parse_publish_envelope_rejects_invalid_json() {
        let err = parse_publish_envelope("not json").unwrap_err();
        assert!(err.contains("parse"));
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
