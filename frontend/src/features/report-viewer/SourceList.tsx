import { ArrowUpRight } from "@phosphor-icons/react";
import { memo } from "react";
import { Eyebrow, FreshnessChip } from "@/components/ui/editorial";
import { cn } from "@/lib/utils";
import type { Source } from "@/types";

interface SourceListProps {
  sources: Source[];
}

export const SourceList = memo(function SourceList({ sources }: SourceListProps) {
  if (sources.length === 0) return null;

  const sorted = [...sources].sort(
    (a, b) => reliabilityRank(b.reliability) - reliabilityRank(a.reliability),
  );

  return (
    <div className="divide-y divide-border border-y border-border">
      {sorted.map((source, index) => (
        <SourceRow key={source.id} source={source} index={index} />
      ))}
    </div>
  );
});

function SourceRow({ source, index }: { source: Source; index: number }) {
  const content = (
    <>
      <span className="font-mono text-[11px] tabular-nums text-muted-foreground">
        {String(index + 1).padStart(2, "0")}
      </span>
      <div className="min-w-0 flex-1 space-y-1.5">
        <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
          <ReliabilityPill reliability={source.reliability} />
          <Eyebrow className="text-muted-foreground/80">
            {source.source_type.replace(/_/g, " ")}
          </Eyebrow>
          {source.publisher && (
            <span className="text-[12px] text-muted-foreground">{source.publisher}</span>
          )}
          <FreshnessChip iso={source.retrieved_at} />
          <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground/60">
            {formatDate(source.retrieved_at)}
          </span>
          {source.last_verification_status === "dead" && (
            <span
              className="font-mono text-[10.5px] font-medium uppercase tracking-[0.14em] text-destructive"
              title={
                source.last_verified_at
                  ? `Verified dead on ${formatDate(source.last_verified_at)}`
                  : "Link dead"
              }
            >
              LINK DEAD
            </span>
          )}
        </div>
        <div className="flex items-start gap-1.5 text-[14.5px] font-medium leading-snug text-foreground">
          <span className="min-w-0 flex-1 truncate">{source.title}</span>
          {source.url && (
            <ArrowUpRight
              size={14}
              className="mt-[3px] shrink-0 text-muted-foreground transition-colors group-hover:text-foreground"
            />
          )}
        </div>
        {source.summary && (
          <p className="line-clamp-2 text-[13px] leading-relaxed text-muted-foreground">
            {source.summary}
          </p>
        )}
      </div>
    </>
  );

  const baseClass = "group flex items-start gap-4 px-1 py-4 transition-colors";

  if (source.url) {
    return (
      <a
        href={source.url}
        target="_blank"
        rel="noreferrer"
        className={cn(baseClass, "hover:bg-muted/40")}
      >
        {content}
      </a>
    );
  }
  return <div className={baseClass}>{content}</div>;
}

export function ReliabilityPill({ reliability }: { reliability: Source["reliability"] }) {
  const accent = reliabilityAccent(reliability);
  return (
    <span className="inline-flex items-center gap-1.5">
      <span className={cn("h-1.5 w-1.5 rounded-full", accent.dot)} aria-hidden />
      <span className="text-[11px] font-medium uppercase tracking-[0.14em] text-foreground">
        {reliability}
      </span>
    </span>
  );
}

function reliabilityRank(reliability: Source["reliability"]) {
  switch (reliability) {
    case "primary":
      return 3;
    case "high":
      return 2;
    case "medium":
      return 1;
    default:
      return 0;
  }
}

function reliabilityAccent(reliability: Source["reliability"]) {
  switch (reliability) {
    case "primary":
      return { dot: "bg-foreground" };
    case "high":
      return { dot: "bg-foreground/70" };
    case "medium":
      return { dot: "bg-foreground/40" };
    default:
      return { dot: "bg-foreground/20" };
  }
}

function formatDate(value: string): string {
  if (!value) return "—";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(date);
}
