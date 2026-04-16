import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import type { AnalysisBlock } from '@/types';

interface AnalysisBlockCardProps {
  block: AnalysisBlock;
}

export function AnalysisBlockCard({ block }: AnalysisBlockCardProps) {
  return (
    <div className="border border-border bg-card p-6 rounded-none">
      <div className="mb-4 flex items-center justify-between text-xs tracking-wider text-muted-foreground">
        <span className="uppercase">{block.kind.replace('_', ' ')}</span>
        <span className="bg-primary/10 px-2 py-0.5 text-primary">
          {Math.round(block.confidence * 100)}% conf
        </span>
      </div>
      <h4 className="mb-3 text-base font-semibold">{block.title}</h4>
      <div className="prose prose-sm max-w-none text-sm text-foreground prose-a:text-primary">
        <ReactMarkdown remarkPlugins={[remarkGfm]}>{block.body}</ReactMarkdown>
      </div>
    </div>
  );
}
