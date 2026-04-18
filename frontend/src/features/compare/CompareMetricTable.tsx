import { Eyebrow, FreshnessChip, SectionHeader } from "@/components/ui/editorial";
import { formatMetricValue } from "@/features/report-viewer/MetricList";
import { cn } from "@/lib/utils";
import type { AnalysisReport, MetricSnapshot } from "@/types";
import { gridCols } from "./CompareHeader";

export function CompareMetricTable({
  reports,
  ids,
  number,
}: {
  reports: Record<string, AnalysisReport | null>;
  ids: string[];
  number: string;
}) {
  const byReport = buildMetricIndex(reports, ids);
  const sharedMetricNames = intersection(ids.map((id) => new Set(Object.keys(byReport[id] ?? {}))));
  const uniqueByReport = ids.map((id) => {
    const keys = Object.keys(byReport[id] ?? {}).filter((k) => !sharedMetricNames.has(k));
    return { id, keys };
  });

  if (sharedMetricNames.size === 0 && uniqueByReport.every((u) => u.keys.length === 0)) {
    return null;
  }

  const sortedShared = [...sharedMetricNames].sort();

  return (
    <section className="space-y-4 pb-14">
      <SectionHeader
        number={number}
        label="Metrics"
        title="Data points"
        meta={
          <span className="font-mono tabular-nums">
            {String(sortedShared.length).padStart(2, "0")} shared
          </span>
        }
      />

      {sortedShared.length > 0 && (
        <div className="border-y border-border">
          <div
            className="grid border-b border-border"
            style={gridCols(ids.length + 1)}
          >
            <div className="px-3 py-2">
              <Eyebrow>Metric</Eyebrow>
            </div>
            {ids.map((id, index) => {
              const entity = reports[id]?.entities?.[0];
              return (
                <div key={id} className={cn("px-3 py-2", index >= 0 && "border-l border-border")}>
                  <span className="truncate text-[12px] font-medium text-foreground">
                    {entity?.symbol || entity?.name || "—"}
                  </span>
                </div>
              );
            })}
          </div>
          {sortedShared.map((name) => (
            <div
              key={name}
              className="grid border-b border-border last:border-b-0"
              style={gridCols(ids.length + 1)}
            >
              <div className="flex items-center px-3 py-3">
                <span className="text-[13px] text-foreground">{name.replace(/_/g, " ")}</span>
              </div>
              {ids.map((id, index) => (
                <MetricCell
                  key={id}
                  metric={byReport[id]?.[name] ?? null}
                  bordered={index >= 0}
                />
              ))}
            </div>
          ))}
        </div>
      )}

      {uniqueByReport.some((u) => u.keys.length > 0) && (
        <div className="mt-2 space-y-3">
          <Eyebrow>Unique metrics</Eyebrow>
          <div className="divide-y divide-border border-y border-border">
            {uniqueByReport.map(({ id, keys }) => {
              if (keys.length === 0) return null;
              const entity = reports[id]?.entities?.[0];
              return (
                <div key={id} className="px-3 py-3">
                  <div className="mb-2 flex items-baseline justify-between">
                    <span className="text-[12.5px] font-medium text-foreground">
                      {entity?.symbol || entity?.name || id}
                    </span>
                    <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground">
                      {String(keys.length).padStart(2, "0")} only here
                    </span>
                  </div>
                  <ul className="flex flex-wrap gap-x-3 gap-y-1 text-[12.5px] text-muted-foreground">
                    {keys.sort().map((key) => (
                      <li key={key}>{key.replace(/_/g, " ")}</li>
                    ))}
                  </ul>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </section>
  );
}

function MetricCell({
  metric,
  bordered,
}: {
  metric: MetricSnapshot | null;
  bordered: boolean;
}) {
  if (!metric) {
    return (
      <div
        className={cn(
          "flex items-center px-3 py-3 text-muted-foreground/50",
          bordered && "border-l border-border",
        )}
      >
        <span className="font-mono text-[11px] tabular-nums">—</span>
      </div>
    );
  }
  const { value, suffix } = formatMetricValue(metric.numeric_value, metric.unit);
  return (
    <div
      className={cn(
        "flex flex-col gap-1 px-3 py-3",
        bordered && "border-l border-border",
      )}
    >
      <div className="flex items-baseline gap-2">
        <span className="font-mono text-[14px] tabular-nums text-foreground">{value}</span>
        {suffix && (
          <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground">
            {suffix}
          </span>
        )}
      </div>
      <FreshnessChip iso={metric.as_of} role="metric" />
    </div>
  );
}

function buildMetricIndex(
  reports: Record<string, AnalysisReport | null>,
  ids: string[],
): Record<string, Record<string, MetricSnapshot>> {
  const out: Record<string, Record<string, MetricSnapshot>> = {};
  for (const id of ids) {
    const report = reports[id];
    if (!report) {
      out[id] = {};
      continue;
    }
    const map: Record<string, MetricSnapshot> = {};
    for (const metric of report.metrics) {
      // Last metric with a given name wins; acceptable for v1 since the
      // ordering is stable per report.
      map[metric.metric] = metric;
    }
    out[id] = map;
  }
  return out;
}

function intersection(sets: Set<string>[]): Set<string> {
  if (sets.length === 0) return new Set();
  const [first, ...rest] = sets;
  const out = new Set<string>();
  for (const key of first) {
    if (rest.every((s) => s.has(key))) out.add(key);
  }
  return out;
}
