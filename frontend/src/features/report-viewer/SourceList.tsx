import type { Source } from '@/types';

interface SourceListProps {
  sources: Source[];
}

export function SourceList({ sources }: SourceListProps) {
  if (sources.length === 0) return null;

  return (
    <div className="space-y-4 pt-8">
      <h3 className="border-b border-border pb-2 text-lg font-medium">Sources</h3>
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        {sources.map(source => (
          <div key={source.id} className="border border-border/50 bg-muted/10 p-4 text-sm">
            <div className="mb-1 flex items-center justify-between text-xs text-muted-foreground">
              <span>{source.reliability}</span>
              <span className="font-mono">{source.id.slice(0, 8)}</span>
            </div>
            <a
              href={source.url || '#'}
              target="_blank"
              rel="noreferrer"
              className="block truncate font-medium text-primary hover:underline"
            >
              {source.title}
            </a>
            <p className="mt-2 line-clamp-3 text-xs text-muted-foreground/80">
              {source.summary}
            </p>
          </div>
        ))}
      </div>
    </div>
  );
}
