use super::config::ServerConfig;
use crate::domain::*;
use crate::infra::db::Database;
use pmcp::{SimpleTool, ToolHandler};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashSet;
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

fn parse_confidence(field: &str, value: Option<f64>) -> Result<f64, pmcp::Error> {
    let v = value.ok_or_else(|| {
        pmcp::Error::Validation(format!("{field}: required, got null — commit to a number"))
    })?;
    if v.is_nan() || !(0.0..=1.0).contains(&v) {
        return Err(pmcp::Error::Validation(format!(
            "{field}: must be in [0.0, 1.0], got {v}"
        )));
    }
    Ok(v)
}

fn parse_probability(field: &str, value: f64) -> Result<f64, pmcp::Error> {
    if value.is_nan() || !(0.0..=1.0).contains(&value) {
        return Err(pmcp::Error::Validation(format!(
            "{field}: must be in [0.0, 1.0], got {value}"
        )));
    }
    Ok(value)
}

fn validate_evidence_ids(
    database: &Database,
    run_id: &str,
    field: &str,
    ids: &[String],
) -> Result<(), pmcp::Error> {
    if ids.is_empty() {
        return Ok(());
    }
    let existing = database
        .existing_source_ids(run_id)
        .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
    let unknown: Vec<&str> = ids
        .iter()
        .filter(|id| !existing.contains(id.as_str()))
        .map(|s| s.as_str())
        .collect();
    if !unknown.is_empty() {
        return Err(pmcp::Error::Validation(format!(
            "{field}: unknown evidence_ids {unknown:?}; submit_source them first"
        )));
    }
    Ok(())
}

fn jaccard_similarity(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let tokens = |phrases: &[String]| -> HashSet<String> {
        phrases
            .iter()
            .flat_map(|s| {
                s.to_lowercase()
                    .split(|c: char| !c.is_alphanumeric())
                    .filter(|t| !t.is_empty() && t.len() > 2)
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
            })
            .collect()
    };
    let set_a = tokens(a);
    let set_b = tokens(b);
    if set_a.is_empty() && set_b.is_empty() {
        return 0.0;
    }
    let intersection: HashSet<_> = set_a.intersection(&set_b).collect();
    let union: HashSet<_> = set_a.union(&set_b).collect();
    if union.is_empty() {
        0.0
    } else {
        intersection.len() as f64 / union.len() as f64
    }
}

#[derive(Debug, Deserialize)]
struct SubmitResearchPlanArgs {
    intent: Option<String>,
    summary: String,
    decision_criteria: Vec<String>,
    planned_checks: Vec<String>,
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
                intent,
                summary: input.summary,
                decision_criteria: input.decision_criteria,
                planned_checks: input.planned_checks,
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
        "required": ["summary", "decision_criteria", "planned_checks"],
        "properties": {
            "intent": { "type": "string", "enum": ["single_equity", "compare_equities", "sector_analysis", "macro_theme", "watchlist", "general_research"] },
            "title": { "type": "string" },
            "summary": { "type": "string" },
            "decision_criteria": { "type": "array", "items": { "type": "string" }, "minItems": 1 },
            "planned_checks": { "type": "array", "items": { "type": "string" }, "minItems": 1 }
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
            let confidence = parse_confidence("confidence", input.confidence)?;
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
                confidence,
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
        "required": ["name", "confidence"],
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
    numeric_value: f64,
    unit: Option<String>,
    period: Option<String>,
    as_of: String,
    source_id: String,
    #[serde(default)]
    prior_value: Option<f64>,
    #[serde(default)]
    change_pct: Option<f64>,
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
            if input.numeric_value.is_nan() {
                return Err(pmcp::Error::Validation(
                    "numeric_value: must be a finite number".to_string(),
                ));
            }
            let database = db(&config).map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            validate_evidence_ids(
                &database,
                &context.run_id,
                "source_id",
                std::slice::from_ref(&input.source_id),
            )?;
            let metric = MetricSnapshot {
                id: input.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                run_id: context.run_id,
                entity_id: input.entity_id,
                metric: input.metric,
                numeric_value: input.numeric_value,
                unit: input.unit,
                period: input.period,
                as_of: input.as_of,
                source_id: input.source_id,
                prior_value: input.prior_value,
                change_pct: input.change_pct,
            };
            database
                .save_metric(&metric)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            Ok(json!({ "status": "ok", "metric_id": metric.id }))
        })
    })
    .with_description("Submit a normalized market, fundamental, valuation, or macro metric with source and as_of metadata. When a prior-period value is known, include prior_value and change_pct so deltas render visually.")
    .with_schema(json!({
        "type": "object",
        "required": ["metric", "numeric_value", "as_of", "source_id"],
        "properties": {
            "id": { "type": "string" },
            "entity_id": { "type": "string" },
            "metric": { "type": "string" },
            "numeric_value": { "type": "number" },
            "unit": { "type": "string" },
            "period": { "type": "string" },
            "as_of": { "type": "string" },
            "source_id": { "type": "string" },
            "prior_value": { "type": "number" },
            "change_pct": { "type": "number" }
        }
    }))
}

#[derive(Debug, Deserialize)]
struct SubmitStructuredArtifactArgs {
    id: Option<String>,
    kind: String,
    title: String,
    summary: String,
    columns: Vec<ArtifactColumn>,
    rows: Vec<Value>,
    series: Option<Vec<ArtifactSeries>>,
    evidence_ids: Vec<String>,
    display_order: Option<i64>,
}

pub fn create_submit_structured_artifact_tool(config: Arc<ServerConfig>) -> impl ToolHandler {
    SimpleTool::new("submit_structured_artifact", move |args: Value, _extra| {
        let config = config.clone();
        Box::pin(async move {
            let input: SubmitStructuredArtifactArgs = serde_json::from_value(args)
                .map_err(|err| pmcp::Error::Validation(err.to_string()))?;
            let context = config
                .load_context()
                .map_err(|err| pmcp::Error::InvalidState(err.to_string()))?;
            let kind = ArtifactKind::from_str(&input.kind).unwrap_or_default();
            let database = db(&config).map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            validate_evidence_ids(
                &database,
                &context.run_id,
                "evidence_ids",
                &input.evidence_ids,
            )?;

            if kind == ArtifactKind::ScenarioMatrix {
                validate_scenario_matrix_rows(&input.columns, &input.rows)?;
            }

            let artifact = StructuredArtifact {
                id: input.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                run_id: context.run_id,
                kind,
                title: input.title,
                summary: input.summary,
                columns: input.columns,
                rows: input.rows,
                series: input.series.unwrap_or_default(),
                evidence_ids: input.evidence_ids,
                display_order: input.display_order.unwrap_or(50),
                created_at: now(),
            };
            database
                .save_structured_artifact(&artifact)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            Ok(json!({ "status": "ok", "artifact_id": artifact.id }))
        })
    })
    .with_description("Submit a source-backed table, comparison matrix, scenario matrix, or lightweight chart for the report. Use area_chart for growth curves and margin-expansion trends that benefit from a filled visual.")
    .with_schema(json!({
        "type": "object",
        "required": ["kind", "title", "summary", "columns", "rows", "evidence_ids"],
        "properties": {
            "id": { "type": "string" },
            "kind": { "type": "string", "enum": ["metric_table", "comparison_matrix", "scenario_matrix", "bar_chart", "line_chart", "area_chart"] },
            "title": { "type": "string" },
            "summary": { "type": "string" },
            "columns": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["key", "label"],
                    "properties": {
                        "key": { "type": "string" },
                        "label": { "type": "string" },
                        "unit": { "type": "string" },
                        "description": { "type": "string" }
                    }
                }
            },
            "rows": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": true
                }
            },
            "series": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["label", "points"],
                    "properties": {
                        "label": { "type": "string" },
                        "points": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["label", "value"],
                                "properties": {
                                    "label": { "type": "string" },
                                    "value": { "type": "number" },
                                    "source_id": { "type": "string" },
                                    "metric_id": { "type": "string" }
                                }
                            }
                        }
                    }
                }
            },
            "evidence_ids": { "type": "array", "items": { "type": "string" }, "minItems": 1 },
            "display_order": { "type": "integer" }
        }
    }))
}

fn validate_scenario_matrix_rows(
    columns: &[ArtifactColumn],
    rows: &[Value],
) -> Result<(), pmcp::Error> {
    let has_probability_column = columns.iter().any(|c| c.key == "probability");
    if !has_probability_column {
        return Ok(());
    }
    let sum: f64 = rows
        .iter()
        .filter_map(|row| row.get("probability").and_then(|v| v.as_f64()))
        .sum();
    if (sum - 1.0).abs() > 0.02 {
        return Err(pmcp::Error::Validation(format!(
            "scenario_matrix probability column sums to {sum:.3}; must sum to 1.0 within 0.02"
        )));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct SubmitBlockArgs {
    id: Option<String>,
    kind: String,
    title: String,
    body: String,
    evidence_ids: Option<Vec<String>>,
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
            let confidence = parse_confidence("confidence", input.confidence)?;
            let importance = Importance::from_str(
                input
                    .importance
                    .as_deref()
                    .ok_or_else(|| pmcp::Error::Validation("importance: required".to_string()))?,
            )
            .map_err(pmcp::Error::Validation)?;
            let evidence_ids = input.evidence_ids.unwrap_or_default();
            let database = db(&config).map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            validate_evidence_ids(&database, &context.run_id, "evidence_ids", &evidence_ids)?;
            let block = AnalysisBlock {
                id: input.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                run_id: context.run_id,
                kind: BlockKind::from_str(&input.kind).unwrap_or_default(),
                title: input.title,
                body: input.body,
                evidence_ids,
                confidence,
                importance,
                display_order: input.display_order.unwrap_or(100),
                created_at: now(),
            };
            database
                .save_block(&block)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            Ok(json!({ "status": "ok", "block_id": block.id }))
        })
    })
    .with_description("Submit a readable, source-backed stock analysis block.")
    .with_schema(json!({
        "type": "object",
        "required": ["kind", "title", "body", "confidence", "importance"],
        "properties": {
            "id": { "type": "string" },
            "kind": { "type": "string", "enum": ["thesis", "business_quality", "financials", "valuation", "peer_comparison", "sector_context", "catalysts", "risks", "technical_context", "open_questions", "other"] },
            "title": { "type": "string" },
            "body": { "type": "string" },
            "evidence_ids": { "type": "array", "items": { "type": "string" } },
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
    what_would_change: Vec<String>,
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
            let confidence = parse_confidence("confidence", input.confidence)?;
            let stance = StanceKind::from_str(&input.stance).unwrap_or_default();

            let database = db(&config).map_err(|err| pmcp::Error::Internal(err.to_string()))?;

            // Cross-field coherence: hedge-everywhere pattern.
            if matches!(stance, StanceKind::Bullish) {
                let blocks = database
                    .get_blocks_for(&context.run_id)
                    .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
                let risk_blocks: Vec<_> = blocks
                    .iter()
                    .filter(|b| b.kind == BlockKind::Risks)
                    .collect();
                if !risk_blocks.is_empty()
                    && risk_blocks.iter().all(|b| b.confidence < 0.3)
                {
                    return Err(pmcp::Error::Validation(
                        "stance=bullish but every risks block has confidence < 0.3; either raise risk confidence or choose a different stance".to_string(),
                    ));
                }
            }

            // Cross-field coherence: blocking uncertainty vs. high stance confidence.
            if confidence > 0.8 {
                let uncertainty = database
                    .get_uncertainty_entries(&context.run_id)
                    .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
                if uncertainty.iter().any(|u| u.blocking) {
                    return Err(pmcp::Error::Validation(
                        "stance confidence > 0.8 while a blocking uncertainty ledger entry is open"
                            .to_string(),
                    ));
                }
            }

            // Cross-field coherence: duplicate-framing check.
            let similarity = jaccard_similarity(&input.key_reasons, &input.what_would_change);
            if similarity > 0.6 {
                return Err(pmcp::Error::Validation(format!(
                    "key_reasons and what_would_change overlap too much (Jaccard={similarity:.2}); what would change the view must be distinct from why you hold it"
                )));
            }

            let final_stance = FinalStance {
                id: input.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                run_id: context.run_id,
                stance,
                horizon: input.horizon,
                confidence,
                summary: input.summary,
                key_reasons: input.key_reasons,
                what_would_change: input.what_would_change,
                disclaimer: RESEARCH_DISCLAIMER.to_string(),
                created_at: now(),
            };
            database
                .save_final_stance(&final_stance)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            Ok(json!({ "status": "ok", "stance_id": final_stance.id }))
        })
    })
    .with_description("Submit the final research stance after all evidence and analysis blocks are submitted.")
    .with_schema(json!({
        "type": "object",
        "required": ["stance", "horizon", "confidence", "summary", "key_reasons", "what_would_change"],
        "properties": {
            "id": { "type": "string" },
            "stance": { "type": "string", "enum": ["bullish", "neutral", "bearish", "mixed", "insufficient_data"] },
            "horizon": { "type": "string" },
            "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
            "summary": { "type": "string" },
            "key_reasons": { "type": "array", "items": { "type": "string" }, "minItems": 1 },
            "what_would_change": { "type": "array", "items": { "type": "string" }, "minItems": 1 }
        }
    }))
}

#[derive(Debug, Deserialize)]
struct SubmitProjectionScenarioArgs {
    label: String,
    target_value: f64,
    target_label: Option<String>,
    upside_pct: Option<f64>,
    probability: f64,
    rationale: String,
    #[serde(default)]
    catalysts: Vec<String>,
    #[serde(default)]
    risks: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SubmitProjectionArgs {
    id: Option<String>,
    entity_id: String,
    horizon: String,
    metric: String,
    current_value: f64,
    current_value_label: Option<String>,
    unit: Option<String>,
    scenarios: Vec<SubmitProjectionScenarioArgs>,
    methodology: String,
    key_assumptions: Vec<String>,
    evidence_ids: Vec<String>,
    confidence: Option<f64>,
}

pub fn create_submit_projection_tool(config: Arc<ServerConfig>) -> impl ToolHandler {
    SimpleTool::new("submit_projection", move |args: Value, _extra| {
        let config = config.clone();
        Box::pin(async move {
            let input: SubmitProjectionArgs = serde_json::from_value(args)
                .map_err(|err| pmcp::Error::Validation(err.to_string()))?;
            let context = config
                .load_context()
                .map_err(|err| pmcp::Error::InvalidState(err.to_string()))?;
            let confidence = parse_confidence("confidence", input.confidence)?;
            let current_value = input.current_value;

            let database = db(&config).map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            validate_evidence_ids(
                &database,
                &context.run_id,
                "evidence_ids",
                &input.evidence_ids,
            )?;

            let mut scenarios = Vec::with_capacity(input.scenarios.len());
            for scenario in input.scenarios {
                let label = ScenarioLabel::from_str(&scenario.label)
                    .map_err(pmcp::Error::Validation)?;
                let probability = parse_probability(
                    &format!("scenarios[{label}].probability"),
                    scenario.probability,
                )?;
                let target_label = scenario
                    .target_label
                    .unwrap_or_else(|| format!("{:.2}", scenario.target_value));
                let upside_pct = scenario.upside_pct.unwrap_or_else(|| {
                    if current_value.abs() > f64::EPSILON {
                        (scenario.target_value - current_value) / current_value
                    } else {
                        0.0
                    }
                });
                scenarios.push(ProjectionScenario {
                    label,
                    target_value: scenario.target_value,
                    target_label,
                    upside_pct,
                    probability,
                    rationale: scenario.rationale,
                    catalysts: scenario.catalysts,
                    risks: scenario.risks,
                });
            }

            let prob_sum: f64 = scenarios.iter().map(|s| s.probability).sum();
            if (prob_sum - 1.0).abs() > 0.02 {
                return Err(pmcp::Error::Validation(format!(
                    "scenario probabilities sum to {prob_sum:.3}; must sum to 1.0 within 0.02"
                )));
            }

            let projection = Projection {
                id: input.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                run_id: context.run_id,
                entity_id: input.entity_id,
                horizon: input.horizon,
                metric: input.metric,
                current_value,
                current_value_label: input
                    .current_value_label
                    .unwrap_or_else(|| format!("{current_value:.2}")),
                unit: input.unit.unwrap_or_default(),
                scenarios,
                methodology: input.methodology,
                key_assumptions: input.key_assumptions,
                evidence_ids: input.evidence_ids,
                confidence,
                disclaimer: RESEARCH_DISCLAIMER.to_string(),
                created_at: now(),
            };
            database
                .save_projection(&projection)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            Ok(json!({ "status": "ok", "projection_id": projection.id }))
        })
    })
    .with_description("Submit a forward-looking projection for a single entity with bull/base/bear scenarios, probabilities, and evidence.")
    .with_schema(json!({
        "type": "object",
        "required": ["entity_id", "horizon", "metric", "current_value", "scenarios", "methodology", "key_assumptions", "evidence_ids", "confidence"],
        "properties": {
            "id": { "type": "string" },
            "entity_id": { "type": "string" },
            "horizon": { "type": "string" },
            "metric": { "type": "string" },
            "current_value": { "type": "number" },
            "current_value_label": { "type": "string" },
            "unit": { "type": "string" },
            "methodology": { "type": "string" },
            "key_assumptions": { "type": "array", "items": { "type": "string" }, "minItems": 1 },
            "evidence_ids": { "type": "array", "items": { "type": "string" }, "minItems": 1 },
            "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
            "scenarios": {
                "type": "array",
                "minItems": 3,
                "items": {
                    "type": "object",
                    "required": ["label", "target_value", "probability", "rationale", "catalysts", "risks"],
                    "properties": {
                        "label": { "type": "string", "enum": ["bull", "base", "bear"] },
                        "target_value": { "type": "number" },
                        "target_label": { "type": "string" },
                        "upside_pct": { "type": "number" },
                        "probability": { "type": "number", "minimum": 0, "maximum": 1 },
                        "rationale": { "type": "string" },
                        "catalysts": { "type": "array", "items": { "type": "string" }, "minItems": 1 },
                        "risks": { "type": "array", "items": { "type": "string" }, "minItems": 1 }
                    }
                }
            }
        }
    }))
}

#[derive(Debug, Deserialize)]
struct SubmitCounterThesisArgs {
    stance_against: String,
    summary: String,
    supporting_evidence_ids: Vec<String>,
    why_we_reject_or_partially_accept: String,
    residual_probability: f64,
}

pub fn create_submit_counter_thesis_tool(config: Arc<ServerConfig>) -> impl ToolHandler {
    SimpleTool::new("submit_counter_thesis", move |args: Value, _extra| {
        let config = config.clone();
        Box::pin(async move {
            let input: SubmitCounterThesisArgs = serde_json::from_value(args)
                .map_err(|err| pmcp::Error::Validation(err.to_string()))?;
            let context = config
                .load_context()
                .map_err(|err| pmcp::Error::InvalidState(err.to_string()))?;
            let stance_against = StanceKind::from_str(&input.stance_against).unwrap_or_default();
            let residual_probability =
                parse_probability("residual_probability", input.residual_probability)?;
            let database = db(&config).map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            validate_evidence_ids(
                &database,
                &context.run_id,
                "supporting_evidence_ids",
                &input.supporting_evidence_ids,
            )?;

            let thesis = CounterThesis {
                id: uuid::Uuid::new_v4().to_string(),
                run_id: context.run_id,
                stance_against,
                summary: input.summary,
                supporting_evidence_ids: input.supporting_evidence_ids,
                why_we_reject_or_partially_accept: input.why_we_reject_or_partially_accept,
                residual_probability,
                created_at: now(),
            };
            database
                .save_counter_thesis(&thesis)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            Ok(json!({ "status": "ok", "counter_thesis_id": thesis.id }))
        })
    })
    .with_description("Submit the strongest good-faith case against the direction you plan to take. Required before submit_final_stance for bullish or bearish stances.")
    .with_schema(json!({
        "type": "object",
        "required": ["stance_against", "summary", "supporting_evidence_ids", "why_we_reject_or_partially_accept", "residual_probability"],
        "properties": {
            "stance_against": { "type": "string", "enum": ["bullish", "neutral", "bearish", "mixed", "insufficient_data"] },
            "summary": { "type": "string" },
            "supporting_evidence_ids": { "type": "array", "items": { "type": "string" }, "minItems": 1 },
            "why_we_reject_or_partially_accept": { "type": "string" },
            "residual_probability": { "type": "number", "minimum": 0, "maximum": 1 }
        }
    }))
}

#[derive(Debug, Deserialize)]
struct SubmitUncertaintyArgs {
    question: String,
    why_it_matters: String,
    attempted_resolution: String,
    blocking: bool,
    #[serde(default)]
    related_decision_criterion: Option<String>,
}

pub fn create_submit_uncertainty_ledger_tool(config: Arc<ServerConfig>) -> impl ToolHandler {
    SimpleTool::new("submit_uncertainty_ledger", move |args: Value, _extra| {
        let config = config.clone();
        Box::pin(async move {
            let input: SubmitUncertaintyArgs = serde_json::from_value(args)
                .map_err(|err| pmcp::Error::Validation(err.to_string()))?;
            let context = config
                .load_context()
                .map_err(|err| pmcp::Error::InvalidState(err.to_string()))?;
            let database = db(&config).map_err(|err| pmcp::Error::Internal(err.to_string()))?;

            let entry = UncertaintyEntry {
                id: uuid::Uuid::new_v4().to_string(),
                run_id: context.run_id,
                question: input.question,
                why_it_matters: input.why_it_matters,
                attempted_resolution: input.attempted_resolution,
                blocking: input.blocking,
                related_decision_criterion: input.related_decision_criterion,
                created_at: now(),
            };
            database
                .save_uncertainty_entry(&entry)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            Ok(json!({ "status": "ok", "uncertainty_id": entry.id }))
        })
    })
    .with_description("Record an open question the analysis could not resolve with trusted evidence. Mark blocking=true when either answer would flip the stance.")
    .with_schema(json!({
        "type": "object",
        "required": ["question", "why_it_matters", "attempted_resolution", "blocking"],
        "properties": {
            "question": { "type": "string" },
            "why_it_matters": { "type": "string" },
            "attempted_resolution": { "type": "string" },
            "blocking": { "type": "boolean" },
            "related_decision_criterion": { "type": "string" }
        }
    }))
}

#[derive(Debug, Deserialize)]
struct SubmitMethodologyArgs {
    approach: String,
    frameworks: Vec<String>,
    data_windows: Vec<String>,
    known_limitations: Vec<String>,
}

pub fn create_submit_methodology_note_tool(config: Arc<ServerConfig>) -> impl ToolHandler {
    SimpleTool::new("submit_methodology_note", move |args: Value, _extra| {
        let config = config.clone();
        Box::pin(async move {
            let input: SubmitMethodologyArgs = serde_json::from_value(args)
                .map_err(|err| pmcp::Error::Validation(err.to_string()))?;
            let context = config
                .load_context()
                .map_err(|err| pmcp::Error::InvalidState(err.to_string()))?;
            if input.known_limitations.is_empty() {
                return Err(pmcp::Error::Validation(
                    "known_limitations: required, must name at least one thing this approach cannot detect".to_string(),
                ));
            }
            if input.data_windows.is_empty() {
                return Err(pmcp::Error::Validation(
                    "data_windows: required, must list at least one source window".to_string(),
                ));
            }
            let database = db(&config).map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            let note = MethodologyNote {
                id: uuid::Uuid::new_v4().to_string(),
                run_id: context.run_id,
                approach: input.approach,
                frameworks: input.frameworks,
                data_windows: input.data_windows,
                known_limitations: input.known_limitations,
                created_at: now(),
            };
            database
                .save_methodology_note(&note)
                .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
            Ok(json!({ "status": "ok", "methodology_id": note.id }))
        })
    })
    .with_description("Submit the run-level methodology once: the approach, frameworks, data windows, and known limitations.")
    .with_schema(json!({
        "type": "object",
        "required": ["approach", "frameworks", "data_windows", "known_limitations"],
        "properties": {
            "approach": { "type": "string" },
            "frameworks": { "type": "array", "items": { "type": "string" } },
            "data_windows": { "type": "array", "items": { "type": "string" }, "minItems": 1 },
            "known_limitations": { "type": "array", "items": { "type": "string" }, "minItems": 1 }
        }
    }))
}

#[derive(Debug, Deserialize)]
struct SubmitDecisionCriterionAnswerArgs {
    criterion: String,
    verdict: String,
    summary: String,
    supporting_block_ids: Vec<String>,
    supporting_evidence_ids: Vec<String>,
}

pub fn create_submit_decision_criterion_answer_tool(config: Arc<ServerConfig>) -> impl ToolHandler {
    SimpleTool::new(
        "submit_decision_criterion_answer",
        move |args: Value, _extra| {
            let config = config.clone();
            Box::pin(async move {
                let input: SubmitDecisionCriterionAnswerArgs = serde_json::from_value(args)
                    .map_err(|err| pmcp::Error::Validation(err.to_string()))?;
                let context = config
                    .load_context()
                    .map_err(|err| pmcp::Error::InvalidState(err.to_string()))?;
                let verdict =
                    CriterionVerdict::from_str(&input.verdict).map_err(pmcp::Error::Validation)?;

                let database =
                    db(&config).map_err(|err| pmcp::Error::Internal(err.to_string()))?;
                validate_evidence_ids(
                    &database,
                    &context.run_id,
                    "supporting_evidence_ids",
                    &input.supporting_evidence_ids,
                )?;

                let existing_blocks = database
                    .existing_block_ids(&context.run_id)
                    .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
                let unknown_blocks: Vec<&str> = input
                    .supporting_block_ids
                    .iter()
                    .filter(|id| !existing_blocks.contains(id.as_str()))
                    .map(|s| s.as_str())
                    .collect();
                if !unknown_blocks.is_empty() {
                    return Err(pmcp::Error::Validation(format!(
                        "supporting_block_ids: unknown block ids {unknown_blocks:?}"
                    )));
                }

                let answer = DecisionCriterionAnswer {
                    id: uuid::Uuid::new_v4().to_string(),
                    run_id: context.run_id,
                    criterion: input.criterion,
                    verdict,
                    summary: input.summary,
                    supporting_block_ids: input.supporting_block_ids,
                    supporting_evidence_ids: input.supporting_evidence_ids,
                    created_at: now(),
                };
                database
                    .save_decision_criterion_answer(&answer)
                    .map_err(|err| pmcp::Error::Internal(err.to_string()))?;
                Ok(json!({ "status": "ok", "answer_id": answer.id }))
            })
        },
    )
    .with_description("Close the loop on each decision criterion named in the research plan. Submit exactly one answer per criterion.")
    .with_schema(json!({
        "type": "object",
        "required": ["criterion", "verdict", "summary", "supporting_block_ids", "supporting_evidence_ids"],
        "properties": {
            "criterion": { "type": "string" },
            "verdict": { "type": "string", "enum": ["confirmed", "refuted", "partially_confirmed", "unresolved"] },
            "summary": { "type": "string" },
            "supporting_block_ids": { "type": "array", "items": { "type": "string" } },
            "supporting_evidence_ids": { "type": "array", "items": { "type": "string" } }
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
