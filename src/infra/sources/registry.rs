use std::sync::OnceLock;
use std::time::Duration;

use super::provider::SourceProvider;
use super::providers;

/// Shared HTTP client used by every provider. Built once — providers should
/// never instantiate their own client. 10s default timeout matches the
/// source-verification probe; retries on 5xx are handled at the call site
/// (`fetch_with_retry` in the providers module) rather than being baked in,
/// because GET and POST semantics differ.
pub fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent(concat!("Bullpen/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("reqwest client")
    })
}

/// Ordered list of built-in providers. First-release shipping set — the
/// order here is the order Settings renders within each category group.
#[must_use]
pub fn all() -> Vec<&'static dyn SourceProvider> {
    vec![
        &providers::tavily::TavilyProvider,
        &providers::brave_search::BraveSearchProvider,
        &providers::sec_edgar::SecEdgarProvider,
        &providers::alpha_vantage::AlphaVantageProvider,
        &providers::fmp::FmpProvider,
        &providers::finnhub::FinnhubProvider,
        &providers::polygon::PolygonProvider,
        &providers::newsapi::NewsApiProvider,
        &providers::finviz::FinvizProvider,
        &providers::stocktwits::StocktwitsProvider,
        &providers::hacker_news::HackerNewsProvider,
        &providers::yahoo_finance::YahooFinanceProvider,
    ]
}

#[must_use]
pub fn get(id: &str) -> Option<&'static dyn SourceProvider> {
    all().into_iter().find(|p| p.descriptor().id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn provider_ids_are_unique() {
        let mut seen = HashSet::new();
        for p in all() {
            assert!(
                seen.insert(p.descriptor().id),
                "duplicate provider id: {}",
                p.descriptor().id
            );
        }
    }

    #[test]
    fn provider_ids_are_snake_case() {
        for p in all() {
            let id = p.descriptor().id;
            assert!(
                id.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "non-snake_case id: {id}"
            );
        }
    }

    #[test]
    fn tool_names_are_unique() {
        let mut seen = HashSet::new();
        for p in all() {
            let name = p.tool_name();
            assert!(seen.insert(name.clone()), "duplicate tool name: {name}");
        }
    }
}
