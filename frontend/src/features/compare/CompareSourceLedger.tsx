import { ArrowUpRight } from "@phosphor-icons/react";
import { Eyebrow, SectionHeader } from "@/components/ui/editorial";
import { ReliabilityPill } from "@/features/report-viewer/SourceList";
import { cn } from "@/lib/utils";
import type { AnalysisReport, Source } from "@/types";

export function CompareSourceLedger({
  reports,
  ids,
  number,
}: {
  reports: Record<string, AnalysisReport | null>;
  ids: string[];
  number: string;
}) {
  const dedup = dedupeSources(reports, ids);
  if (dedup.length === 0) return null;

  return (
    <section className="space-y-4 pb-14">
      <SectionHeader
        number={number}
        label="Sources"
        title="Shared bibliography"
        meta={
          <span className="font-mono tabular-nums">
            {String(dedup.length).padStart(2, "0")} unique
          </span>
        }
      />
      <div className="divide-y divide-border border-y border-border">
        {dedup.map((entry, index) => (
          <div key={entry.key} className="flex items-start gap-4 px-1 py-4">
            <span className="font-mono text-[11px] tabular-nums text-muted-foreground">
              {String(index + 1).padStart(2, "0")}
            </span>
            <div className="min-w-0 flex-1 space-y-1.5">
              <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
                <ReliabilityPill reliability={entry.source.reliability} />
                <Eyebrow className="text-muted-foreground/80">
                  {entry.source.source_type.replace(/_/g, " ")}
                </Eyebrow>
                {entry.source.publisher && (
                  <span className="text-[12px] text-muted-foreground">
                    {entry.source.publisher}
                  </span>
                )}
              </div>
              <div className="flex items-start gap-1.5 text-[14px] font-medium leading-snug text-foreground">
                {entry.source.url ? (
                  <a
                    href={entry.source.url}
                    target="_blank"
                    rel="noreferrer"
                    className="flex min-w-0 flex-1 items-start gap-1.5 hover:underline"
                  >
                    <span className="min-w-0 flex-1 truncate">{entry.source.title}</span>
                    <ArrowUpRight size={14} className="mt-[3px] shrink-0 text-muted-foreground" />
                  </a>
                ) : (
                  <span className="min-w-0 flex-1 truncate">{entry.source.title}</span>
                )}
              </div>
            </div>
            <CitationMatrix presence={entry.presence} ids={ids} />
          </div>
        ))}
      </div>
    </section>
  );
}

function CitationMatrix({ presence, ids }: { presence: Set<string>; ids: string[] }) {
  return (
    <div className="flex shrink-0 items-center gap-1.5 pt-1">
      {ids.map((id, index) => (
        <span
          key={id}
          title={`Analysis ${index + 1}: ${presence.has(id) ? "cited" : "not cited"}`}
          className={cn(
            "h-1.5 w-1.5 rounded-full",
            presence.has(id) ? "bg-foreground" : "bg-foreground/15",
          )}
          aria-hidden
        />
      ))}
    </div>
  );
}

interface DedupEntry {
  key: string;
  source: Source;
  presence: Set<string>;
}

function dedupeSources(
  reports: Record<string, AnalysisReport | null>,
  ids: string[],
): DedupEntry[] {
  const byKey = new Map<string, DedupEntry>();
  for (const id of ids) {
    const report = reports[id];
    if (!report) continue;
    for (const source of report.sources) {
      const key = source.url?.trim() || `${id}::${source.id}`;
      const existing = byKey.get(key);
      if (existing) {
        existing.presence.add(id);
      } else {
        byKey.set(key, { key, source, presence: new Set([id]) });
      }
    }
  }
  return [...byKey.values()].sort((a, b) => b.presence.size - a.presence.size);
}
