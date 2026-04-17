import { getLogoPath } from "@/lib/agents";
import { cn } from "@/lib/utils";
import type { AgentCandidate } from "@/types";

interface AgentStatusListProps {
  agents: AgentCandidate[];
}

export function AgentStatusList({ agents }: AgentStatusListProps) {
  if (agents.length === 0) {
    return (
      <div className="border border-border px-4 py-6 text-sm text-muted-foreground">
        No ACP agents detected on this machine.
      </div>
    );
  }

  return (
    <div className="divide-y divide-border border-y border-border">
      {agents.map((agent, index) => (
        <div
          key={agent.id}
          className="grid grid-cols-[32px_28px_1fr_auto] items-center gap-4 px-1 py-4"
        >
          <span className="font-mono text-[11px] tabular-nums text-muted-foreground">
            {String(index + 1).padStart(2, "0")}
          </span>
          <img
            src={getLogoPath(agent.label)}
            alt=""
            className="h-5 w-5 object-contain opacity-90"
          />
          <div className="flex flex-col gap-0.5">
            <span className="text-[14px] font-medium text-foreground">{agent.label}</span>
            <span className="truncate font-mono text-[11.5px] text-muted-foreground">
              {agent.command || "Not found on PATH"}
            </span>
          </div>
          <StatusTag available={agent.available} />
        </div>
      ))}
    </div>
  );
}

function StatusTag({ available }: { available: boolean }) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 text-[10.5px] font-medium uppercase tracking-[0.14em]",
        available ? "text-foreground" : "text-muted-foreground",
      )}
    >
      <span
        aria-hidden
        className={cn(
          "h-1.5 w-1.5 rounded-full",
          available ? "bg-foreground" : "bg-muted-foreground/40",
        )}
      />
      {available ? "Available" : "Missing"}
    </span>
  );
}
