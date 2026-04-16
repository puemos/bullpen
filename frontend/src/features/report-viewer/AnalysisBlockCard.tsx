import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import type { AnalysisBlock, Source } from '@/types';
import { ConfidenceBadge, getImportanceClasses } from './badge-styles';
import { reportMarkdownComponents } from './markdown-components';

interface AnalysisBlockCardProps {
  block: AnalysisBlock;
  sourceMap?: Map<string, Source>;
}

export function AnalysisBlockCard({ block, sourceMap }: AnalysisBlockCardProps) {
  return (
    <Card>
      <CardHeader className="gap-3">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex flex-wrap gap-2">
            <Badge variant="secondary">{block.kind.replace(/_/g, ' ')}</Badge>
            <Badge className={getImportanceClasses(block.importance)}>
              {block.importance}
            </Badge>
          </div>
          <ConfidenceBadge confidence={block.confidence} />
        </div>
        <CardTitle className="text-base">{block.title}</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="max-w-none text-sm text-foreground [&>*+*]:mt-3">
          <ReactMarkdown remarkPlugins={[remarkGfm]} components={reportMarkdownComponents}>
            {block.body}
          </ReactMarkdown>
        </div>
        {block.evidence_ids.length > 0 && (
          <div className="mt-5 flex flex-wrap items-center gap-2">
            <span className="text-xs text-muted-foreground">Sources:</span>
            <TooltipProvider delayDuration={200}>
              {block.evidence_ids.map(id => {
                const source = sourceMap?.get(id);
                return source ? (
                  <Tooltip key={id}>
                    <TooltipTrigger asChild>
                      <Badge variant="outline" className="max-w-[200px] cursor-default truncate">
                        {source.title}
                      </Badge>
                    </TooltipTrigger>
                    <TooltipContent className="max-w-xs space-y-1 text-left">
                      <p className="font-medium">{source.title}</p>
                      {source.publisher && (
                        <p className="text-muted-foreground">{source.publisher}</p>
                      )}
                      <p className="text-muted-foreground">{source.reliability} reliability</p>
                    </TooltipContent>
                  </Tooltip>
                ) : (
                  <Badge key={id} variant="outline" className="font-mono">
                    {id.slice(0, 8)}
                  </Badge>
                );
              })}
            </TooltipProvider>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
