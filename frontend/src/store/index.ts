import { useSyncExternalStore } from "react";
import type { AppView } from "@/app/navigation";
import { DEFAULT_APP_VIEW } from "@/app/navigation";
import type {
  AnalysisReport,
  AnalysisSummary,
  PlanEntry,
  ProgressItem,
  ProgressItemType,
  RunState,
} from "../types";

interface State {
  view: AppView;
  analyses: AnalysisSummary[];
  selectedAnalysisId: string | null;
  selectedReport: AnalysisReport | null;
  // Single agent selection
  agentId: string;
  modelByAgent: Record<string, string | null>;
  // Per-run state — supports multiple concurrent analyses
  activeRuns: Record<string, RunState>;
  activeAnalysisId: string | null;
  selectedRunTab: string | null;
  // Sub-tab within analysis detail view
  analysisSubTab: "report" | "agent";
  // Compare view: sidebar toggle exposes per-analysis checkboxes; picking
  // 2–4 finalized analyses opens the compare view.
  compareMode: boolean;
  compareAnalysisIds: string[];
  compareReports: Record<string, AnalysisReport | null>;
}

const state: State = {
  view: DEFAULT_APP_VIEW,
  analyses: [],
  selectedAnalysisId: null,
  selectedReport: null,
  agentId: "",
  modelByAgent: {},
  activeRuns: {},
  activeAnalysisId: null,
  selectedRunTab: null,
  analysisSubTab: "agent",
  compareMode: false,
  compareAnalysisIds: [],
  compareReports: {},
};

const listeners = new Set<() => void>();

function emit() {
  for (const listener of listeners) listener();
}

export function setState(partial: Partial<State>) {
  Object.assign(state, partial);
  emit();
}

export function setSelectedReport(next: AnalysisReport | null) {
  state.selectedReport = stableMerge(state.selectedReport, next);
  emit();
}

export function setCompareSelection(ids: string[]) {
  state.compareAnalysisIds = ids;
  // Drop stale report entries whose ids fell out of the selection.
  const keep: Record<string, AnalysisReport | null> = {};
  for (const id of ids) {
    if (id in state.compareReports) keep[id] = state.compareReports[id] ?? null;
  }
  state.compareReports = keep;
  emit();
}

export function setCompareReport(id: string, report: AnalysisReport | null) {
  state.compareReports = { ...state.compareReports, [id]: report };
  emit();
}

export function setCompareMode(next: boolean) {
  state.compareMode = next;
  if (!next) {
    state.compareAnalysisIds = [];
    state.compareReports = {};
  }
  emit();
}

function stableMerge<T>(prev: T, next: T): T {
  if (prev === next) return prev;
  if (prev === null || next === null || typeof prev !== "object" || typeof next !== "object") {
    return next;
  }
  const prevIsArr = Array.isArray(prev);
  const nextIsArr = Array.isArray(next);
  if (prevIsArr !== nextIsArr) return next;
  if (prevIsArr && nextIsArr) {
    const a = prev as unknown as unknown[];
    const b = next as unknown as unknown[];
    if (a.length !== b.length) return next;
    const merged = new Array(b.length);
    let allSame = true;
    for (let i = 0; i < b.length; i++) {
      merged[i] = stableMerge(a[i], b[i]);
      if (merged[i] !== a[i]) allSame = false;
    }
    return allSame ? prev : (merged as unknown as T);
  }
  const a = prev as Record<string, unknown>;
  const b = next as Record<string, unknown>;
  const aKeys = Object.keys(a);
  const bKeys = Object.keys(b);
  if (aKeys.length !== bKeys.length) return next;
  const merged: Record<string, unknown> = {};
  let allSame = true;
  for (const key of bKeys) {
    if (!(key in a)) return next;
    merged[key] = stableMerge(a[key], b[key]);
    if (merged[key] !== a[key]) allSame = false;
  }
  return allSame ? prev : (merged as unknown as T);
}

export function getState(): State {
  return state;
}

export function addRun(runState: RunState) {
  state.activeRuns = { ...state.activeRuns, [runState.runId]: runState };
  if (!state.selectedRunTab) {
    state.selectedRunTab = runState.runId;
  }
  emit();
}

export function updateRunStatus(runId: string, status: RunState["status"]) {
  const run = state.activeRuns[runId];
  if (!run) return;
  state.activeRuns = {
    ...state.activeRuns,
    [runId]: { ...run, status },
  };
  emit();
}

export function addRunProgress(
  runId: string,
  type: ProgressItemType,
  message: string,
  data?: unknown,
) {
  const run = state.activeRuns[runId];
  if (!run) return;
  state.activeRuns = {
    ...state.activeRuns,
    [runId]: {
      ...run,
      progress: [
        ...run.progress,
        {
          id: crypto.randomUUID(),
          type,
          message,
          timestamp: Date.now(),
          data,
        },
      ],
    },
  };
  emit();
}

export function appendRunProgress(runId: string, type: ProgressItemType, delta: string) {
  const run = state.activeRuns[runId];
  if (!run) return;
  const copy = [...run.progress];
  const last = copy[copy.length - 1];
  if (last && last.type === type) {
    copy[copy.length - 1] = { ...last, message: last.message + delta };
  } else {
    copy.push({
      id: crypto.randomUUID(),
      type,
      message: delta,
      timestamp: Date.now(),
    });
  }
  state.activeRuns = {
    ...state.activeRuns,
    [runId]: { ...run, progress: copy },
  };
  emit();
}

export function setRunPlan(runId: string, plan: PlanEntry[]) {
  const run = state.activeRuns[runId];
  if (!run) return;
  state.activeRuns = {
    ...state.activeRuns,
    [runId]: { ...run, plan },
  };
  emit();
}

export function setRunProgress(runId: string, progress: ProgressItem[]) {
  const run = state.activeRuns[runId];
  if (!run) return;
  state.activeRuns = {
    ...state.activeRuns,
    [runId]: { ...run, progress },
  };
  emit();
}

export function clearRuns() {
  state.activeRuns = {};
  state.activeAnalysisId = null;
  state.selectedRunTab = null;
  emit();
}

export function isAnyRunActive(s: State): boolean {
  return Object.values(s.activeRuns).some((r) => r.status === "running");
}

export function useAppStore<T>(selector: (state: State) => T): T {
  return useSyncExternalStore(
    (callback) => {
      listeners.add(callback);
      return () => listeners.delete(callback);
    },
    () => selector(state),
    () => selector(state),
  );
}
