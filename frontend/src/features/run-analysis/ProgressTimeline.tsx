import {
  CheckCircle,
  CircleNotch,
  MagnifyingGlass,
  WarningCircle,
  XCircle,
} from '@phosphor-icons/react';
import { useCallback, useEffect, useMemo, useRef } from 'react';
import MarkdownMessage from '@/components/Agent/MarkdownMessage';
import ToolCallCard from '@/components/Agent/ToolCallCard';
import { Button } from '@/components/ui/button';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { getRunProgress } from '@/shared/api/commands';
import { setRunProgress } from '@/store';
import type { ProgressItem, RunState } from '@/types';
import { getTimelineBlocks, handleProgressEvent } from './progress';

interface ProgressTimelineProps {
  activeRuns: Record<string, RunState>;
  selectedRunTab: string | null;
  onSelectTab: (runId: string) => void;
  onExampleSelect: (prompt: string) => void;
}

const EXAMPLE_PROMPTS = [
  'Compare NVDA to AMD',
  'Analyze the energy sector',
  'Review US regional banks',
];

export function ProgressTimeline({
  activeRuns,
  selectedRunTab,
  onSelectTab,
  onExampleSelect,
}: ProgressTimelineProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const runEntries = Object.values(activeRuns);
  const hasRuns = runEntries.length > 0;
  const currentRun = selectedRunTab ? activeRuns[selectedRunTab] : null;
  const progress = currentRun?.progress ?? [];
  const isRunning = currentRun?.status === 'running';
  const timelineBlocks = useMemo(() => getTimelineBlocks(progress), [progress]);

  const hydrateTab = useCallback(async (runId: string) => {
    const run = activeRuns[runId];
    if (!run || run.progress.length > 0) return;
    try {
      const events = await getRunProgress(runId);
      // Build progress items by replaying events into a temporary array
      const items: ProgressItem[] = [];
      for (const event of events) {
        replayEvent(event, items);
      }
      setRunProgress(runId, items);
    } catch {
      // non-critical — live stream will fill in
    }
  }, [activeRuns]);

  useEffect(() => {
    if (selectedRunTab) {
      hydrateTab(selectedRunTab);
    }
  }, [selectedRunTab, hydrateTab]);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [timelineBlocks, isRunning]);

  return (
    <div className="flex-1 overflow-y-auto px-6 py-6 pb-32" ref={scrollRef}>
      <div className="mx-auto max-w-3xl space-y-8">
        {hasRuns && runEntries.length > 1 && (
          <Tabs
            value={selectedRunTab ?? undefined}
            onValueChange={onSelectTab}
            className="gap-0"
          >
            <TabsList>
              {runEntries.map(run => (
                <TabsTrigger
                  key={run.runId}
                  value={run.runId}
                  className="flex-none px-3 text-xs"
                >
                  <RunStatusIcon status={run.status} />
                  {run.agentLabel}
                </TabsTrigger>
              ))}
            </TabsList>
          </Tabs>
        )}

        {!hasRuns && (
          <div className="flex flex-col items-center justify-center pt-24 text-center text-muted-foreground">
            <MagnifyingGlass size={32} className="mb-4 opacity-20" />
            <p className="text-sm">Enter a research prompt below to begin analysis.</p>
            <div className="mt-8 flex max-w-md flex-wrap justify-center gap-2">
              {EXAMPLE_PROMPTS.map(example => (
                <Button
                  key={example}
                  variant="outline"
                  size="xs"
                  className="bg-card"
                  onClick={() => onExampleSelect(example)}
                >
                  {example}
                </Button>
              ))}
            </div>
          </div>
        )}

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
  );
}

function RunStatusIcon({ status }: { status: RunState['status'] }) {
  switch (status) {
    case 'running':
      return <CircleNotch size={12} className="animate-spin text-primary" />;
    case 'completed':
      return <CheckCircle size={12} className="text-green-500" />;
    case 'error':
      return <XCircle size={12} className="text-destructive" />;
    case 'cancelled':
      return <XCircle size={12} className="text-muted-foreground" />;
  }
}

/**
 * Replay a single persisted event into a progress items array.
 * Used for hydrating tab state from DB.
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
