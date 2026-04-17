mod config;
mod tool;

pub use config::{RunContext, ServerConfig};

use pmcp::{Server, ServerCapabilities};
use std::sync::Arc;

pub async fn run_analysis_mcp_server() -> pmcp::Result<()> {
    let config = Arc::new(ServerConfig::from_args());

    let server = Server::builder()
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
            "finalize_analysis",
            tool::create_finalize_analysis_tool(config.clone()),
        )
        .build()?;

    server.run(pmcp::StdioTransport::new()).await
}
