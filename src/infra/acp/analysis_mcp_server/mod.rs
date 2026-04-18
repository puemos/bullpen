mod config;
mod tool;

pub use config::ServerConfig;

use crate::infra::sources;
use pmcp::{Server, ServerCapabilities};
use std::sync::Arc;

pub async fn run_analysis_mcp_server() -> pmcp::Result<()> {
    let config = Arc::new(ServerConfig::from_args());

    let enabled_sources: Vec<String> = match config.load_context() {
        Ok(ctx) => ctx.enabled_sources,
        Err(_) => Vec::new(),
    };

    let mut builder = Server::builder()
        .name("bullpen-analysis")
        .version(env!("CARGO_PKG_VERSION"))
        .capabilities(ServerCapabilities::tools_only())
        .tool(
            "submit_research_plan",
            tool::create_submit_research_plan_tool(config.clone()),
        )
        .tool(
            "submit_entity_resolution",
            tool::create_submit_entity_resolution_tool(config.clone()),
        )
        .tool(
            "submit_source",
            tool::create_submit_source_tool(config.clone()),
        )
        .tool(
            "verify_source_accessibility",
            tool::create_verify_source_accessibility_tool(config.clone()),
        )
        .tool(
            "submit_metric_snapshot",
            tool::create_submit_metric_snapshot_tool(config.clone()),
        )
        .tool(
            "submit_structured_artifact",
            tool::create_submit_structured_artifact_tool(config.clone()),
        )
        .tool(
            "submit_analysis_block",
            tool::create_submit_analysis_block_tool(config.clone()),
        )
        .tool(
            "submit_final_stance",
            tool::create_submit_final_stance_tool(config.clone()),
        )
        .tool(
            "submit_projection",
            tool::create_submit_projection_tool(config.clone()),
        )
        .tool(
            "submit_counter_thesis",
            tool::create_submit_counter_thesis_tool(config.clone()),
        )
        .tool(
            "submit_uncertainty_ledger",
            tool::create_submit_uncertainty_ledger_tool(config.clone()),
        )
        .tool(
            "submit_methodology_note",
            tool::create_submit_methodology_note_tool(config.clone()),
        )
        .tool(
            "submit_decision_criterion_answer",
            tool::create_submit_decision_criterion_answer_tool(config.clone()),
        )
        .tool(
            "submit_holding_review",
            tool::create_submit_holding_review_tool(config.clone()),
        )
        .tool(
            "submit_allocation_review",
            tool::create_submit_allocation_review_tool(config.clone()),
        )
        .tool(
            "submit_portfolio_risk",
            tool::create_submit_portfolio_risk_tool(config.clone()),
        )
        .tool(
            "submit_rebalancing_suggestion",
            tool::create_submit_rebalancing_suggestion_tool(config.clone()),
        )
        .tool(
            "finalize_analysis",
            tool::create_finalize_analysis_tool(config.clone()),
        );

    for provider in sources::all() {
        let d = provider.descriptor();
        if !enabled_sources.iter().any(|id| id == d.id) {
            continue;
        }
        let api_key = config.source_keys.get(d.id).cloned();
        if d.requires_key && api_key.is_none() {
            eprintln!(
                "source '{}' enabled but no api key provided; skipping tool registration",
                d.id
            );
            continue;
        }
        builder = builder.tool(
            provider.tool_name(),
            tool::create_source_tool(provider, api_key),
        );
    }

    let server = builder.build()?;
    server.run(pmcp::StdioTransport::new()).await
}
