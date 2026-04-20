import { WarningCircle } from "@phosphor-icons/react";
import { CopyButton } from "@/components/ui/copy-button";

interface TimelineErrorBlockProps {
  message: string;
  kind?: string | null;
  details?: string | null;
}

export function TimelineErrorBlock({ message, kind, details }: TimelineErrorBlockProps) {
  const copyText = details ? `${message}\n\n${details}` : message;

  return (
    <div className="border-t border-b border-destructive/30 py-3 text-destructive">
      <div className="flex items-start gap-2">
        <WarningCircle size={15} className="mt-0.5 shrink-0" />
        <div className="min-w-0 flex-1 space-y-2">
          <div className="flex items-start justify-between gap-3">
            <div className="min-w-0">
              <div className="font-mono text-[10.5px] uppercase tracking-[0.14em]">
                {kind ? kind.replaceAll("_", " ") : "Error"}
              </div>
              <p className="mt-1 whitespace-pre-wrap break-words text-[13px] leading-[1.55]">
                {message}
              </p>
            </div>
            <CopyButton text={copyText} className="h-6 w-6 shrink-0" iconSize={12} />
          </div>
          {details && details !== message && (
            <pre className="max-h-44 overflow-auto whitespace-pre-wrap break-words border-t border-destructive/20 pt-2 font-mono text-[11px] leading-[1.5] text-destructive/80">
              {details}
            </pre>
          )}
        </div>
      </div>
    </div>
  );
}
