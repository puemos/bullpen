import { type QueryClient, useQueryClient } from "@tanstack/react-query";
import { useRef } from "react";
import { getAnalysisReport } from "@/shared/api/commands";
import { queryKeys } from "@/shared/api/queries";
import { getState, setSelectedReport } from "@/store";
import type { AnalysisReport, DataChangedPayload } from "@/types";
import { useBackendEvent } from "./useBackendEvent";

const DATA_CHANGED_DEBOUNCE_MS = 400;
const DATA_CHANGED_MAX_WAIT_MS = 1200;

export interface FlushAnalysisDataChangedDeps {
  queryClient: Pick<QueryClient, "invalidateQueries">;
  getSelectedAnalysisId: () => string | null;
  fetchReport: (id: string) => Promise<AnalysisReport | null>;
  setReport: (report: AnalysisReport | null) => void;
}

// AnalysisPage reads `selectedReport` from the Zustand store, not React Query,
// so invalidating the query cache is not enough — also refresh the store copy
// when the currently-selected analysis is among the changed IDs.
export async function flushAnalysisDataChanged(
  changedIds: ReadonlySet<string>,
  deps: FlushAnalysisDataChangedDeps,
): Promise<void> {
  deps.queryClient.invalidateQueries({ queryKey: queryKeys.analyses });
  for (const id of changedIds) {
    deps.queryClient.invalidateQueries({ queryKey: queryKeys.analysis(id) });
    deps.queryClient.invalidateQueries({ queryKey: queryKeys.report(id) });
  }

  const selected = deps.getSelectedAnalysisId();
  if (selected && changedIds.has(selected)) {
    try {
      const report = await deps.fetchReport(selected);
      if (deps.getSelectedAnalysisId() === selected) {
        deps.setReport(report);
      }
    } catch {
      // non-critical; next interaction will refetch
    }
  }
}

export function useQueryInvalidation() {
  const queryClient = useQueryClient();
  const debounceRef = useRef<number | null>(null);
  const maxWaitRef = useRef<number | null>(null);
  const pendingAnalysisIdsRef = useRef<Set<string>>(new Set());

  useBackendEvent<DataChangedPayload>("analysis-data-changed", (payload) => {
    pendingAnalysisIdsRef.current.add(payload.analysis_id);

    const flush = () => {
      if (debounceRef.current !== null) {
        window.clearTimeout(debounceRef.current);
        debounceRef.current = null;
      }
      if (maxWaitRef.current !== null) {
        window.clearTimeout(maxWaitRef.current);
        maxWaitRef.current = null;
      }
      const changedIds = pendingAnalysisIdsRef.current;
      pendingAnalysisIdsRef.current = new Set();

      void flushAnalysisDataChanged(changedIds, {
        queryClient,
        getSelectedAnalysisId: () => getState().selectedAnalysisId,
        fetchReport: getAnalysisReport,
        setReport: setSelectedReport,
      });
    };

    if (debounceRef.current !== null) {
      window.clearTimeout(debounceRef.current);
    }
    debounceRef.current = window.setTimeout(flush, DATA_CHANGED_DEBOUNCE_MS);
    if (maxWaitRef.current === null) {
      maxWaitRef.current = window.setTimeout(flush, DATA_CHANGED_MAX_WAIT_MS);
    }
  });
}
