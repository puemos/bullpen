import { cn } from '@/lib/utils';

export function getImportanceClasses(importance: string): string {
  switch (importance) {
    case 'high':
      return 'border-transparent bg-foreground text-background';
    case 'medium':
      return 'border-transparent bg-secondary text-secondary-foreground';
    case 'low':
      return 'border-muted-foreground/30 text-muted-foreground';
    default:
      return '';
  }
}

export function getStanceClasses(stance: string): string {
  switch (stance) {
    case 'bullish':
      return 'border-emerald-500/40 bg-emerald-500/10 text-emerald-700 dark:text-emerald-400';
    case 'bearish':
      return 'border-red-500/40 bg-red-500/10 text-red-700 dark:text-red-400';
    case 'neutral':
      return 'border-zinc-400/40 bg-zinc-400/10 text-zinc-600 dark:text-zinc-400';
    case 'mixed':
      return 'border-amber-500/40 bg-amber-500/10 text-amber-700 dark:text-amber-400';
    case 'insufficient_data':
      return 'border-muted-foreground/30 bg-muted text-muted-foreground';
    default:
      return '';
  }
}

export function ConfidenceBadge({
  confidence,
  className,
}: {
  confidence: number;
  className?: string;
}) {
  const pct = Math.round(confidence * 100);
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 rounded-md border px-2 py-0.5 text-xs',
        className,
      )}
    >
      <span className="inline-block h-1.5 w-12 overflow-hidden rounded-full bg-muted">
        <span
          className="block h-full rounded-full bg-foreground/60"
          style={{ width: `${pct}%` }}
        />
      </span>
      <span className="tabular-nums">{pct}%</span>
    </span>
  );
}
