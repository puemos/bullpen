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
  planned_checks: string[];
  required_blocks: string[];
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
  as_of: string | null;
  reliability: 'primary' | 'high' | 'medium' | 'low';
  summary: string;
}

export interface MetricSnapshot {
  id: string;
  run_id: string;
  entity_id: string | null;
  metric: string;
  value: string;
  unit: string | null;
  period: string | null;
  as_of: string;
  source_id: string;
  notes: string | null;
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
  | 'scenario_matrix'
  | 'technical_context'
  | 'open_questions'
  | 'other';

export interface AnalysisBlock {
  id: string;
  run_id: string;
  kind: BlockKind;
  title: string;
  body: string;
  evidence_ids: string[];
  entity_ids: string[];
  confidence: number;
  importance: 'high' | 'medium' | 'low' | string;
  display_order: number;
  created_at: string;
}

export interface FinalStance {
  id: string;
  run_id: string;
  stance: 'bullish' | 'neutral' | 'bearish' | 'mixed' | 'insufficient_data';
  horizon: string;
  confidence: number;
  summary: string;
  key_reasons: string[];
  watch_items: string[];
  what_would_change: string[];
  disclaimer: string;
  created_at: string;
}

export interface AnalysisReport {
  analysis: Analysis;
  runs: AnalysisRun[];
  research_plan: ResearchPlan | null;
  entities: Entity[];
  sources: Source[];
  metrics: MetricSnapshot[];
  blocks: AnalysisBlock[];
  final_stance: FinalStance | null;
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
  | { event: 'BlockSubmitted' }
  | { event: 'StanceSubmitted' }
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
