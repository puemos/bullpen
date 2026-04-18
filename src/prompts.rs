use crate::domain::{Analysis, AnalysisIntent, PortfolioDetail, RunContext};
use crate::infra::db::Database;
use handlebars::Handlebars;
use serde_json::{Value, json};

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

pub fn build_portfolio_analysis_prompt(
    run: &RunContext,
    portfolio: &PortfolioDetail,
) -> anyhow::Result<String> {
    let template = include_str!("portfolio_analysis_prompt.hbs");
    let handlebars = Handlebars::new();

    let total_value: Option<f64> = portfolio
        .holdings
        .iter()
        .try_fold(0.0_f64, |acc, h| h.market_value.map(|v| acc + v));
    let total_value_label = total_value.map_or_else(|| "unknown".to_string(), format_money);

    let holdings: Vec<Value> = portfolio
        .holdings
        .iter()
        .map(|h| {
            let price = match (h.market_value, h.quantity) {
                (Some(mv), q) if q.abs() > f64::EPSILON => Some(mv / q),
                _ => None,
            };
            json!({
                "symbol": h.symbol,
                "market": h.market.clone().unwrap_or_default(),
                "name": h.name.clone().unwrap_or_default(),
                "quantity": format_number(h.quantity),
                "price": price.map_or_else(|| "—".to_string(), format_money),
                "market_value": h.market_value.map_or_else(|| "—".to_string(), format_money),
                "weight_pct": h
                    .allocation_pct
                    .map_or_else(|| "—".to_string(), |p| format!("{:.2}", p * 100.0)),
            })
        })
        .collect();

    let as_of = portfolio
        .import_batches
        .first()
        .map_or_else(|| "unknown".to_string(), |batch| batch.imported_at.clone());

    handlebars
        .render_template(
            template,
            &json!({
                "analysis_id": run.analysis_id,
                "run_id": run.run_id,
                "user_prompt": run.user_prompt,
                "agent_id": run.agent_id,
                "portfolio": {
                    "name": portfolio.portfolio.name,
                    "base_currency": portfolio.portfolio.base_currency,
                },
                "snapshot": {
                    "as_of": as_of,
                    "total_value": total_value_label,
                    "count": portfolio.holdings.len(),
                },
                "holdings": holdings,
            }),
        )
        .map_err(Into::into)
}

pub fn build_prompt_for(
    analysis: &Analysis,
    run: &RunContext,
    db: &Database,
) -> anyhow::Result<String> {
    if analysis.intent == AnalysisIntent::Portfolio
        && let Some(portfolio_id) = analysis.portfolio_id.as_deref()
        && let Some(detail) = db.get_portfolio_detail(portfolio_id)?
    {
        return build_portfolio_analysis_prompt(run, &detail);
    }
    build_analysis_prompt(run)
}

fn format_number(value: f64) -> String {
    if value.fract().abs() < 1e-9 {
        format!("{value:.0}")
    } else {
        format!("{value:.4}")
    }
}

fn format_money(value: f64) -> String {
    if value.abs() >= 1000.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AnalysisStatus, PortfolioCsvImportInput};
    use std::path::PathBuf;

    fn fixture_run() -> RunContext {
        RunContext {
            analysis_id: "a".into(),
            run_id: "run-1".into(),
            agent_id: "fake".into(),
            user_prompt: "Review portfolio".into(),
            created_at: chrono::Utc::now().to_rfc3339(),
            enabled_sources: Vec::new(),
        }
    }

    #[test]
    fn dispatcher_picks_portfolio_template_for_portfolio_intent() {
        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let import_result = db
            .import_portfolio_csv(&PortfolioCsvImportInput {
                portfolio_id: None,
                portfolio_name: Some("Core".into()),
                account_id: None,
                account_name: None,
                institution: None,
                account_type: None,
                base_currency: "USD".into(),
                source_name: "snapshot".into(),
                import_kind: crate::domain::PortfolioImportKind::Positions,
                rows: Vec::new(),
            })
            .unwrap();
        let portfolio_id = import_result.portfolio_id.clone();

        let now = chrono::Utc::now().to_rfc3339();
        let portfolio_analysis = Analysis {
            id: "a-p".into(),
            title: "Portfolio review — Core".into(),
            user_prompt: "Review".into(),
            intent: AnalysisIntent::Portfolio,
            status: AnalysisStatus::Running,
            active_run_id: None,
            portfolio_id: Some(portfolio_id),
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        let generic_analysis = Analysis {
            id: "a-g".into(),
            title: "Analyze AAPL".into(),
            user_prompt: "Analyze AAPL".into(),
            intent: AnalysisIntent::SingleEquity,
            status: AnalysisStatus::Running,
            active_run_id: None,
            portfolio_id: None,
            created_at: now.clone(),
            updated_at: now,
        };

        let run = fixture_run();
        let portfolio_prompt = build_prompt_for(&portfolio_analysis, &run, &db).unwrap();
        assert!(portfolio_prompt.contains("<portfolio>"));
        assert!(portfolio_prompt.contains("submit_holding_review"));

        let generic_prompt = build_prompt_for(&generic_analysis, &run, &db).unwrap();
        assert!(!generic_prompt.contains("<portfolio>"));
        assert!(generic_prompt.contains("submit_research_plan"));
    }
}
