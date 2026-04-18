import { Eyebrow, SectionHeader } from "@/components/ui/editorial";
import { formatMetricValue } from "@/features/report-viewer/MetricList";
import { cn } from "@/lib/utils";
import type { AnalysisReport, ScenarioLabel } from "@/types";
import { gridCols, primaryEntityFor, projectionFor } from "./CompareHeader";

const LABELS: ScenarioLabel[] = ["bull", "base", "bear"];

export function CompareProjectionGrid({
  reports,
  ids,
  number,
}: {
  reports: Record<string, AnalysisReport | null>;
  ids: string[];
  number: string;
}) {
  const anyProjection = ids.some((id) => (reports[id]?.projections?.length ?? 0) > 0);
  if (!anyProjection) return null;

  return (
    <section className="space-y-4 pb-14">
      <SectionHeader number={number} label="Projection" title="Forward view" />
      <div className="border-y border-border">
        <div
          className="grid border-b border-border bg-background"
          style={gridCols(ids.length + 1)}
        >
          <div className="px-3 py-2">
            <Eyebrow>Scenario</Eyebrow>
          </div>
          {ids.map((id, index) => {
            const entity = primaryEntityFor(reports[id]);
            return (
              <div
                key={id}
                className={cn("px-3 py-2", index >= 0 && "border-l border-border")}
              >
                <span className="truncate text-[12px] font-medium text-foreground">
                  {entity?.symbol || entity?.name || "—"}
                </span>
              </div>
            );
          })}
        </div>
        {LABELS.map((label) => (
          <div
            key={label}
            className="grid border-b border-border last:border-b-0"
            style={gridCols(ids.length + 1)}
          >
            <div className="flex items-center px-3 py-3">
              <span className="font-mono text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                {label}
              </span>
            </div>
            {ids.map((id, index) => {
              const report = reports[id];
              const entity = primaryEntityFor(report);
              const projection = projectionFor(report, entity?.id);
              const scenario = projection?.scenarios.find((s) => s.label === label);
              if (!scenario || !projection) {
                return (
                  <div
                    key={id}
                    className={cn(
                      "flex items-center px-3 py-3 text-muted-foreground/60",
                      index >= 0 && "border-l border-border",
                    )}
                  >
                    <span className="font-mono text-[11px] tabular-nums">—</span>
                  </div>
                );
              }
              const { value, suffix } = formatMetricValue(scenario.target_value, projection.unit);
              // Compute the pct live from target/current so the display
              // doesn't depend on the (historically inconsistent) stored
              // `upside_pct`. Unit-bug insurance.
              const pct = derivedUpsidePct(scenario.target_value, projection.current_value);
              return (
                <div
                  key={id}
                  className={cn(
                    "flex flex-col gap-1 px-3 py-3",
                    index >= 0 && "border-l border-border",
                  )}
                >
                  <div className="flex items-baseline gap-2">
                    <span className="font-mono text-[14px] tabular-nums text-foreground">
                      {value}
                    </span>
                    {suffix && (
                      <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground">
                        {suffix}
                      </span>
                    )}
                  </div>
                  <div className="flex items-center gap-3">
                    {pct !== null && (
                      <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground">
                        {formatPctFraction(pct, true)}
                      </span>
                    )}
                    <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground">
                      p={Math.round(scenario.probability * 100)}%
                    </span>
                  </div>
                </div>
              );
            })}
          </div>
        ))}
      </div>
    </section>
  );
}

function derivedUpsidePct(target: number, current: number): number | null {
  if (!Number.isFinite(current) || Math.abs(current) < 1e-9) return null;
  return (target - current) / current;
}

function formatPctFraction(fraction: number, signed: boolean): string {
  const rounded = Math.round(fraction * 1000) / 10;
  const body = `${Math.abs(rounded).toFixed(1)}%`;
  if (!signed) return body;
  return rounded > 0 ? `+${body}` : rounded < 0 ? `-${body}` : body;
}
