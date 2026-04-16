use super::config::ServerConfig;
use crate::domain::*;
use crate::infra::db::Database;
use pmcp::{SimpleTool, ToolHandler};
use serde::Deserialize;
use serde_json::{Value, json};
use std::str::FromStr;
use std::sync::Arc;

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn db(config: &ServerConfig) -> anyhow::Result<Database> {
    match &config.db_path {
        Some(path) => Database::open_at(path.clone()),
        None => Database::open(),
    }
}

fn clamp_confidence(value: Option<f64>) -> f64 {
    value.unwrap_or(0.75).clamp(0.0, 1.0)
}

#[derive(Debug, Deserialize)]
struct SubmitResearchPlanArgs {
    intent: Option<String>,
    summary: String,
    planned_checks: Vec<String>,
    required_blocks: Vec<String>,
    title: Option<String>,
}

pub fn create_submit_research_plan_tool(config: Arc<ServerConfig>) -> impl ToolHandler {
    SimpleTool::new("submit_research_plan", move |args: Value, _extra| {
        let config = config.clone();
        Box::pin(async move {
            let input: SubmitResearchPlanArgs = serde_json::from_value(args)
                .map_err(|err| pmcp::Error::Validation(err.to_string()))?;
            let context = config
                .load_context()
                .map_err(|err| pmcp::Error::InvalidState(err.to_string()))?;
            let intent = input
                .intent
                .as_deref()
                .and_then(|v| AnalysisIntent::from_str(v).ok())
                .unwrap_or_default();
            let plan = ResearchPlan {
                id: uuid::Uuid::new_v4().to_string(),
                run_id: context.run_id.clone(),
                intent: intent.clone(),
                summary: input.summary,
                planned_checks: input.planned_checks,
                required_blocks: input.required_blocks,
                created_at: now(),
            };
            let database = db(&config).map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            database
                .save_research_plan(&plan)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            database
                .update_analysis_metadata(&context.analysis_id, input.title.as_deref(), Some(intent))
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            Ok(json!({ "status": "ok", "plan_id": plan.id }))
        })
    })
    .with_description("Submit the interpreted research plan before doing the analysis.")
    .with_schema(json!({
        "type": "object",
        "required": ["summary", "planned_checks", "required_blocks"],
        "properties": {
            "intent": { "type": "string", "enum": ["single_equity", "compare_equities", "sector_analysis", "macro_theme", "watchlist", "general_research"] },
            "title": { "type": "string" },
            "summary": { "type": "string" },
            "planned_checks": { "type": "array", "items": { "type": "string" } },
            "required_blocks": { "type": "array", "items": { "type": "string" } }
        }
    }))
}

#[derive(Debug, Deserialize)]
struct SubmitEntityArgs {
    id: Option<String>,
    symbol: Option<String>,
    name: String,
    exchange: Option<String>,
    asset_type: Option<String>,
    sector: Option<String>,
    country: Option<String>,
    confidence: Option<f64>,
    resolution_notes: Option<String>,
}

pub fn create_submit_entity_resolution_tool(config: Arc<ServerConfig>) -> impl ToolHandler {
    SimpleTool::new("submit_entity_resolution", move |args: Value, _extra| {
        let config = config.clone();
        Box::pin(async move {
            let input: SubmitEntityArgs = serde_json::from_value(args)
                .map_err(|err| pmcp::Error::Validation(err.to_string()))?;
            let context = config
                .load_context()
                .map_err(|err| pmcp::Error::InvalidState(err.to_string()))?;
            let entity = Entity {
                id: input
                    .id
                    .or_else(|| input.symbol.clone())
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                run_id: context.run_id,
                symbol: input.symbol,
                name: input.name,
                exchange: input.exchange,
                asset_type: input.asset_type.unwrap_or_else(|| "equity".to_string()),
                sector: input.sector,
                country: input.country,
                confidence: clamp_confidence(input.confidence),
                resolution_notes: input.resolution_notes,
            };
            db(&config)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?
                .save_entity(&entity)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            Ok(json!({ "status": "ok", "entity_id": entity.id }))
        })
    })
    .with_description(
        "Resolve a ticker, company, ETF, index, sector, or macro entity before citing it.",
    )
    .with_schema(json!({
        "type": "object",
        "required": ["name"],
        "properties": {
            "id": { "type": "string" },
            "symbol": { "type": "string" },
            "name": { "type": "string" },
            "exchange": { "type": "string" },
            "asset_type": { "type": "string" },
            "sector": { "type": "string" },
            "country": { "type": "string" },
            "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
            "resolution_notes": { "type": "string" }
        }
    }))
}

#[derive(Debug, Deserialize)]
struct SubmitSourceArgs {
    id: Option<String>,
    title: String,
    url: Option<String>,
    publisher: Option<String>,
    source_type: Option<String>,
    retrieved_at: Option<String>,
    as_of: Option<String>,
    reliability: Option<String>,
    summary: String,
}

pub fn create_submit_source_tool(config: Arc<ServerConfig>) -> impl ToolHandler {
    SimpleTool::new("submit_source", move |args: Value, _extra| {
        let config = config.clone();
        Box::pin(async move {
            let input: SubmitSourceArgs = serde_json::from_value(args)
                .map_err(|err| pmcp::Error::Validation(err.to_string()))?;
            let context = config
                .load_context()
                .map_err(|err| pmcp::Error::InvalidState(err.to_string()))?;
            let source = Source {
                id: input.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                run_id: context.run_id,
                title: input.title,
                url: input.url,
                publisher: input.publisher,
                source_type: input.source_type.unwrap_or_else(|| "web".to_string()),
                retrieved_at: input.retrieved_at.unwrap_or_else(now),
                as_of: input.as_of,
                reliability: input
                    .reliability
                    .as_deref()
                    .and_then(|v| SourceReliability::from_str(v).ok())
                    .unwrap_or_default(),
                summary: input.summary,
            };
            db(&config)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?
                .save_source(&source)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            Ok(json!({ "status": "ok", "source_id": source.id }))
        })
    })
    .with_description("Submit a source before citing it from an analysis block or metric.")
    .with_schema(json!({
        "type": "object",
        "required": ["title", "summary"],
        "properties": {
            "id": { "type": "string" },
            "title": { "type": "string" },
            "url": { "type": "string" },
            "publisher": { "type": "string" },
            "source_type": { "type": "string" },
            "retrieved_at": { "type": "string" },
            "as_of": { "type": "string" },
            "reliability": { "type": "string", "enum": ["primary", "high", "medium", "low"] },
            "summary": { "type": "string" }
        }
    }))
}

#[derive(Debug, Deserialize)]
struct SubmitMetricArgs {
    id: Option<String>,
    entity_id: Option<String>,
    metric: String,
    value: String,
    unit: Option<String>,
    period: Option<String>,
    as_of: String,
    source_id: String,
    notes: Option<String>,
}

pub fn create_submit_metric_snapshot_tool(config: Arc<ServerConfig>) -> impl ToolHandler {
    SimpleTool::new("submit_metric_snapshot", move |args: Value, _extra| {
        let config = config.clone();
        Box::pin(async move {
            let input: SubmitMetricArgs = serde_json::from_value(args)
                .map_err(|err| pmcp::Error::Validation(err.to_string()))?;
            let context = config
                .load_context()
                .map_err(|err| pmcp::Error::InvalidState(err.to_string()))?;
            let metric = MetricSnapshot {
                id: input.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                run_id: context.run_id,
                entity_id: input.entity_id,
                metric: input.metric,
                value: input.value,
                unit: input.unit,
                period: input.period,
                as_of: input.as_of,
                source_id: input.source_id,
                notes: input.notes,
            };
            db(&config)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?
                .save_metric(&metric)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            Ok(json!({ "status": "ok", "metric_id": metric.id }))
        })
    })
    .with_description("Submit a normalized market, fundamental, valuation, or macro metric with source and as_of metadata.")
    .with_schema(json!({
        "type": "object",
        "required": ["metric", "value", "as_of", "source_id"],
        "properties": {
            "id": { "type": "string" },
            "entity_id": { "type": "string" },
            "metric": { "type": "string" },
            "value": { "type": "string" },
            "unit": { "type": "string" },
            "period": { "type": "string" },
            "as_of": { "type": "string" },
            "source_id": { "type": "string" },
            "notes": { "type": "string" }
        }
    }))
}

#[derive(Debug, Deserialize)]
struct SubmitBlockArgs {
    id: Option<String>,
    kind: String,
    title: String,
    body: String,
    evidence_ids: Option<Vec<String>>,
    entity_ids: Option<Vec<String>>,
    confidence: Option<f64>,
    importance: Option<String>,
    display_order: Option<i64>,
}

pub fn create_submit_analysis_block_tool(config: Arc<ServerConfig>) -> impl ToolHandler {
    SimpleTool::new("submit_analysis_block", move |args: Value, _extra| {
        let config = config.clone();
        Box::pin(async move {
            let input: SubmitBlockArgs = serde_json::from_value(args)
                .map_err(|err| pmcp::Error::Validation(err.to_string()))?;
            let context = config
                .load_context()
                .map_err(|err| pmcp::Error::InvalidState(err.to_string()))?;
            let block = AnalysisBlock {
                id: input.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                run_id: context.run_id,
                kind: BlockKind::from_str(&input.kind).unwrap_or_default(),
                title: input.title,
                body: input.body,
                evidence_ids: input.evidence_ids.unwrap_or_default(),
                entity_ids: input.entity_ids.unwrap_or_default(),
                confidence: clamp_confidence(input.confidence),
                importance: input.importance.unwrap_or_else(|| "medium".to_string()),
                display_order: input.display_order.unwrap_or(100),
                created_at: now(),
            };
            db(&config)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?
                .save_block(&block)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            Ok(json!({ "status": "ok", "block_id": block.id }))
        })
    })
    .with_description("Submit a readable, source-backed stock analysis block.")
    .with_schema(json!({
        "type": "object",
        "required": ["kind", "title", "body"],
        "properties": {
            "id": { "type": "string" },
            "kind": { "type": "string", "enum": ["thesis", "business_quality", "financials", "valuation", "peer_comparison", "sector_context", "catalysts", "risks", "scenario_matrix", "technical_context", "open_questions", "other"] },
            "title": { "type": "string" },
            "body": { "type": "string" },
            "evidence_ids": { "type": "array", "items": { "type": "string" } },
            "entity_ids": { "type": "array", "items": { "type": "string" } },
            "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
            "importance": { "type": "string", "enum": ["high", "medium", "low"] },
            "display_order": { "type": "integer" }
        }
    }))
}

#[derive(Debug, Deserialize)]
struct SubmitFinalStanceArgs {
    id: Option<String>,
    stance: String,
    horizon: String,
    confidence: Option<f64>,
    summary: String,
    key_reasons: Vec<String>,
    watch_items: Vec<String>,
    what_would_change: Vec<String>,
    disclaimer: Option<String>,
}

pub fn create_submit_final_stance_tool(config: Arc<ServerConfig>) -> impl ToolHandler {
    SimpleTool::new("submit_final_stance", move |args: Value, _extra| {
        let config = config.clone();
        Box::pin(async move {
            let input: SubmitFinalStanceArgs = serde_json::from_value(args)
                .map_err(|err| pmcp::Error::Validation(err.to_string()))?;
            let context = config
                .load_context()
                .map_err(|err| pmcp::Error::InvalidState(err.to_string()))?;
            let stance = FinalStance {
                id: input.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                run_id: context.run_id,
                stance: StanceKind::from_str(&input.stance).unwrap_or_default(),
                horizon: input.horizon,
                confidence: clamp_confidence(input.confidence),
                summary: input.summary,
                key_reasons: input.key_reasons,
                watch_items: input.watch_items,
                what_would_change: input.what_would_change,
                disclaimer: input
                    .disclaimer
                    .unwrap_or_else(|| "Research only. Not investment advice.".to_string()),
                created_at: now(),
            };
            db(&config)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?
                .save_final_stance(&stance)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            Ok(json!({ "status": "ok", "stance_id": stance.id }))
        })
    })
    .with_description("Submit the final research stance after all evidence and analysis blocks are submitted.")
    .with_schema(json!({
        "type": "object",
        "required": ["stance", "horizon", "summary", "key_reasons", "watch_items", "what_would_change"],
        "properties": {
            "id": { "type": "string" },
            "stance": { "type": "string", "enum": ["bullish", "neutral", "bearish", "mixed", "insufficient_data"] },
            "horizon": { "type": "string" },
            "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
            "summary": { "type": "string" },
            "key_reasons": { "type": "array", "items": { "type": "string" } },
            "watch_items": { "type": "array", "items": { "type": "string" } },
            "what_would_change": { "type": "array", "items": { "type": "string" } },
            "disclaimer": { "type": "string" }
        }
    }))
}

pub fn create_finalize_analysis_tool(config: Arc<ServerConfig>) -> impl ToolHandler {
    SimpleTool::new("finalize_analysis", move |_args: Value, _extra| {
        let config = config.clone();
        Box::pin(async move {
            let context = config
                .load_context()
                .map_err(|err| pmcp::Error::InvalidState(err.to_string()))?;
            let database = db(&config).map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            let errors = database
                .validate_finalization(&context.run_id)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            if !errors.is_empty() {
                return Err(pmcp::Error::Validation(format!(
                    "analysis is not ready to finalize: {}",
                    errors.join("; ")
                )));
            }
            database
                .update_run_status(&context.run_id, AnalysisStatus::Completed, None)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            database
                .update_analysis_status(&context.analysis_id, AnalysisStatus::Completed)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            Ok(json!({ "status": "ok", "message": "analysis finalized" }))
        })
    })
    .with_description("Finalize the report after all required structured blocks and the final stance have been submitted.")
    .with_schema(json!({
        "type": "object",
        "properties": {}
    }))
}
