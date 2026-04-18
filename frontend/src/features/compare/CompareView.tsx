import { useState } from "react";
import { Eyebrow } from "@/components/ui/editorial";
import { setCompareMode, setState, useAppStore } from "@/store";
import type { AnalysisReport } from "@/types";
import { CompareHeader } from "./CompareHeader";
import { CompareMetricTable } from "./CompareMetricTable";
import { CompareProjectionGrid } from "./CompareProjectionGrid";
import { CompareSourceLedger } from "./CompareSourceLedger";

const CLIP_THESIS_CHARS = 280;

export function CompareView() {
  const ids = useAppStore((s) => s.compareAnalysisIds);
  const reports = useAppStore((s) => s.compareReports);

  if (ids.length === 0) {
    return (
      <div className="flex h-full flex-col items-center justify-center text-muted-foreground">
        <p>No compare selection.</p>
      </div>
    );
  }

  const loaded = ids.every((id) => reports[id] !== undefined);

  const exitCompare = () => {
    setCompareMode(false);
    setState({ view: "new-analysis" });
  };

  return (
    <article className="mx-auto max-w-5xl px-8 pb-32 pt-10">
      <div className="flex items-baseline justify-between gap-4 pb-6">
        <div className="flex flex-col gap-1">
          <Eyebrow>Compare</Eyebrow>
          <h2 className="text-[28px] font-semibold leading-tight tracking-tight text-foreground">
            Side-by-side
          </h2>
        </div>
        <button
          type="button"
          onClick={exitCompare}
          className="font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground transition-colors hover:text-foreground"
        >
          Exit →
        </button>
      </div>

      <CompareHeader reports={reports} ids={ids} />

      <ThesisRow reports={reports} ids={ids} />

      {loaded ? (
        <>
          <CompareProjectionGrid reports={reports} ids={ids} number="01" />
          <CompareMetricTable reports={reports} ids={ids} number="02" />
          <CompareSourceLedger reports={reports} ids={ids} number="03" />
        </>
      ) : (
        <section className="py-16 text-center text-sm text-muted-foreground">
          Loading reports…
        </section>
      )}
    </article>
  );
}

function ThesisRow({
  reports,
  ids,
}: {
  reports: Record<string, AnalysisReport | null>;
  ids: string[];
}) {
  return (
    <div
      className="grid border-b border-border"
      style={{ gridTemplateColumns: `repeat(${ids.length}, minmax(0, 1fr))` }}
    >
      {ids.map((id, index) => {
        const thesis = reports[id]?.blocks.find((b) => b.kind === "thesis");
        return (
          <ThesisCell
            key={id}
            body={thesis?.body ?? ""}
            firstColumn={index === 0}
          />
        );
      })}
    </div>
  );
}

function ThesisCell({ body, firstColumn }: { body: string; firstColumn: boolean }) {
  const [expanded, setExpanded] = useState(false);
  const clippable = body.length > CLIP_THESIS_CHARS;
  const shown = !clippable || expanded ? body : `${body.slice(0, CLIP_THESIS_CHARS)}…`;
  return (
    <div className={firstColumn ? "px-4 py-5" : "border-l border-border px-4 py-5"}>
      <Eyebrow>Thesis</Eyebrow>
      <p className="mt-2 whitespace-pre-wrap text-[13.5px] leading-[1.55] text-foreground/85">
        {shown || <span className="text-muted-foreground/60">— no thesis block —</span>}
      </p>
      {clippable && (
        <button
          type="button"
          onClick={() => setExpanded((v) => !v)}
          className="mt-2 font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground transition-colors hover:text-foreground"
        >
          {expanded ? "Collapse ↑" : "Read more ↓"}
        </button>
      )}
    </div>
  );
}
