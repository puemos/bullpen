import { Eyebrow } from "@/components/ui/editorial";
import { useAppStore } from "@/store";

/**
 * Placeholder — the full side-by-side layout lands with CompareHeader,
 * CompareProjectionGrid, CompareMetricTable, CompareSourceLedger in step 4.
 */
export function CompareView() {
  const ids = useAppStore((s) => s.compareAnalysisIds);
  const reports = useAppStore((s) => s.compareReports);
  return (
    <article className="mx-auto max-w-5xl px-8 pb-32 pt-10">
      <Eyebrow>Compare</Eyebrow>
      <h2 className="mt-3 text-2xl font-semibold leading-tight tracking-tight">
        {ids.length} analyses queued
      </h2>
      <p className="mt-4 max-w-[62ch] text-[14px] leading-[1.55] text-muted-foreground">
        The cross-report compare surface lands next. Selected:
      </p>
      <ul className="mt-4 divide-y divide-border border-y border-border">
        {ids.map((id) => {
          const report = reports[id];
          return (
            <li key={id} className="flex items-baseline justify-between gap-4 px-1 py-3">
              <span className="font-mono text-[12px] tabular-nums text-foreground">{id}</span>
              <span className="font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground">
                {report ? "loaded" : "loading…"}
              </span>
            </li>
          );
        })}
      </ul>
    </article>
  );
}
