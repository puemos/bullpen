import { Eyebrow } from "@/components/ui/editorial";
import { getStanceAccent } from "@/features/report-viewer/badge-styles";
import { cn } from "@/lib/utils";
import type { AnalysisReport, Entity, Projection } from "@/types";

/**
 * Pick the entity this column represents. Prefer the entity tied to the
 * first projection (in a compare-equities report each entity has its own
 * projection); fall back to entities[0] for single-equity reports.
 */
export function primaryEntityFor(report: AnalysisReport | null | undefined): Entity | undefined {
  if (!report) return undefined;
  const firstProjection = report.projections[0];
  if (firstProjection) {
    const match = report.entities.find((e) => e.id === firstProjection.entity_id);
    if (match) return match;
  }
  return report.entities[0];
}

/**
 * The projection whose entity matches the column's header entity. Prevents
 * the compare grid from crossing wires when a report carries multiple
 * entities + multiple projections (e.g. a compare_equities analysis).
 */
export function projectionFor(
  report: AnalysisReport | null | undefined,
  entityId: string | undefined,
): Projection | undefined {
  if (!report) return undefined;
  if (entityId) {
    const match = report.projections.find((p) => p.entity_id === entityId);
    if (match) return match;
  }
  return report.projections[0];
}

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
        const entity = primaryEntityFor(report);
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
              <Eyebrow>{entity?.asset_type ?? report?.analysis.intent ?? "analysis"}</Eyebrow>
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
