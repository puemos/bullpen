export type AnalysisStatus = "queued" | "running" | "completed" | "failed" | "cancelled";

export type AnalysisIntent =
  | "single_equity"
  | "compare_equities"
  | "sector_analysis"
  | "macro_theme"
  | "watchlist"
  | "portfolio"
  | "general_research";

export interface AgentModel {
  id: string;
  name: string;
}

export interface AgentCandidate {
  id: string;
  label: string;
  command: string | null;
  args: string[];
  available: boolean;
  models: AgentModel[];
  supports_model_override: boolean;
}

export interface AppSettings {
  custom_agent_command: string | null;
  custom_agent_args: string[];
  timeout_secs: number;
  source_freshness_days: number;
  disclaimer: string;
  model_by_agent: Record<string, string>;
  enabled_sources: string[];
  sources_with_keys: string[];
}

export type SourceCategoryId =
  | "web_search"
  | "filings"
  | "fundamentals"
  | "market_data"
  | "news"
  | "forums"
  | "screener";

export interface SourceDescriptor {
  id: string;
  display_name: string;
  category: SourceCategoryId;
  requires_key: boolean;
  default_enabled: boolean;
  docs_url: string;
  key_acquisition_url: string | null;
  rate_limit_hint: string | null;
  description: string;
  has_key: boolean;
  enabled: boolean;
}

export interface SourceKeyTestResult {
  status: string;
  message: string;
}

export interface AnalysisSummary {
  id: string;
  title: string;
  user_prompt: string;
  intent: AnalysisIntent;
  status: AnalysisStatus;
  active_run_id: string | null;
  active_run_status: AnalysisStatus | null;
  portfolio_id: string | null;
  block_count: number;
  source_count: number;
  created_at: string;
  updated_at: string;
}

export type PortfolioImportKind = "positions" | "transactions";

export type PortfolioTransactionAction =
  | "buy"
  | "sell"
  | "dividend"
  | "interest"
  | "deposit"
  | "withdrawal"
  | "fee"
  | "tax"
  | "split"
  | "transfer_in"
  | "transfer_out"
  | "other";

export interface PortfolioSummary {
  id: string;
  name: string;
  base_currency: string;
  account_count: number;
  holding_count: number;
  total_market_value: number | null;
  last_import_at: string | null;
  updated_at: string;
}

export interface Portfolio {
  id: string;
  name: string;
  base_currency: string;
  created_at: string;
  updated_at: string;
}

export interface PortfolioAccount {
  id: string;
  portfolio_id: string;
  name: string;
  institution: string | null;
  account_type: string | null;
  base_currency: string;
  created_at: string;
  updated_at: string;
}

export interface PortfolioImportWarning {
  row_index: number | null;
  message: string;
}

export interface PortfolioImportBatch {
  id: string;
  portfolio_id: string;
  account_id: string;
  source_name: string;
  import_kind: PortfolioImportKind;
  imported_at: string;
  row_count: number;
  imported_count: number;
  duplicate_count: number;
  review_count: number;
  warnings: PortfolioImportWarning[];
}

export interface PortfolioPosition {
  id: string;
  portfolio_id: string;
  account_id: string;
  symbol: string;
  market: string | null;
  name: string | null;
  asset_type: string;
  quantity: number;
  price: number | null;
  market_value: number | null;
  cost_basis: number | null;
  currency: string;
  as_of: string | null;
  source_batch_id: string | null;
  updated_at: string;
  notes: string | null;
}

export interface PortfolioTransaction {
  id: string;
  portfolio_id: string;
  account_id: string;
  import_batch_id: string;
  row_index: number;
  trade_date: string | null;
  action: PortfolioTransactionAction;
  symbol: string | null;
  market: string | null;
  name: string | null;
  asset_type: string;
  quantity: number | null;
  price: number | null;
  gross_amount: number | null;
  fees: number | null;
  taxes: number | null;
  currency: string;
  notes: string | null;
  raw_payload: unknown;
  created_at: string;
}

export interface PortfolioHoldingAccount {
  account_id: string;
  account_name: string;
  quantity: number;
  market_value: number | null;
  cost_basis: number | null;
  currency: string;
}

export interface PortfolioHolding {
  symbol: string;
  market: string | null;
  name: string | null;
  asset_type: string;
  quantity: number;
  market_value: number | null;
  cost_basis: number | null;
  currency: string;
  allocation_pct: number | null;
  accounts: PortfolioHoldingAccount[];
}

export interface PortfolioDetail {
  portfolio: Portfolio;
  accounts: PortfolioAccount[];
  holdings: PortfolioHolding[];
  positions: PortfolioPosition[];
  transactions: PortfolioTransaction[];
  import_batches: PortfolioImportBatch[];
  totals_by_currency: [string, number][];
}

export interface PortfolioCsvRow {
  row_index: number;
  raw: Record<string, string>;
  symbol: string | null;
  market: string | null;
  name: string | null;
  asset_type: string | null;
  quantity: number | null;
  price: number | null;
  market_value: number | null;
  cost_basis: number | null;
  gross_amount: number | null;
  fees: number | null;
  taxes: number | null;
  currency: string | null;
  trade_date: string | null;
  action: string | null;
  notes: string | null;
}

export interface PortfolioCsvImportInput {
  portfolio_id: string | null;
  portfolio_name: string | null;
  account_id: string | null;
  account_name: string | null;
  institution: string | null;
  account_type: string | null;
  base_currency: string;
  source_name: string;
  import_kind: PortfolioImportKind;
  rows: PortfolioCsvRow[];
}

export interface PortfolioImportResult {
  portfolio_id: string;
  account_id: string;
  batch_id: string;
  row_count: number;
  imported_count: number;
  duplicate_count: number;
  review_count: number;
  warnings: PortfolioImportWarning[];
  holdings: PortfolioHolding[];
}

export interface Analysis {
  id: string;
  title: string;
  user_prompt: string;
  intent: AnalysisIntent;
  status: AnalysisStatus;
  active_run_id: string | null;
  portfolio_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface AnalysisRun {
  id: string;
  analysis_id: string;
  agent_id: string;
  model_id: string | null;
  prompt_text: string;
  status: AnalysisStatus;
  started_at: string;
  completed_at: string | null;
  error: string | null;
}

export interface ResearchPlan {
  id: string;
  run_id: string;
  intent: AnalysisIntent;
  summary: string;
  decision_criteria: string[];
  planned_checks: string[];
  created_at: string;
}

export interface Entity {
  id: string;
  run_id: string;
  symbol: string | null;
  name: string;
  exchange: string | null;
  asset_type: string;
  sector: string | null;
  country: string | null;
  confidence: number;
  resolution_notes: string | null;
}

export type VerificationStatus = "ok" | "redirect" | "dead" | "timeout" | "forbidden";

export interface Source {
  id: string;
  run_id: string;
  title: string;
  url: string | null;
  publisher: string | null;
  source_type: string;
  retrieved_at: string;
  reliability: "primary" | "high" | "medium" | "low";
  summary: string;
  last_verified_at?: string | null;
  last_verification_status?: VerificationStatus | null;
}

export interface MetricSnapshot {
  id: string;
  run_id: string;
  entity_id: string | null;
  metric: string;
  numeric_value: number;
  unit: string | null;
  period: string | null;
  as_of: string;
  source_id: string;
  prior_value: number | null;
  change_pct: number | null;
}

export type ArtifactKind =
  | "metric_table"
  | "comparison_matrix"
  | "scenario_matrix"
  | "bar_chart"
  | "line_chart"
  | "area_chart"
  | "other";

export interface ArtifactColumn {
  key: string;
  label: string;
  unit: string | null;
  description: string | null;
}

export interface ArtifactPoint {
  label: string;
  value: number;
  source_id: string | null;
  metric_id: string | null;
}

export interface ArtifactSeries {
  label: string;
  points: ArtifactPoint[];
}

export interface StructuredArtifact {
  id: string;
  run_id: string;
  kind: ArtifactKind;
  title: string;
  summary: string;
  columns: ArtifactColumn[];
  rows: Record<string, unknown>[];
  series: ArtifactSeries[];
  evidence_ids: string[];
  display_order: number;
  created_at: string;
}

export type BlockKind =
  | "thesis"
  | "business_quality"
  | "financials"
  | "valuation"
  | "peer_comparison"
  | "sector_context"
  | "catalysts"
  | "risks"
  | "technical_context"
  | "open_questions"
  | "other";

export type Importance = "high" | "medium" | "low";

export interface AnalysisBlock {
  id: string;
  run_id: string;
  kind: BlockKind;
  title: string;
  body: string;
  evidence_ids: string[];
  confidence: number;
  importance: Importance;
  display_order: number;
  created_at: string;
}

export type StanceKind = "bullish" | "neutral" | "bearish" | "mixed" | "insufficient_data";

export interface FinalStance {
  id: string;
  run_id: string;
  stance: StanceKind;
  horizon: string;
  confidence: number;
  summary: string;
  key_reasons: string[];
  what_would_change: string[];
  disclaimer: string;
  created_at: string;
}

export type ScenarioLabel = "bull" | "base" | "bear";

export interface ProjectionScenario {
  label: ScenarioLabel;
  target_value: number;
  target_label: string;
  probability: number;
  rationale: string;
  catalysts: string[];
  risks: string[];
}

export interface Projection {
  id: string;
  run_id: string;
  entity_id: string;
  horizon: string;
  metric: string;
  current_value: number;
  current_value_label: string;
  unit: string;
  scenarios: ProjectionScenario[];
  methodology: string;
  key_assumptions: string[];
  evidence_ids: string[];
  confidence: number;
  disclaimer: string;
  created_at: string;
}

export interface CounterThesis {
  id: string;
  run_id: string;
  stance_against: StanceKind;
  summary: string;
  supporting_evidence_ids: string[];
  why_we_reject_or_partially_accept: string;
  residual_probability: number;
  created_at: string;
}

export interface UncertaintyEntry {
  id: string;
  run_id: string;
  question: string;
  why_it_matters: string;
  attempted_resolution: string;
  blocking: boolean;
  related_decision_criterion: string | null;
  created_at: string;
}

export interface MethodologyNote {
  id: string;
  run_id: string;
  approach: string;
  frameworks: string[];
  data_windows: string[];
  known_limitations: string[];
  created_at: string;
}

export type CriterionVerdict = "confirmed" | "refuted" | "partially_confirmed" | "unresolved";

export interface DecisionCriterionAnswer {
  id: string;
  run_id: string;
  criterion: string;
  verdict: CriterionVerdict;
  summary: string;
  supporting_block_ids: string[];
  supporting_evidence_ids: string[];
  created_at: string;
}

export type HoldingStance = "keep" | "trim" | "add" | "watch" | "exit" | "mixed";
export type AllocationAxis = "asset_class" | "sector" | "geography" | "currency" | "other";
export type RiskLevel = "low" | "medium" | "high";

export interface HoldingReview {
  id: string;
  run_id: string;
  entity_id: string;
  stance: HoldingStance;
  rationale: string;
  key_reasons: string[];
  key_risks: string[];
  confidence: number;
  importance: Importance;
  evidence_ids: string[];
  display_order: number;
  created_at: string;
}

export interface AllocationBucket {
  label: string;
  weight: number;
  commentary: string | null;
}

export interface AllocationDimension {
  dimension: AllocationAxis;
  breakdown: AllocationBucket[];
  concentration_flags: string[];
  overlap_notes: string | null;
}

export interface AllocationReview {
  id: string;
  run_id: string;
  summary: string;
  dimensions: AllocationDimension[];
  evidence_ids: string[];
  confidence: number;
  created_at: string;
}

export interface FactorExposure {
  factor: string;
  level: RiskLevel;
  commentary: string | null;
}

export interface PortfolioRisk {
  id: string;
  run_id: string;
  summary: string;
  factor_exposures: FactorExposure[];
  correlation_notes: string | null;
  macro_sensitivities: string[];
  single_name_risks: string[];
  tail_risks: string[];
  evidence_ids: string[];
  confidence: number;
  created_at: string;
}

export interface RebalancingRow {
  label: string;
  current_weight: number;
  suggested_weight: number;
  delta: number;
  commentary: string | null;
}

export interface RebalancingSuggestion {
  id: string;
  run_id: string;
  rationale: string;
  rows: RebalancingRow[];
  scenarios: string[];
  caveats: string[];
  evidence_ids: string[];
  confidence: number;
  created_at: string;
}

export interface PortfolioScenarioOutcome {
  label: ScenarioLabel;
  probability: number;
  portfolio_return_pct: number;
  projected_value: number | null;
  rationale: string;
  key_drivers: string[];
  watch_indicators: string[];
  evidence_ids: string[];
}

export interface PortfolioStressCase {
  name: string;
  estimated_return_pct: number;
  rationale: string;
  affected_exposures: string[];
  mitigants: string[];
  evidence_ids: string[];
}

export interface PortfolioScenarioAnalysis {
  id: string;
  run_id: string;
  horizon: string;
  base_currency: string;
  current_value: number | null;
  methodology: string;
  key_assumptions: string[];
  scenarios: PortfolioScenarioOutcome[];
  stress_cases: PortfolioStressCase[];
  evidence_ids: string[];
  confidence: number;
  created_at: string;
}

export type PortfolioModelType =
  | "holding_weighted"
  | "asset_class_cma"
  | "factor_overlay"
  | "hybrid";

export interface PortfolioExpectedReturnInput {
  name: string;
  input_type: string;
  weight: number;
  expected_return_pct: number;
  volatility_pct: number | null;
  rationale: string;
  evidence_ids: string[];
}

export interface PortfolioExpectedReturnModel {
  id: string;
  run_id: string;
  horizon: string;
  model_type: PortfolioModelType;
  summary: string;
  expected_return_pct: number;
  volatility_pct: number | null;
  inputs: PortfolioExpectedReturnInput[];
  correlation_assumptions: string[];
  limitations: string[];
  evidence_ids: string[];
  confidence: number;
  created_at: string;
}

export interface AnalysisReport {
  analysis: Analysis;
  runs: AnalysisRun[];
  research_plan: ResearchPlan | null;
  entities: Entity[];
  sources: Source[];
  metrics: MetricSnapshot[];
  artifacts: StructuredArtifact[];
  blocks: AnalysisBlock[];
  final_stance: FinalStance | null;
  projections: Projection[];
  counter_theses: CounterThesis[];
  uncertainty_entries: UncertaintyEntry[];
  methodology_note: MethodologyNote | null;
  decision_criterion_answers: DecisionCriterionAnswer[];
  holding_reviews: HoldingReview[];
  allocation_reviews: AllocationReview[];
  portfolio_risks: PortfolioRisk[];
  rebalancing_suggestions: RebalancingSuggestion[];
  portfolio_scenario_analyses: PortfolioScenarioAnalysis[];
  portfolio_expected_return_models: PortfolioExpectedReturnModel[];
}

export interface PlanEntry {
  content: string;
  priority: string;
  status: string;
}

export interface ToolCallStartedData {
  tool_call_id: string;
  title: string;
  kind: string;
}

export interface ToolCallCompleteData {
  tool_call_id: string;
  status: string;
  title: string;
  raw_input: unknown | null;
  raw_output: unknown | null;
}

export type ProgressEventPayload =
  | { event: "Log"; data: string }
  | { event: "MessageDelta"; data: { id: string; delta: string } }
  | { event: "ThoughtDelta"; data: { id: string; delta: string } }
  | { event: "ToolCallStarted"; data: ToolCallStartedData }
  | { event: "ToolCallComplete"; data: ToolCallCompleteData }
  | { event: "Plan"; data: { entries: PlanEntry[] } }
  | { event: "PlanSubmitted" }
  | { event: "SourceSubmitted" }
  | { event: "MetricSubmitted" }
  | { event: "ArtifactSubmitted" }
  | { event: "BlockSubmitted" }
  | { event: "StanceSubmitted" }
  | { event: "ProjectionSubmitted" }
  | { event: "Completed" }
  | { event: "Error"; data: { message: string } };

export type ProgressItemType =
  | "agent_message"
  | "agent_thought"
  | "tool_call"
  | "tool_result"
  | "plan"
  | "submitted"
  | "completed"
  | "error"
  | "log";

export interface ProgressItem {
  id: string;
  type: ProgressItemType;
  message: string;
  timestamp: number;
  data?: unknown;
}

export type RunStatus = "running" | "completed" | "error" | "cancelled";

export interface RunState {
  runId: string;
  agentId: string;
  agentLabel: string;
  status: RunStatus;
  progress: ProgressItem[];
  plan: PlanEntry[];
}

export interface DataChangedPayload {
  analysis_id: string;
  kind: string;
}
