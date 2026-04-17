import type { Channel } from "@tauri-apps/api/core";
import { invoke } from "@tauri-apps/api/core";
import type {
  AgentCandidate,
  AnalysisReport,
  AnalysisSummary,
  AppSettings,
  ProgressEventPayload,
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

export async function deleteAnalysis(analysisId: string): Promise<void> {
  return invoke("delete_analysis", { analysisId });
}

export async function stopAnalysis(runId: string): Promise<void> {
  return invoke("stop_analysis", { runId });
}

export async function exportAnalysisMarkdown(analysisId: string): Promise<string> {
  return invoke("export_analysis_markdown", { analysisId });
}

export async function createAnalysis(userPrompt: string): Promise<string> {
  return invoke("create_analysis", { userPrompt });
}

export async function setActiveRun(analysisId: string, runId: string): Promise<void> {
  return invoke("set_active_run", { analysisId, runId });
}

export async function getRunProgress(runId: string): Promise<ProgressEventPayload[]> {
  return invoke("get_run_progress", { runId });
}

export async function generateAnalysis(
  userPrompt: string,
  agentId: string,
  modelId: string | null,
  runId: string,
  analysisId: string,
  onProgress: Channel<ProgressEventPayload>,
): Promise<{ analysis_id: string; run_id: string }> {
  return invoke("generate_analysis", {
    userPrompt,
    agentId,
    modelId,
    analysisId,
    runId,
    onProgress,
  });
}
