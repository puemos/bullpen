use async_trait::async_trait;
use serde_json::{Value, json};

use super::super::provider::{
    ProviderCallContext, ProviderDescriptor, SourceCategory, SourceError, SourceProvider,
};
use super::{json_or_upstream, send_with_retry};
use crate::infra::sources::registry::http_client;

const CHART_BASE: &str = "https://query1.finance.yahoo.com/v8/finance/chart";
const SUMMARY_BASE: &str = "https://query1.finance.yahoo.com/v10/finance/quoteSummary";
const ID: &str = "yahoo_finance";

pub struct YahooFinanceProvider;

#[async_trait]
impl SourceProvider for YahooFinanceProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: ID,
            display_name: "Yahoo Finance (unofficial)",
            category: SourceCategory::MarketData,
            requires_key: false,
            default_enabled: false,
            docs_url: "https://query1.finance.yahoo.com/v8/finance/chart",
            key_acquisition_url: None,
            rate_limit_hint: Some("undocumented · may break"),
            description: "Unofficial — may break. No ToS agreement. Use only when licensed sources are unavailable.",
        }
    }

    fn tool_description(&self) -> String {
        "Fetch a Yahoo Finance chart (OHLCV series) or quoteSummary block for a ticker. \
         Unofficial endpoint — may break without notice. Only use as a last-resort price source."
            .to_string()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["endpoint", "symbol"],
            "properties": {
                "endpoint": { "type": "string", "enum": ["chart", "quoteSummary"] },
                "symbol": { "type": "string" },
                "interval": { "type": "string", "enum": ["1d", "1wk", "1mo"], "default": "1d" },
                "range": { "type": "string", "enum": ["1d", "5d", "1mo", "3mo", "6mo", "1y", "2y", "5y", "10y", "ytd", "max"], "default": "3mo" },
                "modules": { "type": "string", "description": "comma-separated modules for quoteSummary" }
            }
        })
    }

    async fn query(
        &self,
        _ctx: ProviderCallContext<'_>,
        args: Value,
    ) -> Result<Value, SourceError> {
        let endpoint = args
            .get("endpoint")
            .and_then(Value::as_str)
            .ok_or_else(|| SourceError::InvalidInput("endpoint required".into()))?;
        let symbol = args
            .get("symbol")
            .and_then(Value::as_str)
            .ok_or_else(|| SourceError::InvalidInput("symbol required".into()))?;

        let url = match endpoint {
            "chart" => {
                let interval = args.get("interval").and_then(Value::as_str).unwrap_or("1d");
                let range = args.get("range").and_then(Value::as_str).unwrap_or("3mo");
                format!("{CHART_BASE}/{symbol}?interval={interval}&range={range}")
            }
            "quoteSummary" => {
                let modules = args
                    .get("modules")
                    .and_then(Value::as_str)
                    .unwrap_or("summaryDetail,financialData,defaultKeyStatistics");
                format!("{SUMMARY_BASE}/{symbol}?modules={modules}")
            }
            other => {
                return Err(SourceError::InvalidInput(format!(
                    "unknown endpoint '{other}'"
                )));
            }
        };

        let resp = send_with_retry(
            || {
                http_client().get(&url).header(
                    "User-Agent",
                    format!("Bullpen/{} research-tool", env!("CARGO_PKG_VERSION")),
                )
            },
            ID,
        )
        .await?;
        json_or_upstream(resp).await
    }
}
