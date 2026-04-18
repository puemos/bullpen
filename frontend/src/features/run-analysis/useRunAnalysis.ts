import { useQueryClient } from "@tanstack/react-query";
import { Channel } from "@tauri-apps/api/core";
import { useState } from "react";
import {
  createAnalysis,
  generateAnalysis,
  getAnalysisReport,
  stopAnalysis,
} from "@/shared/api/commands";
import { queryKeys } from "@/shared/api/queries";
import {
  addRun,
  addRunProgress,
  getState,
  setSelectedReport,
  setState,
  updateRunStatus,
} from "@/store";
import type { AgentCandidate, ProgressEventPayload } from "@/types";
import { handleProgressEvent } from "./progress";

interface UseRunAnalysisOptions {
  agentId: string;
  agents: AgentCandidate[];
  canRun: boolean;
}

export function useRunAnalysis({ agentId, agents, canRun }: UseRunAnalysisOptions) {
  const queryClient = useQueryClient();
  const [localError, setLocalError] = useState<string | null>(null);

  const startWithAnalysisId = (
    analysisId: string,
    prompt: string,
    overrides?: {
      agentId?: string;
      modelId?: string | null;
      enabledSources?: string[] | null;
    },
  ) => {
    const effectiveAgentId = overrides?.agentId ?? agentId;
    const agent = agents.find((a) => a.id === effectiveAgentId);
    const modelId =
      overrides?.modelId !== undefined
        ? overrides.modelId
        : (getState().modelByAgent[effectiveAgentId] ?? null);
    const enabledSources = overrides?.enabledSources ?? null;
    const runId = crypto.randomUUID();

    addRun({
      runId,
      agentId: effectiveAgentId,
      agentLabel: agent?.label || effectiveAgentId,
      status: "running",
      progress: [],
      plan: [],
    });

    setState({
      activeAnalysisId: analysisId,
      selectedAnalysisId: analysisId,
      selectedRunTab: runId,
      view: "analysis",
      analysisSubTab: "agent",
    });

    void queryClient.invalidateQueries({ queryKey: queryKeys.analyses });

    getAnalysisReport(analysisId)
      .then((report) => {
        if (getState().selectedAnalysisId === analysisId) {
          setSelectedReport(report);
        }
      })
      .catch(() => {});

    const onProgress = new Channel<ProgressEventPayload>();
    onProgress.onmessage = (payload) => {
      handleProgressEvent(payload, runId);
      if (payload.event === "Completed") {
        updateRunStatus(runId, "completed");
        finishRun(analysisId);
      } else if (payload.event === "Error") {
        updateRunStatus(runId, "error");
        finishRun(analysisId);
      }
    };

    generateAnalysis(
      prompt,
      effectiveAgentId,
      modelId,
      runId,
      analysisId,
      onProgress,
      enabledSources,
    ).catch((err) => {
      const msg = String(err);
      const isCancelled = msg.includes("cancelled by user");
      updateRunStatus(runId, isCancelled ? "cancelled" : "error");
      if (!isCancelled) {
        addRunProgress(runId, "error", msg);
      }
      finishRun(analysisId);
    });
  };

  const start = async (prompt: string, enabledSources: string[] | null = null) => {
    if (!canRun) return;

    setLocalError(null);

    let analysisId: string;
    try {
      analysisId = await createAnalysis(prompt);
    } catch (err) {
      setLocalError(String(err));
      return;
    }

    startWithAnalysisId(analysisId, prompt, { enabledSources });
  };

  const finishRun = async (analysisId: string) => {
    const current = getState();
    if (current.selectedAnalysisId === analysisId) {
      setState({ analysisSubTab: "report" });
      try {
        const report = await getAnalysisReport(analysisId);
        setSelectedReport(report);
      } catch {
        // non-critical
      }
    }
    await queryClient.invalidateQueries({ queryKey: queryKeys.analyses });
    await queryClient.invalidateQueries({ queryKey: queryKeys.analysis(analysisId) });
  };

  const stop = async (runId?: string) => {
    if (runId) {
      await stopAnalysis(runId);
      addRunProgress(runId, "error", "Stop requested");
    } else {
      const runs = getState().activeRuns;
      await Promise.all(
        Object.values(runs)
          .filter((r) => r.status === "running")
          .map((r) => {
            addRunProgress(r.runId, "error", "Stop requested");
            return stopAnalysis(r.runId);
          }),
      );
    }
  };

  return {
    localError,
    start,
    startWithAnalysisId,
    stop,
  };
}
