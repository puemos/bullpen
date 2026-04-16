import { CheckCircle, WarningCircle } from '@phosphor-icons/react';
import { getLogoPath } from '@/lib/agents';
import type { AgentCandidate } from '@/types';

interface AgentStatusListProps {
  agents: AgentCandidate[];
}

export function AgentStatusList({ agents }: AgentStatusListProps) {
  return (
    <div className="divide-y divide-border rounded-none border border-border text-sm">
      {agents.map(agent => (
        <div key={agent.id} className="flex items-center justify-between p-3">
          <div className="flex items-center gap-3">
            <img
              src={getLogoPath(agent.label)}
              alt={agent.label}
              className="h-5 w-5 object-contain"
            />
            <div className="flex flex-col">
              <strong className="font-medium">{agent.label}</strong>
              <span className="font-mono text-xs tracking-tight text-muted-foreground">
                {agent.command || 'Not found'}
              </span>
            </div>
          </div>
          {agent.available ? (
            <CheckCircle size={16} className="text-primary" />
          ) : (
            <WarningCircle size={16} className="text-destructive" />
          )}
        </div>
      ))}
    </div>
  );
}
