import { memo } from "react";
import { Eyebrow } from "@/components/ui/editorial";
import { cn } from "@/lib/utils";
import type { Entity, Projection, ProjectionScenario, Source } from "@/types";
import { ConfidenceRail } from "./badge-styles";

interface ProjectionViewProps {
  projections: Projection[];
  entityMap: Map<string, Entity>;
  sourceMap: Map<string, Source>;
}

export const ProjectionView = memo(function ProjectionView({
  projections,
  entityMap,
  sourceMap,
}: ProjectionViewProps) {
  if (projections.length === 0) return null;

  return (
    <div className="space-y-14">
      {projections.map((projection, index) => (
        <ProjectionCard
          key={projection.id}
          projection={projection}
          entity={entityMap.get(projection.entity_id) ?? null}
          sourceMap={sourceMap}
          isFirst={index === 0}
        />
      ))}
    </div>
  );
});

function ProjectionCard({
  projection,
  entity,
  sourceMap,
  isFirst,
}: {
  projection: Projection;
  entity: Entity | null;
  sourceMap: Map<string, Source>;
  isFirst: boolean;
}) {
  const orderedScenarios = orderScenarios(projection.scenarios);
  const label = entity?.symbol || entity?.name || projection.entity_id;

  return (
    <article className={cn("space-y-10", !isFirst && "border-t border-border pt-12")}>
      <header className="grid gap-8 lg:grid-cols-[minmax(0,1fr)_260px] lg:gap-14">
        <div className="space-y-4">
          <div className="flex flex-wrap items-center gap-x-3 gap-y-2">
            <Eyebrow>Projection</Eyebrow>
            <Dot />
            <Eyebrow>{projection.horizon}</Eyebrow>
            <Dot />
            <Eyebrow>{formatMetric(projection.metric)}</Eyebrow>
          </div>
          <h3 className="text-[32px] font-semibold leading-[1.05] tracking-[-0.02em] sm:text-[40px]">
            {label}
            <span className="pl-4 font-mono text-[0.6em] tabular-nums text-muted-foreground">
              {projection.current_value_label}
            </span>
          </h3>
          <p className="max-w-[46em] text-[14.5px] leading-[1.65] text-foreground/85">
            <span className="text-muted-foreground">Methodology · </span>
            {projection.methodology}
          </p>
        </div>
        <aside className="space-y-2 lg:border-l lg:border-border lg:pl-6">
          <Eyebrow>Confidence</Eyebrow>
          <ConfidenceRail confidence={projection.confidence} accentClass="bg-foreground/70" />
        </aside>
      </header>

      <ProjectionGauge
        scenarios={orderedScenarios}
        currentValue={projection.current_value}
        currentLabel={projection.current_value_label}
      />

      <ProbabilityBar scenarios={orderedScenarios} />

      <div className="grid gap-0 md:grid-cols-3 md:gap-0 md:divide-x md:divide-border">
        {orderedScenarios.map((scenario, i) => (
          <ScenarioColumn
            key={`${scenario.label}-${i}`}
            scenario={scenario}
            currentValue={projection.current_value}
            indexLabel={String(i + 1).padStart(2, "0")}
          />
        ))}
      </div>

      {projection.key_assumptions.length > 0 && (
        <section className="space-y-3 border-t border-border pt-6">
          <Eyebrow>Key assumptions</Eyebrow>
          <ol className="space-y-2 text-[14px] leading-[1.6] text-foreground/90">
            {projection.key_assumptions.map((assumption, i) => (
              <li key={`${i}-${assumption.slice(0, 32)}`} className="flex gap-3">
                <span className="mt-[0.35em] font-mono text-[10.5px] tabular-nums text-muted-foreground">
                  {String(i + 1).padStart(2, "0")}
                </span>
                <span>{assumption}</span>
              </li>
            ))}
          </ol>
        </section>
      )}

      {projection.evidence_ids.length > 0 && (
        <EvidenceRow ids={projection.evidence_ids} sourceMap={sourceMap} />
      )}

      {projection.disclaimer && (
        <p className="text-[11px] leading-relaxed text-muted-foreground/80">
          {projection.disclaimer}
        </p>
      )}
    </article>
  );
}

type GaugeMarker = {
  key: string;
  x: number;
  labelTop: string;
  labelBottom: string;
  topClassName: string;
  bottomClassName: string;
  dot: { fillClassName: string } | null;
  tick: boolean;
};

function assignSlots(markers: GaugeMarker[], minGap: number): number[] {
  const sorted = markers.map((m, i) => ({ i, x: m.x })).sort((a, b) => a.x - b.x);
  const slots = new Array<number>(markers.length).fill(0);
  const lastXPerSlot: number[] = [];
  for (const { i, x } of sorted) {
    let slot = 0;
    while (slot < lastXPerSlot.length && x - lastXPerSlot[slot] < minGap) {
      slot += 1;
    }
    lastXPerSlot[slot] = x;
    slots[i] = slot;
  }
  return slots;
}

function ProjectionGauge({
  scenarios,
  currentValue,
  currentLabel,
}: {
  scenarios: ProjectionScenario[];
  currentValue: number;
  currentLabel: string;
}) {
  const values = scenarios.map((s) => s.target_value).concat([currentValue]);
  const min = Math.min(...values);
  const max = Math.max(...values);
  const span = max - min || Math.max(1, Math.abs(max) * 0.1);
  const width = 880;
  const height = 116;
  const padX = 40;
  const midY = height / 2;

  const project = (value: number) => padX + ((value - min) / span) * (width - padX * 2);

  const markers: GaugeMarker[] = [
    {
      key: "now",
      x: project(currentValue),
      labelTop: "now",
      labelBottom: currentLabel,
      topClassName: "fill-muted-foreground text-[10px] font-mono uppercase tracking-[0.16em]",
      bottomClassName: "fill-foreground text-[11px] font-mono tabular-nums",
      dot: null,
      tick: true,
    },
    ...scenarios.map((scenario, i): GaugeMarker => {
      const accent = scenarioAccent(scenario.label);
      return {
        key: `${scenario.label}-${i}`,
        x: project(scenario.target_value),
        labelTop: scenario.label,
        labelBottom: scenario.target_label,
        topClassName: cn("text-[10px] font-mono uppercase tracking-[0.16em]", accent.text),
        bottomClassName: "fill-foreground text-[11px] font-mono tabular-nums",
        dot: { fillClassName: accent.fill },
        tick: false,
      };
    }),
  ];

  const MIN_GAP = 64;
  const SLOT_STEP = 18;
  const BASE_TOP_OFFSET = 14;
  const BASE_BOTTOM_OFFSET = 24;
  const slots = assignSlots(markers, MIN_GAP);

  return (
    <div className="space-y-3">
      <div className="border-y border-border py-4">
        <svg viewBox={`0 0 ${width} ${height}`} role="img" className="h-28 w-full">
          <line
            x1={padX}
            x2={width - padX}
            y1={midY}
            y2={midY}
            stroke="currentColor"
            strokeWidth="1"
            className="text-border"
          />
          {markers.map((marker, i) => {
            const slot = slots[i];
            const topY = midY - (BASE_TOP_OFFSET + slot * SLOT_STEP);
            const bottomY = midY + (BASE_BOTTOM_OFFSET + slot * SLOT_STEP);
            const tickExtent = 14 + slot * SLOT_STEP;
            return (
              <g key={marker.key}>
                {marker.tick && (
                  <line
                    x1={marker.x}
                    x2={marker.x}
                    y1={midY - tickExtent}
                    y2={midY + tickExtent}
                    stroke="currentColor"
                    strokeWidth="1.5"
                    className="text-foreground"
                  />
                )}
                {slot > 0 && !marker.tick && (
                  <>
                    <line
                      x1={marker.x}
                      x2={marker.x}
                      y1={midY - 6}
                      y2={topY + 4}
                      stroke="currentColor"
                      strokeWidth="1"
                      className="text-border"
                    />
                    <line
                      x1={marker.x}
                      x2={marker.x}
                      y1={midY + 6}
                      y2={bottomY - 10}
                      stroke="currentColor"
                      strokeWidth="1"
                      className="text-border"
                    />
                  </>
                )}
                {marker.dot && (
                  <circle cx={marker.x} cy={midY} r={4.5} className={marker.dot.fillClassName} />
                )}
                <text x={marker.x} y={topY} textAnchor="middle" className={marker.topClassName}>
                  {marker.labelTop}
                </text>
                <text
                  x={marker.x}
                  y={bottomY}
                  textAnchor="middle"
                  className={marker.bottomClassName}
                >
                  {marker.labelBottom}
                </text>
              </g>
            );
          })}
        </svg>
      </div>
      <div className="flex flex-wrap gap-x-4 gap-y-1 font-mono text-[11px] tabular-nums text-muted-foreground">
        {scenarios.map((scenario, i) => (
          <span key={`${scenario.label}-${i}`}>
            {scenario.label} · {scenario.target_label} (
            {formatUpside(currentValue, scenario.target_value)})
          </span>
        ))}
      </div>
    </div>
  );
}

function ProbabilityBar({ scenarios }: { scenarios: ProjectionScenario[] }) {
  const total = scenarios.reduce((sum, s) => sum + s.probability, 0) || 1;
  return (
    <div className="space-y-2">
      <div className="flex items-baseline justify-between">
        <Eyebrow>Probability weight</Eyebrow>
        <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground">
          {Math.round(total * 100)}%
        </span>
      </div>
      <div className="flex h-[6px] w-full overflow-hidden bg-border/60">
        {scenarios.map((scenario, i) => {
          const pct = (scenario.probability / total) * 100;
          const accent = scenarioAccent(scenario.label);
          return (
            <div
              key={`${scenario.label}-${i}`}
              className={cn("h-full", accent.bar)}
              style={{ width: `${pct}%` }}
            />
          );
        })}
      </div>
      <div className="flex flex-wrap gap-x-4 gap-y-1 font-mono text-[11px] tabular-nums text-muted-foreground">
        {scenarios.map((scenario, i) => (
          <span key={`${scenario.label}-${i}`} className="inline-flex items-center gap-1.5">
            <span
              className={cn("h-1.5 w-1.5 rounded-full", scenarioAccent(scenario.label).bar)}
              aria-hidden
            />
            {scenario.label} {Math.round(scenario.probability * 100)}%
          </span>
        ))}
      </div>
    </div>
  );
}

function ScenarioColumn({
  scenario,
  currentValue,
  indexLabel,
}: {
  scenario: ProjectionScenario;
  currentValue: number;
  indexLabel: string;
}) {
  const accent = scenarioAccent(scenario.label);

  return (
    <div className="flex flex-col gap-4 px-0 py-6 md:px-6 md:first:pl-0 md:last:pr-0">
      <div className="space-y-2">
        <div className="flex items-baseline gap-2">
          <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground">
            {indexLabel}
          </span>
          <Eyebrow className={accent.text}>{scenario.label}</Eyebrow>
        </div>
        <div className="flex items-baseline gap-3">
          <span className={cn("text-2xl font-semibold tracking-tight", accent.text)}>
            {scenario.target_label}
          </span>
          <span className="font-mono text-[12px] tabular-nums text-muted-foreground">
            {formatUpside(currentValue, scenario.target_value)}
          </span>
        </div>
        <div className="font-mono text-[10.5px] tabular-nums text-muted-foreground">
          {Math.round(scenario.probability * 100)}% probability
        </div>
      </div>

      <div className={cn("h-[2px] w-10", accent.bar)} aria-hidden />

      <p className="text-[14px] leading-[1.6] text-foreground/90">{scenario.rationale}</p>

      {scenario.catalysts.length > 0 && (
        <ScenarioList label="Catalysts" items={scenario.catalysts} markerClass={accent.bar} />
      )}
      {scenario.risks.length > 0 && (
        <ScenarioList label="Risks" items={scenario.risks} markerClass="bg-muted-foreground/50" />
      )}
    </div>
  );
}

function ScenarioList({
  label,
  items,
  markerClass,
}: {
  label: string;
  items: string[];
  markerClass: string;
}) {
  return (
    <div className="space-y-2">
      <Eyebrow>{label}</Eyebrow>
      <ul className="space-y-1.5 text-[13.5px] leading-[1.55] text-foreground/85">
        {items.map((item, i) => (
          <li key={`${i}-${item.slice(0, 32)}`} className="flex gap-2.5">
            <span
              className={cn("mt-[0.65em] h-1 w-1 shrink-0 rounded-full", markerClass)}
              aria-hidden
            />
            <span>{item}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}

function EvidenceRow({ ids, sourceMap }: { ids: string[]; sourceMap: Map<string, Source> }) {
  return (
    <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1.5 border-t border-border pt-4">
      <Eyebrow className="shrink-0">Evidence</Eyebrow>
      {ids.map((id, index) => {
        const source = sourceMap.get(id);
        const label = source?.title ?? id.slice(0, 8);
        const href = source?.url ?? "#";
        return (
          <a
            key={id}
            href={href}
            target="_blank"
            rel="noreferrer"
            className="inline-flex items-baseline gap-1.5 text-[12.5px] text-foreground/80 underline-offset-4 hover:underline"
          >
            <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground">
              {String(index + 1).padStart(2, "0")}
            </span>
            <span className="max-w-[24ch] truncate">{label}</span>
          </a>
        );
      })}
    </div>
  );
}

function orderScenarios(scenarios: ProjectionScenario[]): ProjectionScenario[] {
  const rank: Record<string, number> = { bear: 0, base: 1, bull: 2 };
  return [...scenarios].sort((a, b) => {
    const ra = rank[a.label.toLowerCase()] ?? 99;
    const rb = rank[b.label.toLowerCase()] ?? 99;
    return ra - rb;
  });
}

function scenarioAccent(label: string): {
  fill: string;
  text: string;
  bar: string;
} {
  switch (label.toLowerCase()) {
    case "bull":
      return {
        fill: "fill-emerald-600 dark:fill-emerald-400",
        text: "text-emerald-700 dark:text-emerald-400",
        bar: "bg-emerald-600 dark:bg-emerald-400",
      };
    case "bear":
      return {
        fill: "fill-red-600 dark:fill-red-400",
        text: "text-red-700 dark:text-red-400",
        bar: "bg-red-600 dark:bg-red-400",
      };
    default:
      return {
        fill: "fill-foreground",
        text: "text-foreground",
        bar: "bg-foreground/70",
      };
  }
}

function formatMetric(metric: string) {
  return metric.replace(/_/g, " ");
}

function formatUpside(current: number, target: number): string {
  if (!Number.isFinite(current) || Math.abs(current) < 1e-9) return "—";
  const pct = ((target - current) / current) * 100;
  const sign = pct >= 0 ? "+" : "";
  return `${sign}${pct.toFixed(1)}%`;
}

function Dot() {
  return <span className="h-1 w-1 rounded-full bg-border" aria-hidden />;
}
