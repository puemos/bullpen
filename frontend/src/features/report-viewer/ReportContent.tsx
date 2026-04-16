import { ChartLineUp } from '@phosphor-icons/react';
import { useMemo } from 'react';
import {
  getAnalysisReport,
  setActiveRun,
} from '@/shared/api/commands';
import { setState, useAppStore } from '@/store';
import type { Entity, Source } from '@/types';
import { AnalysisSection } from './AnalysisSection';
import { ArgumentSpine } from './ArgumentSpine';
import { Eyebrow, SectionHeader } from '@/components/ui/editorial';
import { MetricList } from './MetricList';
import { ProjectionView } from './ProjectionView';
import { ReportHero } from './ReportHero';
import { SourceList } from './SourceList';
import { StructuredArtifactView } from './StructuredArtifactView';

export function ReportContent() {
  const report = useAppStore(state => state.selectedReport);
  const selectedAnalysisId = useAppStore(state => state.selectedAnalysisId);
  const sourceMap = useMemo(
    () =>
      report
        ? new Map<string, Source>(report.sources.map(s => [s.id, s]))
        : new Map<string, Source>(),
    [report?.sources],
  );
  const entityMap = useMemo(
    () =>
      report
        ? new Map<string, Entity>(report.entities.map(e => [e.id, e]))
        : new Map<string, Entity>(),
    [report?.entities],
  );

  if (!selectedAnalysisId) {
    return (
      <div className="flex h-full flex-col items-center justify-center text-muted-foreground">
        <ChartLineUp size={32} className="mb-4 opacity-20" />
        <p>No report selected.</p>
      </div>
    );
  }

  if (!report) {
    return (
      <div className="flex h-full items-center justify-center text-sm">
        Loading report...
      </div>
    );
  }

  const switchRun = async (runId: string) => {
    await setActiveRun(report.analysis.id, runId);
    const updated = await getAnalysisReport(report.analysis.id, runId);
    setState({ selectedReport: updated });
  };

  const plan = report.research_plan;
  const hasProjections = report.projections.length > 0;
  const hasMetrics = report.metrics.length > 0;
  const hasEvidence = report.artifacts.length > 0;
  const hasAnalysis = report.blocks.length > 0;
  const hasSources = report.sources.length > 0;

  const sectionFlags = {
    hasProjections,
    hasMetrics,
    hasEvidence,
    hasAnalysis,
    hasSources,
  };

  return (
    <article className="mx-auto max-w-5xl px-8 pb-32">
      <div className="pt-10 pb-14">
        <ReportHero report={report} onSwitchRun={switchRun} />
      </div>

      {report.final_stance && (
        <section className="pb-14">
          <ArgumentSpine stance={report.final_stance} />
        </section>
      )}

      {plan?.decision_criteria?.length ? (
        <section className="pb-12">
          <DecisionCriteria criteria={plan.decision_criteria} />
        </section>
      ) : null}

      {(hasProjections || hasMetrics || hasEvidence || hasAnalysis || hasSources) && (
        <SectionJumpNav {...sectionFlags} />
      )}

      {hasProjections && (
        <section className="space-y-8 pb-16">
          <SectionHeader
            number={sectionNumber(sectionFlags, 'projections')}
            label="Projection"
            title="Forward view"
            meta={<span className="font-mono tabular-nums">{report.projections.length.toString().padStart(2, '0')} {report.projections.length === 1 ? 'target' : 'targets'}</span>}
            id="projections"
          />
          <ProjectionView
            projections={report.projections}
            entityMap={entityMap}
            sourceMap={sourceMap}
          />
        </section>
      )}

      {hasMetrics && (
        <section className="space-y-8 pb-16">
          <SectionHeader
            number={sectionNumber(sectionFlags, 'metrics')}
            label="Metrics"
            title="Data points"
            meta={<span className="font-mono tabular-nums">{report.metrics.length.toString().padStart(2, '0')} tracked</span>}
            id="metrics"
          />
          <MetricList
            metrics={report.metrics}
            entityMap={entityMap}
            sourceMap={sourceMap}
          />
        </section>
      )}

      {hasEvidence && (
        <section className="space-y-2 pb-16">
          <SectionHeader
            number={sectionNumber(sectionFlags, 'evidence')}
            label="Evidence"
            title="Structured evidence"
            meta={<span className="font-mono tabular-nums">{report.artifacts.length.toString().padStart(2, '0')} artifacts</span>}
            id="evidence"
          />
          <div>
            {report.artifacts.map((artifact, index) => (
              <StructuredArtifactView
                key={artifact.id}
                artifact={artifact}
                isFirst={index === 0}
              />
            ))}
          </div>
        </section>
      )}

      {hasAnalysis && (
        <section className="space-y-8 pb-16">
          <SectionHeader
            number={sectionNumber(sectionFlags, 'analysis')}
            label="Analysis"
            title="The deeper read"
            meta={<span className="font-mono tabular-nums">{report.blocks.length.toString().padStart(2, '0')} blocks</span>}
            id="analysis"
          />
          <AnalysisSection blocks={report.blocks} sourceMap={sourceMap} />
        </section>
      )}

      {hasSources && (
        <section className="space-y-8 pb-16">
          <SectionHeader
            number={sectionNumber(sectionFlags, 'sources')}
            label="Sources"
            title="Bibliography"
            meta={<span className="font-mono tabular-nums">{report.sources.length.toString().padStart(2, '0')} cited</span>}
            id="sources"
          />
          <SourceList sources={report.sources} />
        </section>
      )}
    </article>
  );
}

type SectionFlags = {
  hasProjections: boolean;
  hasMetrics: boolean;
  hasEvidence: boolean;
  hasAnalysis: boolean;
  hasSources: boolean;
};

type SectionKey = 'projections' | 'metrics' | 'evidence' | 'analysis' | 'sources';

function sectionNumber(flags: SectionFlags, which: SectionKey): string {
  const order: SectionKey[] = ['projections', 'metrics', 'evidence', 'analysis', 'sources'];
  const present = new Set<SectionKey>();
  if (flags.hasProjections) present.add('projections');
  if (flags.hasMetrics) present.add('metrics');
  if (flags.hasEvidence) present.add('evidence');
  if (flags.hasAnalysis) present.add('analysis');
  if (flags.hasSources) present.add('sources');
  const seq = order.filter(key => present.has(key));
  const idx = seq.indexOf(which);
  return String(idx + 1).padStart(2, '0');
}

function DecisionCriteria({ criteria }: { criteria: string[] }) {
  return (
    <div className="flex flex-col gap-3 border-t border-border pt-5">
      <Eyebrow>Decision criteria</Eyebrow>
      <ol className="divide-y divide-border/60 text-[13.5px] text-foreground/85">
        {criteria.map((criterion, index) => (
          <li
            key={criterion}
            className="flex items-baseline gap-3 py-2 first:pt-0 last:pb-0"
          >
            <span className="shrink-0 font-mono text-[10.5px] tabular-nums text-muted-foreground">
              {String(index + 1).padStart(2, '0')}
            </span>
            <span className="leading-[1.55]">
              {criterion.replace(/^\s*\d+[.)]\s+/, '')}
            </span>
          </li>
        ))}
      </ol>
    </div>
  );
}

function SectionJumpNav({
  hasProjections,
  hasMetrics,
  hasEvidence,
  hasAnalysis,
  hasSources,
}: SectionFlags) {
  const items: { href: string; label: string }[] = [];
  if (hasProjections) items.push({ href: '#projections', label: 'Projection' });
  if (hasMetrics) items.push({ href: '#metrics', label: 'Metrics' });
  if (hasEvidence) items.push({ href: '#evidence', label: 'Evidence' });
  if (hasAnalysis) items.push({ href: '#analysis', label: 'Analysis' });
  if (hasSources) items.push({ href: '#sources', label: 'Sources' });

  return (
    <nav className="sticky top-0 z-20 -mx-8 mb-8 border-y border-border bg-background/90 px-8 py-3 backdrop-blur">
      <div className="flex items-center gap-6">
        <Eyebrow>Contents</Eyebrow>
        <div className="flex items-center gap-5 text-[12.5px]">
          {items.map(item => (
            <a
              key={item.href}
              href={item.href}
              className="text-muted-foreground transition-colors hover:text-foreground"
            >
              {item.label}
            </a>
          ))}
        </div>
      </div>
    </nav>
  );
}
