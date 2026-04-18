use async_trait::async_trait;
use serde_json::{Value, json};

use super::super::provider::{
    ProviderCallContext, ProviderDescriptor, SourceCategory, SourceError, SourceProvider,
};
use super::{json_or_upstream, send_with_retry};
use crate::infra::sources::registry::http_client;

const BASE: &str = "https://api.search.brave.com/res/v1";
const ID: &str = "brave_search";

pub struct BraveSearchProvider;

#[async_trait]
impl SourceProvider for BraveSearchProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: ID,
            display_name: "Brave Search",
            category: SourceCategory::WebSearch,
            requires_key: true,
            default_enabled: false,
            docs_url: "https://api-dashboard.search.brave.com/app/documentation",
            key_acquisition_url: Some("https://api-dashboard.search.brave.com"),
            rate_limit_hint: Some("$5/mo free credits, ≤ 50 qps"),
            description: "Independent web index; good fallback for queries Google/Bing miss.",
        }
    }

    fn tool_name(&self) -> String {
        "brave_search".to_string()
    }

    fn tool_description(&self) -> String {
        "Search the Brave independent web index. Returns organic results. Useful as a second \
         opinion to Tavily."
            .to_string()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["q"],
            "properties": {
                "q": { "type": "string", "description": "search query" },
                "count": { "type": "integer", "minimum": 1, "maximum": 20, "default": 10 },
                "country": { "type": "string", "default": "US" },
                "freshness": { "type": "string", "enum": ["pd", "pw", "pm", "py"], "description": "past day/week/month/year" }
            }
        })
    }

    async fn query(&self, ctx: ProviderCallContext<'_>, args: Value) -> Result<Value, SourceError> {
        let key = ctx.api_key.ok_or(SourceError::MissingKey(ID))?;
        let query = args
            .get("q")
            .and_then(Value::as_str)
            .ok_or_else(|| SourceError::InvalidInput("q required".into()))?;
        let count = args.get("count").and_then(Value::as_u64).unwrap_or(10);
        let country = args.get("country").and_then(Value::as_str).unwrap_or("US");

        let mut params: Vec<(&str, String)> = vec![
            ("q", query.to_string()),
            ("count", count.to_string()),
            ("country", country.to_string()),
        ];
        if let Some(freshness) = args.get("freshness").and_then(Value::as_str) {
            params.push(("freshness", freshness.to_string()));
        }

        let url = format!("{BASE}/web/search");
        let resp = send_with_retry(
            || {
                http_client()
                    .get(&url)
                    .header("X-Subscription-Token", key)
                    .header("Accept", "application/json")
                    .query(&params)
            },
            ID,
        )
        .await?;
        json_or_upstream(resp).await
    }
}
