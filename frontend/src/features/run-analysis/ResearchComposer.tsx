import { Play, WarningCircle } from '@phosphor-icons/react';
import AgentSelector from '@/components/Agent/AgentSelector';
import { Button } from '@/components/ui/button';
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
  return (
    <div className="absolute bottom-0 left-0 right-0 flex justify-center border-t border-border bg-background p-4">
      <div className="w-full max-w-3xl">
        {!selectedAgent?.available && (
          <div className="mb-2 flex items-center gap-2 text-xs text-destructive">
            <WarningCircle size={14} /> Configure an ACP agent binary before running analysis.
          </div>
        )}
        {localError && (
          <div className="mb-2 flex items-center gap-2 text-xs text-destructive">
            <WarningCircle size={14} /> {localError}
          </div>
        )}
        <div className="flex flex-col gap-2 border border-border bg-card p-2 shadow-sm transition-colors focus-within:border-primary/40">
          <Textarea
            className="min-h-[72px] w-full resize-none border-0 bg-transparent px-2 py-2 text-sm shadow-none outline-none placeholder:text-muted-foreground/50 focus-visible:border-transparent focus-visible:ring-0"
            rows={3}
            value={prompt}
            onChange={event => onPromptChange(event.target.value)}
            placeholder="What would you like to investigate?"
          />
          <div className="flex items-center justify-between border-t border-border/50 px-1 pt-2">
            <AgentSelector
              agents={agents}
              selectedAgentId={agentId}
              onSelect={id => setState({ agentId: id })}
            />
            <Button
              variant="default"
              size="sm"
              className="flex items-center gap-1.5 font-semibold"
              disabled={!canRun}
              onClick={onRun}
            >
              <Play size={14} weight="fill" />
              Run
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
