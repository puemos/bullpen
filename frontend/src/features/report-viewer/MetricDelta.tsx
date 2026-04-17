import { cn } from "@/lib/utils";

interface MetricDeltaProps {
  changePct: number | null;
  priorValue: number | null;
  className?: string;
}

export function MetricDelta({ changePct, priorValue, className }: MetricDeltaProps) {
  if (changePct === null && priorValue === null) return null;
  const pct = changePct ?? 0;
  const isPositive = pct > 0;
  const isNegative = pct < 0;
  const arrow = isPositive ? "↑" : isNegative ? "↓" : "·";
  const color = isPositive
    ? "text-emerald-700 dark:text-emerald-400"
    : isNegative
      ? "text-red-700 dark:text-red-400"
      : "text-muted-foreground";
  const sign = isPositive ? "+" : "";
  return (
    <span
      className={cn(
        "inline-flex items-baseline gap-1 font-mono text-[11px] tabular-nums",
        color,
        className,
      )}
      title={priorValue !== null ? `prior ${priorValue}` : undefined}
    >
      <span>{arrow}</span>
      {changePct !== null && (
        <span>
          {sign}
          {(pct * 100).toFixed(1)}%
        </span>
      )}
    </span>
  );
}
