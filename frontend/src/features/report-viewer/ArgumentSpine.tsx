import type { FinalStance } from '@/types';
import { getStanceAccent } from './badge-styles';
import { Eyebrow } from '@/components/ui/editorial';

interface ArgumentSpineProps {
  stance: FinalStance;
}

export function ArgumentSpine({ stance }: ArgumentSpineProps) {
  const accent = getStanceAccent(stance.stance);
  const hasAny =
    stance.key_reasons.length + stance.what_would_change.length > 0;
  if (!hasAny) return null;

  return (
    <section className="grid gap-10 md:grid-cols-2 md:gap-8">
      <SpineColumn
        number="01"
        label="The case"
        items={stance.key_reasons}
        markerClass={accent.dot}
        markerStyle="tick"
      />
      <SpineColumn
        number="02"
        label="Would change our mind"
        items={stance.what_would_change}
        markerClass="bg-foreground/60"
        markerStyle="dot"
      />
    </section>
  );
}

function SpineColumn({
  number,
  label,
  items,
  markerClass,
  markerStyle,
}: {
  number: string;
  label: string;
  items: string[];
  markerClass: string;
  markerStyle: 'tick' | 'dot';
}) {
  return (
    <div className="flex flex-col gap-5">
      <div className="flex items-baseline gap-2 border-b border-border pb-3">
        <span className="font-mono text-[10.5px] font-medium tabular-nums text-muted-foreground">
          {number}
        </span>
        <Eyebrow>{label}</Eyebrow>
      </div>
      {items.length === 0 ? (
        <p className="text-sm italic text-muted-foreground/70">None stated.</p>
      ) : (
        <ol className="space-y-4 text-[15px] leading-[1.55] text-foreground">
          {items.map((item, index) => (
            <li key={`${index}-${item.slice(0, 32)}`} className="flex gap-3">
              {markerStyle === 'tick' ? (
                <span className={`mt-[0.55em] h-[2px] w-3 shrink-0 ${markerClass}`} aria-hidden />
              ) : (
                <span
                  className={`mt-[0.7em] h-1 w-1 shrink-0 rounded-full ${markerClass}`}
                  aria-hidden
                />
              )}
              <span>{item}</span>
            </li>
          ))}
        </ol>
      )}
    </div>
  );
}
