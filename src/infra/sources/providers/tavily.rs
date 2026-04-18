use async_trait::async_trait;
use serde_json::{Value, json};

use super::super::provider::{
    ProviderCallContext, ProviderDescriptor, SourceCategory, SourceError, SourceProvider,
};
use super::{json_or_upstream, send_with_retry};
use crate::infra::sources::registry::http_client;

const BASE: &str = "https://api.tavily.com";
const ID: &str = "tavily";

pub struct TavilyProvider;

#[async_trait]
impl SourceProvider for TavilyProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: ID,
            display_name: "Tavily",
            category: SourceCategory::WebSearch,
            requires_key: true,
            default_enabled: false,
            docs_url: "https://docs.tavily.com",
            key_acquisition_url: Some("https://app.tavily.com"),
            rate_limit_hint: Some("credit-based (1/basic, 2/advanced)"),
            description: "AI-native web + news search; returns clean snippets with sources.",
        }
    }

    fn tool_name(&self) -> String {
        "tavily_search".to_string()
    }

    fn tool_description(&self) -> String {
        "Search the live web via Tavily. Good for news, press releases, blog posts, and \
         general-purpose grounding. Prefer this over scraping when a URL is not already known."
            .to_string()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["query"],
            "properties": {
                "query": { "type": "string", "description": "free-text search query" },
                "search_depth": { "type": "string", "enum": ["basic", "advanced"], "default": "basic" },
                "max_results": { "type": "integer", "minimum": 1, "maximum": 20, "default": 5 },
                "include_answer": { "type": "boolean", "default": false },
                "topic": { "type": "string", "enum": ["general", "news", "finance"], "default": "general" }
            }
        })
    }

    async fn query(&self, ctx: ProviderCallContext<'_>, args: Value) -> Result<Value, SourceError> {
        let key = ctx.api_key.ok_or(SourceError::MissingKey(ID))?;
        let query = args
            .get("query")
            .and_then(Value::as_str)
            .ok_or_else(|| SourceError::InvalidInput("query required".into()))?;
        let mut body = json!({
            "query": query,
            "search_depth": args.get("search_depth").and_then(Value::as_str).unwrap_or("basic"),
            "max_results": args.get("max_results").and_then(Value::as_u64).unwrap_or(5),
            "include_answer": args.get("include_answer").and_then(Value::as_bool).unwrap_or(false),
            "topic": args.get("topic").and_then(Value::as_str).unwrap_or("general"),
        });
        if let Some(topic) = body.get("topic").and_then(Value::as_str)
            && topic != "general"
        {
            body["topic"] = json!(topic);
        }

        let url = format!("{BASE}/search");
        let resp = send_with_retry(
            || {
                http_client()
                    .post(&url)
                    .bearer_auth(key)
                    .header("Content-Type", "application/json")
                    .json(&body)
            },
            ID,
        )
        .await?;
        json_or_upstream(resp).await
    }
}
