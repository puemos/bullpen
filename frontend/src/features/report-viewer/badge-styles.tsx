import { cn } from '@/lib/utils';

export function getImportanceClasses(importance: string): string {
  switch (importance) {
    case 'high':
      return 'border-foreground/20 bg-foreground text-background';
    case 'medium':
      return 'border-border bg-transparent text-foreground';
    case 'low':
      return 'border-border bg-transparent text-muted-foreground';
    default:
      return '';
  }
}

export interface StanceAccent {
  tick: string;
  text: string;
  rule: string;
  dot: string;
  label: string;
}

export function getStanceAccent(stance: string): StanceAccent {
  switch (stance) {
    case 'bullish':
      return {
        tick: 'bg-emerald-600 dark:bg-emerald-400',
        text: 'text-emerald-700 dark:text-emerald-400',
        rule: 'bg-emerald-600/80 dark:bg-emerald-400/80',
        dot: 'bg-emerald-600 dark:bg-emerald-400',
        label: 'Bullish',
      };
    case 'bearish':
      return {
        tick: 'bg-red-600 dark:bg-red-400',
        text: 'text-red-700 dark:text-red-400',
        rule: 'bg-red-600/80 dark:bg-red-400/80',
        dot: 'bg-red-600 dark:bg-red-400',
        label: 'Bearish',
      };
    case 'mixed':
      return {
        tick: 'bg-amber-500 dark:bg-amber-400',
        text: 'text-amber-700 dark:text-amber-400',
        rule: 'bg-amber-500/80 dark:bg-amber-400/80',
        dot: 'bg-amber-500 dark:bg-amber-400',
        label: 'Mixed',
      };
    case 'neutral':
      return {
        tick: 'bg-zinc-500 dark:bg-zinc-400',
        text: 'text-zinc-700 dark:text-zinc-300',
        rule: 'bg-zinc-500/70 dark:bg-zinc-400/70',
        dot: 'bg-zinc-500 dark:bg-zinc-400',
        label: 'Neutral',
      };
    default:
      return {
        tick: 'bg-muted-foreground/60',
        text: 'text-muted-foreground',
        rule: 'bg-muted-foreground/40',
        dot: 'bg-muted-foreground/60',
        label: 'Insufficient data',
      };
  }
}

export function getStanceClasses(stance: string): string {
  const accent = getStanceAccent(stance);
  return cn('border-transparent bg-transparent', accent.text);
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
        'inline-flex items-center gap-2 font-mono text-[10.5px] tabular-nums text-muted-foreground',
        className,
      )}
    >
      <span className="inline-block h-px w-10 bg-border">
        <span
          className="block h-full bg-foreground/70"
          style={{ width: `${pct}%` }}
        />
      </span>
      <span>{pct}%</span>
    </span>
  );
}

export function ConfidenceRail({
  confidence,
  accentClass,
  className,
}: {
  confidence: number;
  accentClass: string;
  className?: string;
}) {
  const pct = Math.round(confidence * 100);
  return (
    <div className={cn('flex items-center gap-3', className)}>
      <div className="relative h-px flex-1 overflow-hidden bg-border">
        <div
          className={cn('absolute inset-y-0 left-0', accentClass)}
          style={{ width: `${pct}%` }}
        />
      </div>
      <span className="font-mono text-xs tabular-nums text-foreground">
        {pct}%
      </span>
    </div>
  );
}
