import { useState } from 'react';
import { useAppStore } from '@/store';
import type { AgentCandidate } from '@/types';
import { ProgressTimeline } from './ProgressTimeline';
import { ResearchComposer } from './ResearchComposer';
import { useRunAnalysis } from './useRunAnalysis';

interface ResearchPageProps {
  agents: AgentCandidate[];
  onDone: () => Promise<void>;
}

export function ResearchPage({ agents, onDone }: ResearchPageProps) {
  const [prompt, setPrompt] = useState('');
  const agentId = useAppStore(state => state.agentId);
  const isRunning = useAppStore(state => state.isRunning);
  const progress = useAppStore(state => state.progress);

  const selectedAgent = agents.find(agent => agent.id === agentId);
  const canRun = prompt.trim().length > 0 && !!selectedAgent?.available && !isRunning;
  const { localError, start, stop } = useRunAnalysis({ agentId, canRun, onDone });

  return (
    <div className="relative flex h-full flex-col bg-background">
      <ProgressTimeline
        isRunning={isRunning}
        progress={progress}
        onExampleSelect={setPrompt}
      />
      <ResearchComposer
        agentId={agentId}
        agents={agents}
        canRun={canRun}
        isRunning={isRunning}
        localError={localError}
        prompt={prompt}
        selectedAgent={selectedAgent}
        onPromptChange={setPrompt}
        onRun={() => start(prompt)}
        onStop={stop}
      />
    </div>
  );
}
