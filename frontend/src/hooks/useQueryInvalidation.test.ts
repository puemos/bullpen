import { describe, expect, it, type Mock, vi } from "vitest";
import { queryKeys } from "@/shared/api/queries";
import type { AnalysisReport } from "@/types";
import {
  type FlushAnalysisDataChangedDeps,
  flushAnalysisDataChanged,
} from "./useQueryInvalidation";

function makeReport(id: string, activeRunId: string | null): AnalysisReport {
  return {
    analysis: {
      id,
      title: "t",
      user_prompt: "p",
      intent: null,
      status: "running",
      active_run_id: activeRunId,
      portfolio_id: null,
      created_at: "",
      updated_at: "",
    },
    runs: [],
    blocks: [],
    sources: [],
  } as unknown as AnalysisReport;
}

type FetchReport = FlushAnalysisDataChangedDeps["fetchReport"];
type SetReport = FlushAnalysisDataChangedDeps["setReport"];

interface TestDeps extends FlushAnalysisDataChangedDeps {
  invalidateQueries: Mock;
  fetchReport: FetchReport & Mock;
  setReport: SetReport & Mock;
}

function makeDeps(overrides: Partial<FlushAnalysisDataChangedDeps> = {}): TestDeps {
  const invalidateQueries = vi.fn();
  const fetchReport =
    (overrides.fetchReport as (FetchReport & Mock) | undefined) ??
    vi.fn<FetchReport>(async (id) => makeReport(id, "run-fresh"));
  const setReport = (overrides.setReport as (SetReport & Mock) | undefined) ?? vi.fn<SetReport>();
  return {
    queryClient: overrides.queryClient ?? { invalidateQueries },
    getSelectedAnalysisId: overrides.getSelectedAnalysisId ?? (() => null),
    fetchReport,
    setReport,
    invalidateQueries,
  };
}

describe("flushAnalysisDataChanged", () => {
  it("invalidates the analyses list and per-id keys for every changed id", async () => {
    const deps = makeDeps();
    await flushAnalysisDataChanged(new Set(["a-1", "a-2"]), deps);

    expect(deps.invalidateQueries).toHaveBeenCalledWith({
      queryKey: queryKeys.analyses,
    });
    expect(deps.invalidateQueries).toHaveBeenCalledWith({
      queryKey: queryKeys.analysis("a-1"),
    });
    expect(deps.invalidateQueries).toHaveBeenCalledWith({
      queryKey: queryKeys.report("a-1"),
    });
    expect(deps.invalidateQueries).toHaveBeenCalledWith({
      queryKey: queryKeys.analysis("a-2"),
    });
    expect(deps.invalidateQueries).toHaveBeenCalledWith({
      queryKey: queryKeys.report("a-2"),
    });
  });

  // Regression guard: the Zustand `selectedReport` must be refreshed when the
  // currently-selected analysis changes. Without this, `active_run_id` stays
  // stale and AnalysisPage renders "No agent activity for this analysis."
  it("refreshes the Zustand selected report when the selected analysis changed", async () => {
    const deps = makeDeps({
      getSelectedAnalysisId: () => "a-1",
      fetchReport: vi.fn<FetchReport>(async () => makeReport("a-1", "run-42")),
    });

    await flushAnalysisDataChanged(new Set(["a-1"]), deps);

    expect(deps.fetchReport).toHaveBeenCalledWith("a-1");
    expect(deps.setReport).toHaveBeenCalledTimes(1);
    const report = deps.setReport.mock.calls[0][0] as AnalysisReport;
    expect(report.analysis.active_run_id).toBe("run-42");
  });

  it("does not refetch the selected report when its id is not among changed ids", async () => {
    const deps = makeDeps({
      getSelectedAnalysisId: () => "a-1",
    });

    await flushAnalysisDataChanged(new Set(["a-2"]), deps);

    expect(deps.fetchReport).not.toHaveBeenCalled();
    expect(deps.setReport).not.toHaveBeenCalled();
  });

  it("does not update the store when the selection changes during the fetch", async () => {
    let selected: string | null = "a-1";
    const deps = makeDeps({
      getSelectedAnalysisId: () => selected,
      fetchReport: vi.fn<FetchReport>(async () => {
        selected = "a-2";
        return makeReport("a-1", "run-42");
      }),
    });

    await flushAnalysisDataChanged(new Set(["a-1"]), deps);

    expect(deps.fetchReport).toHaveBeenCalledWith("a-1");
    expect(deps.setReport).not.toHaveBeenCalled();
  });

  it("swallows fetch errors without calling setReport", async () => {
    const deps = makeDeps({
      getSelectedAnalysisId: () => "a-1",
      fetchReport: vi.fn<FetchReport>(async () => {
        throw new Error("boom");
      }),
    });

    await expect(flushAnalysisDataChanged(new Set(["a-1"]), deps)).resolves.toBeUndefined();
    expect(deps.setReport).not.toHaveBeenCalled();
  });
});
