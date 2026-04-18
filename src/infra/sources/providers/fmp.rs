use async_trait::async_trait;
use serde_json::{Value, json};

use super::super::provider::{
    ProviderCallContext, ProviderDescriptor, SourceCategory, SourceError, SourceProvider,
};
use super::{json_or_upstream, send_with_retry};
use crate::infra::sources::registry::http_client;

const BASE: &str = "https://financialmodelingprep.com/api/v3";
const ID: &str = "fmp";

pub struct FmpProvider;

#[async_trait]
impl SourceProvider for FmpProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: ID,
            display_name: "Financial Modeling Prep",
            category: SourceCategory::Fundamentals,
            requires_key: true,
            default_enabled: false,
            docs_url: "https://site.financialmodelingprep.com/developer/docs",
            key_acquisition_url: Some(
                "https://site.financialmodelingprep.com/developer/docs/api-keys",
            ),
            rate_limit_hint: Some("250 req/day free"),
            description: "Profile, income statements, quotes; higher free quota than AV.",
        }
    }

    fn tool_description(&self) -> String {
        "Fetch fundamentals, profile, quotes, or financial statements from Financial Modeling Prep."
            .to_string()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["endpoint", "symbol"],
            "properties": {
                "endpoint": {
                    "type": "string",
                    "enum": ["profile", "quote", "income-statement", "balance-sheet-statement", "cash-flow-statement", "ratios"]
                },
                "symbol": { "type": "string" },
                "limit": { "type": "integer", "minimum": 1, "maximum": 40, "default": 5 },
                "period": { "type": "string", "enum": ["annual", "quarter"], "default": "annual" }
            }
        })
    }

    async fn query(&self, ctx: ProviderCallContext<'_>, args: Value) -> Result<Value, SourceError> {
        let key = ctx.api_key.ok_or(SourceError::MissingKey(ID))?;
        let endpoint = args
            .get("endpoint")
            .and_then(Value::as_str)
            .ok_or_else(|| SourceError::InvalidInput("endpoint required".into()))?;
        let symbol = args
            .get("symbol")
            .and_then(Value::as_str)
            .ok_or_else(|| SourceError::InvalidInput("symbol required".into()))?;

        let url = format!("{BASE}/{endpoint}/{symbol}");

        let mut params: Vec<(&str, String)> = vec![("apikey", key.to_string())];
        if let Some(limit) = args.get("limit").and_then(Value::as_u64) {
            params.push(("limit", limit.to_string()));
        }
        if let Some(period) = args.get("period").and_then(Value::as_str) {
            params.push(("period", period.to_string()));
        }

        let resp = send_with_retry(|| http_client().get(&url).query(&params), ID).await?;
        json_or_upstream(resp).await
    }
}
