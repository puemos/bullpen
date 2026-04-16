import { useState } from 'react';
import { MagnifyingGlass } from '@phosphor-icons/react';
import { Button } from '@/components/ui/button';
import { useAppStore } from '@/store';
import type { AgentCandidate } from '@/types';
import { ResearchComposer } from './ResearchComposer';
import { useRunAnalysis } from './useRunAnalysis';

const EXAMPLE_PROMPTS = [
  'Compare NVDA to AMD',
  'Analyze the energy sector',
  'Review US regional banks',
];

interface ResearchPageProps {
  agents: AgentCandidate[];
  onDone: () => Promise<void>;
}

export function ResearchPage({ agents, onDone }: ResearchPageProps) {
  const [prompt, setPrompt] = useState('');
  const agentId = useAppStore(state => state.agentId);

  const selectedAgent = agents.find(agent => agent.id === agentId);
  const canRun = prompt.trim().length > 0 && !!selectedAgent?.available;

  const { localError, start } = useRunAnalysis({
    agentId,
    agents,
    canRun,
    onDone,
  });

  return (
    <div className="relative flex h-full flex-col bg-background">
      <div className="flex-1 overflow-y-auto px-6 py-6 pb-32">
        <div className="mx-auto max-w-3xl">
          <div className="flex flex-col items-center justify-center pt-24 text-center text-muted-foreground">
            <MagnifyingGlass size={32} className="mb-4 opacity-20" />
            <p className="text-sm">Enter a research prompt to begin.</p>
            <div className="mt-8 flex max-w-md flex-wrap justify-center gap-2">
              {EXAMPLE_PROMPTS.map(example => (
                <Button
                  key={example}
                  variant="outline"
                  size="xs"
                  className="bg-card"
                  onClick={() => setPrompt(example)}
                >
                  {example}
                </Button>
              ))}
            </div>
          </div>
        </div>
      </div>
      <ResearchComposer
        agentId={agentId}
        agents={agents}
        canRun={canRun}
        localError={localError}
        prompt={prompt}
        selectedAgent={selectedAgent}
        onPromptChange={setPrompt}
        onRun={() => start(prompt)}
      />
    </div>
  );
}
