import { ChartLine, ChartLineUp, Newspaper, Table as TableIcon } from '@phosphor-icons/react';
import { useMemo } from 'react';
import { Badge } from '@/components/ui/badge';
import {
  Card,
  CardContent,
  CardHeader,
} from '@/components/ui/card';
import { Separator } from '@/components/ui/separator';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  getAnalysisReport,
  setActiveRun,
} from '@/shared/api/commands';
import { setState, useAppStore } from '@/store';
import type { Source } from '@/types';
import { ConfidenceBadge, getStanceClasses } from './badge-styles';
import { AnalysisBlockCard } from './AnalysisBlockCard';
import { FinalStanceView } from './FinalStanceView';
import { SourceList } from './SourceList';
import { StructuredArtifactView } from './StructuredArtifactView';

export function ReportContent() {
  const report = useAppStore(state => state.selectedReport);
  const selectedAnalysisId = useAppStore(state => state.selectedAnalysisId);
  const sourceMap = useMemo(
    () => report ? new Map<string, Source>(report.sources.map(s => [s.id, s])) : new Map<string, Source>(),
    [report?.sources],
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
    return <div className="flex h-full items-center justify-center text-sm">Loading report...</div>;
  }

  const switchRun = async (runId: string) => {
    await setActiveRun(report.analysis.id, runId);
    const updated = await getAnalysisReport(report.analysis.id, runId);
    setState({ selectedReport: updated });
  };

  const hasMultipleRuns = report.runs.length > 1;
  const activeRunId = report.analysis.active_run_id;
  const plan = report.research_plan;

  return (
    <div className="mx-auto max-w-5xl space-y-8 p-8 pb-32">
      <Card>
        <CardHeader className="gap-0">
          <div className="flex flex-wrap gap-2">
            <Badge className={getStanceClasses(report.final_stance?.stance || '')}>
              {formatLabel(report.final_stance?.stance || 'no stance')}
            </Badge>
            <Badge variant="secondary">{formatLabel(report.analysis.intent)}</Badge>
            {report.final_stance && (
              <ConfidenceBadge confidence={report.final_stance.confidence} />
            )}
          </div>
        </CardHeader>
        <CardContent className="space-y-5">
          <Separator />
          <div className="grid gap-4 text-sm sm:grid-cols-3">
            <ReportMetric label="Sources" value={report.sources.length} icon={<Newspaper size={14} />} />
            <ReportMetric label="Metrics" value={report.metrics.length} icon={<ChartLine size={14} />} />
            <ReportMetric label="Artifacts" value={report.artifacts.length} icon={<TableIcon size={14} />} />
          </div>
          {plan?.decision_criteria?.length ? (
            <div className="space-y-3">
              <h3 className="text-sm font-medium">Decision Criteria</h3>
              <div className="flex flex-wrap gap-2">
                {plan.decision_criteria.map(criteria => (
                  <Badge key={criteria} variant="outline">
                    {criteria}
                  </Badge>
                ))}
              </div>
            </div>
          ) : null}
        </CardContent>
      </Card>

      {hasMultipleRuns && (
        <Tabs value={activeRunId ?? undefined} onValueChange={switchRun} className="gap-0">
          <TabsList>
            {report.runs.map(run => (
              <TabsTrigger
                key={run.id}
                value={run.id}
                className="flex-none px-3 text-xs"
              >
                {run.agent_id}
                <span className="ml-1.5 text-[10px] text-muted-foreground">
                  ({run.status})
                </span>
              </TabsTrigger>
            ))}
          </TabsList>
        </Tabs>
      )}

      {report.final_stance && <FinalStanceView stance={report.final_stance} />}

      {report.artifacts.length > 0 && (
        <section className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-medium">Structured Evidence</h3>
            <Badge variant="secondary">{report.artifacts.length} artifacts</Badge>
          </div>
          <div className="space-y-4">
            {report.artifacts.map(artifact => (
              <StructuredArtifactView key={artifact.id} artifact={artifact} />
            ))}
          </div>
        </section>
      )}

      {report.blocks.length > 0 && (
        <section className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-medium">Analysis</h3>
            <Badge variant="secondary">{report.blocks.length} blocks</Badge>
          </div>
          {report.blocks.map(block => (
            <AnalysisBlockCard key={block.id} block={block} sourceMap={sourceMap} />
          ))}
        </section>
      )}

      <SourceList sources={report.sources} />
    </div>
  );
}

function ReportMetric({ label, value, icon }: { label: string; value: number; icon: React.ReactNode }) {
  return (
    <div className="rounded-md border bg-muted/20 px-4 py-3">
      <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
        {icon}
        {label}
      </div>
      <div className="mt-1 text-xl font-semibold tabular-nums">{value}</div>
    </div>
  );
}

function formatLabel(value: string) {
  return value.replace(/_/g, ' ');
}
