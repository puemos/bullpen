use async_trait::async_trait;
use serde_json::{Value, json};

use super::super::provider::{
    ProviderCallContext, ProviderDescriptor, SourceCategory, SourceError, SourceProvider,
};
use super::{json_or_upstream, send_with_retry};
use crate::infra::sources::registry::http_client;

const BASE: &str = "https://api.stocktwits.com/api/2";
const ID: &str = "stocktwits";

pub struct StocktwitsProvider;

#[async_trait]
impl SourceProvider for StocktwitsProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: ID,
            display_name: "StockTwits",
            category: SourceCategory::Forums,
            requires_key: false,
            default_enabled: false,
            docs_url: "https://api.stocktwits.com/developers/docs",
            key_acquisition_url: None,
            rate_limit_hint: Some("~200/hr unauthenticated"),
            description: "Retail trader sentiment stream. Signal quality varies; corroborate.",
        }
    }

    fn tool_description(&self) -> String {
        "Fetch the StockTwits public message stream for a ticker (`endpoint=\"symbol\"`) or the \
         currently trending symbols (`endpoint=\"trending\"`). Unauthenticated, rate-limited."
            .to_string()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["endpoint"],
            "properties": {
                "endpoint": { "type": "string", "enum": ["symbol", "trending"] },
                "symbol": { "type": "string", "description": "required when endpoint=symbol" }
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
            "symbol" => {
                let symbol = args
                    .get("symbol")
                    .and_then(Value::as_str)
                    .ok_or_else(|| SourceError::InvalidInput("symbol required".into()))?;
                format!("{BASE}/streams/symbol/{symbol}.json")
            }
            "trending" => format!("{BASE}/trending/symbols.json"),
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
