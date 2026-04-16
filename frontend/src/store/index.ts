import { useSyncExternalStore } from 'react';
import { DEFAULT_APP_VIEW } from '@/app/navigation';
import type { AppView } from '@/app/navigation';
import type {
  AnalysisReport,
  AnalysisSummary,
  PlanEntry,
  ProgressItem,
  ProgressItemType,
} from '../types';

interface State {
  view: AppView;
  analyses: AnalysisSummary[];
  selectedAnalysisId: string | null;
  selectedReport: AnalysisReport | null;
  isRunning: boolean;
  activeRunId: string | null;
  progress: ProgressItem[];
  plan: PlanEntry[];
  agentId: string;
}

const state: State = {
  view: DEFAULT_APP_VIEW,
  analyses: [],
  selectedAnalysisId: null,
  selectedReport: null,
  isRunning: false,
  activeRunId: null,
  progress: [],
  plan: [],
  agentId: '',
};

const listeners = new Set<() => void>();

function emit() {
  for (const listener of listeners) listener();
}

export function setState(partial: Partial<State>) {
  Object.assign(state, partial);
  emit();
}

export function getState(): State {
  return state;
}

export function addProgress(type: ProgressItemType, message: string, data?: unknown) {
  state.progress = [
    ...state.progress,
    {
      id: crypto.randomUUID(),
      type,
      message,
      timestamp: Date.now(),
      data,
    },
  ];
  emit();
}

export function appendProgress(type: ProgressItemType, delta: string) {
  const copy = [...state.progress];
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
  state.progress = copy;
  emit();
}

export function clearProgress() {
  state.progress = [];
  state.plan = [];
  emit();
}

export function useAppStore<T>(selector: (state: State) => T): T {
  return useSyncExternalStore(
    callback => {
      listeners.add(callback);
      return () => listeners.delete(callback);
    },
    () => selector(state),
    () => selector(state)
  );
}
