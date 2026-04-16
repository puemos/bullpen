import { useState } from 'react';
import { Eyebrow } from '@/components/ui/editorial';
import { useAppStore } from '@/store';
import type { AgentCandidate } from '@/types';
import { ResearchComposer } from './ResearchComposer';
import { useRunAnalysis } from './useRunAnalysis';

interface ExamplePrompt {
  tag: string;
  text: string;
}

const EXAMPLE_PROMPTS: ExamplePrompt[] = [
  {
    tag: 'Compare',
    text: 'Compare NVDA to AMD across AI compute margins and supply constraints.',
  },
  {
    tag: 'Sector',
    text: "Is the energy sector's dividend growth sustainable through 2027?",
  },
  {
    tag: 'Stress',
    text: 'Stress-test US regional banks under a 300bps rate-hike shock.',
  },
  {
    tag: 'Single',
    text: 'Build the bull and bear case for TSM, focusing on geopolitical risk.',
  },
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
    <div className="relative flex h-full min-h-0 flex-col overflow-y-auto bg-background">
      <div className="mx-auto flex w-full max-w-3xl flex-1 flex-col px-8 pt-24 pb-16">
        <div className="mb-10">
          <Eyebrow>New analysis</Eyebrow>
        </div>

        <h1 className="mb-12 text-[56px] font-semibold leading-[0.98] tracking-[-0.03em] sm:text-[72px]">
          What do you
          <br />
          want to know?
        </h1>

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

        <div className="mt-20 space-y-5 border-t border-border pt-8">
          <div className="flex items-baseline justify-between">
            <Eyebrow>Start from an example</Eyebrow>
            <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground/70">
              {String(EXAMPLE_PROMPTS.length).padStart(2, '0')}
            </span>
          </div>

          <ol className="divide-y divide-border border-y border-border">
            {EXAMPLE_PROMPTS.map((example, index) => (
              <li key={example.text}>
                <button
                  type="button"
                  onClick={() => setPrompt(example.text)}
                  className="group grid w-full grid-cols-[32px_80px_1fr] items-baseline gap-4 px-1 py-4 text-left transition-colors hover:bg-muted/40"
                >
                  <span className="font-mono text-[11px] tabular-nums text-muted-foreground">
                    {String(index + 1).padStart(2, '0')}
                  </span>
                  <span className="text-[10.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground">
                    {example.tag}
                  </span>
                  <span className="text-[15px] leading-snug text-foreground group-hover:text-foreground">
                    {example.text}
                  </span>
                </button>
              </li>
            ))}
          </ol>
        </div>
      </div>
    </div>
  );
}
