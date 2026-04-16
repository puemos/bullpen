import { CaretDown, Check } from "@phosphor-icons/react";
import { cn } from "@/lib/utils";
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
} from "@/components/ui/dropdown-menu";
import type { AgentCandidate } from "@/types";
import { getLogoPath } from "@/lib/agents";

interface AgentSelectorProps {
  agents: AgentCandidate[];
  selectedAgentId: string;
  onSelect: (agentId: string) => void;
  disabled?: boolean;
}

export default function AgentSelector({
  agents,
  selectedAgentId,
  onSelect,
  disabled,
}: AgentSelectorProps) {
  const selectedAgent = agents.find((a) => a.id === selectedAgentId);

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild disabled={disabled}>
        <button
          className={cn(
            "flex items-center gap-2 px-3 py-1.5 rounded-md transition-all border outline-none",
            "border-border/50 hover:border-border hover:bg-muted/30 focus-visible:ring-2 focus-visible:ring-ring",
            "data-[state=open]:bg-muted/50 data-[state=open]:border-border",
            disabled && "opacity-50 cursor-not-allowed",
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
            {selectedAgent?.label || "Select Agent"}
            {!selectedAgent?.available && selectedAgent && " (offline)"}
          </span>
          <CaretDown size={12} className="text-muted-foreground ml-1" weight="bold" />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align="start"
        className="w-[200px] rounded-md bg-popover/95 backdrop-blur-xl border-border shadow-md p-1"
      >
        {agents.map((agent) => {
          const isSelected = agent.id === selectedAgentId;
          const isUnavailable = !agent.available;

          if (isUnavailable) {
            return (
              <div 
                key={agent.id}
                className="flex items-center gap-2 px-2 py-1.5 rounded text-xs w-full text-left opacity-40 cursor-not-allowed"
              >
                <img src={getLogoPath(agent.label)} alt={agent.label} className="w-3.5 h-3.5 object-contain opacity-80" />
                <span className="flex-1 truncate font-medium">{agent.label}</span>
                <span className="text-[10px] text-muted-foreground">offline</span>
              </div>
            );
          }

          return (
            <button
              key={agent.id}
              onClick={() => onSelect(agent.id)}
              className={cn(
                "flex items-center gap-2 px-2 py-1.5 rounded text-xs transition-colors w-full text-left outline-none",
                isSelected
                  ? "bg-accent/50 text-accent-foreground font-medium"
                  : "text-muted-foreground hover:bg-muted hover:text-foreground",
              )}
            >
              <img src={getLogoPath(agent.label)} alt={agent.label} className="w-3.5 h-3.5 object-contain opacity-80" />
              <span className="flex-1 truncate">{agent.label}</span>
              {isSelected && <Check size={12} weight="bold" />}
            </button>
          );
        })}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
