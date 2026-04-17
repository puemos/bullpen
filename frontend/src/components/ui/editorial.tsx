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
