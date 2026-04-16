import { CircleNotch, Copy, Stop, Trash } from '@phosphor-icons/react';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import MarkdownMessage from '@/components/Agent/MarkdownMessage';
import ToolCallCard from '@/components/Agent/ToolCallCard';
import { Button } from '@/components/ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { ReportContent } from '@/features/report-viewer/ReportContent';
import {
  deleteAnalysis,
  exportAnalysisMarkdown,
  getRunProgress,
  stopAnalysis,
} from '@/shared/api/commands';
import { addRun, addRunProgress, setRunProgress, setState, useAppStore } from '@/store';
import type { ProgressItem, RunState } from '@/types';
import { getTimelineBlocks } from '@/features/run-analysis/progress';
import { WarningCircle } from '@phosphor-icons/react';

interface AnalysisPageProps {
  onRefresh: () => Promise<void>;
}

export function AnalysisPage({ onRefresh }: AnalysisPageProps) {
  const selectedAnalysisId = useAppStore(state => state.selectedAnalysisId);
  const report = useAppStore(state => state.selectedReport);
  const analyses = useAppStore(state => state.analyses);
  const activeRuns = useAppStore(state => state.activeRuns);
  const subTab = useAppStore(state => state.analysisSubTab);
  const [copyState, setCopyState] = useState<string | null>(null);

  const selectedAnalysis = useMemo(
    () => analyses.find(analysis => analysis.id === selectedAnalysisId) ?? null,
    [analyses, selectedAnalysisId]
  );

  // Find the run for this analysis
  const currentRun = useMemo(() => {
    if (!selectedAnalysisId) return null;
    return Object.values(activeRuns).find(
      r => r.runId === report?.analysis.active_run_id
    ) ?? null;
  }, [activeRuns, selectedAnalysisId, report]);

  const activeRunMeta = report?.runs.find(r => r.id === report.analysis.active_run_id);
  const runId = currentRun?.runId ?? report?.analysis.active_run_id ?? null;
  const isRunning = currentRun?.status === 'running';
  const title = report?.analysis.title ?? selectedAnalysis?.title ?? 'Analysis';
  const prompt = report?.analysis.user_prompt ?? selectedAnalysis?.user_prompt ?? null;

  const remove = useCallback(async () => {
    if (!report) return;

    await deleteAnalysis(report.analysis.id);
    setState({ selectedAnalysisId: null, selectedReport: null, view: 'new-analysis' });
    await onRefresh();
  }, [onRefresh, report]);

  const copyMarkdown = useCallback(async () => {
    if (!report) return;

    const markdown = await exportAnalysisMarkdown(report.analysis.id);
    await navigator.clipboard.writeText(markdown);
    setCopyState('Copied!');
    setTimeout(() => setCopyState(null), 1500);
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
      onValueChange={value =>
        setState({ analysisSubTab: value as 'report' | 'agent' })
      }
      className="h-full gap-0"
    >
      <div className="shrink-0 border-b border-border bg-background">
        <div className="mx-auto flex max-w-5xl flex-col gap-4 px-8 py-6">
          <div className="space-y-2">
            <h1 className="text-2xl font-semibold tracking-tight">{title}</h1>
            {prompt && (
              <p className="max-w-3xl text-sm text-muted-foreground">{prompt}</p>
            )}
          </div>

          <div className="flex flex-wrap items-center justify-between gap-3">
            <TabsList>
              <TabsTrigger value="report" className="flex-none px-4">
                Report
              </TabsTrigger>
              <TabsTrigger value="agent" className="flex-none px-4">
                Agent
                {isRunning && (
                  <CircleNotch size={12} className="animate-spin text-primary" />
                )}
              </TabsTrigger>
            </TabsList>

            {report && (
              <div className="flex gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  className="flex items-center gap-1.5"
                  onClick={copyMarkdown}
                >
                  <Copy size={14} /> {copyState || 'Markdown'}
                </Button>
                <Button
                  variant="destructive"
                  size="sm"
                  className="flex items-center gap-1.5"
                  onClick={remove}
                >
                  <Trash size={14} /> Delete
                </Button>
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
          agentLabel={activeRunMeta?.agent_id ?? null}
        />
      </TabsContent>
    </Tabs>
  );
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
    getRunProgress(runId).then(events => {
      const items: ProgressItem[] = [];
      for (const event of events) {
        replayEvent(event, items);
      }
      // Create a RunState if one doesn't exist in memory (e.g. past completed analysis)
      if (!run) {
        addRun({
          runId,
          agentId: '',
          agentLabel: agentLabel || 'Agent',
          status: 'completed',
          progress: items,
          plan: [],
        });
      } else {
        setRunProgress(runId, items);
      }
    }).catch(() => {
      // non-critical
    });
  }, [runId, run, agentLabel]);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [timelineBlocks, isRunning]);

  const handleStop = useCallback(async () => {
    if (!runId) return;
    addRunProgress(runId, 'error', 'Stop requested');
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
        <div className="mx-auto max-w-3xl space-y-8">
          {timelineBlocks.map(block => {
            if (block.type === 'message') {
              return (
                <div key={block.id}>
                  <MarkdownMessage text={block.content} />
                </div>
              );
            }

            if (block.type === 'tool') {
              return (
                <div key={block.id}>
                  <div className="max-w-[400px]">
                    <ToolCallCard
                      title={block.title}
                      toolName={block.toolName}
                      toolKind={block.kind}
                      arguments={block.arguments}
                      result={block.result}
                      status={block.status}
                    />
                  </div>
                </div>
              );
            }

            if (block.type === 'error') {
              return (
                <div key={block.id} className="flex items-center gap-2 text-xs text-destructive">
                  <WarningCircle size={14} /> {block.content}
                </div>
              );
            }

            return null;
          })}

          {isRunning && (
            <div className="flex animate-pulse items-center gap-2 text-xs text-muted-foreground">
              <div className="h-1.5 w-1.5 bg-primary/50" />
              Agent is working...
            </div>
          )}
        </div>
      </div>

      {isRunning && (
        <div className="flex shrink-0 justify-center border-t border-border p-4">
          <Button
            variant="destructive"
            size="sm"
            className="flex items-center gap-1.5"
            onClick={handleStop}
          >
            <Stop size={14} weight="fill" />
            Stop
          </Button>
        </div>
      )}
    </div>
  );
}

/**
 * Replay a single persisted event into a progress items array.
 */
function replayEvent(payload: import('@/types').ProgressEventPayload, items: ProgressItem[]) {
  const push = (type: ProgressItem['type'], message: string, data?: unknown) => {
    items.push({ id: crypto.randomUUID(), type, message, timestamp: Date.now(), data });
  };
  const appendLast = (type: ProgressItem['type'], delta: string) => {
    const last = items[items.length - 1];
    if (last && last.type === type) {
      items[items.length - 1] = { ...last, message: last.message + delta };
    } else {
      items.push({ id: crypto.randomUUID(), type, message: delta, timestamp: Date.now() });
    }
  };

  switch (payload.event) {
    case 'MessageDelta':
      appendLast('agent_message', payload.data.delta);
      break;
    case 'ThoughtDelta':
      appendLast('agent_thought', payload.data.delta);
      break;
    case 'ToolCallStarted':
      push('tool_call', payload.data.title, payload.data);
      break;
    case 'ToolCallComplete':
      push('tool_result', `${payload.data.title || 'tool'} ${payload.data.status}`, payload.data);
      break;
    case 'Plan':
      push('plan', 'Plan updated', payload.data);
      break;
    case 'PlanSubmitted':
      push('submitted', 'Research plan submitted');
      break;
    case 'SourceSubmitted':
      push('submitted', 'Source submitted');
      break;
    case 'MetricSubmitted':
      push('submitted', 'Metric submitted');
      break;
    case 'ArtifactSubmitted':
      push('submitted', 'Structured artifact submitted');
      break;
    case 'BlockSubmitted':
      push('submitted', 'Analysis block submitted');
      break;
    case 'StanceSubmitted':
      push('submitted', 'Final stance submitted');
      break;
    case 'Completed':
      push('completed', 'Analysis complete');
      break;
    case 'Error':
      push('error', payload.data.message);
      break;
    case 'Log':
      push('log', payload.data);
      break;
  }
}
