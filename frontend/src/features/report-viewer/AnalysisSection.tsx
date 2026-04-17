import { memo } from "react";
import { Eyebrow } from "@/components/ui/editorial";
import type { AnalysisBlock, BlockKind, Source } from "@/types";
import { AnalysisBlockCard } from "./AnalysisBlockCard";

interface AnalysisSectionProps {
  blocks: AnalysisBlock[];
  sourceMap: Map<string, Source>;
}

interface BlockGroup {
  id: string;
  label: string;
  kinds: BlockKind[];
}

const GROUPS: BlockGroup[] = [
  { id: "thesis", label: "Thesis & Business Quality", kinds: ["thesis", "business_quality"] },
  {
    id: "financials",
    label: "Financial Case",
    kinds: ["financials", "valuation", "peer_comparison"],
  },
  { id: "context", label: "Context", kinds: ["sector_context", "technical_context"] },
  { id: "path", label: "Path Ahead", kinds: ["catalysts", "risks"] },
  { id: "open", label: "Open Questions", kinds: ["open_questions", "other"] },
];

export const AnalysisSection = memo(function AnalysisSection({
  blocks,
  sourceMap,
}: AnalysisSectionProps) {
  if (blocks.length === 0) return null;

  const sorted = [...blocks].sort((a, b) => a.display_order - b.display_order);
  const grouped = GROUPS.map((group) => ({
    ...group,
    blocks: sorted.filter((b) => group.kinds.includes(b.kind as BlockKind)),
  })).filter((group) => group.blocks.length > 0);

  return (
    <div className="space-y-14">
      {grouped.map((group) => (
        <div key={group.id} className="space-y-2">
          <div className="sticky top-12 z-10 -mx-8 flex h-11 items-center border-b border-border bg-background px-8">
            <div className="flex flex-1 items-baseline justify-between gap-4">
              <Eyebrow>{group.label}</Eyebrow>
              <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground">
                {String(group.blocks.length).padStart(2, "0")}
              </span>
            </div>
          </div>
          <div>
            {group.blocks.map((block, index) => (
              <AnalysisBlockCard
                key={block.id}
                block={block}
                sourceMap={sourceMap}
                isFirstInGroup={index === 0}
              />
            ))}
          </div>
        </div>
      ))}
    </div>
  );
});
