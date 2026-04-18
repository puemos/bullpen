use crate::domain::PortfolioCsvRow;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CsvField {
    Symbol,
    Market,
    Name,
    AssetType,
    Quantity,
    Price,
    MarketValue,
    CostBasis,
    GrossAmount,
    Fees,
    Taxes,
    Currency,
    TradeDate,
    Action,
    Notes,
}

impl CsvField {
    #[must_use]
    pub const fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::Symbol => &[
                "symbol",
                "ticker",
                "isin",
                "cusip",
                "instrument",
                "security",
            ],
            Self::Market => &["market", "exchange", "venue", "mic", "listing"],
            Self::Name => &["name", "description", "instrument name", "security name"],
            Self::AssetType => &["asset type", "type", "category", "asset class"],
            Self::Quantity => &["quantity", "qty", "shares", "units"],
            Self::Price => &["price", "unit price", "last price"],
            Self::MarketValue => &["market value", "value", "current value"],
            Self::CostBasis => &["cost basis", "book value", "cost", "invested"],
            Self::GrossAmount => &["gross amount", "amount", "net amount", "total"],
            Self::Fees => &["fees", "commission", "commissions"],
            Self::Taxes => &["tax", "taxes", "withholding"],
            Self::Currency => &["currency", "ccy"],
            Self::TradeDate => &["date", "as of", "trade date", "settlement date"],
            Self::Action => &["action", "operation"],
            Self::Notes => &["notes", "note", "memo", "comment"],
        }
    }

    const ALL: [Self; 15] = [
        Self::Symbol,
        Self::Market,
        Self::Name,
        Self::AssetType,
        Self::Quantity,
        Self::Price,
        Self::MarketValue,
        Self::CostBasis,
        Self::GrossAmount,
        Self::Fees,
        Self::Taxes,
        Self::Currency,
        Self::TradeDate,
        Self::Action,
        Self::Notes,
    ];
}

fn normalize_header(header: &str) -> String {
    let mut out = String::new();
    for ch in header.trim().chars() {
        let decomposed = unicode_normalization::UnicodeNormalization::nfd(std::iter::once(ch));
        for c in decomposed {
            if c.is_ascii_alphabetic() || c.is_ascii_digit() {
                out.push(c.to_ascii_lowercase());
            } else if (c.is_whitespace() || c == '_' || c == '-') && !out.ends_with(' ') {
                out.push(' ');
            }
        }
    }
    out.trim().to_string()
}

#[must_use]
pub fn infer_field(header: &str) -> Option<CsvField> {
    let normalized = normalize_header(header);
    if normalized.is_empty() {
        return None;
    }
    for field in CsvField::ALL {
        if field.aliases().iter().any(|alias| normalized == *alias) {
            return Some(field);
        }
    }
    CsvField::ALL.into_iter().find(|field| {
        field
            .aliases()
            .iter()
            .any(|alias| normalized.contains(alias))
    })
}

#[must_use]
pub fn build_header_mapping(headers: &[String]) -> HashMap<CsvField, Option<String>> {
    let mut mapping: HashMap<CsvField, Option<String>> = HashMap::new();
    for field in CsvField::ALL {
        mapping.insert(field, None);
    }
    for header in headers {
        if let Some(field) = infer_field(header) {
            if mapping.get(&field).is_none_or(Option::is_none) {
                mapping.insert(field, Some(header.clone()));
            }
        }
    }
    mapping
}

#[must_use]
pub fn default_headers(count: usize) -> Vec<String> {
    let names: &[&str] = if count >= 5 {
        &["symbol", "market", "quantity", "price", "currency"]
    } else {
        &["symbol", "quantity", "price", "currency"]
    };
    (0..count)
        .map(|i| {
            names
                .get(i)
                .map_or_else(|| format!("col_{}", i + 1), |s| (*s).to_string())
        })
        .collect()
}

#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn pick_string(raw: &HashMap<String, String>, header: Option<&str>) -> Option<String> {
    let header = header?;
    let value = raw.get(header)?.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn pick_number(raw: &HashMap<String, String>, header: Option<&str>) -> Option<f64> {
    let value = pick_string(raw, header)?;
    let stripped: String = value
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == ',' || *c == '.' || *c == '-')
        .collect();
    if stripped.is_empty() {
        return None;
    }
    let last_comma = stripped.rfind(',');
    let last_dot = stripped.rfind('.');
    let normalized = match (last_comma, last_dot) {
        (Some(c), Some(d)) => {
            if c > d {
                stripped.replace('.', "").replace(',', ".")
            } else {
                stripped.replace(',', "")
            }
        }
        (Some(_), None) => stripped.replace(',', "."),
        _ => stripped,
    };
    normalized.parse::<f64>().ok().filter(|n| n.is_finite())
}

#[must_use]
pub fn parse_csv_records(text: &str) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut cell = String::new();
    let mut in_quotes = false;
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        let next = chars.get(i + 1).copied();
        if ch == '"' {
            if in_quotes && next == Some('"') {
                cell.push('"');
                i += 1;
            } else {
                in_quotes = !in_quotes;
            }
        } else if (ch == ',' || ch == '\t') && !in_quotes {
            row.push(cell);
            cell = String::new();
        } else if (ch == '\n' || ch == '\r') && !in_quotes {
            if ch == '\r' && next == Some('\n') {
                i += 1;
            }
            row.push(cell);
            rows.push(row);
            row = Vec::new();
            cell = String::new();
        } else {
            cell.push(ch);
        }
        i += 1;
    }
    if !cell.is_empty() || !row.is_empty() {
        row.push(cell);
        rows.push(row);
    }
    rows
}

#[must_use]
pub fn parse_portfolio_csv(text: &str) -> Vec<PortfolioCsvRow> {
    let records: Vec<Vec<String>> = parse_csv_records(text)
        .into_iter()
        .filter(|r| r.iter().any(|c| !c.trim().is_empty()))
        .collect();
    if records.is_empty() {
        return Vec::new();
    }
    let headers: Vec<String> = records[0]
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let trimmed = h.trim();
            if trimmed.is_empty() {
                format!("col_{}", i + 1)
            } else {
                trimmed.to_string()
            }
        })
        .collect();
    let looks_like_header = headers.iter().any(|h| infer_field(h).is_some());
    let data = if looks_like_header {
        &records[1..]
    } else {
        &records[..]
    };
    let header_row = if looks_like_header {
        headers
    } else {
        default_headers(records[0].len())
    };
    let mapping = build_header_mapping(&header_row);
    let mut rows: Vec<PortfolioCsvRow> = Vec::new();
    for (index, record) in data.iter().enumerate() {
        if record.iter().all(|c| c.trim().is_empty()) {
            continue;
        }
        let mut raw: HashMap<String, String> = HashMap::new();
        for (col_idx, header) in header_row.iter().enumerate() {
            raw.insert(
                header.clone(),
                record.get(col_idx).map_or("", |s| s.trim()).to_string(),
            );
        }
        rows.push(PortfolioCsvRow {
            row_index: index + 1,
            raw: raw.clone(),
            symbol: pick_string(
                &raw,
                mapping.get(&CsvField::Symbol).and_then(|o| o.as_deref()),
            ),
            market: pick_string(
                &raw,
                mapping.get(&CsvField::Market).and_then(|o| o.as_deref()),
            ),
            name: pick_string(
                &raw,
                mapping.get(&CsvField::Name).and_then(|o| o.as_deref()),
            ),
            asset_type: pick_string(
                &raw,
                mapping.get(&CsvField::AssetType).and_then(|o| o.as_deref()),
            ),
            quantity: pick_number(
                &raw,
                mapping.get(&CsvField::Quantity).and_then(|o| o.as_deref()),
            ),
            price: pick_number(
                &raw,
                mapping.get(&CsvField::Price).and_then(|o| o.as_deref()),
            ),
            market_value: pick_number(
                &raw,
                mapping
                    .get(&CsvField::MarketValue)
                    .and_then(|o| o.as_deref()),
            ),
            cost_basis: pick_number(
                &raw,
                mapping.get(&CsvField::CostBasis).and_then(|o| o.as_deref()),
            ),
            gross_amount: pick_number(
                &raw,
                mapping
                    .get(&CsvField::GrossAmount)
                    .and_then(|o| o.as_deref()),
            ),
            fees: pick_number(
                &raw,
                mapping.get(&CsvField::Fees).and_then(|o| o.as_deref()),
            ),
            taxes: pick_number(
                &raw,
                mapping.get(&CsvField::Taxes).and_then(|o| o.as_deref()),
            ),
            currency: pick_string(
                &raw,
                mapping.get(&CsvField::Currency).and_then(|o| o.as_deref()),
            ),
            trade_date: pick_string(
                &raw,
                mapping.get(&CsvField::TradeDate).and_then(|o| o.as_deref()),
            ),
            action: pick_string(
                &raw,
                mapping.get(&CsvField::Action).and_then(|o| o.as_deref()),
            ),
            notes: pick_string(
                &raw,
                mapping.get(&CsvField::Notes).and_then(|o| o.as_deref()),
            ),
        });
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_csv_simple_comma() {
        let csv = "a,b,c\n1,2,3";
        let records = parse_csv_records(csv);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["a", "b", "c"]);
        assert_eq!(records[1], vec!["1", "2", "3"]);
    }

    #[test]
    fn parse_csv_tab_delimiter() {
        let csv = "a\tb\tc\n1\t2\t3";
        let records = parse_csv_records(csv);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["a", "b", "c"]);
        assert_eq!(records[1], vec!["1", "2", "3"]);
    }

    #[test]
    fn parse_csv_quoted_fields() {
        let csv = "\"a,b\",c\n1,2";
        let records = parse_csv_records(csv);
        assert_eq!(records[0], vec!["a,b", "c"]);
    }

    #[test]
    fn parse_csv_escaped_quotes() {
        let csv = "\"a\"\"b\",c\n1,2";
        let records = parse_csv_records(csv);
        assert_eq!(records[0], vec!["a\"b", "c"]);
    }

    #[test]
    fn parse_csv_crlf() {
        let csv = "a,b\r\n1,2\r\n3,4";
        let records = parse_csv_records(csv);
        assert_eq!(records.len(), 3);
        assert_eq!(records[2], vec!["3", "4"]);
    }

    #[test]
    fn infer_field_exact_match() {
        assert_eq!(infer_field("symbol"), Some(CsvField::Symbol));
        assert_eq!(infer_field("Symbol"), Some(CsvField::Symbol));
        assert_eq!(infer_field("SYMBOL"), Some(CsvField::Symbol));
    }

    #[test]
    fn infer_field_substring() {
        assert_eq!(infer_field("ticker symbol"), Some(CsvField::Symbol));
        assert_eq!(infer_field("current value"), Some(CsvField::MarketValue));
    }

    #[test]
    fn infer_field_unicode() {
        assert_eq!(infer_field("Symbôle"), Some(CsvField::Symbol));
        assert_eq!(infer_field("Prïce"), Some(CsvField::Price));
    }

    #[test]
    fn pick_number_us_format() {
        let mut raw = HashMap::new();
        raw.insert("price".to_string(), "1,234.56".to_string());
        assert_eq!(pick_number(&raw, Some("price")), Some(1234.56));
    }

    #[test]
    fn pick_number_european_format() {
        let mut raw = HashMap::new();
        raw.insert("price".to_string(), "1.234,56".to_string());
        assert_eq!(pick_number(&raw, Some("price")), Some(1234.56));
    }

    #[test]
    fn parse_portfolio_csv_integration() {
        let csv =
            "Symbol,Market,Quantity,Price,Currency\nAAPL,NASDAQ,10,190.50,USD\nGOOG,,5,140,USD";
        let rows = parse_portfolio_csv(csv);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].symbol, Some("AAPL".to_string()));
        assert_eq!(rows[0].market, Some("NASDAQ".to_string()));
        assert_eq!(rows[0].quantity, Some(10.0));
        assert_eq!(rows[0].price, Some(190.5));
        assert_eq!(rows[0].currency, Some("USD".to_string()));
        assert_eq!(rows[1].symbol, Some("GOOG".to_string()));
        assert_eq!(rows[1].market, None);
        assert_eq!(rows[1].quantity, Some(5.0));
    }
}
