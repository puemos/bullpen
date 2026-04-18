use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceCategory {
    WebSearch,
    Filings,
    Fundamentals,
    MarketData,
    News,
    Forums,
    Screener,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderDescriptor {
    pub id: &'static str,
    pub display_name: &'static str,
    pub category: SourceCategory,
    pub requires_key: bool,
    pub default_enabled: bool,
    pub docs_url: &'static str,
    pub key_acquisition_url: Option<&'static str>,
    /// Short free-tier notice (e.g. `"25 req/day free"`) rendered beside the key input.
    pub rate_limit_hint: Option<&'static str>,
    /// Explanatory caption rendered under the provider name in Settings.
    pub description: &'static str,
}

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("missing api key for provider {0}")]
    MissingKey(&'static str),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("provider returned error {status}: {message}")]
    Upstream { status: u16, message: String },
    #[error("rate limited by provider {0}")]
    RateLimited(&'static str),
    #[error("parse failed: {0}")]
    ParseFailed(String),
    #[error("http error: {0}")]
    Http(String),
    #[error("unexpected response shape")]
    Shape,
}

impl From<reqwest::Error> for SourceError {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(err.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct ProviderCallContext<'a> {
    pub api_key: Option<&'a str>,
}

/// The `SourceProvider` trait is what ties a provider module into the MCP
/// tool surface. Each provider corresponds to exactly one MCP tool whose
/// name is `<id>_search` (or similar) and whose JSON schema is what the
/// agent sees when deciding how to call it.
#[async_trait]
pub trait SourceProvider: Send + Sync {
    fn descriptor(&self) -> ProviderDescriptor;

    /// The MCP tool name exposed to the agent. Defaults to `"<id>_query"`.
    fn tool_name(&self) -> String {
        format!("{}_query", self.descriptor().id)
    }

    fn tool_description(&self) -> String;

    fn input_schema(&self) -> Value;

    async fn query(&self, ctx: ProviderCallContext<'_>, args: Value) -> Result<Value, SourceError>;
}
