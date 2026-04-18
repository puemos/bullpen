import { CircleNotch, MagnifyingGlass, Plus, X } from "@phosphor-icons/react";
import {
  type CSSProperties,
  type KeyboardEvent,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import type { AppView } from "@/app/navigation";
import { Button } from "@/components/ui/button";
import { Eyebrow } from "@/components/ui/editorial";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";
import { cn } from "@/lib/utils";
import type { AnalysisSummary, PortfolioSummary } from "@/types";

interface AppSidebarProps {
  analyses: AnalysisSummary[];
  portfolios: PortfolioSummary[];
  currentView: AppView;
  selectedAnalysisId: string | null;
  selectedPortfolioId: string | null;
  onViewChange: (view: AppView) => void;
  onSelectAnalysis: (analysisId: string) => void | Promise<void>;
  onSelectPortfolio: (portfolioId: string) => void | Promise<void>;
  onNewPortfolio: () => void | Promise<void>;
  currentVersion: string | null;
  updateAvailable: boolean;
  onUpdateClick: () => void;
}

const SECTION_CAP = 5;
const SEARCH_MIN = 15;

export function AppSidebar({
  analyses,
  portfolios,
  currentView,
  selectedAnalysisId,
  selectedPortfolioId,
  onViewChange,
  onSelectAnalysis,
  onSelectPortfolio,
  onNewPortfolio,
  currentVersion,
  updateAvailable,
  onUpdateClick,
}: AppSidebarProps) {
  const [analysesExpanded, setAnalysesExpanded] = useState(false);
  const [portfoliosExpanded, setPortfoliosExpanded] = useState(false);
  const [searchActive, setSearchActive] = useState(false);
  const [search, setSearch] = useState("");

  const filteredAnalyses = useMemo(() => {
    const query = search.trim().toLowerCase();
    if (!query) return analyses;
    return analyses.filter((analysis) => analysis.title.toLowerCase().includes(query));
  }, [analyses, search]);

  const visibleAnalyses = analysesExpanded
    ? filteredAnalyses
    : filteredAnalyses.slice(0, SECTION_CAP);
  const hiddenAnalysesCount = Math.max(0, filteredAnalyses.length - SECTION_CAP);

  const visiblePortfolios = portfoliosExpanded
    ? portfolios
    : portfolios.slice(0, SECTION_CAP);
  const hiddenPortfoliosCount = Math.max(0, portfolios.length - SECTION_CAP);

  return (
    <Sidebar className="border-r border-sidebar-border bg-sidebar" variant="sidebar">
      <div data-tauri-drag-region className="h-10 shrink-0" />

      <SidebarContent className="overflow-y-auto">
        <SidebarGroup className="px-4 py-0">
          <SidebarGroupContent className="flex flex-col gap-6">
            {/* ── Analyses ── */}
            <section>
              {searchActive ? (
                <SearchHeader
                  value={search}
                  onChange={setSearch}
                  onClose={() => {
                    setSearch("");
                    setSearchActive(false);
                  }}
                />
              ) : (
                <SectionHeaderRow
                  label="Analyses"
                  count={analyses.length}
                  showSearch={analyses.length >= SEARCH_MIN}
                  onSearchClick={() => setSearchActive(true)}
                  onAddClick={() => onViewChange("new-analysis")}
                />
              )}

              {analyses.length === 0 ? (
                <EmptyCta
                  label="New analysis"
                  onClick={() => onViewChange("new-analysis")}
                  isActive={currentView === "new-analysis"}
                />
              ) : filteredAnalyses.length === 0 ? (
                <p className="mt-2 px-2 text-[12px] leading-[1.55] text-sidebar-foreground/55">
                  No match for "{search.trim()}".
                </p>
              ) : (
                <>
                  <SidebarMenu className="mt-2 gap-1.5">
                    {visibleAnalyses.map((analysis) => (
                      <SidebarMenuItem key={analysis.id} className="sidebar-report-row">
                        <SidebarMenuButton
                          asChild
                          isActive={
                            currentView === "analysis" && selectedAnalysisId === analysis.id
                          }
                          className="h-auto items-start px-2 py-2 text-[13px] font-normal data-[active=true]:font-normal data-[active=true]:bg-sidebar-accent/70"
                        >
                          <Button
                            type="button"
                            variant="ghost"
                            size="xs"
                            className="h-auto min-w-0 flex-col items-stretch justify-start gap-1 px-2 py-0 text-[13px]"
                            onClick={() => {
                              void onSelectAnalysis(analysis.id);
                            }}
                          >
                            <MarqueeTitle title={analysis.title} />
                            <AnalysisMeta analysis={analysis} />
                          </Button>
                        </SidebarMenuButton>
                      </SidebarMenuItem>
                    ))}
                  </SidebarMenu>

                  <Expander
                    hiddenCount={hiddenAnalysesCount}
                    expanded={analysesExpanded}
                    onToggle={() => setAnalysesExpanded((prev) => !prev)}
                  />
                </>
              )}
            </section>

            {/* ── Portfolios ── */}
            <section>
              <SectionHeaderRow
                label="Portfolios"
                count={portfolios.length}
                showSearch={false}
                onSearchClick={() => {
                  /* no-op */
                }}
                onAddClick={() => {
                  void onNewPortfolio();
                }}
              />

              {portfolios.length === 0 ? (
                <EmptyCta
                  label="New portfolio"
                  onClick={() => {
                    void onNewPortfolio();
                  }}
                  isActive={false}
                />
              ) : (
                <>
                  <SidebarMenu className="mt-2 gap-0.5">
                    {visiblePortfolios.map((portfolio) => (
                      <SidebarMenuItem key={portfolio.id}>
                        <SidebarMenuButton
                          asChild
                          isActive={
                            currentView === "portfolio" && selectedPortfolioId === portfolio.id
                          }
                          className="h-7 items-center px-2 text-[13px] font-normal data-[active=true]:bg-sidebar-accent/70"
                        >
                          <Button
                            type="button"
                            variant="ghost"
                            size="xs"
                            className="h-7 min-w-0 items-center justify-between gap-2 px-2 text-[13px]"
                            onClick={() => {
                              void onSelectPortfolio(portfolio.id);
                            }}
                          >
                            <MarqueeTitle title={portfolio.name} />
                            <span className="shrink-0 font-mono text-[10px] uppercase tracking-[0.14em] text-sidebar-foreground/50">
                              {portfolio.base_currency}
                            </span>
                          </Button>
                        </SidebarMenuButton>
                      </SidebarMenuItem>
                    ))}
                  </SidebarMenu>

                  <Expander
                    hiddenCount={hiddenPortfoliosCount}
                    expanded={portfoliosExpanded}
                    onToggle={() => setPortfoliosExpanded((prev) => !prev)}
                  />
                </>
              )}
            </section>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter className="gap-2 border-t border-sidebar-border px-4 pb-4 pt-2">
        <div className="flex items-center justify-between gap-2">
          <button
            type="button"
            onClick={() => onViewChange("settings")}
            className={cn(
              "text-[12.5px] transition-colors",
              currentView === "settings"
                ? "text-sidebar-foreground"
                : "text-sidebar-foreground/50 hover:text-sidebar-foreground",
            )}
          >
            Settings
          </button>
          {updateAvailable && (
            <button
              type="button"
              onClick={onUpdateClick}
              className="font-mono text-[10.5px] uppercase tracking-[0.14em] text-sidebar-foreground/50 transition-colors hover:text-sidebar-foreground"
            >
              Update ↑
            </button>
          )}
        </div>
        {currentVersion && (
          <span className="font-mono text-[10.5px] tabular-nums text-sidebar-foreground/35">
            v{currentVersion}
          </span>
        )}
      </SidebarFooter>
    </Sidebar>
  );
}

function SectionHeaderRow({
  label,
  count,
  showSearch,
  onSearchClick,
  onAddClick,
}: {
  label: string;
  count: number;
  showSearch: boolean;
  onSearchClick: () => void;
  onAddClick: () => void;
}) {
  return (
    <div className="flex h-7 items-center justify-between">
      <Eyebrow>{label}</Eyebrow>
      <div className="flex items-center gap-2">
        <span className="font-mono text-[10.5px] tabular-nums text-sidebar-foreground/40">
          {String(count).padStart(2, "0")}
        </span>
        {showSearch && (
          <IconButton ariaLabel={`Search ${label.toLowerCase()}`} onClick={onSearchClick}>
            <MagnifyingGlass size={13} weight="bold" />
          </IconButton>
        )}
        <IconButton ariaLabel={`New ${label.slice(0, -1).toLowerCase()}`} onClick={onAddClick}>
          <Plus size={13} weight="bold" />
        </IconButton>
      </div>
    </div>
  );
}

function IconButton({
  children,
  ariaLabel,
  onClick,
}: {
  children: React.ReactNode;
  ariaLabel: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      aria-label={ariaLabel}
      onClick={onClick}
      className="flex h-6 w-6 items-center justify-center text-sidebar-foreground/50 transition-colors hover:text-sidebar-foreground"
    >
      {children}
    </button>
  );
}

function SearchHeader({
  value,
  onChange,
  onClose,
}: {
  value: string;
  onChange: (next: string) => void;
  onClose: () => void;
}) {
  const inputRef = useRef<HTMLInputElement>(null);
  useLayoutEffect(() => {
    inputRef.current?.focus();
  }, []);
  const handleKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
    }
  };
  return (
    <div className="flex h-7 items-center gap-2">
      <MagnifyingGlass size={13} weight="bold" className="text-sidebar-foreground/45" />
      <input
        ref={inputRef}
        type="text"
        value={value}
        onChange={(event) => onChange(event.target.value)}
        onKeyDown={handleKeyDown}
        onBlur={() => {
          if (!value) onClose();
        }}
        placeholder="Search analyses"
        className="flex-1 bg-transparent text-[12.5px] text-sidebar-foreground outline-none placeholder:text-sidebar-foreground/40"
      />
      <IconButton ariaLabel="Close search" onClick={onClose}>
        <X size={12} weight="bold" />
      </IconButton>
    </div>
  );
}

function EmptyCta({
  label,
  onClick,
  isActive,
}: {
  label: string;
  onClick: () => void;
  isActive: boolean;
}) {
  return (
    <SidebarMenu className="mt-2">
      <SidebarMenuItem>
        <SidebarMenuButton
          isActive={isActive}
          onClick={onClick}
          className="h-8 text-[13px] font-normal"
        >
          <span className="flex items-center gap-2">
            <span aria-hidden className="font-mono text-muted-foreground">
              +
            </span>
            <span>{label}</span>
          </span>
        </SidebarMenuButton>
      </SidebarMenuItem>
    </SidebarMenu>
  );
}

function Expander({
  hiddenCount,
  expanded,
  onToggle,
}: {
  hiddenCount: number;
  expanded: boolean;
  onToggle: () => void;
}) {
  if (hiddenCount === 0 && !expanded) return null;
  const label = expanded ? "Show less ↑" : `${hiddenCount} more ↓`;
  return (
    <button
      type="button"
      onClick={onToggle}
      className="mt-2 flex h-6 w-full items-center px-2 font-mono text-[10.5px] uppercase tracking-[0.14em] text-sidebar-foreground/45 transition-colors hover:text-sidebar-foreground"
    >
      {label}
    </button>
  );
}

function AnalysisMeta({ analysis }: { analysis: AnalysisSummary }) {
  const running =
    analysis.active_run_status === "running" || analysis.active_run_status === "queued";

  return (
    <span className="flex items-center gap-1.5 pl-0 font-mono text-[10px] uppercase tracking-[0.14em] text-sidebar-foreground/45">
      {running ? (
        <>
          <CircleNotch size={10} className="animate-spin text-primary" />
          <span className="text-primary">Running</span>
        </>
      ) : (
        <>
          <span>{statusLabel(analysis)}</span>
          <span aria-hidden className="text-sidebar-foreground/25">
            ·
          </span>
          <span className="tabular-nums">{String(analysis.block_count).padStart(2, "0")}b</span>
          <span aria-hidden className="text-sidebar-foreground/25">
            ·
          </span>
          <span className="tabular-nums">{String(analysis.source_count).padStart(2, "0")}s</span>
        </>
      )}
    </span>
  );
}

function statusLabel(analysis: AnalysisSummary): string {
  switch (analysis.status) {
    case "completed":
      return "Done";
    case "failed":
      return "Failed";
    case "cancelled":
      return "Stopped";
    case "queued":
      return "Queued";
    case "running":
      return "Running";
    default:
      return analysis.status;
  }
}

function MarqueeTitle({ title }: { title: string }) {
  const containerRef = useRef<HTMLSpanElement>(null);
  const textRef = useRef<HTMLSpanElement>(null);
  const [metrics, setMetrics] = useState({ scrollable: false, distance: 0 });

  useLayoutEffect(() => {
    const measure = () => {
      const container = containerRef.current;
      const text = textRef.current;
      if (!container || !text) return;

      const measuredDistance = Math.max(0, text.scrollWidth - container.clientWidth);
      const titleDistance = title.length > 24 ? Math.round(title.length * 6.5) : 0;
      const distance = Math.max(measuredDistance, titleDistance);
      setMetrics({ scrollable: distance > 2, distance });
    };

    measure();

    const observer = new ResizeObserver(measure);
    if (containerRef.current) observer.observe(containerRef.current);
    if (textRef.current) observer.observe(textRef.current);

    return () => observer.disconnect();
  }, [title]);

  const style = {
    "--marquee-offset": `-${metrics.distance}px`,
    "--marquee-duration": `${Math.min(18, Math.max(8, metrics.distance / 14 + 7))}s`,
  } as CSSProperties;

  return (
    <span
      ref={containerRef}
      className="sidebar-report-title text-sidebar-foreground"
      data-scrollable={metrics.scrollable ? "true" : undefined}
      style={style}
    >
      <span ref={textRef} className="sidebar-report-title-inner">
        {title}
      </span>
    </span>
  );
}
