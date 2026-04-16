use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

pub type AnalysisId = String;
pub type AnalysisRunId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisStatus {
    Queued,
    Running,
    #[default]
    Completed,
    Failed,
    Cancelled,
}

impl fmt::Display for AnalysisStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        };
        write!(f, "{value}")
    }
}

impl FromStr for AnalysisStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "queued" => Ok(Self::Queued),
            "running" | "in_progress" => Ok(Self::Running),
            "completed" | "done" => Ok(Self::Completed),
            "failed" | "error" => Ok(Self::Failed),
            "cancelled" | "canceled" => Ok(Self::Cancelled),
            _ => Err(format!("unknown analysis status: {value}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisIntent {
    SingleEquity,
    CompareEquities,
    SectorAnalysis,
    MacroTheme,
    Watchlist,
    #[default]
    GeneralResearch,
}

impl fmt::Display for AnalysisIntent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::SingleEquity => "single_equity",
            Self::CompareEquities => "compare_equities",
            Self::SectorAnalysis => "sector_analysis",
            Self::MacroTheme => "macro_theme",
            Self::Watchlist => "watchlist",
            Self::GeneralResearch => "general_research",
        };
        write!(f, "{value}")
    }
}

impl FromStr for AnalysisIntent {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "single_equity" => Ok(Self::SingleEquity),
            "compare_equities" => Ok(Self::CompareEquities),
            "sector_analysis" => Ok(Self::SectorAnalysis),
            "macro_theme" => Ok(Self::MacroTheme),
            "watchlist" => Ok(Self::Watchlist),
            "general_research" => Ok(Self::GeneralResearch),
            _ => Ok(Self::GeneralResearch),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    pub id: AnalysisId,
    pub title: String,
    pub user_prompt: String,
    pub intent: AnalysisIntent,
    pub status: AnalysisStatus,
    pub active_run_id: Option<AnalysisRunId>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRun {
    pub id: AnalysisRunId,
    pub analysis_id: AnalysisId,
    pub agent_id: String,
    pub prompt_text: String,
    pub status: AnalysisStatus,
    pub started_at: String,
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub id: AnalysisId,
    pub title: String,
    pub user_prompt: String,
    pub intent: AnalysisIntent,
    pub status: AnalysisStatus,
    pub active_run_id: Option<AnalysisRunId>,
    pub active_run_status: Option<AnalysisStatus>,
    pub block_count: usize,
    pub source_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub run_id: AnalysisRunId,
    pub symbol: Option<String>,
    pub name: String,
    pub exchange: Option<String>,
    pub asset_type: String,
    pub sector: Option<String>,
    pub country: Option<String>,
    pub confidence: f64,
    pub resolution_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchPlan {
    pub id: String,
    pub run_id: AnalysisRunId,
    pub intent: AnalysisIntent,
    pub summary: String,
    pub decision_criteria: Vec<String>,
    pub planned_checks: Vec<String>,
    pub required_blocks: Vec<String>,
    pub required_artifacts: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SourceReliability {
    Primary,
    High,
    #[default]
    Medium,
    Low,
}

impl fmt::Display for SourceReliability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Primary => "primary",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        };
        write!(f, "{value}")
    }
}

impl FromStr for SourceReliability {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "primary" => Ok(Self::Primary),
            "high" => Ok(Self::High),
            "medium" => Ok(Self::Medium),
            "low" => Ok(Self::Low),
            _ => Ok(Self::Medium),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub run_id: AnalysisRunId,
    pub title: String,
    pub url: Option<String>,
    pub publisher: Option<String>,
    pub source_type: String,
    pub retrieved_at: String,
    pub as_of: Option<String>,
    pub reliability: SourceReliability,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    pub id: String,
    pub run_id: AnalysisRunId,
    pub entity_id: Option<String>,
    pub metric: String,
    pub value: String,
    pub numeric_value: Option<f64>,
    pub unit: Option<String>,
    pub period: Option<String>,
    pub as_of: String,
    pub source_id: String,
    pub notes: Option<String>,
    #[serde(default)]
    pub prior_value: Option<f64>,
    #[serde(default)]
    pub change_pct: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    MetricTable,
    ComparisonMatrix,
    ScenarioMatrix,
    BarChart,
    LineChart,
    AreaChart,
    #[default]
    Other,
}

impl fmt::Display for ArtifactKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::MetricTable => "metric_table",
            Self::ComparisonMatrix => "comparison_matrix",
            Self::ScenarioMatrix => "scenario_matrix",
            Self::BarChart => "bar_chart",
            Self::LineChart => "line_chart",
            Self::AreaChart => "area_chart",
            Self::Other => "other",
        };
        write!(f, "{value}")
    }
}

impl FromStr for ArtifactKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "metric_table" => Ok(Self::MetricTable),
            "comparison_matrix" => Ok(Self::ComparisonMatrix),
            "scenario_matrix" => Ok(Self::ScenarioMatrix),
            "bar_chart" => Ok(Self::BarChart),
            "line_chart" => Ok(Self::LineChart),
            "area_chart" => Ok(Self::AreaChart),
            _ => Ok(Self::Other),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactColumn {
    pub key: String,
    pub label: String,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactPoint {
    pub label: String,
    pub value: f64,
    #[serde(default)]
    pub source_id: Option<String>,
    #[serde(default)]
    pub metric_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactSeries {
    pub label: String,
    pub points: Vec<ArtifactPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredArtifact {
    pub id: String,
    pub run_id: AnalysisRunId,
    pub kind: ArtifactKind,
    pub title: String,
    pub summary: String,
    pub columns: Vec<ArtifactColumn>,
    pub rows: Vec<serde_json::Value>,
    pub series: Vec<ArtifactSeries>,
    pub evidence_ids: Vec<String>,
    pub entity_ids: Vec<String>,
    pub display_order: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BlockKind {
    Thesis,
    BusinessQuality,
    Financials,
    Valuation,
    PeerComparison,
    SectorContext,
    Catalysts,
    Risks,
    ScenarioMatrix,
    TechnicalContext,
    OpenQuestions,
    #[default]
    Other,
}

impl fmt::Display for BlockKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Thesis => "thesis",
            Self::BusinessQuality => "business_quality",
            Self::Financials => "financials",
            Self::Valuation => "valuation",
            Self::PeerComparison => "peer_comparison",
            Self::SectorContext => "sector_context",
            Self::Catalysts => "catalysts",
            Self::Risks => "risks",
            Self::ScenarioMatrix => "scenario_matrix",
            Self::TechnicalContext => "technical_context",
            Self::OpenQuestions => "open_questions",
            Self::Other => "other",
        };
        write!(f, "{value}")
    }
}

impl FromStr for BlockKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "thesis" => Ok(Self::Thesis),
            "business_quality" => Ok(Self::BusinessQuality),
            "financials" => Ok(Self::Financials),
            "valuation" => Ok(Self::Valuation),
            "peer_comparison" => Ok(Self::PeerComparison),
            "sector_context" => Ok(Self::SectorContext),
            "catalysts" => Ok(Self::Catalysts),
            "risks" => Ok(Self::Risks),
            "scenario_matrix" => Ok(Self::ScenarioMatrix),
            "technical_context" => Ok(Self::TechnicalContext),
            "open_questions" => Ok(Self::OpenQuestions),
            _ => Ok(Self::Other),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisBlock {
    pub id: String,
    pub run_id: AnalysisRunId,
    pub kind: BlockKind,
    pub title: String,
    pub body: String,
    pub evidence_ids: Vec<String>,
    pub entity_ids: Vec<String>,
    pub confidence: f64,
    pub importance: String,
    pub display_order: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StanceKind {
    Bullish,
    Neutral,
    Bearish,
    Mixed,
    #[default]
    InsufficientData,
}

impl fmt::Display for StanceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Bullish => "bullish",
            Self::Neutral => "neutral",
            Self::Bearish => "bearish",
            Self::Mixed => "mixed",
            Self::InsufficientData => "insufficient_data",
        };
        write!(f, "{value}")
    }
}

impl FromStr for StanceKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "bullish" => Ok(Self::Bullish),
            "neutral" => Ok(Self::Neutral),
            "bearish" => Ok(Self::Bearish),
            "mixed" => Ok(Self::Mixed),
            "insufficient_data" => Ok(Self::InsufficientData),
            _ => Ok(Self::InsufficientData),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalStance {
    pub id: String,
    pub run_id: AnalysisRunId,
    pub stance: StanceKind,
    pub horizon: String,
    pub confidence: f64,
    pub summary: String,
    pub key_reasons: Vec<String>,
    pub watch_items: Vec<String>,
    pub what_would_change: Vec<String>,
    pub disclaimer: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionScenario {
    pub label: String,
    pub target_value: f64,
    pub target_label: String,
    pub upside_pct: f64,
    pub probability: f64,
    pub rationale: String,
    pub catalysts: Vec<String>,
    pub risks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Projection {
    pub id: String,
    pub run_id: AnalysisRunId,
    pub entity_id: String,
    pub horizon: String,
    pub metric: String,
    pub current_value: f64,
    pub current_value_label: String,
    pub unit: String,
    pub scenarios: Vec<ProjectionScenario>,
    pub methodology: String,
    pub key_assumptions: Vec<String>,
    pub evidence_ids: Vec<String>,
    pub confidence: f64,
    pub disclaimer: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub analysis: Analysis,
    pub runs: Vec<AnalysisRun>,
    pub research_plan: Option<ResearchPlan>,
    pub entities: Vec<Entity>,
    pub sources: Vec<Source>,
    pub metrics: Vec<MetricSnapshot>,
    pub artifacts: Vec<StructuredArtifact>,
    pub blocks: Vec<AnalysisBlock>,
    pub final_stance: Option<FinalStance>,
    pub projections: Vec<Projection>,
}
