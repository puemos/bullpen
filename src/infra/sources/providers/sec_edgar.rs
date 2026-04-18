use async_trait::async_trait;
use serde_json::{Value, json};

use super::super::provider::{
    ProviderCallContext, ProviderDescriptor, SourceCategory, SourceError, SourceProvider,
};
use super::{json_or_upstream, send_with_retry};
use crate::infra::sources::registry::http_client;

const BASE: &str = "https://data.sec.gov";
const ID: &str = "sec_edgar";

/// SEC EDGAR requires a descriptive `User-Agent` with a contact email —
/// anonymous requests are blocked with 403. We concatenate the app name
/// with an optional user-configured contact via the `SEC_EDGAR_USER_AGENT`
/// env var, falling back to a generic string. The Settings helper text
/// instructs users to set this for compliance with SEC fair-access policy.
fn user_agent() -> String {
    std::env::var("SEC_EDGAR_USER_AGENT").unwrap_or_else(|_| {
        format!(
            "Bullpen/{} research-tool (contact: set SEC_EDGAR_USER_AGENT env)",
            env!("CARGO_PKG_VERSION")
        )
    })
}

pub struct SecEdgarProvider;

#[async_trait]
impl SourceProvider for SecEdgarProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: ID,
            display_name: "SEC EDGAR",
            category: SourceCategory::Filings,
            requires_key: false,
            default_enabled: true,
            docs_url: "https://www.sec.gov/os/accessing-edgar-data",
            key_acquisition_url: None,
            rate_limit_hint: Some("10 req/sec · requires User-Agent with contact email"),
            description: "Primary source for U.S. filings: 10-K, 10-Q, 8-K. Set SEC_EDGAR_USER_AGENT to your email.",
        }
    }

    fn tool_name(&self) -> String {
        "sec_edgar_lookup".to_string()
    }

    fn tool_description(&self) -> String {
        "Fetch SEC EDGAR filing metadata or company facts by CIK. Use `endpoint=\"submissions\"` \
         for the filing index and `endpoint=\"companyfacts\"` for XBRL financial concepts."
            .to_string()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["endpoint", "cik"],
            "properties": {
                "endpoint": { "type": "string", "enum": ["submissions", "companyfacts"] },
                "cik": { "type": "string", "description": "10-digit zero-padded CIK, e.g. 0000320193" }
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
        let cik = args
            .get("cik")
            .and_then(Value::as_str)
            .ok_or_else(|| SourceError::InvalidInput("cik required".into()))?;
        let cik_padded = normalize_cik(cik)?;

        let url = match endpoint {
            "submissions" => format!("{BASE}/submissions/CIK{cik_padded}.json"),
            "companyfacts" => format!("{BASE}/api/xbrl/companyfacts/CIK{cik_padded}.json"),
            other => {
                return Err(SourceError::InvalidInput(format!(
                    "unknown endpoint '{other}'"
                )));
            }
        };

        let resp = send_with_retry(
            || {
                http_client()
                    .get(&url)
                    .header("User-Agent", user_agent())
                    .header("Accept", "application/json")
            },
            ID,
        )
        .await?;
        json_or_upstream(resp).await
    }
}

fn normalize_cik(input: &str) -> Result<String, SourceError> {
    let digits: String = input.chars().filter(char::is_ascii_digit).collect();
    if digits.is_empty() || digits.len() > 10 {
        return Err(SourceError::InvalidInput(format!("invalid cik: {input}")));
    }
    Ok(format!("{digits:0>10}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pads_short_cik() {
        assert_eq!(normalize_cik("320193").unwrap(), "0000320193");
        assert_eq!(normalize_cik("CIK0000320193").unwrap(), "0000320193");
    }

    #[test]
    fn rejects_non_digits() {
        assert!(normalize_cik("abc").is_err());
    }
}
