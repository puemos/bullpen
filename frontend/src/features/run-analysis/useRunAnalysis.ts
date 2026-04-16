import { Channel } from '@tauri-apps/api/core';
import { useState } from 'react';
import {
  createAnalysis,
  generateAnalysis,
  getAnalysisReport,
  stopAnalysis,
} from '@/shared/api/commands';
import {
  addRun,
  addRunProgress,
  getState,
  setState,
  updateRunStatus,
} from '@/store';
import type { AgentCandidate, ProgressEventPayload } from '@/types';
import { handleProgressEvent } from './progress';

interface UseRunAnalysisOptions {
  agentId: string;
  agents: AgentCandidate[];
  canRun: boolean;
  onDone: () => Promise<void>;
}

export function useRunAnalysis({ agentId, agents, canRun, onDone }: UseRunAnalysisOptions) {
  const [localError, setLocalError] = useState<string | null>(null);

  const start = async (prompt: string) => {
    if (!canRun) return;

    setLocalError(null);

    const agent = agents.find(a => a.id === agentId);

    let analysisId: string;
    try {
      analysisId = await createAnalysis(prompt);
    } catch (err) {
      setLocalError(String(err));
      return;
    }

    const runId = crypto.randomUUID();

    addRun({
      runId,
      agentId,
      agentLabel: agent?.label || agentId,
      status: 'running',
      progress: [],
      plan: [],
    });

    // Navigate to analysis detail with agent tab active
    setState({
      activeAnalysisId: analysisId,
      selectedAnalysisId: analysisId,
      selectedRunTab: runId,
      view: 'analysis',
      analysisSubTab: 'agent',
    });

    // Refresh analyses list so the sidebar shows the new entry immediately
    void onDone();

    // Fetch initial report so the report tab has data
    getAnalysisReport(analysisId).then(report => {
      // Only update if we're still viewing this analysis
      if (getState().selectedAnalysisId === analysisId) {
        setState({ selectedReport: report });
      }
    }).catch(() => {});

    const onProgress = new Channel<ProgressEventPayload>();
    onProgress.onmessage = payload => {
      handleProgressEvent(payload, runId);
      if (payload.event === 'Completed') {
        updateRunStatus(runId, 'completed');
        finishRun(analysisId);
      } else if (payload.event === 'Error') {
        updateRunStatus(runId, 'error');
        finishRun(analysisId);
      }
    };

    generateAnalysis(prompt, agentId, runId, analysisId, onProgress).catch(err => {
      const msg = String(err);
      const isCancelled = msg.includes('cancelled by user');
      updateRunStatus(runId, isCancelled ? 'cancelled' : 'error');
      if (!isCancelled) {
        addRunProgress(runId, 'error', msg);
      }
      finishRun(analysisId);
    });
  };

  const finishRun = async (analysisId: string) => {
    // Switch to report tab if we're still viewing this analysis
    const current = getState();
    if (current.selectedAnalysisId === analysisId) {
      setState({ analysisSubTab: 'report' });
      try {
        const report = await getAnalysisReport(analysisId);
        setState({ selectedReport: report });
      } catch {
        // non-critical
      }
    }
    await onDone();
  };

  const stop = async (runId?: string) => {
    if (runId) {
      await stopAnalysis(runId);
      addRunProgress(runId, 'error', 'Stop requested');
    } else {
      const runs = getState().activeRuns;
      await Promise.all(
        Object.values(runs)
          .filter(r => r.status === 'running')
          .map(r => {
            addRunProgress(r.runId, 'error', 'Stop requested');
            return stopAnalysis(r.runId);
          })
      );
    }
  };

  return {
    localError,
    start,
    stop,
  };
}
