import { invoke } from '@tauri-apps/api/core';
import type { Channel } from '@tauri-apps/api/core';
import type {
  AgentCandidate,
  AppSettings,
  AnalysisReport,
  AnalysisSummary,
  ProgressEventPayload,
} from '@/types';

export async function getAgents(): Promise<AgentCandidate[]> {
  return invoke('get_agents');
}

export async function getSettings(): Promise<AppSettings> {
  return invoke('get_settings');
}

export async function updateSettings(config: AppSettings): Promise<AppSettings> {
  return invoke('update_settings', { config });
}

export async function getAllAnalyses(): Promise<AnalysisSummary[]> {
  return invoke('get_all_analyses');
}

export async function getAnalysisReport(analysisId: string): Promise<AnalysisReport | null> {
  return invoke('get_analysis_report', { analysisId });
}

export async function deleteAnalysis(analysisId: string): Promise<void> {
  return invoke('delete_analysis', { analysisId });
}

export async function stopAnalysis(runId: string): Promise<void> {
  return invoke('stop_analysis', { runId });
}

export async function exportAnalysisMarkdown(analysisId: string): Promise<string> {
  return invoke('export_analysis_markdown', { analysisId });
}

export async function generateAnalysis(
  userPrompt: string,
  agentId: string,
  runId: string,
  onProgress: Channel<ProgressEventPayload>
): Promise<{ analysis_id: string; run_id: string }> {
  return invoke('generate_analysis', {
    userPrompt,
    agentId,
    runId,
    onProgress,
  });
}
