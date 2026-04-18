use async_trait::async_trait;
use serde_json::{Value, json};

use super::super::provider::{
    ProviderCallContext, ProviderDescriptor, SourceCategory, SourceError, SourceProvider,
};
use super::{json_or_upstream, send_with_retry};
use crate::infra::sources::registry::http_client;

const ID: &str = "polygon";

/// Polygon's docs redirected to massive.com during the 2025 rebrand; the
/// REST host at `api.polygon.io` still resolves. The base URL is overridable
/// via `BULLPEN_POLYGON_BASE_URL` so we can patch the default without a full
/// release if they actually move.
fn base_url() -> String {
    std::env::var("BULLPEN_POLYGON_BASE_URL")
        .unwrap_or_else(|_| "https://api.polygon.io".to_string())
}

pub struct PolygonProvider;

#[async_trait]
impl SourceProvider for PolygonProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: ID,
            display_name: "Polygon.io",
            category: SourceCategory::MarketData,
            requires_key: true,
            default_enabled: false,
            docs_url: "https://polygon.io/docs",
            key_acquisition_url: Some("https://polygon.io/dashboard/api-keys"),
            rate_limit_hint: Some("5 req/min free tier"),
            description: "Aggregates, ticker reference, reference data. BULLPEN_POLYGON_BASE_URL overrides host.",
        }
    }

    fn tool_description(&self) -> String {
        "Fetch price aggregates or ticker reference data from Polygon. \
         `endpoint=\"aggregates\"` needs multiplier/timespan/from/to; \
         `endpoint=\"ticker_details\"` needs ticker only."
            .to_string()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["endpoint", "ticker"],
            "properties": {
                "endpoint": { "type": "string", "enum": ["aggregates", "ticker_details"] },
                "ticker": { "type": "string" },
                "multiplier": { "type": "integer", "minimum": 1 },
                "timespan": { "type": "string", "enum": ["minute", "hour", "day", "week", "month", "quarter", "year"] },
                "from": { "type": "string", "description": "YYYY-MM-DD" },
                "to": { "type": "string", "description": "YYYY-MM-DD" }
            }
        })
    }

    async fn query(&self, ctx: ProviderCallContext<'_>, args: Value) -> Result<Value, SourceError> {
        let key = ctx.api_key.ok_or(SourceError::MissingKey(ID))?;
        let endpoint = args
            .get("endpoint")
            .and_then(Value::as_str)
            .ok_or_else(|| SourceError::InvalidInput("endpoint required".into()))?;
        let ticker = args
            .get("ticker")
            .and_then(Value::as_str)
            .ok_or_else(|| SourceError::InvalidInput("ticker required".into()))?;

        let base = base_url();
        let url = match endpoint {
            "ticker_details" => format!("{base}/v3/reference/tickers/{ticker}"),
            "aggregates" => {
                let multiplier = args
                    .get("multiplier")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| SourceError::InvalidInput("multiplier required".into()))?;
                let timespan = args
                    .get("timespan")
                    .and_then(Value::as_str)
                    .ok_or_else(|| SourceError::InvalidInput("timespan required".into()))?;
                let from = args
                    .get("from")
                    .and_then(Value::as_str)
                    .ok_or_else(|| SourceError::InvalidInput("from required".into()))?;
                let to = args
                    .get("to")
                    .and_then(Value::as_str)
                    .ok_or_else(|| SourceError::InvalidInput("to required".into()))?;
                format!("{base}/v2/aggs/ticker/{ticker}/range/{multiplier}/{timespan}/{from}/{to}")
            }
            other => {
                return Err(SourceError::InvalidInput(format!(
                    "unknown endpoint '{other}'"
                )));
            }
        };

        let resp = send_with_retry(|| http_client().get(&url).bearer_auth(key), ID).await?;
        json_or_upstream(resp).await
    }
}
