import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import type { Source } from '@/types';

interface SourceListProps {
  sources: Source[];
}

export function SourceList({ sources }: SourceListProps) {
  if (sources.length === 0) return null;

  return (
    <section className="space-y-4 pt-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-medium">Sources</h3>
        <Badge variant="secondary">{sources.length} sources</Badge>
      </div>
      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
        {sources.map(source => (
          <Card key={source.id} className="gap-2 py-3">
            <CardHeader className="gap-1.5 px-4">
              <div className="flex items-center gap-2">
                <Badge variant={source.reliability === 'primary' ? 'default' : 'outline'}>
                  {source.reliability}
                </Badge>
                <Badge variant="secondary" className="text-[10px]">
                  {source.source_type.replace(/_/g, ' ')}
                </Badge>
              </div>
              <CardTitle className="truncate text-sm">
                <a
                  href={source.url || '#'}
                  target="_blank"
                  rel="noreferrer"
                  className="text-primary hover:underline"
                >
                  {source.title}
                </a>
              </CardTitle>
              {source.publisher && (
                <p className="text-xs text-muted-foreground">{source.publisher}</p>
              )}
            </CardHeader>
            <CardContent className="px-4">
              <p className="line-clamp-3 text-xs text-muted-foreground">
                {source.summary}
              </p>
              <p className="mt-2 text-xs text-muted-foreground">
                Retrieved {source.retrieved_at}
              </p>
            </CardContent>
          </Card>
        ))}
      </div>
    </section>
  );
}
