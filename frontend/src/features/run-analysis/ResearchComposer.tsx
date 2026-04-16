import { ArrowRight, WarningCircle } from '@phosphor-icons/react';
import type { KeyboardEvent } from 'react';
import AgentSelector from '@/components/Agent/AgentSelector';
import { Textarea } from '@/components/ui/textarea';
import { setState } from '@/store';
import type { AgentCandidate } from '@/types';

interface ResearchComposerProps {
  agentId: string;
  agents: AgentCandidate[];
  canRun: boolean;
  localError: string | null;
  prompt: string;
  selectedAgent: AgentCandidate | undefined;
  onPromptChange: (prompt: string) => void;
  onRun: () => void;
}

export function ResearchComposer({
  agentId,
  agents,
  canRun,
  localError,
  prompt,
  selectedAgent,
  onPromptChange,
  onRun,
}: ResearchComposerProps) {
  const handleKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
      event.preventDefault();
      if (canRun) onRun();
    }
  };

  return (
    <div className="space-y-4">
      <div className="border-t border-b border-border">
        <Textarea
          className="min-h-[140px] w-full resize-none border-0 bg-transparent px-0 py-5 text-[22px] leading-[1.35] tracking-[-0.01em] shadow-none outline-none placeholder:text-muted-foreground/40 focus-visible:border-transparent focus-visible:ring-0 md:text-[22px]"
          rows={4}
          value={prompt}
          onChange={event => onPromptChange(event.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Type your research question…"
          autoFocus
        />
      </div>

      <div className="flex items-center justify-between gap-4">
        <AgentSelector
          agents={agents}
          selectedAgentId={agentId}
          onSelect={id => setState({ agentId: id })}
        />
        <div className="flex items-center gap-4">
          <span className="hidden font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground/70 sm:inline">
            {canRun ? '⌘ + ↵ to run' : ''}
          </span>
          <button
            type="button"
            disabled={!canRun}
            onClick={onRun}
            className="group inline-flex items-center gap-2 border border-foreground bg-foreground px-4 py-2 text-[13px] font-medium text-background transition-colors hover:bg-background hover:text-foreground disabled:border-border disabled:bg-transparent disabled:text-muted-foreground/60"
          >
            <span>Run analysis</span>
            <ArrowRight
              size={14}
              weight="bold"
              className="transition-transform group-enabled:group-hover:translate-x-0.5"
            />
          </button>
        </div>
      </div>

      {(!selectedAgent?.available || localError) && (
        <div className="space-y-2 pt-2">
          {!selectedAgent?.available && (
            <div className="flex items-center gap-2 text-xs text-destructive">
              <WarningCircle size={14} />
              <span>Configure an ACP agent binary before running analysis.</span>
            </div>
          )}
          {localError && (
            <div className="flex items-center gap-2 text-xs text-destructive">
              <WarningCircle size={14} />
              <span>{localError}</span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
