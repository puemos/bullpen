import { Eyebrow } from "@/components/ui/editorial";
import { getStanceAccent } from "@/features/report-viewer/badge-styles";
import { cn } from "@/lib/utils";
import type { AnalysisReport } from "@/types";

export function CompareHeader({
  reports,
  ids,
}: {
  reports: Record<string, AnalysisReport | null>;
  ids: string[];
}) {
  return (
    <div className="grid border-y border-border" style={gridCols(ids.length)}>
      {ids.map((id, index) => {
        const report = reports[id];
        const entity = report?.entities?.[0];
        const stance = report?.final_stance;
        const accent = getStanceAccent(stance?.stance ?? "insufficient_data");
        return (
          <div
            key={id}
            className={cn("flex flex-col gap-2 px-4 py-5", index > 0 && "border-l border-border")}
          >
            <div className="flex items-baseline gap-2">
              <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground">
                {String(index + 1).padStart(2, "0")}
              </span>
              <Eyebrow>{entity?.asset_type ?? "analysis"}</Eyebrow>
            </div>
            <div className="truncate text-[18px] font-semibold leading-tight tracking-tight text-foreground">
              {entity?.symbol || entity?.name || report?.analysis.title || "—"}
            </div>
            <div className="flex items-center gap-2">
              <span className={cn("h-1.5 w-1.5 rounded-full", accent.dot)} aria-hidden />
              <span className={cn("text-[12px] font-medium uppercase tracking-[0.14em]", accent.text)}>
                {accent.label}
              </span>
              {stance && (
                <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground">
                  {Math.round(stance.confidence * 100)}%
                </span>
              )}
            </div>
            {stance?.horizon && (
              <span className="font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground">
                {stance.horizon}
              </span>
            )}
          </div>
        );
      })}
    </div>
  );
}

export function gridCols(count: number): { gridTemplateColumns: string } {
  return { gridTemplateColumns: `repeat(${count}, minmax(0, 1fr))` };
}
