import { useLayoutEffect, useRef, useState, type CSSProperties } from "react";
import { CircleNotch } from "@phosphor-icons/react";
import { Button } from "@/components/ui/button";
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
import type { AppView } from "@/app/navigation";
import type { AnalysisSummary } from "@/types";

interface AppSidebarProps {
  analyses: AnalysisSummary[];
  currentView: AppView;
  selectedAnalysisId: string | null;
  onViewChange: (view: AppView) => void;
  onSelectAnalysis: (analysisId: string) => void | Promise<void>;
}

export function AppSidebar({
  analyses,
  currentView,
  selectedAnalysisId,
  onViewChange,
  onSelectAnalysis,
}: AppSidebarProps) {
  return (
    <Sidebar className="border-r border-sidebar-border bg-sidebar" variant="sidebar">
      <div data-tauri-drag-region className="h-10 shrink-0" />

      <SidebarContent className="gap-0 overflow-hidden">
        <SidebarGroup className="min-h-0 flex-1 px-3 py-0">
          <SidebarGroupContent className="flex min-h-0 flex-1 flex-col">
            <div className="mb-2 flex items-center justify-between">
              <div className="px-1 text-[13px] font-semibold tracking-tight text-sidebar-foreground">
                Analysis
              </div>
              {analyses.length > 0 && (
                <span className="px-1 font-mono text-[11px] text-sidebar-foreground/50">
                  {analyses.length}
                </span>
              )}
            </div>

            <SidebarMenu className="mb-3">
              <SidebarMenuItem>
                <SidebarMenuButton
                  isActive={currentView === "new-analysis"}
                  onClick={() => onViewChange("new-analysis")}
                  className="h-8 text-[13px]"
                >
                  <span>+ New analysis</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>

            {analyses.length > 0 && (
              <ScrollArea className="min-h-0 flex-1 pr-1">
                <SidebarMenu className="gap-0.5">
                  {analyses.map((analysis) => (
                    <SidebarMenuItem key={analysis.id} className="sidebar-report-row">
                      <SidebarMenuButton
                        asChild
                        isActive={
                          currentView === "analysis" &&
                          selectedAnalysisId === analysis.id
                        }
                        className="h-8 px-2 text-[13px] font-normal data-[active=true]:font-medium"
                      >
                        <Button
                          type="button"
                          variant="ghost"
                          size="xs"
                          className="h-8 min-w-0 justify-start gap-2 px-2 text-[13px]"
                          onClick={() => {
                            void onSelectAnalysis(analysis.id);
                          }}
                        >
                          <AnalysisStatus analysis={analysis} />
                          <MarqueeTitle title={analysis.title} />
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

      <SidebarFooter className="border-t border-sidebar-border p-1">
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton
              isActive={currentView === "settings"}
              onClick={() => onViewChange("settings")}
              className="h-8 text-[13px]"
            >
              <span>Settings</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  );
}

function AnalysisStatus({ analysis }: { analysis: AnalysisSummary }) {
  if (
    analysis.active_run_status === "running" ||
    analysis.active_run_status === "queued"
  ) {
    return <CircleNotch className="animate-spin text-primary" />;
  }

  const statusClass =
    analysis.status === "completed"
      ? "bg-green-500/50"
      : analysis.status === "failed"
        ? "bg-destructive/60"
        : "bg-primary/45";

  return (
    <span
      aria-hidden="true"
      className={`size-1.5 shrink-0 rounded-full ${statusClass}`}
    />
  );
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
      className="sidebar-report-title"
      data-scrollable={metrics.scrollable ? "true" : undefined}
      style={style}
    >
      <span ref={textRef} className="sidebar-report-title-inner">
        {title}
      </span>
    </span>
  );
}
