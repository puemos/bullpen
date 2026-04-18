import { WarningCircle } from "@phosphor-icons/react";
import { type CSSProperties, useCallback, useEffect, useRef, useState } from "react";
import { Toaster, toast } from "sonner";
import { AppSidebar } from "@/app/AppSidebar";
import { SidebarInset, SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";
import { AnalysisPage } from "@/features/analysis/AnalysisPage";
import { PortfolioPage } from "@/features/portfolio/PortfolioPage";
import { ResearchPage } from "@/features/run-analysis/ResearchPage";
import { SettingsPage } from "@/features/settings/SettingsPage";
import { UpdateDialog } from "@/features/updates/UpdateDialog";
import { useBackendEvent } from "@/hooks/useBackendEvent";
import { useUpdateCheck } from "@/hooks/useUpdateCheck";
import {
  getAgents,
  getAllAnalyses,
  getAnalysisReport,
  getPortfolioDetail,
  getPortfolios,
  getSettings,
} from "@/shared/api/commands";
import { getState, setSelectedReport, setState, useAppStore } from "@/store";
import type { AgentCandidate, DataChangedPayload } from "@/types";
import type { AppView } from "./navigation";

const DATA_CHANGED_DEBOUNCE_MS = 400;
const DATA_CHANGED_MAX_WAIT_MS = 1200;

export function App() {
  const view = useAppStore((state) => state.view);
  const analyses = useAppStore((state) => state.analyses);
  const portfolios = useAppStore((state) => state.portfolios);
  const selectedAnalysisId = useAppStore((state) => state.selectedAnalysisId);
  const selectedPortfolioId = useAppStore((state) => state.selectedPortfolioId);
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
      const [nextAgents, nextAnalyses, nextPortfolios, settings] = await Promise.all([
        getAgents(),
        getAllAnalyses(),
        getPortfolios(),
        getSettings(),
      ]);
      const selected = getState().selectedAnalysisId;
      const selectedStillExists = selected
        ? nextAnalyses.some((analysis) => analysis.id === selected)
        : false;
      const selectedPortfolio = getState().selectedPortfolioId;
      const selectedPortfolioStillExists = selectedPortfolio
        ? nextPortfolios.some((portfolio) => portfolio.id === selectedPortfolio)
        : false;

      setAgents(nextAgents);
      setState({
        analyses: nextAnalyses,
        portfolios: nextPortfolios,
        ...(selected && !selectedStillExists
          ? { selectedAnalysisId: null, selectedReport: null }
          : {}),
        ...(selectedPortfolio && !selectedPortfolioStillExists
          ? { selectedPortfolioId: null, selectedPortfolio: null }
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
      if (selectedPortfolio && selectedPortfolioStillExists) {
        const portfolio = await getPortfolioDetail(selectedPortfolio);
        setState({ selectedPortfolio: portfolio });
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

  const selectPortfolio = useCallback(async (portfolioId: string) => {
    setError(null);
    const summary = getState().portfolios.find((p) => p.id === portfolioId);

    // Optimistic: swap the header to the new portfolio's summary immediately so
    // switching feels instant. The actual holdings arrive when the fetch
    // resolves and we replace the shell.
    const optimistic = summary
      ? {
          portfolio: {
            id: summary.id,
            name: summary.name,
            base_currency: summary.base_currency,
            created_at: summary.updated_at,
            updated_at: summary.updated_at,
          },
          accounts: [],
          holdings: [],
          positions: [],
          transactions: [],
          import_batches: [],
          totals_by_currency: [],
        }
      : null;

    setState({
      selectedPortfolioId: portfolioId,
      selectedPortfolio: optimistic,
      view: "portfolio",
    });

    try {
      const portfolio = await getPortfolioDetail(portfolioId);
      if (getState().selectedPortfolioId === portfolioId) {
        setState({ selectedPortfolio: portfolio });
      }
    } catch (err) {
      setError(String(err));
    }
  }, []);

  const newPortfolio = useCallback(() => {
    setError(null);
    // Switch to the portfolio view's empty-create state so the user picks a
    // base currency before the portfolio is actually persisted.
    setState({
      selectedPortfolioId: null,
      selectedPortfolio: null,
      view: "portfolio",
    });
  }, []);

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if (!(event.metaKey || event.ctrlKey) || !event.shiftKey) return;
      const target = event.target as HTMLElement | null;
      const tag = target?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || target?.isContentEditable) return;
      const key = event.key.toLowerCase();
      if (key === "a") {
        event.preventDefault();
        setState({ view: "new-analysis" });
      } else if (key === "p") {
        event.preventDefault();
        newPortfolio();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [newPortfolio]);

  const changeView = useCallback((nextView: AppView) => {
    setError(null);
    setState({ view: nextView });
  }, []);

  return (
    <SidebarProvider
      className="h-screen min-h-0 overflow-hidden bg-background text-foreground"
      style={{ "--sidebar-width": "17rem" } as CSSProperties}
    >
      <AppSidebar
        analyses={analyses}
        portfolios={portfolios}
        currentView={view}
        selectedAnalysisId={selectedAnalysisId}
        selectedPortfolioId={selectedPortfolioId}
        onViewChange={changeView}
        onSelectAnalysis={selectAnalysis}
        onSelectPortfolio={selectPortfolio}
        onNewPortfolio={newPortfolio}
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
          {view === "portfolio" && (
            <PortfolioPage agents={agents} onRefresh={refresh} onSelectAnalysis={selectAnalysis} />
          )}
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
