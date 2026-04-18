import {
  CaretDown,
  CircleNotch,
  DotsThree,
  FileArrowUp,
  SpinnerGap,
  WarningCircle,
} from "@phosphor-icons/react";
import {
  type ChangeEvent,
  type KeyboardEvent as ReactKeyboardEvent,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Eyebrow, SectionHeader } from "@/components/ui/editorial";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useRunAnalysis } from "@/features/run-analysis/useRunAnalysis";
import { getLogoPath } from "@/lib/agents";
import {
  createPortfolio,
  createPortfolioAnalysis,
  deletePortfolio,
  getPortfolioDetail,
  getPriceHistory,
  getSettings,
  importPortfolioCsv,
  renamePortfolio,
  updateSettings,
} from "@/shared/api/commands";
import { getState, setState, useAppStore } from "@/store";
import type {
  AgentCandidate,
  AnalysisSummary,
  PortfolioCsvImportInput,
  PortfolioCsvRow,
  PortfolioDetail,
  PortfolioHolding,
} from "@/types";

async function persistModelByAgent(map: Record<string, string | null>) {
  try {
    const settings = await getSettings();
    const next: Record<string, string> = {};
    for (const [id, value] of Object.entries(map)) {
      if (value) next[id] = value;
    }
    await updateSettings({ ...settings, model_by_agent: next });
  } catch {
    // non-critical
  }
}

type CsvField = keyof Omit<PortfolioCsvRow, "row_index" | "raw">;

interface PortfolioPageProps {
  agents: AgentCandidate[];
  onRefresh: () => Promise<void> | void;
  onSelectAnalysis: (analysisId: string) => void | Promise<void>;
}

const CURRENCY_OPTIONS = ["USD", "EUR", "GBP", "CHF", "JPY", "CAD", "AUD", "SEK", "NOK"] as const;

const FIELD_ALIASES: Record<CsvField, string[]> = {
  symbol: ["symbol", "ticker", "isin", "cusip", "instrument", "security"],
  market: ["market", "exchange", "venue", "mic", "listing"],
  name: ["name", "description", "instrument name", "security name"],
  asset_type: ["asset type", "type", "category", "asset class"],
  quantity: ["quantity", "qty", "shares", "units"],
  price: ["price", "unit price", "last price"],
  market_value: ["market value", "value", "current value"],
  cost_basis: ["cost basis", "book value", "cost", "invested"],
  gross_amount: ["gross amount", "amount", "net amount", "total"],
  fees: ["fees", "commission", "commissions"],
  taxes: ["tax", "taxes", "withholding"],
  currency: ["currency", "ccy"],
  trade_date: ["date", "as of", "trade date", "settlement date"],
  action: ["action", "operation"],
  notes: ["notes", "note", "memo", "comment"],
};

export function PortfolioPage({ agents, onRefresh, onSelectAnalysis }: PortfolioPageProps) {
  const portfolios = useAppStore((state) => state.portfolios);
  const selectedPortfolioId = useAppStore((state) => state.selectedPortfolioId);
  const selectedPortfolio = useAppStore((state) => state.selectedPortfolio);
  const [loadingPortfolio, setLoadingPortfolio] = useState(false);
  const [creating, setCreating] = useState(false);
  const [createCurrency, setCreateCurrency] = useState<string>("USD");

  const selectPortfolio = async (portfolioId: string) => {
    setLoadingPortfolio(true);
    setState({ selectedPortfolioId: portfolioId, selectedPortfolio: null, view: "portfolio" });
    try {
      const detail = await getPortfolioDetail(portfolioId);
      setState({ selectedPortfolio: detail });
    } catch (err) {
      toast.error("Could not load portfolio", { description: String(err) });
    } finally {
      setLoadingPortfolio(false);
    }
  };

  useEffect(() => {
    // If an id is selected but the detail wasn't loaded yet, fetch it.
    // Don't auto-select "first portfolio" here — leaving both null is how the
    // "+ New portfolio" flow asks to show the EmptyCreate picker.
    if (selectedPortfolioId && !selectedPortfolio) {
      void selectPortfolio(selectedPortfolioId);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedPortfolioId, selectedPortfolio]);

  const handleCreate = async () => {
    setCreating(true);
    try {
      const portfolio = await createPortfolio("Portfolio", createCurrency);
      toast.success("Portfolio created", { description: `${portfolio.name} · ${portfolio.base_currency}` });
      await onRefresh();
      const detail = await getPortfolioDetail(portfolio.id);
      setState({
        selectedPortfolioId: portfolio.id,
        selectedPortfolio: detail,
        view: "portfolio",
      });
    } catch (err) {
      toast.error("Could not create portfolio", { description: String(err) });
    } finally {
      setCreating(false);
    }
  };

  return (
    <main className="flex h-full min-h-0 flex-col overflow-hidden">
      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="mx-auto w-full max-w-5xl px-5 pb-12 pt-12 md:px-8">
          {selectedPortfolio ? (
            <PortfolioView
              detail={selectedPortfolio}
              loading={loadingPortfolio}
              agents={agents}
              onRefresh={onRefresh}
              onSelectAnalysis={onSelectAnalysis}
            />
          ) : (
            <EmptyCreate
              disabled={creating}
              currency={createCurrency}
              onCurrencyChange={setCreateCurrency}
              onCreate={handleCreate}
            />
          )}
        </div>
      </div>
    </main>
  );
}

function EmptyCreate({
  disabled,
  currency,
  onCurrencyChange,
  onCreate,
}: {
  disabled: boolean;
  currency: string;
  onCurrencyChange: (value: string) => void;
  onCreate: () => void;
}) {
  return (
    <section className="space-y-5">
      <Eyebrow>Portfolio workspace</Eyebrow>
      <h1 className="max-w-[760px] text-[42px] font-semibold leading-[0.98] md:text-[64px]">
        Create a portfolio.
      </h1>
      <p className="max-w-[62ch] text-[15px] leading-[1.65] text-muted-foreground">
        Set up a portfolio, paste or upload the current holdings snapshot, and run portfolio-level
        research when you want it.
      </p>
      <div className="flex flex-wrap items-end gap-3 pt-1">
        <div className="space-y-2">
          <FieldLabel label="Base currency" />
          <Select value={currency} onValueChange={onCurrencyChange}>
            <SelectTrigger className="h-10 w-[140px] rounded-none font-mono uppercase shadow-none">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {CURRENCY_OPTIONS.map((code) => (
                <SelectItem key={code} value={code} className="font-mono">
                  {code}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <Button
          type="button"
          disabled={disabled}
          onClick={onCreate}
          className="h-10 rounded-none border border-foreground bg-foreground text-background shadow-none hover:bg-background hover:text-foreground"
        >
          {disabled && <SpinnerGap size={14} className="animate-spin" />}
          Create a portfolio
        </Button>
      </div>
    </section>
  );
}

function PortfolioView({
  detail,
  loading,
  agents,
  onRefresh,
  onSelectAnalysis,
}: {
  detail: PortfolioDetail;
  loading: boolean;
  agents: AgentCandidate[];
  onRefresh: () => Promise<void> | void;
  onSelectAnalysis: (analysisId: string) => void | Promise<void>;
}) {
  const [snapshotText, setSnapshotText] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [analysisStarting, setAnalysisStarting] = useState(false);
  const [editingName, setEditingName] = useState(false);
  const [draftName, setDraftName] = useState(detail.portfolio.name);
  const [renaming, setRenaming] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const lastImportAt = detail.import_batches[0]?.imported_at ?? null;
  const baseCurrency = detail.portfolio.base_currency || "USD";

  useEffect(() => {
    if (!editingName) setDraftName(detail.portfolio.name);
  }, [detail.portfolio.name, editingName]);

  const storeAgentId = useAppStore((state) => state.agentId);
  const hasAnyAvailableAgent = agents.some((agent) => agent.available);
  const availableAgents = agents.filter((agent) => agent.available);

  const { startWithAnalysisId } = useRunAnalysis({
    agentId: storeAgentId,
    agents,
    canRun: hasAnyAvailableAgent,
    onDone: async () => {
      await onRefresh();
    },
  });

  const totalsByCurrency = useMemo(() => {
    const totals = new Map<string, number>();
    for (const holding of detail.holdings) {
      if (holding.market_value === null) continue;
      const code = holding.currency || baseCurrency;
      totals.set(code, (totals.get(code) ?? 0) + holding.market_value);
    }
    return Array.from(totals.entries()).sort(([a], [b]) =>
      a === baseCurrency ? -1 : b === baseCurrency ? 1 : a.localeCompare(b),
    );
  }, [detail.holdings, baseCurrency]);

  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleFile = async (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) return;
    try {
      const text = await file.text();
      setSnapshotText(text);
    } catch (err) {
      toast.error("Could not read file", { description: String(err) });
    }
  };

  const handleUpdate = async () => {
    const trimmed = snapshotText.trim();
    if (!trimmed) {
      toast.error("Paste or upload a snapshot first");
      return;
    }
    const rows = parseSnapshotRows(trimmed);
    if (rows.length === 0) {
      toast.error("No importable rows detected");
      return;
    }
    setSubmitting(true);
    try {
      const input: PortfolioCsvImportInput = {
        portfolio_id: detail.portfolio.id,
        portfolio_name: detail.portfolio.name,
        account_id: null,
        account_name: "Current snapshot",
        institution: null,
        account_type: "snapshot",
        base_currency: baseCurrency,
        source_name: "snapshot update",
        import_kind: "positions",
        rows,
      };
      const result = await importPortfolioCsv(input);
      toast.success("Snapshot updated", {
        description: `${result.imported_count} rows imported, ${result.review_count} need review.`,
      });
      const fresh = await getPortfolioDetail(result.portfolio_id);
      setState({ selectedPortfolio: fresh });
      setSnapshotText("");
    } catch (err) {
      toast.error("Update failed", { description: String(err) });
    } finally {
      setSubmitting(false);
    }
  };

  const commitRename = async () => {
    const next = draftName.trim();
    setEditingName(false);
    if (!next || next === detail.portfolio.name) {
      setDraftName(detail.portfolio.name);
      return;
    }
    setRenaming(true);
    try {
      await renamePortfolio(detail.portfolio.id, next);
      // Swap in the new name immediately so the header doesn't flicker back.
      setState({
        selectedPortfolio: {
          ...detail,
          portfolio: {
            ...detail.portfolio,
            name: next,
            updated_at: new Date().toISOString(),
          },
        },
      });
      await onRefresh();
    } catch (err) {
      setDraftName(detail.portfolio.name);
      toast.error("Rename failed", { description: String(err) });
    } finally {
      setRenaming(false);
    }
  };

  const cancelRename = () => {
    setDraftName(detail.portfolio.name);
    setEditingName(false);
  };

  const handleNameKeyDown = (event: ReactKeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Enter") {
      event.preventDefault();
      void commitRename();
    } else if (event.key === "Escape") {
      event.preventDefault();
      cancelRename();
    }
  };

  const handleDelete = async () => {
    const confirmed = window.confirm(
      `Delete "${detail.portfolio.name}"? This removes its holdings and snapshot history. Analyses already created from it stay.`,
    );
    if (!confirmed) return;
    setDeleting(true);
    try {
      await deletePortfolio(detail.portfolio.id);
      toast.success("Portfolio deleted");
      // Clear selection; App.tsx refresh will pick a new default or show EmptyCreate.
      setState({
        selectedPortfolioId: null,
        selectedPortfolio: null,
        view: "portfolio",
      });
      await onRefresh();
    } catch (err) {
      toast.error("Delete failed", { description: String(err) });
    } finally {
      setDeleting(false);
    }
  };

  const startAnalysisWith = async (pickedAgentId: string, pickedModelId: string | null) => {
    const agent = agents.find((candidate) => candidate.id === pickedAgentId);
    if (!agent?.available) {
      toast.error("That agent isn't available. Configure it in Settings.");
      return;
    }

    // Persist the choice so future clicks remember it.
    const prevMap = getState().modelByAgent;
    const nextMap: Record<string, string | null> = { ...prevMap, [pickedAgentId]: pickedModelId };
    setState({ agentId: pickedAgentId, modelByAgent: nextMap });
    void persistModelByAgent(nextMap);

    setAnalysisStarting(true);
    try {
      const analysisId = await createPortfolioAnalysis(detail.portfolio.id, null);
      const defaultPrompt = `Review the current snapshot of portfolio "${detail.portfolio.name}" (${baseCurrency}): concentration, allocation, risk, and non-prescriptive rebalancing scenarios.`;
      startWithAnalysisId(analysisId, defaultPrompt, {
        agentId: pickedAgentId,
        modelId: pickedModelId,
      });
    } catch (err) {
      toast.error("Could not start analysis", { description: String(err) });
    } finally {
      setAnalysisStarting(false);
    }
  };

  const sortedHoldings = useMemo(
    () =>
      [...detail.holdings].sort((a, b) => {
        const aw = a.allocation_pct ?? 0;
        const bw = b.allocation_pct ?? 0;
        return bw - aw;
      }),
    [detail.holdings],
  );

  const placeholderCurrency = baseCurrency;
  const placeholder = `Paste CSV (market is optional — use it to pin a listing):\nSymbol, Market, Quantity, Price, Currency\nAAPL, NASDAQ, 10, 190, ${placeholderCurrency}`;

  return (
    <div className="space-y-10">
      <header className="flex flex-wrap items-start justify-between gap-6">
        <div className="min-w-0 space-y-3">
          <Eyebrow>Portfolio</Eyebrow>
          {editingName ? (
            <div className="space-y-1">
              <Input
                autoFocus
                value={draftName}
                onChange={(event) => setDraftName(event.target.value)}
                onBlur={() => void commitRename()}
                onKeyDown={handleNameKeyDown}
                disabled={renaming}
                className="h-auto max-w-[640px] rounded-none border-0 border-b border-border bg-transparent p-0 text-[34px] font-semibold leading-[1.02] tracking-[-0.02em] shadow-none focus-visible:border-foreground focus-visible:ring-0 md:text-[48px]"
                aria-label="Portfolio name"
              />
              <div className="font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground">
                ↵ save · esc cancel
              </div>
            </div>
          ) : (
            <button
              type="button"
              onClick={() => setEditingName(true)}
              className="group block max-w-[640px] truncate text-left text-[34px] font-semibold leading-[1.02] tracking-[-0.02em] text-foreground hover:text-foreground md:text-[48px]"
              title="Click to rename"
            >
              <span className="border-b border-transparent group-hover:border-border">
                {detail.portfolio.name}
              </span>
            </button>
          )}
          <PortfolioMeta
            baseCurrency={baseCurrency}
            holdingCount={detail.holdings.length}
            totals={totalsByCurrency}
            lastImportAt={lastImportAt}
          />
        </div>
        <div className="flex items-center gap-1">
          <RunAnalysisMenu
            agents={agents}
            availableAgents={availableAgents}
            disabled={
              analysisStarting || detail.holdings.length === 0 || !hasAnyAvailableAgent
            }
            running={analysisStarting}
            onPick={(pickedAgentId, pickedModelId) => {
              void startAnalysisWith(pickedAgentId, pickedModelId);
            }}
          />
          <PortfolioOverflowMenu
            onRename={() => setEditingName(true)}
            onDelete={() => void handleDelete()}
            deleting={deleting}
          />
        </div>
      </header>

      {!hasAnyAvailableAgent && (
        <div className="flex items-center gap-2 text-xs text-destructive">
          <WarningCircle size={14} />
          <span>Configure an ACP agent binary in Settings before running analysis.</span>
        </div>
      )}

      <section className="space-y-5">
        <SectionHeader number="01" label="Holdings" title="Current allocation" />
        {loading ? (
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <SpinnerGap size={14} className="animate-spin" />
            Loading
          </div>
        ) : sortedHoldings.length === 0 ? (
          <div className="border-t border-border py-6 text-sm leading-[1.6] text-muted-foreground">
            Update the snapshot to build the holdings view.
          </div>
        ) : (
          <div className="overflow-x-auto">
            <div className="min-w-[700px] divide-y divide-border border-t border-border">
              <div className="grid grid-cols-[minmax(150px,1.2fr)_80px_100px_100px_120px_90px] gap-3 py-2 font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground">
                <span>Instrument</span>
                <span className="text-right">30d</span>
                <span className="text-right">Qty</span>
                <span className="text-right">Price</span>
                <span className="text-right">Value</span>
                <span className="text-right">Weight</span>
              </div>
              {sortedHoldings.map((holding) => (
                <HoldingRow
                  key={`${holding.symbol}-${holding.market ?? ""}-${holding.currency}`}
                  holding={holding}
                  baseCurrency={baseCurrency}
                />
              ))}
            </div>
          </div>
        )}
      </section>

      <PortfolioAnalysesSection
        portfolioId={detail.portfolio.id}
        onSelectAnalysis={onSelectAnalysis}
      />

      <section className="space-y-5">
        <SectionHeader number="03" label="Snapshot" title="Update current holdings" />
        <div className="space-y-3 border-t border-border pt-5">
          <textarea
            value={snapshotText}
            onChange={(event) => setSnapshotText(event.target.value)}
            placeholder={placeholder}
            className="min-h-[180px] w-full border border-border bg-transparent p-3 font-mono text-[12.5px] leading-[1.5] shadow-none outline-none focus:border-foreground"
          />
          <div className="flex flex-wrap items-center gap-3">
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={() => fileInputRef.current?.click()}
              className="h-9 rounded-none border border-border shadow-none"
            >
              <FileArrowUp size={14} />
              Upload CSV
            </Button>
            <input
              ref={fileInputRef}
              type="file"
              accept=".csv,text/csv,text/plain"
              className="hidden"
              onChange={handleFile}
            />
            <Button
              type="button"
              disabled={submitting || snapshotText.trim().length === 0}
              onClick={handleUpdate}
              className="h-9 rounded-none border border-foreground bg-foreground text-background shadow-none hover:bg-background hover:text-foreground"
            >
              {submitting && <SpinnerGap size={14} className="animate-spin" />}
              Update snapshot
            </Button>
          </div>
        </div>
      </section>
    </div>
  );
}

function PortfolioMeta({
  baseCurrency,
  holdingCount,
  totals,
  lastImportAt,
}: {
  baseCurrency: string;
  holdingCount: number;
  totals: [string, number][];
  lastImportAt: string | null;
}) {
  // Empty state: don't fake "00 holdings" or an em-dash. The important fact is
  // "no snapshot yet" — everything else is noise.
  if (holdingCount === 0) {
    return (
      <div className="flex flex-wrap items-center gap-3 font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground">
        <span>{baseCurrency}</span>
        <span aria-hidden className="text-muted-foreground/50">
          ·
        </span>
        <span>No snapshot yet</span>
      </div>
    );
  }

  const segments: string[] = [
    baseCurrency,
    `${String(holdingCount).padStart(2, "0")} holdings`,
  ];
  if (totals.length > 0) {
    segments.push(totals.map(([code, sum]) => formatMoney(sum, code)).join(" · "));
  }
  if (lastImportAt) {
    segments.push(`Updated ${formatDate(lastImportAt)}`);
  }

  return (
    <div className="flex flex-wrap items-center gap-3 font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground">
      {segments.map((segment, index) => (
        <span key={segment} className="flex items-center gap-3">
          {index > 0 && (
            <span aria-hidden className="text-muted-foreground/50">
              ·
            </span>
          )}
          <span className="tabular-nums">{segment}</span>
        </span>
      ))}
    </div>
  );
}

function PortfolioOverflowMenu({
  onRename,
  onDelete,
  deleting,
}: {
  onRename: () => void;
  onDelete: () => void;
  deleting: boolean;
}) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          aria-label="Portfolio actions"
          className="h-10 w-10 rounded-none border border-transparent p-0 shadow-none hover:border-border hover:bg-transparent data-[state=open]:border-border"
        >
          <DotsThree size={18} weight="bold" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align="end"
        className="w-[180px] rounded-md border-border bg-popover/95 p-1 shadow-md backdrop-blur-xl"
      >
        <DropdownMenuItem
          onSelect={() => {
            // Defer so the menu animates closed before the input focuses.
            setTimeout(onRename, 0);
          }}
          className="gap-2 text-xs"
        >
          Rename
        </DropdownMenuItem>
        <DropdownMenuItem
          disabled={deleting}
          onSelect={() => {
            setTimeout(onDelete, 0);
          }}
          className="gap-2 text-xs text-destructive focus:text-destructive"
        >
          {deleting ? "Deleting…" : "Delete"}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function RunAnalysisMenu({
  agents,
  availableAgents,
  disabled,
  running,
  onPick,
}: {
  agents: AgentCandidate[];
  availableAgents: AgentCandidate[];
  disabled: boolean;
  running: boolean;
  onPick: (agentId: string, modelId: string | null) => void;
}) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild disabled={disabled}>
        <Button
          type="button"
          disabled={disabled}
          className="h-10 rounded-none border border-foreground bg-foreground text-background shadow-none hover:bg-background hover:text-foreground disabled:border-border disabled:bg-transparent disabled:text-muted-foreground/60"
        >
          {running && <SpinnerGap size={14} className="animate-spin" />}
          <span>Run analysis</span>
          <CaretDown size={12} weight="bold" className="ml-1" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align="end"
        className="w-[240px] rounded-md border-border bg-popover/95 p-1 shadow-md backdrop-blur-xl"
      >
        {availableAgents.length === 0 && (
          <DropdownMenuItem disabled className="text-xs text-muted-foreground">
            No available agents — open Settings
          </DropdownMenuItem>
        )}
        {agents.map((agent) => {
          const isUnavailable = !agent.available;
          const hasModels = agent.models.length > 0;

          if (hasModels && !isUnavailable) {
            return (
              <DropdownMenuSub key={agent.id}>
                <DropdownMenuSubTrigger className="gap-2 text-xs">
                  <img
                    src={getLogoPath(agent.label)}
                    alt={agent.label}
                    className="h-3.5 w-3.5 object-contain opacity-80"
                  />
                  <span className="flex-1 truncate">{agent.label}</span>
                </DropdownMenuSubTrigger>
                <DropdownMenuSubContent className="w-[220px] p-1">
                  <DropdownMenuItem
                    className="gap-2 text-xs"
                    onSelect={() => onPick(agent.id, null)}
                  >
                    <span className="flex-1 truncate">Default</span>
                  </DropdownMenuItem>
                  {agent.models.map((model) => (
                    <DropdownMenuItem
                      key={model.id}
                      className="gap-2 text-xs"
                      onSelect={() => onPick(agent.id, model.id)}
                    >
                      <span className="flex-1 truncate">{model.name}</span>
                    </DropdownMenuItem>
                  ))}
                </DropdownMenuSubContent>
              </DropdownMenuSub>
            );
          }

          return (
            <DropdownMenuItem
              key={agent.id}
              disabled={isUnavailable}
              onSelect={() => onPick(agent.id, null)}
              className="gap-2 text-xs"
            >
              <img
                src={getLogoPath(agent.label)}
                alt={agent.label}
                className="h-3.5 w-3.5 object-contain opacity-80"
              />
              <span className="flex-1 truncate">{agent.label}</span>
              {isUnavailable && (
                <span className="text-[10px] text-muted-foreground">offline</span>
              )}
            </DropdownMenuItem>
          );
        })}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function HoldingRow({
  holding,
  baseCurrency,
}: {
  holding: PortfolioHolding;
  baseCurrency: string;
}) {
  const price =
    holding.market_value !== null && holding.quantity !== 0
      ? holding.market_value / holding.quantity
      : null;
  const currency = holding.currency || baseCurrency;
  return (
    <div className="grid grid-cols-[minmax(150px,1.2fr)_80px_100px_100px_120px_90px] items-center gap-3 py-3 text-[13px]">
      <div className="min-w-0">
        <div className="flex items-baseline gap-2">
          <span className="truncate font-medium">{holding.symbol}</span>
          {holding.market && (
            <span className="font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground">
              {holding.market}
            </span>
          )}
        </div>
        <div className="truncate text-xs text-muted-foreground">
          {holding.name ?? holding.asset_type}
        </div>
      </div>
      <HoldingSparkline symbol={holding.symbol} market={holding.market} />
      <span className="text-right font-mono tabular-nums">{formatNumber(holding.quantity)}</span>
      <span className="text-right font-mono tabular-nums">
        {price !== null ? formatMoney(price, currency) : "—"}
      </span>
      <span className="text-right font-mono tabular-nums">
        {holding.market_value !== null ? formatMoney(holding.market_value, currency) : "—"}
      </span>
      <span className="text-right font-mono tabular-nums">
        {holding.allocation_pct !== null ? formatPercent(holding.allocation_pct) : "—"}
      </span>
    </div>
  );
}

function PortfolioAnalysesSection({
  portfolioId,
  onSelectAnalysis,
}: {
  portfolioId: string;
  onSelectAnalysis: (analysisId: string) => void | Promise<void>;
}) {
  const analyses = useAppStore((state) => state.analyses);
  const linked = useMemo(
    () => analyses.filter((analysis) => analysis.portfolio_id === portfolioId),
    [analyses, portfolioId],
  );

  return (
    <section className="space-y-5">
      <SectionHeader number="02" label="Analyses" title="Linked research" />
      {linked.length === 0 ? (
        <div className="border-t border-border py-6 text-sm leading-[1.6] text-muted-foreground">
          No analyses yet. Run one with the "Run analysis" action above.
        </div>
      ) : (
        <div className="divide-y divide-border border-t border-border">
          {linked.map((analysis) => (
            <AnalysisRow
              key={analysis.id}
              analysis={analysis}
              onSelect={() => void onSelectAnalysis(analysis.id)}
            />
          ))}
        </div>
      )}
    </section>
  );
}

function AnalysisRow({
  analysis,
  onSelect,
}: {
  analysis: AnalysisSummary;
  onSelect: () => void;
}) {
  const running =
    analysis.active_run_status === "running" || analysis.active_run_status === "queued";
  const statusText = running ? "RUNNING" : analysisStatusLabel(analysis);

  return (
    <button
      type="button"
      onClick={onSelect}
      className="grid w-full grid-cols-[minmax(0,1fr)_auto] items-baseline gap-4 py-3 text-left transition-colors hover:bg-muted/30"
    >
      <div className="min-w-0">
        <div className="truncate text-[14px] font-medium">{analysis.title}</div>
        <div className="mt-1 font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground">
          {formatDate(analysis.updated_at)}
          <span aria-hidden className="mx-2 text-muted-foreground/50">
            ·
          </span>
          <span className="tabular-nums">
            {String(analysis.block_count).padStart(2, "0")}b
          </span>
          <span aria-hidden className="mx-2 text-muted-foreground/50">
            ·
          </span>
          <span className="tabular-nums">
            {String(analysis.source_count).padStart(2, "0")}s
          </span>
        </div>
      </div>
      <span
        className={`flex items-center gap-1.5 font-mono text-[10.5px] uppercase tracking-[0.14em] ${running ? "text-primary" : "text-muted-foreground"}`}
      >
        {running && <CircleNotch size={10} className="animate-spin" />}
        {statusText}
      </span>
    </button>
  );
}

function analysisStatusLabel(analysis: AnalysisSummary): string {
  switch (analysis.status) {
    case "completed":
      return "DONE";
    case "failed":
      return "FAILED";
    case "cancelled":
      return "STOPPED";
    case "running":
      return "RUNNING";
    case "queued":
      return "QUEUED";
    default:
      return String(analysis.status).toUpperCase();
  }
}

const priceHistoryCache = new Map<string, Promise<number[]>>();

function fetchPriceHistoryCached(symbol: string, market: string | null): Promise<number[]> {
  const key = `${symbol}|${market ?? ""}`;
  const hit = priceHistoryCache.get(key);
  if (hit) return hit;
  const pending = getPriceHistory(symbol, market).catch(() => [] as number[]);
  priceHistoryCache.set(key, pending);
  return pending;
}

function HoldingSparkline({ symbol, market }: { symbol: string; market: string | null }) {
  const [series, setSeries] = useState<number[] | null>(null);

  useEffect(() => {
    let cancelled = false;
    fetchPriceHistoryCached(symbol, market).then((values) => {
      if (!cancelled) setSeries(values);
    });
    return () => {
      cancelled = true;
    };
  }, [symbol, market]);

  if (!series || series.length < 2) {
    return <span aria-hidden className="block text-right text-muted-foreground/40">—</span>;
  }

  const min = Math.min(...series);
  const max = Math.max(...series);
  const span = max - min || 1;
  const width = 72;
  const height = 22;
  const step = width / (series.length - 1);
  const points = series
    .map((value, index) => {
      const x = index * step;
      const y = height - ((value - min) / span) * height;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
  const up = series[series.length - 1] >= series[0];

  return (
    <svg
      viewBox={`0 0 ${width} ${height}`}
      className={`ml-auto block h-[22px] w-[72px] ${up ? "text-foreground" : "text-muted-foreground"}`}
      aria-hidden
    >
      <polyline
        points={points}
        fill="none"
        stroke="currentColor"
        strokeWidth={1}
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

function FieldLabel({ label }: { label: string }) {
  return (
    <span className="font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground">
      {label}
    </span>
  );
}

function parseSnapshotRows(text: string): PortfolioCsvRow[] {
  const records = parseCsvRecords(text);
  if (records.length === 0) return [];
  const headers = records[0].map((header, index) => header.trim() || `col_${index + 1}`);
  const looksLikeHeader = headers.some((header) => inferField(header) !== null);
  const data = looksLikeHeader ? records.slice(1) : records;
  const headerRow = looksLikeHeader ? headers : defaultHeaders(records[0].length);
  const mapping = buildHeaderMapping(headerRow);
  const rows: PortfolioCsvRow[] = [];
  data.forEach((record, index) => {
    if (record.every((cell) => !cell.trim())) return;
    const raw: Record<string, string> = {};
    headerRow.forEach((header, columnIndex) => {
      raw[header] = record[columnIndex]?.trim() ?? "";
    });
    rows.push({
      row_index: index + 1,
      raw,
      symbol: pickString(raw, mapping.symbol),
      market: pickString(raw, mapping.market),
      name: pickString(raw, mapping.name),
      asset_type: pickString(raw, mapping.asset_type),
      quantity: pickNumber(raw, mapping.quantity),
      price: pickNumber(raw, mapping.price),
      market_value: pickNumber(raw, mapping.market_value),
      cost_basis: pickNumber(raw, mapping.cost_basis),
      gross_amount: pickNumber(raw, mapping.gross_amount),
      fees: pickNumber(raw, mapping.fees),
      taxes: pickNumber(raw, mapping.taxes),
      currency: pickString(raw, mapping.currency),
      trade_date: pickString(raw, mapping.trade_date),
      action: pickString(raw, mapping.action),
      notes: pickString(raw, mapping.notes),
    });
  });
  return rows;
}

function defaultHeaders(count: number): string[] {
  const names = ["symbol", "quantity", "price", "currency"];
  return Array.from({ length: count }, (_, index) => names[index] ?? `col_${index + 1}`);
}

function buildHeaderMapping(headers: string[]): Record<CsvField, string | null> {
  const mapping = (Object.keys(FIELD_ALIASES) as CsvField[]).reduce(
    (acc, field) => {
      acc[field] = null;
      return acc;
    },
    {} as Record<CsvField, string | null>,
  );
  headers.forEach((header) => {
    const field = inferField(header);
    if (field && !mapping[field]) mapping[field] = header;
  });
  return mapping;
}

function inferField(header: string): CsvField | null {
  const normalized = header
    .toLowerCase()
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[^a-z0-9]+/g, " ")
    .trim();
  if (!normalized) return null;
  const entries = Object.entries(FIELD_ALIASES) as [CsvField, string[]][];
  for (const [field, aliases] of entries) {
    if (aliases.some((alias) => normalized === alias)) return field;
  }
  for (const [field, aliases] of entries) {
    if (aliases.some((alias) => normalized.includes(alias))) return field;
  }
  return null;
}

function pickString(raw: Record<string, string>, header: string | null): string | null {
  if (!header) return null;
  const value = raw[header]?.trim();
  return value ? value : null;
}

function pickNumber(raw: Record<string, string>, header: string | null): number | null {
  const value = pickString(raw, header);
  if (!value) return null;
  const stripped = value.replace(/[^0-9,.\-]/g, "");
  const lastComma = stripped.lastIndexOf(",");
  const lastDot = stripped.lastIndexOf(".");
  let normalized = stripped;
  if (lastComma > -1 && lastDot > -1) {
    const decimal = lastComma > lastDot ? "," : ".";
    const thousands = decimal === "," ? "." : ",";
    normalized = stripped.replaceAll(thousands, "").replace(decimal, ".");
  } else if (lastComma > -1) {
    normalized = stripped.replace(",", ".");
  }
  const parsed = Number.parseFloat(normalized);
  return Number.isFinite(parsed) ? parsed : null;
}

function parseCsvRecords(text: string): string[][] {
  const rows: string[][] = [];
  let row: string[] = [];
  let cell = "";
  let inQuotes = false;

  for (let index = 0; index < text.length; index++) {
    const char = text[index];
    const next = text[index + 1];
    if (char === '"') {
      if (inQuotes && next === '"') {
        cell += '"';
        index += 1;
      } else {
        inQuotes = !inQuotes;
      }
    } else if ((char === "," || char === "\t") && !inQuotes) {
      row.push(cell);
      cell = "";
    } else if ((char === "\n" || char === "\r") && !inQuotes) {
      if (char === "\r" && next === "\n") index += 1;
      row.push(cell);
      rows.push(row);
      row = [];
      cell = "";
    } else {
      cell += char;
    }
  }

  if (cell || row.length > 0) {
    row.push(cell);
    rows.push(row);
  }

  return rows;
}

function formatMoney(value: number, currency: string): string {
  try {
    return new Intl.NumberFormat(undefined, {
      style: "currency",
      currency: currency || "USD",
      maximumFractionDigits: Math.abs(value) >= 1000 ? 0 : 2,
    }).format(value);
  } catch {
    // Fallback if the currency code isn't recognized by Intl.
    return `${value.toFixed(Math.abs(value) >= 1000 ? 0 : 2)} ${currency}`;
  }
}

function formatNumber(value: number): string {
  return new Intl.NumberFormat(undefined, { maximumFractionDigits: 4 }).format(value);
}

function formatPercent(value: number): string {
  return new Intl.NumberFormat(undefined, {
    style: "percent",
    maximumFractionDigits: 1,
  }).format(value);
}

function formatDate(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(date);
}
