import { Copy, DownloadSimple, Stop, Trash, WarningCircle } from "@phosphor-icons/react";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import MarkdownMessage from "@/components/Agent/MarkdownMessage";
import ToolCallCard from "@/components/Agent/ToolCallCard";
import { Dot, Eyebrow } from "@/components/ui/editorial";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ReportContent } from "@/features/report-viewer/ReportContent";
import { getTimelineBlocks } from "@/features/run-analysis/progress";
import {
  deleteAnalysis,
  exportAnalysisHtml,
  exportAnalysisMarkdown,
  getRunProgress,
  stopAnalysis,
} from "@/shared/api/commands";
import { addRun, addRunProgress, setRunProgress, setState, useAppStore } from "@/store";
import type { Analysis, AnalysisReport, AnalysisSummary, ProgressItem, RunState } from "@/types";

interface AnalysisPageProps {
  onRefresh: () => Promise<void>;
}

export function AnalysisPage({ onRefresh }: AnalysisPageProps) {
  const selectedAnalysisId = useAppStore((state) => state.selectedAnalysisId);
  const report = useAppStore((state) => state.selectedReport);
  const analyses = useAppStore((state) => state.analyses);
  const activeRuns = useAppStore((state) => state.activeRuns);
  const subTab = useAppStore((state) => state.analysisSubTab);
  const [copyState, setCopyState] = useState<string | null>(null);
  const [exportState, setExportState] = useState<string | null>(null);

  const selectedAnalysis = useMemo(
    () => analyses.find((analysis) => analysis.id === selectedAnalysisId) ?? null,
    [analyses, selectedAnalysisId],
  );

  // Find the run for this analysis
  const currentRun = useMemo(() => {
    if (!selectedAnalysisId) return null;
    return (
      Object.values(activeRuns).find((r) => r.runId === report?.analysis.active_run_id) ?? null
    );
  }, [activeRuns, selectedAnalysisId, report]);

  const activeRunMeta = report?.runs.find((r) => r.id === report.analysis.active_run_id);
  const runId = currentRun?.runId ?? report?.analysis.active_run_id ?? null;
  const isRunning = currentRun?.status === "running";
  const title = report?.analysis.title ?? selectedAnalysis?.title ?? "Analysis";
  const prompt = report?.analysis.user_prompt ?? selectedAnalysis?.user_prompt ?? null;

  const remove = useCallback(async () => {
    if (!report) return;

    await deleteAnalysis(report.analysis.id);
    setState({ selectedAnalysisId: null, selectedReport: null, view: "new-analysis" });
    await onRefresh();
  }, [onRefresh, report]);

  const copyMarkdown = useCallback(async () => {
    if (!report) return;

    const markdown = await exportAnalysisMarkdown(report.analysis.id);
    await writeText(markdown);
    setCopyState("Copied!");
    setTimeout(() => setCopyState(null), 1500);
  }, [report]);

  const exportHtml = useCallback(async () => {
    if (!report) return;
    setExportState("Exporting…");
    try {
      const result = await exportAnalysisHtml(report.analysis.id);
      if (result) {
        setExportState("Saved");
      } else {
        setExportState(null);
        return;
      }
    } catch (err) {
      console.error("export html failed:", err);
      setExportState("Failed");
    }
    setTimeout(() => setExportState(null), 1800);
  }, [report]);

  if (!selectedAnalysisId) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
        No analysis selected.
      </div>
    );
  }

  return (
    <Tabs
      value={subTab}
      onValueChange={(value) => setState({ analysisSubTab: value as "report" | "agent" })}
      className="h-full gap-0"
    >
      <div className="shrink-0 border-b border-border bg-background">
        <div className="mx-auto flex max-w-5xl flex-col gap-6 px-8 pt-10 pb-5">
          <AnalysisMetaLine analysis={selectedAnalysis} report={report} isRunning={isRunning} />

          <div className="space-y-4">
            <h1 className="text-[34px] font-semibold leading-[1.05] tracking-[-0.02em]">{title}</h1>
            {prompt && (
              <p className="max-w-[62ch] text-[14.5px] leading-[1.55] text-muted-foreground">
                {prompt}
              </p>
            )}
          </div>

          <div className="flex flex-wrap items-center justify-between gap-3">
            <TabsList className="h-auto w-fit gap-6 rounded-none bg-transparent p-0">
              <TabsTrigger
                value="report"
                className="h-auto flex-none rounded-none border-0 border-b-2 border-transparent bg-transparent px-0 py-2 font-mono text-[11px] font-medium uppercase tracking-[0.16em] text-muted-foreground shadow-none data-[state=active]:border-foreground data-[state=active]:bg-transparent data-[state=active]:text-foreground data-[state=active]:shadow-none"
              >
                Report
              </TabsTrigger>
              <TabsTrigger
                value="agent"
                className="h-auto flex-none rounded-none border-0 border-b-2 border-transparent bg-transparent px-0 py-2 font-mono text-[11px] font-medium uppercase tracking-[0.16em] text-muted-foreground shadow-none data-[state=active]:border-foreground data-[state=active]:bg-transparent data-[state=active]:text-foreground data-[state=active]:shadow-none"
              >
                Agent
                {isRunning && <Dot className="ml-1 size-1.5 animate-pulse bg-primary" />}
              </TabsTrigger>
            </TabsList>

            {report && (
              <div className="flex items-center gap-5 text-[12.5px]">
                <button
                  type="button"
                  onClick={copyMarkdown}
                  className="inline-flex items-center gap-1.5 text-muted-foreground transition-colors hover:text-foreground"
                >
                  <Copy size={13} />
                  <span>{copyState || "Copy as markdown"}</span>
                </button>
                <span aria-hidden className="h-3 w-px bg-border" />
                <button
                  type="button"
                  onClick={exportHtml}
                  className="inline-flex items-center gap-1.5 text-muted-foreground transition-colors hover:text-foreground"
                >
                  <DownloadSimple size={13} />
                  <span>{exportState || "Export HTML"}</span>
                </button>
                <span aria-hidden className="h-3 w-px bg-border" />
                <button
                  type="button"
                  onClick={remove}
                  className="inline-flex items-center gap-1.5 text-muted-foreground transition-colors hover:text-destructive"
                >
                  <Trash size={13} />
                  <span>Delete</span>
                </button>
              </div>
            )}
          </div>
        </div>
      </div>

      <TabsContent value="report" className="mt-0 min-h-0 overflow-auto">
        <ReportContent />
      </TabsContent>
      <TabsContent value="agent" className="mt-0 min-h-0 overflow-hidden">
        <AgentTimeline
          runId={runId}
          run={currentRun}
          isRunning={isRunning}
          agentLabel={
            activeRunMeta
              ? activeRunMeta.agent_id +
                (activeRunMeta.model_id ? ` · ${activeRunMeta.model_id}` : "")
              : null
          }
        />
      </TabsContent>
    </Tabs>
  );
}

function AnalysisMetaLine({
  analysis,
  report,
  isRunning,
}: {
  analysis: Analysis | AnalysisSummary | null;
  report: AnalysisReport | null;
  isRunning: boolean;
}) {
  const intent = analysis?.intent;
  const status = report?.analysis.status ?? analysis?.status;
  const created = report?.analysis.created_at ?? analysis?.created_at;

  return (
    <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
      <Eyebrow>Analysis</Eyebrow>
      {intent && (
        <>
          <Dot />
          <Eyebrow>{intent.replace(/_/g, " ")}</Eyebrow>
        </>
      )}
      {status && (
        <>
          <Dot />
          <Eyebrow className={isRunning ? "text-primary" : undefined}>
            {isRunning ? "Running" : status}
          </Eyebrow>
        </>
      )}
      {created && (
        <>
          <Dot />
          <Eyebrow>{formatCreated(created)}</Eyebrow>
        </>
      )}
    </div>
  );
}

function formatCreated(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(date);
}

function AgentTimeline({
  runId,
  run,
  isRunning,
  agentLabel,
}: {
  runId: string | null;
  run: RunState | null;
  isRunning: boolean;
  agentLabel: string | null;
}) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const progress = run?.progress ?? [];
  const timelineBlocks = useMemo(() => getTimelineBlocks(progress), [progress]);

  // Hydrate progress from DB if we have a runId but no in-memory progress
  useEffect(() => {
    if (!runId || (run && run.progress.length > 0)) return;
    getRunProgress(runId)
      .then((events) => {
        const items: ProgressItem[] = [];
        for (const event of events) {
          replayEvent(event, items);
        }
        // Create a RunState if one doesn't exist in memory (e.g. past completed analysis)
        if (!run) {
          addRun({
            runId,
            agentId: "",
            agentLabel: agentLabel || "Agent",
            status: "completed",
            progress: items,
            plan: [],
          });
        } else {
          setRunProgress(runId, items);
        }
      })
      .catch(() => {
        // non-critical
      });
  }, [runId, run, agentLabel]);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, []);

  const handleStop = useCallback(async () => {
    if (!runId) return;
    addRunProgress(runId, "error", "Stop requested");
    await stopAnalysis(runId);
  }, [runId]);

  if (!runId) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
        No agent activity for this analysis.
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      <div className="flex-1 overflow-y-auto px-6 py-6" ref={scrollRef}>
        <div className="mx-auto max-w-3xl">
          {timelineBlocks.map((block) => {
            if (block.type === "message") {
              return (
                <div key={block.id} className="py-4">
                  <MarkdownMessage text={block.content} />
                </div>
              );
            }

            if (block.type === "tool") {
              return (
                <div key={block.id}>
                  <ToolCallCard
                    title={block.title}
                    toolName={block.toolName}
                    toolKind={block.kind}
                    arguments={block.arguments}
                    result={block.result}
                    status={block.status}
                  />
                </div>
              );
            }

            if (block.type === "error") {
              return (
                <div
                  key={block.id}
                  className="flex items-center gap-2 py-1 text-xs text-destructive"
                >
                  <WarningCircle size={14} /> {block.content}
                </div>
              );
            }

            return null;
          })}

          {isRunning && (
            <div className="flex animate-pulse items-center gap-2 py-2 text-xs text-muted-foreground">
              <Dot className="size-1.5 bg-primary" />
              Agent is working...
            </div>
          )}
        </div>
      </div>

      {isRunning && (
        <div className="shrink-0 border-t border-border">
          <div className="mx-auto max-w-3xl px-6 py-3">
            <button
              type="button"
              onClick={handleStop}
              className="inline-flex items-center gap-1.5 text-[12.5px] text-muted-foreground transition-colors hover:text-destructive"
            >
              <Stop size={13} weight="fill" />
              <span>Stop</span>
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

/**
 * Replay a single persisted event into a progress items array.
 */
function replayEvent(payload: import("@/types").ProgressEventPayload, items: ProgressItem[]) {
  const push = (type: ProgressItem["type"], message: string, data?: unknown) => {
    items.push({ id: crypto.randomUUID(), type, message, timestamp: Date.now(), data });
  };
  const appendLast = (type: ProgressItem["type"], delta: string) => {
    const last = items[items.length - 1];
    if (last && last.type === type) {
      items[items.length - 1] = { ...last, message: last.message + delta };
    } else {
      items.push({ id: crypto.randomUUID(), type, message: delta, timestamp: Date.now() });
    }
  };

  switch (payload.event) {
    case "MessageDelta":
      appendLast("agent_message", payload.data.delta);
      break;
    case "ThoughtDelta":
      appendLast("agent_thought", payload.data.delta);
      break;
    case "ToolCallStarted":
      push("tool_call", payload.data.title, payload.data);
      break;
    case "ToolCallComplete":
      push("tool_result", `${payload.data.title || "tool"} ${payload.data.status}`, payload.data);
      break;
    case "Plan":
      push("plan", "Plan updated", payload.data);
      break;
    case "PlanSubmitted":
      push("submitted", "Research plan submitted");
      break;
    case "SourceSubmitted":
      push("submitted", "Source submitted");
      break;
    case "MetricSubmitted":
      push("submitted", "Metric submitted");
      break;
    case "ArtifactSubmitted":
      push("submitted", "Structured artifact submitted");
      break;
    case "BlockSubmitted":
      push("submitted", "Analysis block submitted");
      break;
    case "StanceSubmitted":
      push("submitted", "Final stance submitted");
      break;
    case "Completed":
      push("completed", "Analysis complete");
      break;
    case "Error":
      push("error", payload.data.message);
      break;
    case "Log":
      push("log", payload.data);
      break;
  }
}
