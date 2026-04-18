import { CircleNotch } from "@phosphor-icons/react";
import { type CSSProperties, useLayoutEffect, useRef, useState } from "react";
import type { AppView } from "@/app/navigation";
import { Button } from "@/components/ui/button";
import { Eyebrow } from "@/components/ui/editorial";
import { ScrollArea } from "@/components/ui/scroll-area";
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
import type { AnalysisSummary } from "@/types";

interface AppSidebarProps {
  analyses: AnalysisSummary[];
  currentView: AppView;
  selectedAnalysisId: string | null;
  onViewChange: (view: AppView) => void;
  onSelectAnalysis: (analysisId: string) => void | Promise<void>;
  currentVersion: string | null;
  updateAvailable: boolean;
  onUpdateClick: () => void;
}

export function AppSidebar({
  analyses,
  currentView,
  selectedAnalysisId,
  onViewChange,
  onSelectAnalysis,
  currentVersion,
  updateAvailable,
  onUpdateClick,
}: AppSidebarProps) {
  return (
    <Sidebar className="border-r border-sidebar-border bg-sidebar" variant="sidebar">
      <div data-tauri-drag-region className="h-10 shrink-0" />

      <SidebarContent className="gap-0 overflow-hidden">
        <SidebarGroup className="min-h-0 flex-1 px-4 py-0">
          <SidebarGroupContent className="flex min-h-0 flex-1 flex-col">
            <div className="mb-4 flex items-baseline justify-between">
              <Eyebrow>Analyses</Eyebrow>
              {analyses.length > 0 && (
                <span className="font-mono text-[10.5px] tabular-nums text-sidebar-foreground/40">
                  {String(analyses.length).padStart(2, "0")}
                </span>
              )}
            </div>

            <SidebarMenu className="mb-5">
              <SidebarMenuItem>
                <SidebarMenuButton
                  isActive={currentView === "new-analysis"}
                  onClick={() => onViewChange("new-analysis")}
                  className="h-8 text-[13px] font-normal"
                >
                  <span className="flex items-center gap-2">
                    <span aria-hidden className="font-mono text-muted-foreground">
                      +
                    </span>
                    <span>New analysis</span>
                  </span>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>

            {analyses.length > 0 && (
              <ScrollArea className="-mx-1 min-h-0 flex-1 px-1">
                <SidebarMenu className="gap-1.5">
                  {analyses.map((analysis) => (
                    <SidebarMenuItem key={analysis.id} className="sidebar-report-row">
                      <SidebarMenuButton
                        asChild
                        isActive={currentView === "analysis" && selectedAnalysisId === analysis.id}
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
              </ScrollArea>
            )}
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter className="gap-2 px-4 pb-4 pt-2">
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
