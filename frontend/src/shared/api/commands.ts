import type { Channel } from "@tauri-apps/api/core";
import { invoke } from "@tauri-apps/api/core";
import type {
  AgentCandidate,
  AnalysisReport,
  AnalysisSummary,
  AppSettings,
  Portfolio,
  PortfolioCsvImportInput,
  PortfolioDetail,
  PortfolioImportResult,
  PortfolioSummary,
  ProgressEventPayload,
  SourceDescriptor,
  SourceKeyTestResult,
} from "@/types";

export async function getAgents(): Promise<AgentCandidate[]> {
  return invoke("get_agents");
}

export async function getSettings(): Promise<AppSettings> {
  return invoke("get_settings");
}

export async function updateSettings(config: AppSettings): Promise<AppSettings> {
  return invoke("update_settings", { config });
}

export async function getAllAnalyses(): Promise<AnalysisSummary[]> {
  return invoke("get_all_analyses");
}

export async function getAnalysisReport(
  analysisId: string,
  runId?: string,
): Promise<AnalysisReport | null> {
  return invoke("get_analysis_report", { analysisId, runId: runId ?? null });
}

export async function getStanceStaleMetrics(
  analysisId: string,
  runId?: string,
): Promise<string[]> {
  return invoke("get_stance_stale_metrics", { analysisId, runId: runId ?? null });
}

export async function deleteAnalysis(analysisId: string): Promise<void> {
  return invoke("delete_analysis", { analysisId });
}

export async function createPortfolio(
  name: string,
  baseCurrency: string,
): Promise<Portfolio> {
  return invoke("create_portfolio", { name, baseCurrency });
}

export async function getPortfolios(): Promise<PortfolioSummary[]> {
  return invoke("get_portfolios");
}

export async function getPortfolioDetail(portfolioId: string): Promise<PortfolioDetail | null> {
  return invoke("get_portfolio_detail", { portfolioId });
}

export async function importPortfolioCsv(
  input: PortfolioCsvImportInput,
): Promise<PortfolioImportResult> {
  return invoke("import_portfolio_csv", { input });
}

export async function deletePortfolio(portfolioId: string): Promise<void> {
  return invoke("delete_portfolio", { portfolioId });
}

export async function renamePortfolio(portfolioId: string, name: string): Promise<Portfolio> {
  return invoke("rename_portfolio", { portfolioId, name });
}

export async function stopAnalysis(runId: string): Promise<void> {
  return invoke("stop_analysis", { runId });
}

export async function getPriceHistory(
  symbol: string,
  market: string | null,
): Promise<number[]> {
  return invoke("get_price_history", { symbol, market });
}

export async function exportAnalysisMarkdown(analysisId: string): Promise<string> {
  return invoke("export_analysis_markdown", { analysisId });
}

export interface ExportedHtml {
  path: string;
  size_bytes: number;
}

export async function exportAnalysisHtml(analysisId: string): Promise<ExportedHtml | null> {
  return invoke("export_analysis_html", { analysisId });
}

export interface PublishedReport {
  url: string;
  delete_token: string;
  site_id: string;
  provider: string;
}

export async function publishAnalysisHtml(analysisId: string): Promise<PublishedReport> {
  return invoke("publish_analysis_html", { analysisId });
}

export async function createAnalysis(userPrompt: string): Promise<string> {
  return invoke("create_analysis", { userPrompt, portfolioId: null });
}

export async function createPortfolioAnalysis(
  portfolioId: string,
  userPrompt: string | null,
): Promise<string> {
  return invoke("create_analysis", {
    userPrompt: userPrompt ?? "",
    portfolioId,
  });
}

export async function setActiveRun(analysisId: string, runId: string): Promise<void> {
  return invoke("set_active_run", { analysisId, runId });
}

export async function getRunProgress(runId: string): Promise<ProgressEventPayload[]> {
  return invoke("get_run_progress", { runId });
}

export async function getAppVersion(): Promise<string> {
  return invoke("get_app_version");
}

export async function runSelfUpdate(): Promise<void> {
  return invoke("run_self_update");
}

export async function generateAnalysis(
  userPrompt: string,
  agentId: string,
  modelId: string | null,
  runId: string,
  analysisId: string,
  onProgress: Channel<ProgressEventPayload>,
  enabledSources: string[] | null = null,
): Promise<{ analysis_id: string; run_id: string }> {
  return invoke("generate_analysis", {
    userPrompt,
    agentId,
    modelId,
    analysisId,
    runId,
    enabledSources,
    onProgress,
  });
}

export async function listSources(): Promise<SourceDescriptor[]> {
  return invoke("list_sources");
}

export async function refreshSourceKeyStatus(): Promise<SourceDescriptor[]> {
  return invoke("refresh_source_key_status");
}

export async function setSourceKey(providerId: string, key: string): Promise<void> {
  return invoke("set_source_key", { args: { provider_id: providerId, key } });
}

export async function clearSourceKey(providerId: string): Promise<void> {
  return invoke("clear_source_key", { providerId });
}

export async function testSourceKey(providerId: string): Promise<SourceKeyTestResult> {
  return invoke("test_source_key", { providerId });
}

export async function setEnabledSources(ids: string[]): Promise<string[]> {
  return invoke("set_enabled_sources", { ids });
}
