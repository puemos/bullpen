export type AnalysisStatus = 'queued' | 'running' | 'completed' | 'failed' | 'cancelled';

export type AnalysisIntent =
  | 'single_equity'
  | 'compare_equities'
  | 'sector_analysis'
  | 'macro_theme'
  | 'watchlist'
  | 'general_research';

export interface AgentCandidate {
  id: string;
  label: string;
  command: string | null;
  args: string[];
  available: boolean;
}

export interface AppSettings {
  custom_agent_command: string | null;
  custom_agent_args: string[];
  timeout_secs: number;
  source_freshness_days: number;
  disclaimer: string;
}

export interface AnalysisSummary {
  id: string;
  title: string;
  user_prompt: string;
  intent: AnalysisIntent;
  status: AnalysisStatus;
  active_run_id: string | null;
  active_run_status: AnalysisStatus | null;
  block_count: number;
  source_count: number;
  created_at: string;
  updated_at: string;
}

export interface Analysis {
  id: string;
  title: string;
  user_prompt: string;
  intent: AnalysisIntent;
  status: AnalysisStatus;
  active_run_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface AnalysisRun {
  id: string;
  analysis_id: string;
  agent_id: string;
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

export interface Source {
  id: string;
  run_id: string;
  title: string;
  url: string | null;
  publisher: string | null;
  source_type: string;
  retrieved_at: string;
  reliability: 'primary' | 'high' | 'medium' | 'low';
  summary: string;
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
  | 'metric_table'
  | 'comparison_matrix'
  | 'scenario_matrix'
  | 'bar_chart'
  | 'line_chart'
  | 'area_chart'
  | 'other';

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
  | 'thesis'
  | 'business_quality'
  | 'financials'
  | 'valuation'
  | 'peer_comparison'
  | 'sector_context'
  | 'catalysts'
  | 'risks'
  | 'technical_context'
  | 'open_questions'
  | 'other';

export type Importance = 'high' | 'medium' | 'low';

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

export type StanceKind = 'bullish' | 'neutral' | 'bearish' | 'mixed' | 'insufficient_data';

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

export type ScenarioLabel = 'bull' | 'base' | 'bear';

export interface ProjectionScenario {
  label: ScenarioLabel;
  target_value: number;
  target_label: string;
  upside_pct: number;
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

export type CriterionVerdict =
  | 'confirmed'
  | 'refuted'
  | 'partially_confirmed'
  | 'unresolved';

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
  | { event: 'Log'; data: string }
  | { event: 'MessageDelta'; data: { id: string; delta: string } }
  | { event: 'ThoughtDelta'; data: { id: string; delta: string } }
  | { event: 'ToolCallStarted'; data: ToolCallStartedData }
  | { event: 'ToolCallComplete'; data: ToolCallCompleteData }
  | { event: 'Plan'; data: { entries: PlanEntry[] } }
  | { event: 'PlanSubmitted' }
  | { event: 'SourceSubmitted' }
  | { event: 'MetricSubmitted' }
  | { event: 'ArtifactSubmitted' }
  | { event: 'BlockSubmitted' }
  | { event: 'StanceSubmitted' }
  | { event: 'ProjectionSubmitted' }
  | { event: 'Completed' }
  | { event: 'Error'; data: { message: string } };

export type ProgressItemType =
  | 'agent_message'
  | 'agent_thought'
  | 'tool_call'
  | 'tool_result'
  | 'plan'
  | 'submitted'
  | 'completed'
  | 'error'
  | 'log';

export interface ProgressItem {
  id: string;
  type: ProgressItemType;
  message: string;
  timestamp: number;
  data?: unknown;
}

export type RunStatus = 'running' | 'completed' | 'error' | 'cancelled';

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
