import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { Eyebrow } from "@/components/ui/editorial";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import type { AnalysisBlock, Source } from "@/types";
import { ConfidenceBadge } from "./badge-styles";
import { reportMarkdownComponents } from "./markdown-components";

interface AnalysisBlockCardProps {
  block: AnalysisBlock;
  sourceMap?: Map<string, Source>;
  isFirstInGroup?: boolean;
}

export function AnalysisBlockCard({ block, sourceMap, isFirstInGroup }: AnalysisBlockCardProps) {
  return (
    <article
      className={cn(
        "grid gap-6 py-8 md:grid-cols-[180px_minmax(0,1fr)] md:gap-10",
        !isFirstInGroup && "border-t border-border",
      )}
    >
      <header className="flex flex-col gap-2 md:sticky md:top-28 md:self-start">
        <div className="flex items-center gap-2">
          <ImportanceGlyph importance={block.importance} />
          <Eyebrow>{String(block.importance)}</Eyebrow>
        </div>
        <h3 className="text-[17px] font-semibold leading-snug tracking-tight text-foreground">
          {block.title}
        </h3>
        <ConfidenceBadge confidence={block.confidence} />
      </header>

      <div className="min-w-0 space-y-5">
        <div className="max-w-[62ch] text-[15px] leading-[1.65] text-foreground/90 [&>*+*]:mt-4">
          <ReactMarkdown remarkPlugins={[remarkGfm]} components={reportMarkdownComponents}>
            {block.body}
          </ReactMarkdown>
        </div>
        {block.evidence_ids.length > 0 && (
          <EvidenceRow ids={block.evidence_ids} sourceMap={sourceMap} />
        )}
      </div>
    </article>
  );
}

function ImportanceGlyph({ importance }: { importance: string }) {
  const cls =
    importance === "high"
      ? "bg-foreground"
      : importance === "medium"
        ? "bg-foreground/50"
        : "bg-foreground/20";
  return <span className={cn("h-2 w-2 shrink-0 rounded-full", cls)} aria-hidden />;
}

function EvidenceRow({ ids, sourceMap }: { ids: string[]; sourceMap?: Map<string, Source> }) {
  return (
    <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1.5 border-t border-border pt-4">
      <Eyebrow className="shrink-0">Cited</Eyebrow>
      <TooltipProvider delayDuration={150}>
        {ids.map((id, index) => {
          const source = sourceMap?.get(id);
          const label = source?.title ?? id.slice(0, 8);
          return (
            <Tooltip key={id}>
              <TooltipTrigger asChild>
                <a
                  href={source?.url || "#"}
                  target="_blank"
                  rel="noreferrer"
                  className="inline-flex items-baseline gap-1.5 text-[12.5px] text-foreground/80 underline-offset-4 hover:underline"
                >
                  <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground">
                    {String(index + 1).padStart(2, "0")}
                  </span>
                  <span className="max-w-[24ch] truncate">{label}</span>
                </a>
              </TooltipTrigger>
              {source && (
                <TooltipContent
                  variant="editorial"
                  sideOffset={6}
                  className="max-w-xs space-y-1.5 text-left"
                >
                  <span className="block font-mono text-[10px] uppercase tracking-[0.16em] text-muted-foreground">
                    Source
                  </span>
                  <p className="text-[13px] font-medium leading-snug text-foreground">
                    {source.title}
                  </p>
                  {source.publisher && (
                    <p className="text-[12px] text-foreground/70">{source.publisher}</p>
                  )}
                  <p className="font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground">
                    {source.reliability} reliability
                  </p>
                </TooltipContent>
              )}
            </Tooltip>
          );
        })}
      </TooltipProvider>
    </div>
  );
}
