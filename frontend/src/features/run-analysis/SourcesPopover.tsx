import { ArrowUpRight, CaretDown, Check } from "@phosphor-icons/react";
import { useEffect, useRef, useState } from "react";
import { setState } from "@/store";
import type { SourceDescriptor } from "@/types";

interface SourcesPopoverProps {
  sources: SourceDescriptor[];
  selected: Set<string>;
  onToggle: (id: string) => void;
}

export function SourcesPopover({ sources, selected, onToggle }: SourcesPopoverProps) {
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onDocClick = (event: MouseEvent) => {
      if (rootRef.current && !rootRef.current.contains(event.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", onDocClick);
    return () => document.removeEventListener("mousedown", onDocClick);
  }, [open]);

  const availableCount = sources.filter((s) => !s.requires_key || s.has_key).length;
  const activeCount = sources.filter(
    (s) => selected.has(s.id) && (!s.requires_key || s.has_key),
  ).length;

  return (
    <div ref={rootRef} className="relative">
      <button
        type="button"
        onClick={() => setOpen((prev) => !prev)}
        className="inline-flex items-center gap-2 border border-border px-3 py-1.5 font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground hover:text-foreground"
      >
        <span>RUN SOURCES</span>
        <span className="tabular-nums text-foreground">
          {String(activeCount).padStart(2, "0")} / {String(availableCount).padStart(2, "0")}
        </span>
        <CaretDown size={10} weight="bold" />
      </button>
      {open && (
        <div className="absolute bottom-full left-0 z-20 mb-2 w-80 border border-border bg-background">
          <div className="border-b border-border px-3 py-2 font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground">
            Per-run sources
          </div>
          <div className="max-h-80 divide-y divide-border overflow-auto">
            {sources.length === 0 && (
              <div className="px-3 py-3 text-[12.5px] text-muted-foreground">
                No data sources enabled in Settings.
              </div>
            )}
            {sources.map((src) => {
              const isSelected = selected.has(src.id);
              const missingKey = src.requires_key && !src.has_key;
              return (
                <button
                  key={src.id}
                  type="button"
                  disabled={missingKey}
                  onClick={() => onToggle(src.id)}
                  className="flex w-full items-center justify-between gap-3 px-3 py-2 text-left hover:bg-muted/40 disabled:cursor-not-allowed disabled:opacity-50"
                >
                  <span className="min-w-0 flex-1">
                    <span className="block text-[13px]">{src.display_name}</span>
                    <span className="block font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground">
                      {src.category.replace("_", " ")}
                      {missingKey ? " · no key" : ""}
                    </span>
                  </span>
                  <span className="flex h-4 w-4 items-center justify-center border border-border">
                    {isSelected && !missingKey ? <Check size={10} weight="bold" /> : null}
                  </span>
                </button>
              );
            })}
          </div>
          <button
            type="button"
            onClick={() => {
              setState({ view: "settings" });
              setOpen(false);
            }}
            className="flex w-full items-center justify-between border-t border-border px-3 py-2.5 font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground transition-colors hover:bg-muted/40 hover:text-foreground"
          >
            <span>Manage sources</span>
            <ArrowUpRight size={12} weight="bold" />
          </button>
        </div>
      )}
    </div>
  );
}
