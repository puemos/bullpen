use crate::infra::acp::analysis_mcp_server::RunContext;
use handlebars::Handlebars;
use serde_json::json;

pub fn build_analysis_prompt(run: &RunContext) -> anyhow::Result<String> {
    let template = include_str!("analysis_prompt.hbs");
    let handlebars = Handlebars::new();
    handlebars
        .render_template(
            template,
            &json!({
                "analysis_id": run.analysis_id,
                "run_id": run.run_id,
                "user_prompt": run.user_prompt,
                "agent_id": run.agent_id,
            }),
        )
        .map_err(Into::into)
}
