use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

pub type PortfolioId = String;
pub type PortfolioAccountId = String;
pub type PortfolioImportBatchId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PortfolioImportKind {
    #[default]
    Positions,
    Transactions,
}

impl fmt::Display for PortfolioImportKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Positions => "positions",
            Self::Transactions => "transactions",
        };
        write!(f, "{value}")
    }
}

impl FromStr for PortfolioImportKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "transactions" | "transaction" | "ledger" => Ok(Self::Transactions),
            _ => Ok(Self::Positions),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PortfolioTransactionAction {
    Buy,
    Sell,
    Dividend,
    Interest,
    Deposit,
    Withdrawal,
    Fee,
    Tax,
    Split,
    TransferIn,
    TransferOut,
    #[default]
    Other,
}

impl fmt::Display for PortfolioTransactionAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Buy => "buy",
            Self::Sell => "sell",
            Self::Dividend => "dividend",
            Self::Interest => "interest",
            Self::Deposit => "deposit",
            Self::Withdrawal => "withdrawal",
            Self::Fee => "fee",
            Self::Tax => "tax",
            Self::Split => "split",
            Self::TransferIn => "transfer_in",
            Self::TransferOut => "transfer_out",
            Self::Other => "other",
        };
        write!(f, "{value}")
    }
}

impl FromStr for PortfolioTransactionAction {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let normalized = value.trim().to_ascii_lowercase().replace([' ', '-'], "_");
        match normalized.as_str() {
            "buy" | "bought" | "purchase" | "purchased" | "acquisto" => Ok(Self::Buy),
            "sell" | "sold" | "sale" | "vendita" => Ok(Self::Sell),
            "dividend" | "dividends" | "dividendo" => Ok(Self::Dividend),
            "interest" | "coupon" | "cedola" => Ok(Self::Interest),
            "deposit" | "cash_deposit" | "contribution" | "contributo" => Ok(Self::Deposit),
            "withdrawal" | "cash_withdrawal" | "prelievo" => Ok(Self::Withdrawal),
            "fee" | "fees" | "commission" | "commissione" => Ok(Self::Fee),
            "tax" | "taxes" | "imposta" | "ritenuta" => Ok(Self::Tax),
            "split" | "stock_split" => Ok(Self::Split),
            "transfer_in" | "transferin" => Ok(Self::TransferIn),
            "transfer_out" | "transferout" => Ok(Self::TransferOut),
            _ => Ok(Self::Other),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub id: PortfolioId,
    pub name: String,
    pub base_currency: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioAccount {
    pub id: PortfolioAccountId,
    pub portfolio_id: PortfolioId,
    pub name: String,
    pub institution: Option<String>,
    pub account_type: Option<String>,
    pub base_currency: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioImportBatch {
    pub id: PortfolioImportBatchId,
    pub portfolio_id: PortfolioId,
    pub account_id: PortfolioAccountId,
    pub source_name: String,
    pub import_kind: PortfolioImportKind,
    pub imported_at: String,
    pub row_count: usize,
    pub imported_count: usize,
    pub duplicate_count: usize,
    pub review_count: usize,
    pub warnings: Vec<PortfolioImportWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioImportWarning {
    pub row_index: Option<usize>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioPosition {
    pub id: String,
    pub portfolio_id: PortfolioId,
    pub account_id: PortfolioAccountId,
    pub symbol: String,
    pub market: Option<String>,
    pub name: Option<String>,
    pub asset_type: String,
    pub quantity: f64,
    pub price: Option<f64>,
    pub market_value: Option<f64>,
    pub cost_basis: Option<f64>,
    pub currency: String,
    pub as_of: Option<String>,
    pub source_batch_id: Option<PortfolioImportBatchId>,
    pub updated_at: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioTransaction {
    pub id: String,
    pub portfolio_id: PortfolioId,
    pub account_id: PortfolioAccountId,
    pub import_batch_id: PortfolioImportBatchId,
    pub row_index: usize,
    pub trade_date: Option<String>,
    pub action: PortfolioTransactionAction,
    pub symbol: Option<String>,
    pub market: Option<String>,
    pub name: Option<String>,
    pub asset_type: String,
    pub quantity: Option<f64>,
    pub price: Option<f64>,
    pub gross_amount: Option<f64>,
    pub fees: Option<f64>,
    pub taxes: Option<f64>,
    pub currency: String,
    pub notes: Option<String>,
    pub raw_payload: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioHoldingAccount {
    pub account_id: PortfolioAccountId,
    pub account_name: String,
    pub quantity: f64,
    pub market_value: Option<f64>,
    pub cost_basis: Option<f64>,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioHolding {
    pub symbol: String,
    pub market: Option<String>,
    pub name: Option<String>,
    pub asset_type: String,
    pub quantity: f64,
    pub market_value: Option<f64>,
    pub cost_basis: Option<f64>,
    pub currency: String,
    pub allocation_pct: Option<f64>,
    pub accounts: Vec<PortfolioHoldingAccount>,
}

#[must_use]
pub fn portfolio_holding_entity_id(symbol: &str, market: Option<&str>) -> String {
    let mut id = String::from("holding:");
    id.push_str(&normalize_holding_id_part(symbol));
    if let Some(market) = market
        && !market.trim().is_empty()
    {
        id.push(':');
        id.push_str(&normalize_holding_id_part(market));
    }
    id
}

fn normalize_holding_id_part(value: &str) -> String {
    let mut out = String::new();
    for ch in value.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if matches!(ch, '.' | '-' | '_') {
            out.push(ch);
        }
    }
    if out.is_empty() {
        "unknown".to_string()
    } else {
        out
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSummary {
    pub id: PortfolioId,
    pub name: String,
    pub base_currency: String,
    pub account_count: usize,
    pub holding_count: usize,
    pub total_market_value: Option<f64>,
    pub last_import_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioDetail {
    pub portfolio: Portfolio,
    pub accounts: Vec<PortfolioAccount>,
    pub holdings: Vec<PortfolioHolding>,
    pub positions: Vec<PortfolioPosition>,
    pub transactions: Vec<PortfolioTransaction>,
    pub import_batches: Vec<PortfolioImportBatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioCsvRow {
    pub row_index: usize,
    #[serde(default)]
    pub raw: HashMap<String, String>,
    pub symbol: Option<String>,
    pub market: Option<String>,
    pub name: Option<String>,
    pub asset_type: Option<String>,
    pub quantity: Option<f64>,
    pub price: Option<f64>,
    pub market_value: Option<f64>,
    pub cost_basis: Option<f64>,
    pub gross_amount: Option<f64>,
    pub fees: Option<f64>,
    pub taxes: Option<f64>,
    pub currency: Option<String>,
    pub trade_date: Option<String>,
    pub action: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioCsvImportInput {
    pub portfolio_id: Option<PortfolioId>,
    pub portfolio_name: Option<String>,
    pub account_id: Option<PortfolioAccountId>,
    pub account_name: Option<String>,
    pub institution: Option<String>,
    pub account_type: Option<String>,
    pub base_currency: String,
    pub source_name: String,
    pub import_kind: PortfolioImportKind,
    pub rows: Vec<PortfolioCsvRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioImportResult {
    pub portfolio_id: PortfolioId,
    pub account_id: PortfolioAccountId,
    pub batch_id: PortfolioImportBatchId,
    pub row_count: usize,
    pub imported_count: usize,
    pub duplicate_count: usize,
    pub review_count: usize,
    pub warnings: Vec<PortfolioImportWarning>,
    pub holdings: Vec<PortfolioHolding>,
}
