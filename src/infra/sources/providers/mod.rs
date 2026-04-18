pub mod alpha_vantage;
pub mod brave_search;
pub mod finnhub;
pub mod finviz;
pub mod fmp;
pub mod hacker_news;
pub mod newsapi;
pub mod polygon;
pub mod sec_edgar;
pub mod stocktwits;
pub mod tavily;
pub mod yahoo_finance;

use super::provider::SourceError;
use reqwest::{RequestBuilder, Response};

/// Send an HTTP request with one retry on 5xx / connection errors.
///
/// Providers clone the request via a closure so we can retry without
/// reading the builder twice. 10s timeout is inherited from the shared
/// client.
pub async fn send_with_retry(
    build: impl Fn() -> RequestBuilder,
    provider_id: &'static str,
) -> Result<Response, SourceError> {
    let mut last_err: Option<SourceError> = None;
    for attempt in 0..=1 {
        match build().send().await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_server_error() && attempt == 0 {
                    last_err = Some(SourceError::Upstream {
                        status: status.as_u16(),
                        message: "server error, retrying".to_string(),
                    });
                } else if status.as_u16() == 429 {
                    return Err(SourceError::RateLimited(provider_id));
                } else {
                    return Ok(resp);
                }
            }
            Err(err) if attempt == 0 && (err.is_timeout() || err.is_connect()) => {
                last_err = Some(SourceError::Http(err.to_string()));
            }
            Err(err) => return Err(SourceError::Http(err.to_string())),
        }
    }
    Err(last_err.unwrap_or(SourceError::Shape))
}

/// Return the JSON body of a successful response, mapping non-2xx into
/// canonical `SourceError::Upstream` with a truncated body. Bodies are
/// capped at 512 bytes in the error path to avoid leaking keys that some
/// providers echo back in error messages.
pub async fn json_or_upstream(resp: Response) -> Result<serde_json::Value, SourceError> {
    let status = resp.status();
    if status.is_success() {
        resp.json::<serde_json::Value>()
            .await
            .map_err(|err| SourceError::ParseFailed(err.to_string()))
    } else {
        let text = resp.text().await.unwrap_or_default();
        let truncated: String = text.chars().take(512).collect();
        Err(SourceError::Upstream {
            status: status.as_u16(),
            message: truncated,
        })
    }
}
