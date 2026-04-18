use async_trait::async_trait;
use serde_json::{Value, json};

use super::super::provider::{
    ProviderCallContext, ProviderDescriptor, SourceCategory, SourceError, SourceProvider,
};
use super::{json_or_upstream, send_with_retry};
use crate::infra::sources::registry::http_client;

const BASE: &str = "https://finnhub.io/api/v1";
const ID: &str = "finnhub";

pub struct FinnhubProvider;

#[async_trait]
impl SourceProvider for FinnhubProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: ID,
            display_name: "Finnhub",
            category: SourceCategory::MarketData,
            requires_key: true,
            default_enabled: false,
            docs_url: "https://finnhub.io/docs/api",
            key_acquisition_url: Some("https://finnhub.io/dashboard"),
            rate_limit_hint: Some("60 req/min free"),
            description: "Company profile, real-time quotes, curated company news feed.",
        }
    }

    fn tool_description(&self) -> String {
        "Fetch a company profile (`profile2`), a real-time `quote`, or recent `company-news` \
         from Finnhub by ticker. `from`/`to` must be yyyy-mm-dd for news."
            .to_string()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["endpoint", "symbol"],
            "properties": {
                "endpoint": { "type": "string", "enum": ["profile2", "quote", "company-news"] },
                "symbol": { "type": "string" },
                "from": { "type": "string", "description": "news from date (YYYY-MM-DD)" },
                "to": { "type": "string", "description": "news to date (YYYY-MM-DD)" }
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

        let (url, param_key) = match endpoint {
            "profile2" => (format!("{BASE}/stock/profile2"), "symbol"),
            "quote" => (format!("{BASE}/quote"), "symbol"),
            "company-news" => (format!("{BASE}/company-news"), "symbol"),
            other => {
                return Err(SourceError::InvalidInput(format!(
                    "unknown endpoint '{other}'"
                )));
            }
        };

        let mut params: Vec<(&str, String)> = vec![(param_key, symbol.to_string())];
        if endpoint == "company-news" {
            let from = args
                .get("from")
                .and_then(Value::as_str)
                .ok_or_else(|| SourceError::InvalidInput("from date required".into()))?;
            let to = args
                .get("to")
                .and_then(Value::as_str)
                .ok_or_else(|| SourceError::InvalidInput("to date required".into()))?;
            params.push(("from", from.to_string()));
            params.push(("to", to.to_string()));
        }

        let resp = send_with_retry(
            || {
                http_client()
                    .get(&url)
                    .header("X-Finnhub-Token", key)
                    .query(&params)
            },
            ID,
        )
        .await?;
        json_or_upstream(resp).await
    }
}
