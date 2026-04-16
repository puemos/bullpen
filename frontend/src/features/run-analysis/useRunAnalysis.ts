import { Channel } from '@tauri-apps/api/core';
import { useState } from 'react';
import {
  generateAnalysis,
  getAnalysisReport,
  stopAnalysis,
} from '@/shared/api/commands';
import {
  addProgress,
  clearProgress,
  setState,
  useAppStore,
} from '@/store';
import type { ProgressEventPayload } from '@/types';
import { handleProgressEvent } from './progress';

interface UseRunAnalysisOptions {
  agentId: string;
  canRun: boolean;
  onDone: () => Promise<void>;
}

export function useRunAnalysis({ agentId, canRun, onDone }: UseRunAnalysisOptions) {
  const activeRunId = useAppStore(state => state.activeRunId);
  const [localError, setLocalError] = useState<string | null>(null);

  const start = async (prompt: string) => {
    if (!canRun) return;

    setLocalError(null);
    clearProgress();

    const runId = crypto.randomUUID();
    setState({ isRunning: true, activeRunId: runId });

    const onProgress = new Channel<ProgressEventPayload>();
    onProgress.onmessage = payload => {
      handleProgressEvent(payload);
      if (payload.event === 'Completed' || payload.event === 'Error') {
        setState({ isRunning: false, activeRunId: null });
        onDone().catch(console.error);
      }
    };

    try {
      const result = await generateAnalysis(prompt, agentId, runId, onProgress);
      setState({
        selectedAnalysisId: result.analysis_id,
        view: 'reports',
      });
      const report = await getAnalysisReport(result.analysis_id);
      setState({ selectedReport: report });
      await onDone();
    } catch (err) {
      setLocalError(String(err));
      setState({ isRunning: false, activeRunId: null });
    }
  };

  const stop = async () => {
    if (!activeRunId) return;
    await stopAnalysis(activeRunId);
    addProgress('error', 'Stop requested');
  };

  return {
    localError,
    start,
    stop,
  };
}
