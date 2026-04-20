import { Check } from "@phosphor-icons/react";
import { type FormEvent, useEffect, useState } from "react";
import { DropdownMenuItem, DropdownMenuSeparator } from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import type { AgentCandidate } from "@/types";

type SelectAgentModel = (agentId: string, modelId: string | null) => void;

interface AgentModelOptionsProps {
  agent: AgentCandidate;
  activeModelId: string | null;
  isSelected: boolean;
  onSelect: SelectAgentModel;
  showChecks?: boolean;
}

export function hasAgentModelChoices(agent: AgentCandidate) {
  return agent.models.length > 0 || agent.supports_model_override;
}

export function getAgentModelLabel(agent: AgentCandidate, modelId: string | null) {
  if (!modelId) return null;
  return agent.models.find((model) => model.id === modelId)?.name ?? modelId;
}

export function isCustomAgentModel(agent: AgentCandidate, modelId: string | null) {
  return Boolean(modelId && !agent.models.some((model) => model.id === modelId));
}

export function AgentModelOptions({
  agent,
  activeModelId,
  isSelected,
  onSelect,
  showChecks = true,
}: AgentModelOptionsProps) {
  const customModelId =
    agent.supports_model_override && isCustomAgentModel(agent, activeModelId) ? activeModelId : "";

  return (
    <>
      <DropdownMenuItem
        onSelect={() => onSelect(agent.id, null)}
        className={cn(
          "gap-2 text-xs",
          showChecks &&
            isSelected &&
            activeModelId === null &&
            "bg-accent/50 font-medium text-accent-foreground",
        )}
      >
        <span className="flex-1 truncate">Agent default</span>
        {showChecks && isSelected && activeModelId === null && <Check size={12} weight="bold" />}
      </DropdownMenuItem>

      {agent.models.map((model) => {
        const isModelSelected = isSelected && model.id === activeModelId;
        return (
          <DropdownMenuItem
            key={model.id}
            onSelect={() => onSelect(agent.id, model.id)}
            className={cn(
              "gap-2 text-xs",
              showChecks && isModelSelected && "bg-accent/50 font-medium text-accent-foreground",
            )}
          >
            <span className="flex-1 truncate">{model.name}</span>
            {showChecks && isModelSelected && <Check size={12} weight="bold" />}
          </DropdownMenuItem>
        );
      })}

      {agent.supports_model_override && (
        <>
          <DropdownMenuSeparator />
          <CustomModelForm
            agent={agent}
            activeModelId={customModelId}
            onSelect={onSelect}
            showCheck={showChecks && isSelected && Boolean(customModelId)}
          />
        </>
      )}
    </>
  );
}

function CustomModelForm({
  agent,
  activeModelId,
  onSelect,
  showCheck,
}: {
  agent: AgentCandidate;
  activeModelId: string | null;
  onSelect: SelectAgentModel;
  showCheck: boolean;
}) {
  const [value, setValue] = useState(activeModelId ?? "");

  useEffect(() => {
    setValue(activeModelId ?? "");
  }, [activeModelId]);

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const next = value.trim();
    if (!next) return;
    onSelect(agent.id, next);
  };

  return (
    <form
      className="space-y-2 px-2 py-2"
      onSubmit={handleSubmit}
      onClick={(event) => event.stopPropagation()}
      onKeyDown={(event) => event.stopPropagation()}
    >
      <div className="flex items-center gap-2">
        <label
          htmlFor={`${agent.id}-custom-model`}
          className="flex-1 font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground"
        >
          Custom model
        </label>
        {showCheck && <Check size={12} weight="bold" className="text-foreground" />}
      </div>
      <div className="flex gap-1.5">
        <Input
          id={`${agent.id}-custom-model`}
          value={value}
          onChange={(event) => setValue(event.target.value)}
          placeholder="model id"
          className="h-7 rounded-none border-border px-2 font-mono text-[11.5px] shadow-none focus-visible:ring-0"
        />
        <button
          type="submit"
          className="h-7 border border-foreground bg-foreground px-2 font-mono text-[10.5px] uppercase tracking-[0.14em] text-background transition-colors hover:bg-background hover:text-foreground disabled:border-border disabled:bg-transparent disabled:text-muted-foreground"
          disabled={!value.trim()}
        >
          Use
        </button>
      </div>
    </form>
  );
}
