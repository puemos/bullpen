//! Lightweight price-history fetcher used for the portfolio sparklines.
//!
//! Hits Yahoo Finance's public v8 chart endpoint, parses daily closes, and
//! caches results in memory with a 1-hour TTL keyed by (symbol, market). On
//! any network / parse failure we return an empty series so the UI silently
//! drops the sparkline rather than surfacing an error — the chart is a
//! decorative glanceable, not a load-bearing signal.
//!
//! The exchange-suffix mapping is intentionally best-effort: for markets we
//! don't know we fall through to the raw symbol, which is already correct
//! for US listings.

use anyhow::Result;
use serde::Deserialize;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

const CACHE_TTL: Duration = Duration::from_secs(60 * 60);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(8);

struct CacheEntry {
    series: Vec<f64>,
    inserted_at: Instant,
}

type Cache = std::collections::HashMap<String, CacheEntry>;

fn cache() -> &'static Mutex<Cache> {
    static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(Cache::new()))
}

fn cache_key(symbol: &str, market: Option<&str>) -> String {
    format!(
        "{}|{}",
        symbol.to_ascii_uppercase(),
        market.unwrap_or_default().to_ascii_uppercase()
    )
}

fn cache_get(key: &str) -> Option<Vec<f64>> {
    let guard = cache().lock().ok()?;
    let entry = guard.get(key)?;
    if entry.inserted_at.elapsed() > CACHE_TTL {
        return None;
    }
    Some(entry.series.clone())
}

fn cache_put(key: String, series: Vec<f64>) {
    if let Ok(mut guard) = cache().lock() {
        guard.insert(
            key,
            CacheEntry {
                series,
                inserted_at: Instant::now(),
            },
        );
    }
}

/// Map a market/exchange code to Yahoo's ticker suffix. US listings take no
/// suffix; the rest follow Yahoo's documented conventions.
fn yahoo_symbol(symbol: &str, market: Option<&str>) -> String {
    let Some(market) = market.map(str::trim).filter(|m| !m.is_empty()) else {
        return symbol.to_string();
    };
    let code = market.to_ascii_uppercase();
    let suffix = match code.as_str() {
        "XETRA" | "ETR" | "FRA" | "BER" | "DE" => ".DE",
        "LSE" | "LON" | "UK" => ".L",
        "TSX" | "TOR" => ".TO",
        "TSXV" | "VEN" => ".V",
        "SIX" | "SWX" | "CH" => ".SW",
        "BIT" | "MIL" | "MI" => ".MI",
        "EPA" | "PAR" | "FR" => ".PA",
        "AMS" | "NL" => ".AS",
        "EBR" | "BRU" | "BE" => ".BR",
        "BME" | "MAD" | "MC" | "ES" => ".MC",
        "ISE" | "IR" => ".IR",
        "JSE" | "JO" => ".JO",
        "HKEX" | "HKG" | "HK" => ".HK",
        "TSE" | "JPX" | "JP" => ".T",
        "ASX" | "AU" => ".AX",
        "KRX" | "KR" => ".KS",
        "TPE" | "TW" => ".TW",
        "BMV" | "MX" => ".MX",
        "B3" | "SAO" | "BR" => ".SA",
        _ => "",
    };
    format!("{symbol}{suffix}")
}

#[derive(Debug, Deserialize)]
struct ChartResponse {
    chart: Chart,
}

#[derive(Debug, Deserialize)]
struct Chart {
    #[serde(default)]
    result: Option<Vec<ChartResult>>,
}

#[derive(Debug, Deserialize)]
struct ChartMeta {
    #[serde(rename = "shortName")]
    short_name: Option<String>,
    #[serde(rename = "longName")]
    long_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChartResult {
    meta: ChartMeta,
    indicators: Indicators,
}

#[derive(Debug, Deserialize)]
struct Indicators {
    #[serde(default)]
    quote: Vec<Quote>,
}

#[derive(Debug, Deserialize)]
struct Quote {
    #[serde(default)]
    close: Vec<Option<f64>>,
}

struct NameCacheEntry {
    name: Option<String>,
    inserted_at: Instant,
}

type NameCache = std::collections::HashMap<String, NameCacheEntry>;

fn name_cache() -> &'static Mutex<NameCache> {
    static CACHE: OnceLock<Mutex<NameCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(NameCache::new()))
}

#[allow(clippy::option_option)]
fn name_cache_get(key: &str) -> Option<Option<String>> {
    let guard = name_cache().lock().ok()?;
    let entry = guard.get(key)?;
    if entry.inserted_at.elapsed() > CACHE_TTL {
        return None;
    }
    Some(entry.name.clone())
}

fn name_cache_put(key: String, name: Option<String>) {
    if let Ok(mut guard) = name_cache().lock() {
        guard.insert(
            key,
            NameCacheEntry {
                name,
                inserted_at: Instant::now(),
            },
        );
    }
}

/// Fetch a human-readable company/fund name for a symbol.
/// Uses the same Yahoo Finance chart endpoint as sparklines, extracting `meta.shortName`.
/// Returns `None` on any network or parse failure.
pub async fn fetch_symbol_name(symbol: &str, market: Option<&str>) -> Result<Option<String>> {
    let trimmed = symbol.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let key = cache_key(trimmed, market);
    if let Some(cached) = name_cache_get(&key) {
        return Ok(cached);
    }

    let ticker = yahoo_symbol(trimmed, market);
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=1d",
        urlencode(&ticker)
    );

    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/122.0 Safari/537.36",
        )
        .build()?;

    let response = client.get(&url).send().await?;
    if !response.status().is_success() {
        name_cache_put(key, None);
        return Ok(None);
    }

    let body: ChartResponse = response.json().await?;
    let name = body
        .chart
        .result
        .and_then(|r| r.into_iter().next())
        .and_then(|res| res.meta.short_name.or(res.meta.long_name));

    name_cache_put(key, name.clone());
    Ok(name)
}

pub async fn fetch_price_history(symbol: &str, market: Option<&str>) -> Result<Vec<f64>> {
    let trimmed = symbol.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let key = cache_key(trimmed, market);
    if let Some(cached) = cache_get(&key) {
        return Ok(cached);
    }

    let ticker = yahoo_symbol(trimmed, market);
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=1mo",
        urlencode(&ticker)
    );

    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/122.0 Safari/537.36",
        )
        .build()?;

    let response = client.get(&url).send().await?;
    if !response.status().is_success() {
        // Cache empty so we don't hammer the API for known-missing symbols.
        cache_put(key, Vec::new());
        return Ok(Vec::new());
    }

    let body: ChartResponse = response.json().await?;
    let result = body.chart.result.and_then(|r| r.into_iter().next());

    // Populate the name cache for free while we have the response.
    if let Some(ref res) = result {
        let name = res.meta.short_name.clone().or(res.meta.long_name.clone());
        name_cache_put(key.clone(), name);
    }

    let closes: Vec<f64> = result
        .and_then(|res| res.indicators.quote.into_iter().next())
        .map(|quote| quote.close.into_iter().flatten().collect())
        .unwrap_or_default();

    cache_put(key, closes.clone());
    Ok(closes)
}

fn urlencode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn us_listing_takes_no_suffix() {
        assert_eq!(yahoo_symbol("AAPL", Some("NASDAQ")), "AAPL");
        assert_eq!(yahoo_symbol("IBM", Some("NYSE")), "IBM");
    }

    #[test]
    fn european_listings_map_to_yahoo_suffixes() {
        assert_eq!(yahoo_symbol("SAP", Some("XETRA")), "SAP.DE");
        assert_eq!(yahoo_symbol("VOD", Some("LSE")), "VOD.L");
        assert_eq!(yahoo_symbol("NESN", Some("SIX")), "NESN.SW");
    }

    #[test]
    fn unknown_market_falls_through_to_bare_symbol() {
        assert_eq!(yahoo_symbol("AAPL", Some("MYSTERYEX")), "AAPL");
        assert_eq!(yahoo_symbol("AAPL", None), "AAPL");
        assert_eq!(yahoo_symbol("AAPL", Some("   ")), "AAPL");
    }
}
