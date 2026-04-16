import { MagnifyingGlass, WarningCircle } from '@phosphor-icons/react';
import { useEffect, useMemo, useRef } from 'react';
import MarkdownMessage from '@/components/Agent/MarkdownMessage';
import ToolCallCard from '@/components/Agent/ToolCallCard';
import type { ProgressItem } from '@/types';
import { getTimelineBlocks } from './progress';

interface ProgressTimelineProps {
  isRunning: boolean;
  progress: ProgressItem[];
  onExampleSelect: (prompt: string) => void;
}

const EXAMPLE_PROMPTS = [
  'Compare NVDA to AMD',
  'Analyze the energy sector',
  'Review US regional banks',
];

export function ProgressTimeline({
  isRunning,
  progress,
  onExampleSelect,
}: ProgressTimelineProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const timelineBlocks = useMemo(() => getTimelineBlocks(progress), [progress]);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [timelineBlocks, isRunning]);

  return (
    <div className="flex-1 overflow-y-auto px-6 py-6 pb-32" ref={scrollRef}>
      <div className="mx-auto max-w-3xl space-y-8">
        {timelineBlocks.length === 0 && !isRunning && (
          <div className="flex flex-col items-center justify-center pt-24 text-center text-muted-foreground">
            <MagnifyingGlass size={32} className="mb-4 opacity-20" />
            <p className="text-sm">Enter a research prompt below to begin analysis.</p>
            <div className="mt-8 flex max-w-md flex-wrap justify-center gap-2">
              {EXAMPLE_PROMPTS.map(example => (
                <button
                  key={example}
                  className="border border-border bg-card px-3 py-1.5 text-xs font-medium transition-colors hover:bg-muted"
                  onClick={() => onExampleSelect(example)}
                >
                  {example}
                </button>
              ))}
            </div>
          </div>
        )}

        {timelineBlocks.map(block => {
          if (block.type === 'message') {
            return (
              <div key={block.id} className="border-l border-border/30 pl-8">
                <MarkdownMessage text={block.content} />
              </div>
            );
          }

          if (block.type === 'tool') {
            return (
              <div key={block.id} className="border-l-2 border-primary/20 bg-muted/10 pl-4">
                <div className="max-w-[400px]">
                  <ToolCallCard
                    title={block.title}
                    toolName={block.toolName || block.title}
                    arguments={block.arguments}
                    result={block.result}
                    status={block.status}
                    displayTitle={null}
                  />
                </div>
              </div>
            );
          }

          if (block.type === 'error') {
            return (
              <div key={block.id} className="flex items-center gap-2 pl-4 text-xs text-destructive">
                <WarningCircle size={14} /> {block.content}
              </div>
            );
          }

          return (
            <div key={block.id} className="pl-4 text-xs italic text-muted-foreground/60">
              {block.content}
            </div>
          );
        })}

        {isRunning && (
          <div className="flex animate-pulse items-center gap-2 pl-8 text-xs text-muted-foreground">
            <div className="h-1.5 w-1.5 bg-primary/50" />
            Agent is working...
          </div>
        )}
      </div>
    </div>
  );
}
