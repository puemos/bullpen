import { memo } from "react";
import { Dot, Eyebrow } from "@/components/ui/editorial";
import type { AnalysisReport, FinalStance } from "@/types";
import { ConfidenceRail, getStanceAccent } from "./badge-styles";

interface ReportHeroProps {
  report: AnalysisReport;
  onSwitchRun: (runId: string) => void;
}

export const ReportHero = memo(function ReportHero({ report, onSwitchRun }: ReportHeroProps) {
  const stance = report.final_stance;
  const accent = getStanceAccent(stance?.stance ?? "");
  const asOf = formatDate(stance?.created_at ?? report.analysis.updated_at);
  const horizon = stance?.horizon || "—";
  const activeRunId = report.analysis.active_run_id;

  return (
    <header className="space-y-10">
      <div className="flex flex-wrap items-center gap-x-3 gap-y-2">
        <Eyebrow>Final stance</Eyebrow>
        <Dot />
        <Eyebrow>{horizon}</Eyebrow>
        <Dot />
        <Eyebrow>As of {asOf}</Eyebrow>
      </div>

      <div className="grid gap-10 lg:grid-cols-[minmax(0,1fr)_280px] lg:gap-16">
        <div className="space-y-8">
          <StanceHeadline accent={accent} stance={stance} />

          {stance?.summary && (
            <p className="max-w-[34em] text-xl font-normal leading-[1.45] tracking-[-0.005em] text-foreground">
              {stance.summary}
            </p>
          )}
        </div>

        <aside className="space-y-6 lg:border-l lg:border-border lg:pl-8">
          <div className="space-y-2">
            <Eyebrow>Confidence</Eyebrow>
            <ConfidenceRail confidence={stance?.confidence ?? 0} accentClass={accent.rule} />
          </div>

          {report.runs.length > 1 && activeRunId && (
            <div className="space-y-2">
              <Eyebrow>Run</Eyebrow>
              <RunSwitcher runs={report.runs} activeRunId={activeRunId} onSwitch={onSwitchRun} />
            </div>
          )}

          {stance?.disclaimer && (
            <p className="text-[11px] leading-relaxed text-muted-foreground/80">
              {stance.disclaimer}
            </p>
          )}
        </aside>
      </div>

      <StatFooter
        sources={report.sources.length}
        artifacts={report.artifacts.length}
        blocks={report.blocks.length}
        metrics={report.metrics.length}
        entities={report.entities.length}
        projections={report.projections.length}
      />
    </header>
  );
});

function StanceHeadline({
  accent,
  stance,
}: {
  accent: ReturnType<typeof getStanceAccent>;
  stance: FinalStance | null;
}) {
  return (
    <div className="relative pl-6 sm:pl-8">
      <span className={`absolute left-0 top-1 bottom-1 w-[3px] ${accent.tick}`} aria-hidden />
      <div
        className={`text-[64px] font-semibold uppercase leading-[0.95] tracking-[-0.035em] sm:text-[84px] ${accent.text}`}
      >
        {(stance?.stance || "unknown").replace(/_/g, " ")}
      </div>
      {stance?.stance === "insufficient_data" && (
        <p className="mt-3 text-sm text-muted-foreground">
          Not enough reliable evidence to take a position.
        </p>
      )}
    </div>
  );
}

function StatFooter({
  sources,
  artifacts,
  blocks,
  metrics,
  entities,
  projections,
}: {
  sources: number;
  artifacts: number;
  blocks: number;
  metrics: number;
  entities: number;
  projections: number;
}) {
  return (
    <div className="border-t border-border pt-5">
      <dl className="grid grid-cols-2 gap-y-4 sm:grid-cols-3 lg:grid-cols-6">
        <Stat label="Entities" value={entities} />
        <Stat label="Sources" value={sources} />
        <Stat label="Artifacts" value={artifacts} />
        <Stat label="Analysis blocks" value={blocks} />
        <Stat label="Data points" value={metrics} />
        <Stat label="Projections" value={projections} />
      </dl>
    </div>
  );
}

function Stat({ label, value }: { label: string; value: number }) {
  return (
    <div className="flex flex-col gap-1">
      <dt>
        <Eyebrow>{label}</Eyebrow>
      </dt>
      <dd className="font-mono text-2xl font-medium tabular-nums text-foreground">{value}</dd>
    </div>
  );
}

function RunSwitcher({
  runs,
  activeRunId,
  onSwitch,
}: {
  runs: AnalysisReport["runs"];
  activeRunId: string;
  onSwitch: (runId: string) => void;
}) {
  return (
    <div className="flex flex-wrap gap-1">
      {runs.map((run) => {
        const active = run.id === activeRunId;
        return (
          <button
            key={run.id}
            type="button"
            onClick={() => onSwitch(run.id)}
            className={
              active
                ? "border border-foreground bg-foreground px-2.5 py-1 text-[11px] font-medium text-background"
                : "border border-border bg-transparent px-2.5 py-1 text-[11px] text-muted-foreground hover:border-foreground/40 hover:text-foreground"
            }
          >
            {run.agent_id}
            {run.model_id ? ` · ${run.model_id}` : ""}
          </button>
        );
      })}
    </div>
  );
}

function formatDate(value: string): string {
  if (!value) return "—";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(date);
}
