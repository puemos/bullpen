use async_trait::async_trait;
use serde_json::{Value, json};

use super::super::provider::{
    ProviderCallContext, ProviderDescriptor, SourceCategory, SourceError, SourceProvider,
};
use super::{json_or_upstream, send_with_retry};
use crate::infra::sources::registry::http_client;

const BASE: &str = "https://www.alphavantage.co/query";
const ID: &str = "alpha_vantage";

pub struct AlphaVantageProvider;

#[async_trait]
impl SourceProvider for AlphaVantageProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: ID,
            display_name: "Alpha Vantage",
            category: SourceCategory::Fundamentals,
            requires_key: true,
            default_enabled: false,
            docs_url: "https://www.alphavantage.co/documentation/",
            key_acquisition_url: Some("https://www.alphavantage.co/support/#api-key"),
            rate_limit_hint: Some("25 req/day free · 5 req/min"),
            description: "Fundamentals, income statements, daily OHLC time series.",
        }
    }

    fn tool_description(&self) -> String {
        "Fetch fundamentals or price series from Alpha Vantage. Supports OVERVIEW, \
         INCOME_STATEMENT, BALANCE_SHEET, CASH_FLOW, TIME_SERIES_DAILY, GLOBAL_QUOTE. \
         Watch the 25/day free-tier limit."
            .to_string()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["function", "symbol"],
            "properties": {
                "function": {
                    "type": "string",
                    "enum": [
                        "OVERVIEW", "INCOME_STATEMENT", "BALANCE_SHEET", "CASH_FLOW",
                        "EARNINGS", "TIME_SERIES_DAILY", "TIME_SERIES_WEEKLY",
                        "TIME_SERIES_MONTHLY", "GLOBAL_QUOTE"
                    ]
                },
                "symbol": { "type": "string", "description": "ticker, e.g. AAPL" },
                "outputsize": { "type": "string", "enum": ["compact", "full"], "default": "compact" }
            }
        })
    }

    async fn query(&self, ctx: ProviderCallContext<'_>, args: Value) -> Result<Value, SourceError> {
        let key = ctx.api_key.ok_or(SourceError::MissingKey(ID))?;
        let function = args
            .get("function")
            .and_then(Value::as_str)
            .ok_or_else(|| SourceError::InvalidInput("function required".into()))?;
        let symbol = args
            .get("symbol")
            .and_then(Value::as_str)
            .ok_or_else(|| SourceError::InvalidInput("symbol required".into()))?;

        let mut params: Vec<(&str, String)> = vec![
            ("function", function.to_string()),
            ("symbol", symbol.to_string()),
            ("apikey", key.to_string()),
        ];
        if let Some(outputsize) = args.get("outputsize").and_then(Value::as_str) {
            params.push(("outputsize", outputsize.to_string()));
        }

        let resp = send_with_retry(|| http_client().get(BASE).query(&params), ID).await?;
        let body = json_or_upstream(resp).await?;

        // Alpha Vantage returns 200 with a JSON body like {"Note": "…rate limit…"}
        // or {"Information": "…"} for throttling. Translate those to proper
        // errors so the agent doesn't treat them as valid data.
        if let Some(obj) = body.as_object() {
            if let Some(note) = obj.get("Note").and_then(Value::as_str)
                && note.to_lowercase().contains("limit")
            {
                return Err(SourceError::RateLimited(ID));
            }
            if let Some(info) = obj.get("Information").and_then(Value::as_str)
                && info.to_lowercase().contains("rate")
            {
                return Err(SourceError::RateLimited(ID));
            }
        }
        Ok(body)
    }
}
