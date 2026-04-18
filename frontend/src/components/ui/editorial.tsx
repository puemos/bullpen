import { cn } from "@/lib/utils";

export function Eyebrow({
  children,
  className,
}: {
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <span
      className={cn(
        "inline-block text-[10.5px] font-medium uppercase tracking-[0.18em] text-muted-foreground",
        className,
      )}
    >
      {children}
    </span>
  );
}

export function SectionHeader({
  number,
  label,
  title,
  meta,
  id,
  className,
}: {
  number?: string;
  label: string;
  title?: string;
  meta?: React.ReactNode;
  id?: string;
  className?: string;
}) {
  return (
    <header id={id} className={cn("scroll-mt-24 space-y-3 border-t border-border pt-6", className)}>
      <div className="flex items-end justify-between gap-4">
        <div className="flex items-baseline gap-3">
          {number && (
            <span className="font-mono text-[10.5px] font-medium tabular-nums text-muted-foreground">
              {number}
            </span>
          )}
          <Eyebrow>{label}</Eyebrow>
        </div>
        {meta && <div className="text-xs text-muted-foreground">{meta}</div>}
      </div>
      {title && <h2 className="text-2xl font-semibold leading-tight tracking-tight">{title}</h2>}
    </header>
  );
}

export function HairlineDivider({ className }: { className?: string }) {
  return <div className={cn("h-px w-full bg-border", className)} />;
}

export function MetaRow({ items, className }: { items: React.ReactNode[]; className?: string }) {
  return (
    <div className={cn("flex flex-wrap items-center gap-x-3 gap-y-2", className)}>
      {items.map((item, index) => (
        <span key={index} className="flex items-center gap-x-3">
          {index > 0 && <Dot />}
          {item}
        </span>
      ))}
    </div>
  );
}

export function Dot({ className }: { className?: string }) {
  return <span className={cn("h-1 w-1 rounded-full bg-border", className)} aria-hidden />;
}

export type FreshnessBucket = "fresh" | "aging" | "stale" | "very_stale";

export function freshnessBucket(ageDays: number): FreshnessBucket {
  if (ageDays <= 7) return "fresh";
  if (ageDays <= 30) return "aging";
  if (ageDays <= 180) return "stale";
  return "very_stale";
}

export function ageDaysFrom(iso: string, now: Date = new Date()): number | null {
  if (!iso) return null;
  const parsed = new Date(iso);
  if (Number.isNaN(parsed.getTime())) return null;
  const ms = now.getTime() - parsed.getTime();
  return Math.max(0, Math.floor(ms / 86_400_000));
}

/**
 * Colour-graded freshness label for a source retrieval or metric `as_of`
 * stamp. `variant="source"` and `variant="metric"` both render the same way
 * today; the prop is kept so downstream styling can diverge without changing
 * callers.
 */
export function FreshnessChip({
  iso,
  variant,
  now,
  className,
}: {
  iso: string;
  variant: "source" | "metric";
  now?: Date;
  className?: string;
}) {
  const age = ageDaysFrom(iso, now);
  if (age === null) {
    return (
      <span
        className={cn(
          "font-mono text-[10.5px] tabular-nums uppercase tracking-[0.14em] text-muted-foreground/60",
          className,
        )}
        title="Unparseable date"
      >
        ? AGE
      </span>
    );
  }
  const bucket = freshnessBucket(age);
  const tone = bucketTone(bucket);
  const label = formatAgeLabel(iso, age);
  const prefix = variant === "source" ? "RETR" : "AS OF";
  return (
    <span
      className={cn(
        "inline-flex items-baseline gap-1 font-mono text-[10.5px] tabular-nums uppercase tracking-[0.14em]",
        tone,
        className,
      )}
      title={`${prefix} ${iso} · ${age}d old`}
    >
      <span className="text-muted-foreground/70">{prefix}</span>
      <span>{label}</span>
    </span>
  );
}

function formatAgeLabel(iso: string, ageDays: number): string {
  if (ageDays <= 90) return `${ageDays}d`;
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return `${ageDays}d`;
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    year: "numeric",
  })
    .format(date)
    .toUpperCase();
}

function bucketTone(bucket: FreshnessBucket): string {
  switch (bucket) {
    case "fresh":
      return "text-muted-foreground";
    case "aging":
      return "text-foreground";
    case "stale":
      return "text-destructive/80";
    case "very_stale":
      return "text-destructive font-medium";
  }
}
