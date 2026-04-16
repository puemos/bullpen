import { ChartLineUp, Copy, Trash } from '@phosphor-icons/react';
import { useState } from 'react';
import { Button } from '@/components/ui/button';
import {
  deleteAnalysis,
  exportAnalysisMarkdown,
} from '@/shared/api/commands';
import { setState, useAppStore } from '@/store';
import { AnalysisBlockCard } from './AnalysisBlockCard';
import { FinalStanceView } from './FinalStanceView';
import { SourceList } from './SourceList';

interface ReportsPageProps {
  onRefresh: () => Promise<void>;
}

export function ReportsPage({ onRefresh }: ReportsPageProps) {
  const report = useAppStore(state => state.selectedReport);
  const selectedAnalysisId = useAppStore(state => state.selectedAnalysisId);
  const [copyState, setCopyState] = useState<string | null>(null);

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

  const remove = async () => {
    await deleteAnalysis(report.analysis.id);
    setState({ selectedAnalysisId: null, selectedReport: null, view: 'research' });
    await onRefresh();
  };

  const copyMarkdown = async () => {
    const markdown = await exportAnalysisMarkdown(report.analysis.id);
    await navigator.clipboard.writeText(markdown);
    setCopyState('Copied!');
    setTimeout(() => setCopyState(null), 1500);
  };

  return (
    <div className="mx-auto max-w-4xl space-y-12 p-8 pb-32">
      <div className="flex items-start justify-between border-b border-border pb-6">
        <div>
          <span className="text-xs uppercase tracking-widest text-muted-foreground">
            {report.final_stance?.stance || 'no stance'}
          </span>
          <h1 className="mt-1 text-2xl font-semibold tracking-tight">
            {report.analysis.title}
          </h1>
          <p className="mt-2 text-sm text-muted-foreground">
            {report.analysis.user_prompt}
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            className="flex items-center gap-1.5"
            onClick={copyMarkdown}
          >
            <Copy size={14} /> {copyState || 'Markdown'}
          </Button>
          <Button
            variant="destructive"
            size="sm"
            className="flex items-center gap-1.5"
            onClick={remove}
          >
            <Trash size={14} /> Delete
          </Button>
        </div>
      </div>

      {report.final_stance && <FinalStanceView stance={report.final_stance} />}

      {report.blocks.length > 0 && (
        <div className="space-y-6">
          <h3 className="border-b border-border pb-2 text-lg font-medium">Analysis</h3>
          {report.blocks.map(block => (
            <AnalysisBlockCard key={block.id} block={block} />
          ))}
        </div>
      )}

      <SourceList sources={report.sources} />
    </div>
  );
}
