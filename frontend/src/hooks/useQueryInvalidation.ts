import { useQueryClient } from "@tanstack/react-query";
import { useRef } from "react";
import { queryKeys } from "@/shared/api/queries";
import type { DataChangedPayload } from "@/types";
import { useBackendEvent } from "./useBackendEvent";

const DATA_CHANGED_DEBOUNCE_MS = 400;
const DATA_CHANGED_MAX_WAIT_MS = 1200;

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

      queryClient.invalidateQueries({ queryKey: queryKeys.analyses });
      for (const id of changedIds) {
        queryClient.invalidateQueries({ queryKey: queryKeys.analysis(id) });
        queryClient.invalidateQueries({ queryKey: queryKeys.report(id) });
      }
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
