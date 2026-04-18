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
import { useQueryInvalidation } from "@/hooks/useQueryInvalidation";
import { useUpdateCheck } from "@/hooks/useUpdateCheck";
import { getAnalysisReport, getPortfolioDetail } from "@/shared/api/commands";
import { useAgents, useAnalyses, usePortfolios, useSettings } from "@/shared/api/queries";
import { getState, setSelectedReport, setState, useAppStore } from "@/store";
import type { AppView } from "./navigation";

export function App() {
  const view = useAppStore((state) => state.view);
  const selectedAnalysisId = useAppStore((state) => state.selectedAnalysisId);
  const selectedPortfolioId = useAppStore((state) => state.selectedPortfolioId);
  const [error, setError] = useState<string | null>(null);
  const { currentVersion, updateAvailable } = useUpdateCheck();
  const [updateDialogOpen, setUpdateDialogOpen] = useState(false);
  const toastFiredRef = useRef(false);

  const { data: agents = [] } = useAgents();
  const { data: analyses = [] } = useAnalyses();
  const { data: portfolios = [] } = usePortfolios();
  const { data: settings } = useSettings();

  useQueryInvalidation();

  useEffect(() => {
    if (settings) {
      const currentAgentId = getState().agentId;
      if (!currentAgentId && agents.length > 0) {
        setState({
          agentId: agents.find((agent) => agent.available)?.id || agents[0]?.id || "",
        });
      }
      setState({ modelByAgent: { ...(settings.model_by_agent ?? {}) } });
    }
  }, [settings, agents]);

  useEffect(() => {
    const selected = getState().selectedAnalysisId;
    if (selected && !analyses.some((a) => a.id === selected)) {
      setState({ selectedAnalysisId: null, selectedReport: null });
    }
  }, [analyses]);

  useEffect(() => {
    const selected = getState().selectedPortfolioId;
    if (selected && !portfolios.some((p) => p.id === selected)) {
      setState({ selectedPortfolioId: null, selectedPortfolio: null });
    }
  }, [portfolios]);

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

  const selectPortfolio = useCallback(
    async (portfolioId: string) => {
      setError(null);
      const summary = portfolios.find((p) => p.id === portfolioId);

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
    },
    [portfolios],
  );

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
          {view === "new-analysis" && <ResearchPage agents={agents} />}
          {view === "analysis" && <AnalysisPage />}
          {view === "portfolio" && (
            <PortfolioPage agents={agents} onSelectAnalysis={selectAnalysis} />
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
