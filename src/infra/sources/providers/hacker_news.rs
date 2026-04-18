use async_trait::async_trait;
use serde_json::{Value, json};

use super::super::provider::{
    ProviderCallContext, ProviderDescriptor, SourceCategory, SourceError, SourceProvider,
};
use super::{json_or_upstream, send_with_retry};
use crate::infra::sources::registry::http_client;

const BASE: &str = "https://hacker-news.firebaseio.com/v0";
const ID: &str = "hacker_news";

pub struct HackerNewsProvider;

#[async_trait]
impl SourceProvider for HackerNewsProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: ID,
            display_name: "Hacker News",
            category: SourceCategory::Forums,
            requires_key: false,
            default_enabled: false,
            docs_url: "https://github.com/HackerNews/API",
            key_acquisition_url: None,
            rate_limit_hint: Some("no published limit"),
            description: "Tech IPO and product chatter. Useful for finding narratives early.",
        }
    }

    fn tool_description(&self) -> String {
        "Fetch the current `topstories` list (`endpoint=\"topstories\"`) or a specific item by id \
         (`endpoint=\"item\"` with `item_id`). Pair with tavily_search to drill into story URLs."
            .to_string()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["endpoint"],
            "properties": {
                "endpoint": { "type": "string", "enum": ["topstories", "item"] },
                "item_id": { "type": "integer" }
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
        let url = match endpoint {
            "topstories" => format!("{BASE}/topstories.json"),
            "item" => {
                let id = args
                    .get("item_id")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| SourceError::InvalidInput("item_id required".into()))?;
                format!("{BASE}/item/{id}.json")
            }
            other => {
                return Err(SourceError::InvalidInput(format!(
                    "unknown endpoint '{other}'"
                )));
            }
        };

        let resp = send_with_retry(|| http_client().get(&url), ID).await?;
        json_or_upstream(resp).await
    }
}
