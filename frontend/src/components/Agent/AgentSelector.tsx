import { CaretDown, Check } from "@phosphor-icons/react";
import {
  AgentModelOptions,
  getAgentModelLabel,
  hasAgentModelChoices,
} from "@/components/Agent/AgentModelOptions";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { getLogoPath } from "@/lib/agents";
import { cn } from "@/lib/utils";
import type { AgentCandidate } from "@/types";

interface AgentSelectorProps {
  agents: AgentCandidate[];
  selectedAgentId: string;
  modelByAgent: Record<string, string | null>;
  onSelect: (agentId: string, modelId: string | null) => void;
  disabled?: boolean;
}

export default function AgentSelector({
  agents,
  selectedAgentId,
  modelByAgent,
  onSelect,
  disabled,
}: AgentSelectorProps) {
  const selectedAgent = agents.find((a) => a.id === selectedAgentId);
  const selectedModelId = selectedAgent ? (modelByAgent[selectedAgent.id] ?? null) : null;
  const selectedModelName = selectedAgent
    ? getAgentModelLabel(selectedAgent, selectedModelId)
    : null;

  const triggerLabel = selectedAgent
    ? selectedModelName
      ? `${selectedAgent.label} · ${selectedModelName}`
      : selectedAgent.label
    : "Select Agent";

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild disabled={disabled}>
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={disabled}
          className={cn(
            "h-8 border-border/50 bg-transparent px-3 text-xs hover:border-border hover:bg-muted/30",
            "data-[state=open]:bg-muted/50 data-[state=open]:border-border",
          )}
        >
          <span className="flex items-center gap-2 text-xs font-medium text-foreground">
            {selectedAgent && (
              <img
                src={getLogoPath(selectedAgent.label)}
                alt={selectedAgent.label}
                className="w-4 h-4 object-contain"
              />
            )}
            {triggerLabel}
            {!selectedAgent?.available && selectedAgent && " (offline)"}
          </span>
          <CaretDown size={12} className="text-muted-foreground ml-1" weight="bold" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align="start"
        className="w-[220px] rounded-md bg-popover/95 backdrop-blur-xl border-border shadow-md p-1"
      >
        {agents.map((agent) => {
          const isSelected = agent.id === selectedAgentId;
          const isUnavailable = !agent.available;
          const hasModels = hasAgentModelChoices(agent);
          const activeModelId = modelByAgent[agent.id] ?? null;

          if (hasModels && !isUnavailable) {
            return (
              <DropdownMenuSub key={agent.id}>
                <DropdownMenuSubTrigger
                  className={cn(
                    "gap-2 text-xs",
                    isSelected && "bg-accent/50 text-accent-foreground font-medium",
                  )}
                >
                  <img
                    src={getLogoPath(agent.label)}
                    alt={agent.label}
                    className="w-3.5 h-3.5 object-contain opacity-80"
                  />
                  <span className="flex-1 truncate">{agent.label}</span>
                  {isSelected && <Check size={12} weight="bold" className="shrink-0" />}
                </DropdownMenuSubTrigger>
                <DropdownMenuSubContent className="w-[200px] p-1">
                  <AgentModelOptions
                    agent={agent}
                    activeModelId={activeModelId}
                    isSelected={isSelected}
                    onSelect={onSelect}
                  />
                </DropdownMenuSubContent>
              </DropdownMenuSub>
            );
          }

          return (
            <DropdownMenuItem
              key={agent.id}
              disabled={isUnavailable}
              onSelect={() => onSelect(agent.id, null)}
              className={cn(
                "gap-2 text-xs",
                isSelected && "bg-accent/50 text-accent-foreground font-medium",
              )}
            >
              <img
                src={getLogoPath(agent.label)}
                alt={agent.label}
                className="w-3.5 h-3.5 object-contain opacity-80"
              />
              <span className="flex-1 truncate">{agent.label}</span>
              {isUnavailable && <span className="text-[10px] text-muted-foreground">offline</span>}
              {isSelected && <Check size={12} weight="bold" />}
            </DropdownMenuItem>
          );
        })}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
