use async_trait::async_trait;
use serde_json::{Value, json};

use super::super::provider::{
    ProviderCallContext, ProviderDescriptor, SourceCategory, SourceError, SourceProvider,
};
use super::{json_or_upstream, send_with_retry};
use crate::infra::sources::registry::http_client;

const BASE: &str = "https://newsapi.org/v2";
const ID: &str = "newsapi";

pub struct NewsApiProvider;

#[async_trait]
impl SourceProvider for NewsApiProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: ID,
            display_name: "NewsAPI",
            category: SourceCategory::News,
            requires_key: true,
            default_enabled: false,
            docs_url: "https://newsapi.org/docs",
            key_acquisition_url: Some("https://newsapi.org/register"),
            rate_limit_hint: Some("100 req/day dev · articles 24h+ old only"),
            description: "Broad news aggregator; free tier is delayed but useful for context.",
        }
    }

    fn tool_description(&self) -> String {
        "Search news articles via NewsAPI `everything` endpoint. Note: free tier only returns \
         articles at least 24 hours old."
            .to_string()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["q"],
            "properties": {
                "q": { "type": "string" },
                "from": { "type": "string", "description": "ISO date" },
                "to": { "type": "string", "description": "ISO date" },
                "language": { "type": "string", "default": "en" },
                "sort_by": { "type": "string", "enum": ["publishedAt", "relevancy", "popularity"], "default": "publishedAt" },
                "page_size": { "type": "integer", "minimum": 1, "maximum": 100, "default": 20 }
            }
        })
    }

    async fn query(&self, ctx: ProviderCallContext<'_>, args: Value) -> Result<Value, SourceError> {
        let key = ctx.api_key.ok_or(SourceError::MissingKey(ID))?;
        let q = args
            .get("q")
            .and_then(Value::as_str)
            .ok_or_else(|| SourceError::InvalidInput("q required".into()))?;

        let mut params: Vec<(&str, String)> = vec![
            ("q", q.to_string()),
            (
                "language",
                args.get("language")
                    .and_then(Value::as_str)
                    .unwrap_or("en")
                    .to_string(),
            ),
            (
                "sortBy",
                args.get("sort_by")
                    .and_then(Value::as_str)
                    .unwrap_or("publishedAt")
                    .to_string(),
            ),
            (
                "pageSize",
                args.get("page_size")
                    .and_then(Value::as_u64)
                    .unwrap_or(20)
                    .to_string(),
            ),
        ];
        if let Some(from) = args.get("from").and_then(Value::as_str) {
            params.push(("from", from.to_string()));
        }
        if let Some(to) = args.get("to").and_then(Value::as_str) {
            params.push(("to", to.to_string()));
        }

        let url = format!("{BASE}/everything");
        let resp = send_with_retry(
            || {
                http_client()
                    .get(&url)
                    .header("X-Api-Key", key)
                    .query(&params)
            },
            ID,
        )
        .await?;
        json_or_upstream(resp).await
    }
}
