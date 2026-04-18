import { WarningCircle } from "@phosphor-icons/react";
import { type CSSProperties, useCallback, useEffect, useRef, useState } from "react";
import { toast, Toaster } from "sonner";
import { AppSidebar } from "@/app/AppSidebar";
import { SidebarInset, SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";
import { AnalysisPage } from "@/features/analysis/AnalysisPage";
import { CompareView } from "@/features/compare/CompareView";
import { ResearchPage } from "@/features/run-analysis/ResearchPage";
import { SettingsPage } from "@/features/settings/SettingsPage";
import { UpdateDialog } from "@/features/updates/UpdateDialog";
import { useBackendEvent } from "@/hooks/useBackendEvent";
import { useUpdateCheck } from "@/hooks/useUpdateCheck";
import {
  getAgents,
  getAllAnalyses,
  getAnalysisReport,
  getSettings,
  loadCompareReports,
} from "@/shared/api/commands";
import {
  getState,
  setCompareMode,
  setCompareReport,
  setCompareSelection,
  setSelectedReport,
  setState,
  useAppStore,
} from "@/store";
import type { AgentCandidate, DataChangedPayload } from "@/types";
import { COMPARE_MAX, COMPARE_MIN } from "./navigation";
import type { AppView } from "./navigation";

const DATA_CHANGED_DEBOUNCE_MS = 400;
const DATA_CHANGED_MAX_WAIT_MS = 1200;

export function App() {
  const view = useAppStore((state) => state.view);
  const analyses = useAppStore((state) => state.analyses);
  const selectedAnalysisId = useAppStore((state) => state.selectedAnalysisId);
  const compareMode = useAppStore((state) => state.compareMode);
  const compareAnalysisIds = useAppStore((state) => state.compareAnalysisIds);
  const [agents, setAgents] = useState<AgentCandidate[]>([]);
  const [error, setError] = useState<string | null>(null);
  const { currentVersion, updateAvailable } = useUpdateCheck();
  const [updateDialogOpen, setUpdateDialogOpen] = useState(false);
  const toastFiredRef = useRef(false);

  useEffect(() => {
    if (!updateAvailable || toastFiredRef.current) return;
    toastFiredRef.current = true;
    toast("Update available", {
      description: `v${updateAvailable.latestVersion} is ready to install.`,
      action: {
        label: "Details",
        onClick: () => setUpdateDialogOpen(true),
      },
      duration: 12000,
    });
  }, [updateAvailable]);

  const openUpdateDialog = useCallback(() => setUpdateDialogOpen(true), []);

  const refresh = useCallback(async () => {
    setError(null);
    try {
      const [nextAgents, nextAnalyses, settings] = await Promise.all([
        getAgents(),
        getAllAnalyses(),
        getSettings(),
      ]);
      const selected = getState().selectedAnalysisId;
      const selectedStillExists = selected
        ? nextAnalyses.some((analysis) => analysis.id === selected)
        : false;

      setAgents(nextAgents);
      setState({
        analyses: nextAnalyses,
        ...(selected && !selectedStillExists
          ? { selectedAnalysisId: null, selectedReport: null }
          : {}),
        agentId:
          getState().agentId ||
          nextAgents.find((agent) => agent.available)?.id ||
          nextAgents[0]?.id ||
          "",
        modelByAgent: { ...(settings.model_by_agent ?? {}) },
      });

      if (selected && selectedStillExists) {
        const report = await getAnalysisReport(selected);
        setSelectedReport(report);
      }
    } catch (err) {
      setError(String(err));
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const debounceRef = useRef<number | null>(null);
  const maxWaitRef = useRef<number | null>(null);
  const pendingAnalysisIdsRef = useRef<Set<string>>(new Set());

  useBackendEvent<DataChangedPayload>("analysis-data-changed", (payload) => {
    pendingAnalysisIdsRef.current.add(payload.analysis_id);

    const flush = async () => {
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
      try {
        const nextAnalyses = await getAllAnalyses();
        const selected = getState().selectedAnalysisId;
        const selectedStillExists = selected
          ? nextAnalyses.some((analysis) => analysis.id === selected)
          : false;

        setState({
          analyses: nextAnalyses,
          ...(selected && !selectedStillExists
            ? { selectedAnalysisId: null, selectedReport: null }
            : {}),
        });

        if (selected && selectedStillExists && changedIds.has(selected)) {
          const report = await getAnalysisReport(selected);
          if (getState().selectedAnalysisId === selected) {
            setSelectedReport(report);
          }
        }
      } catch (err) {
        setError(String(err));
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

  useEffect(
    () => () => {
      if (debounceRef.current !== null) {
        window.clearTimeout(debounceRef.current);
      }
      if (maxWaitRef.current !== null) {
        window.clearTimeout(maxWaitRef.current);
      }
    },
    [],
  );

  const selectAnalysis = useCallback(async (analysisId: string) => {
    setError(null);

    // Check if this analysis has an active in-memory run
    const runs = getState().activeRuns;
    const isRunning = Object.values(runs).some(
      (r) => r.status === "running" && getState().activeAnalysisId === analysisId,
    );

    setState({
      selectedAnalysisId: analysisId,
      selectedReport: null,
      view: "analysis",
      analysisSubTab: isRunning ? "agent" : "report",
    });

    try {
      const report = await getAnalysisReport(analysisId);
      // Refine: if the run is still running according to the report, show agent tab
      const runIsActive =
        report?.analysis.active_run_id &&
        Object.values(getState().activeRuns).some(
          (r) => r.runId === report.analysis.active_run_id && r.status === "running",
        );
      setSelectedReport(report);
      if (runIsActive) {
        setState({ analysisSubTab: "agent" });
      }
    } catch (err) {
      setError(String(err));
    }
  }, []);

  const changeView = useCallback((nextView: AppView) => {
    setError(null);
    setState({ view: nextView });
  }, []);

  const toggleCompareMode = useCallback(() => {
    setError(null);
    setCompareMode(!getState().compareMode);
  }, []);

  const toggleCompareSelection = useCallback((analysisId: string) => {
    const current = getState().compareAnalysisIds;
    if (current.includes(analysisId)) {
      setCompareSelection(current.filter((id) => id !== analysisId));
      return;
    }
    if (current.length >= COMPARE_MAX) return;
    setCompareSelection([...current, analysisId]);
  }, []);

  const submitCompare = useCallback(async () => {
    const ids = getState().compareAnalysisIds;
    if (ids.length < COMPARE_MIN || ids.length > COMPARE_MAX) return;
    setState({ view: "compare" });
    try {
      const reports = await loadCompareReports(ids);
      for (const [id, report] of Object.entries(reports)) {
        setCompareReport(id, report);
      }
    } catch (err) {
      setError(String(err));
    }
  }, []);

  return (
    <SidebarProvider
      className="h-screen min-h-0 overflow-hidden bg-background text-foreground"
      style={{ "--sidebar-width": "17rem" } as CSSProperties}
    >
      <AppSidebar
        analyses={analyses}
        currentView={view}
        selectedAnalysisId={selectedAnalysisId}
        compareMode={compareMode}
        compareAnalysisIds={compareAnalysisIds}
        onViewChange={changeView}
        onSelectAnalysis={selectAnalysis}
        onToggleCompareMode={toggleCompareMode}
        onToggleCompareSelection={toggleCompareSelection}
        onSubmitCompare={() => {
          void submitCompare();
        }}
        currentVersion={currentVersion}
        updateAvailable={Boolean(updateAvailable)}
        onUpdateClick={openUpdateDialog}
      />
      <SidebarInset className="h-screen min-w-0 overflow-hidden">
        <div data-tauri-drag-region className="absolute left-0 right-0 top-0 z-10 h-3" />
        <header className="flex h-10 shrink-0 items-center gap-2 border-b border-border px-3 md:hidden">
          <SidebarTrigger />
        </header>
        {error && (
          <div className="mx-6 mt-4 flex items-center gap-2 rounded-md bg-destructive/10 px-4 py-2 text-sm text-destructive">
            <WarningCircle size={16} />
            {error}
          </div>
        )}
        <div className="min-h-0 flex-1 overflow-hidden">
          {view === "new-analysis" && <ResearchPage agents={agents} onDone={refresh} />}
          {view === "analysis" && <AnalysisPage onRefresh={refresh} />}
          {view === "compare" && <CompareView />}
          {view === "settings" && <SettingsPage agents={agents} />}
        </div>
      </SidebarInset>
      {updateAvailable && currentVersion && (
        <UpdateDialog
          open={updateDialogOpen}
          onOpenChange={setUpdateDialogOpen}
          currentVersion={currentVersion}
          updateInfo={updateAvailable}
        />
      )}
      <Toaster
        position="bottom-right"
        toastOptions={{
          classNames: {
            toast:
              "!rounded-none !border !border-border !bg-background !text-foreground !shadow-none",
            description: "!text-muted-foreground",
          },
        }}
      />
    </SidebarProvider>
  );
}
