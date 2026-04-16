import type { FinalStance } from '@/types';

interface FinalStanceViewProps {
  stance: FinalStance;
}

export function FinalStanceView({ stance }: FinalStanceViewProps) {
  return (
    <div className="border border-border bg-muted/5 p-6">
      <h3 className="mb-4 text-lg font-medium">Final Conclusion</h3>
      <p className="mb-6 text-sm leading-relaxed">{stance.summary}</p>
      <div className="grid grid-cols-2 gap-8 text-sm">
        <div>
          <h4 className="mb-2 font-semibold">Key Reasons</h4>
          <ul className="list-disc space-y-1 pl-4 text-muted-foreground">
            {stance.key_reasons.map(reason => (
              <li key={reason}>{reason}</li>
            ))}
          </ul>
        </div>
        <div>
          <h4 className="mb-2 font-semibold">Watch Items</h4>
          <ul className="list-disc space-y-1 pl-4 text-muted-foreground">
            {stance.watch_items.map(item => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </div>
      </div>
    </div>
  );
}
