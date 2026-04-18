use async_trait::async_trait;
use scraper::{Html, Selector};
use serde::Serialize;
use serde_json::{Value, json};

use super::super::provider::{
    ProviderCallContext, ProviderDescriptor, SourceCategory, SourceError, SourceProvider,
};
use super::send_with_retry;
use crate::infra::sources::registry::http_client;

const BASE: &str = "https://finviz.com";
const ID: &str = "finviz";

pub struct FinvizProvider;

#[async_trait]
impl SourceProvider for FinvizProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: ID,
            display_name: "Finviz",
            category: SourceCategory::Screener,
            requires_key: false,
            default_enabled: false,
            docs_url: "https://finviz.com/help/screener.ashx",
            key_acquisition_url: Some("https://finviz.com/elite.ashx"),
            rate_limit_hint: Some("HTML scrape · cap a few req/min"),
            description: "Community-favorite snapshot: valuation + profitability + technicals in one fetch. \
                 Parsed server-side to a flat JSON shape.",
        }
    }

    fn tool_description(&self) -> String {
        "Fetch the Finviz quote snapshot for a ticker. Returns parsed \
         `{ symbol, company_name, sector, industry, country, exchange, last_price, \
         metrics: {P/E, EPS (ttm), Market Cap, …}, description }`. \
         Finviz's HTML can shift without notice — if this returns `parse_failed`, fall back \
         to another fundamentals source and cite that instead."
            .to_string()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["symbol"],
            "properties": {
                "symbol": { "type": "string" }
            }
        })
    }

    async fn query(&self, ctx: ProviderCallContext<'_>, args: Value) -> Result<Value, SourceError> {
        let symbol = args
            .get("symbol")
            .and_then(Value::as_str)
            .ok_or_else(|| SourceError::InvalidInput("symbol required".into()))?;

        let mut url = format!("{BASE}/quote.ashx?t={symbol}");
        if let Some(key) = ctx.api_key {
            url.push_str("&auth=");
            url.push_str(key);
        }

        let resp = send_with_retry(
            || {
                http_client().get(&url).header(
                    "User-Agent",
                    format!("Bullpen/{} research-tool", env!("CARGO_PKG_VERSION")),
                )
            },
            ID,
        )
        .await?;
        let status = resp.status();
        if !status.is_success() {
            return Err(SourceError::Upstream {
                status: status.as_u16(),
                message: "finviz non-2xx".to_string(),
            });
        }
        let html = resp
            .text()
            .await
            .map_err(|err| SourceError::ParseFailed(err.to_string()))?;

        let parsed = parse_quote_page(&html, symbol)?;
        serde_json::to_value(parsed).map_err(|err| SourceError::ParseFailed(err.to_string()))
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ParsedQuote {
    pub symbol: String,
    pub company_name: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub country: Option<String>,
    pub exchange: Option<String>,
    pub last_price: Option<f64>,
    /// Key→value snapshot: keys are Finviz labels as-rendered
    /// (e.g. "P/E", "EPS (ttm)", "Market Cap"). Values are returned as
    /// trimmed strings so the agent can decide how to parse `%`, `B`, `M`,
    /// date strings, and compound forms like `"1.08 (0.40%)"`.
    pub metrics: serde_json::Map<String, Value>,
    pub description: Option<String>,
}

/// Parse a Finviz `quote.ashx` page into a flat, agent-friendly shape.
///
/// This is a load-bearing function: we purposely do *not* coerce numeric
/// values here. Finviz mixes percentages, magnitudes (`B`/`M`/`K`), dates,
/// and compound strings (`"1.08 (0.40%)"`) in the same column, and the
/// agent is better positioned to interpret each metric in the context of
/// its own reasoning. We trim and preserve the raw string.
pub fn parse_quote_page(html: &str, requested_symbol: &str) -> Result<ParsedQuote, SourceError> {
    let doc = Html::parse_document(html);

    let mut quote = ParsedQuote {
        symbol: requested_symbol.to_uppercase(),
        company_name: None,
        sector: None,
        industry: None,
        country: None,
        exchange: None,
        last_price: None,
        metrics: serde_json::Map::new(),
        description: None,
    };

    // Header: ticker + company name.
    // <h1 data-ticker="AAPL">AAPL</h1>
    // <h2 class="quote-header_ticker-wrapper_company …"><a>Apple Inc</a></h2>
    if let Some(h1) = doc
        .select(&select("h1.quote-header_ticker-wrapper_ticker"))
        .next()
    {
        let text = collapse(&h1.text().collect::<String>());
        if !text.is_empty() {
            quote.symbol = text.to_uppercase();
        }
    }
    if let Some(h2) = doc
        .select(&select("h2.quote-header_ticker-wrapper_company"))
        .next()
    {
        let text = collapse(&h2.text().collect::<String>());
        if !text.is_empty() {
            quote.company_name = Some(text);
        }
    }

    // Classification chips: sector · industry · country · exchange are the
    // first four <a> children of `.quote-links` (after which navigation
    // links like Chart/Compare/Options appear).
    if let Some(chips) = doc.select(&select("div.quote-links")).next() {
        let links: Vec<String> = chips
            .select(&select("a.tab-link"))
            .map(|a| collapse(&a.text().collect::<String>()))
            .filter(|s| !s.is_empty())
            .collect();
        quote.sector = links.first().cloned();
        quote.industry = links.get(1).cloned();
        quote.country = links.get(2).cloned();
        quote.exchange = links.get(3).cloned();
    }

    // Last close price.
    if let Some(el) = doc
        .select(&select("strong.quote-price_wrapper_price"))
        .next()
    {
        let raw = collapse(&el.text().collect::<String>());
        quote.last_price = parse_number(&raw);
    }

    // Snapshot metrics table. Each <td.snapshot-td2> alternates label then
    // value across the row. We pair adjacent label/value cells regardless
    // of which row they fall in — Finviz emits them in a consistent order.
    let cell_sel = select("td.snapshot-td2");
    let label_sel = select("div.snapshot-td-label");
    let value_sel = select("div.snapshot-td-content");

    let mut pending_label: Option<String> = None;
    for cell in doc.select(&cell_sel) {
        if let Some(label_el) = cell.select(&label_sel).next() {
            let text = collapse(&label_el.text().collect::<String>());
            if !text.is_empty() {
                pending_label = Some(text);
            }
        } else if let Some(value_el) = cell.select(&value_sel).next()
            && let Some(label) = pending_label.take()
        {
            let value = collapse(&value_el.text().collect::<String>());
            if !value.is_empty() && value != "-" {
                quote.metrics.insert(label, Value::String(value));
            }
        }
    }

    if quote.metrics.is_empty() {
        return Err(SourceError::ParseFailed(
            "no metrics found — Finviz markup may have changed".to_string(),
        ));
    }

    // Company bio (single long paragraph). Finviz emits two `.quote_profile`
    // cells on the page — the first contains the bio; the second is a funds
    // placeholder that's often empty.
    for el in doc.select(&select("td.fullview-profile div.quote_profile-bio")) {
        let text = collapse(&el.text().collect::<String>());
        if text.len() > 40 {
            quote.description = Some(text);
            break;
        }
    }

    Ok(quote)
}

fn select(css: &str) -> Selector {
    // scraper::Selector::parse fails at compile-time for literal bad CSS,
    // and these literals are hand-written constants. Unwrap is sound.
    Selector::parse(css).expect("hard-coded css selector")
}

/// Collapse whitespace runs and trim.
fn collapse(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = true;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

/// Parse a Finviz-formatted number. Handles magnitude suffixes
/// (`B` / `M` / `K`), percent signs, and plain decimals. Returns `None`
/// for compound strings like `"1.08 (0.40%)"` — the caller already has the
/// raw string in the metrics map.
fn parse_number(s: &str) -> Option<f64> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return None;
    }
    let (body, mult) = match trimmed.chars().last() {
        Some('B') => (&trimmed[..trimmed.len() - 1], 1_000_000_000.0),
        Some('M') => (&trimmed[..trimmed.len() - 1], 1_000_000.0),
        Some('K') => (&trimmed[..trimmed.len() - 1], 1_000.0),
        Some('%') => (&trimmed[..trimmed.len() - 1], 1.0),
        _ => (trimmed, 1.0),
    };
    body.replace(',', "").parse::<f64>().ok().map(|n| n * mult)
}

#[cfg(test)]
mod tests {
    use super::*;

    const AAPL_FIXTURE: &str = include_str!("../../../../tests/fixtures/finviz/aapl.html");

    #[test]
    fn parses_aapl_fixture_header() {
        let quote = parse_quote_page(AAPL_FIXTURE, "aapl").expect("parse");
        assert_eq!(quote.symbol, "AAPL");
        assert_eq!(quote.company_name.as_deref(), Some("Apple Inc"));
        assert_eq!(quote.sector.as_deref(), Some("Technology"));
        assert_eq!(quote.industry.as_deref(), Some("Consumer Electronics"));
        assert_eq!(quote.country.as_deref(), Some("USA"));
        assert_eq!(quote.exchange.as_deref(), Some("NASD"));
    }

    #[test]
    fn parses_aapl_fixture_price() {
        let quote = parse_quote_page(AAPL_FIXTURE, "aapl").expect("parse");
        // Last close at time of capture was 270.23; we match with a tolerance
        // in case the fixture gets refreshed.
        let price = quote.last_price.expect("last_price present");
        assert!((250.0..320.0).contains(&price), "unexpected price: {price}");
    }

    #[test]
    fn parses_aapl_fixture_metrics() {
        let quote = parse_quote_page(AAPL_FIXTURE, "aapl").expect("parse");
        // Hit rate matters more than exact values, since Finviz refreshes
        // daily. Expect the stable keys to be present.
        for key in [
            "P/E",
            "EPS (ttm)",
            "Market Cap",
            "Sales",
            "Dividend TTM",
            "52W High",
            "52W Low",
            "Beta",
            "ROE",
            "Debt/Eq",
            "Prev Close",
        ] {
            assert!(
                quote.metrics.contains_key(key),
                "missing key {key:?}; got {:?}",
                quote.metrics.keys().collect::<Vec<_>>()
            );
        }
        // Percent values keep their `%` suffix (agent interprets).
        let roe = quote
            .metrics
            .get("ROE")
            .and_then(Value::as_str)
            .expect("roe str");
        assert!(roe.ends_with('%'), "roe should retain % suffix: {roe}");
    }

    #[test]
    fn parses_aapl_fixture_description() {
        let quote = parse_quote_page(AAPL_FIXTURE, "aapl").expect("parse");
        let desc = quote.description.as_deref().expect("description");
        assert!(
            desc.to_lowercase().contains("apple"),
            "description should mention apple: {desc}"
        );
        assert!(desc.len() > 80, "description should be substantive");
    }

    #[test]
    fn parse_fails_loudly_on_empty_html() {
        let err = parse_quote_page("<html><body>no data</body></html>", "aapl").unwrap_err();
        match err {
            SourceError::ParseFailed(msg) => {
                assert!(msg.contains("no metrics"), "got: {msg}");
            }
            other => panic!("expected ParseFailed, got {other:?}"),
        }
    }

    #[test]
    fn parse_number_handles_magnitude_suffixes() {
        assert_eq!(parse_number("3967.28B"), Some(3_967_280_000_000.0));
        assert_eq!(parse_number("435.62B"), Some(435_620_000_000.0));
        assert_eq!(parse_number("126.77M"), Some(126_770_000.0));
        assert_eq!(parse_number("1.5K"), Some(1_500.0));
        assert_eq!(parse_number("34.19"), Some(34.19));
        assert_eq!(parse_number("12.5%"), Some(12.5));
        assert_eq!(parse_number("1,234.56"), Some(1234.56));
        assert_eq!(parse_number("-"), None);
        assert_eq!(parse_number(""), None);
    }

    #[test]
    fn collapse_normalises_whitespace() {
        assert_eq!(collapse("  hello \n  world \t  "), "hello world");
        assert_eq!(collapse("one"), "one");
        assert_eq!(collapse(""), "");
    }

    #[test]
    fn serialised_output_is_flat_json() {
        let quote = parse_quote_page(AAPL_FIXTURE, "aapl").expect("parse");
        let value = serde_json::to_value(&quote).expect("serialise");
        // Agent-friendly top-level keys only.
        assert!(value.get("symbol").is_some());
        assert!(value.get("metrics").and_then(Value::as_object).is_some());
        // Never expose raw html.
        assert!(value.get("html").is_none());
    }
}
