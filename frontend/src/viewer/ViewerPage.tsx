import { Dot, Eyebrow } from "@/components/ui/editorial";
import { ReportContent } from "@/features/report-viewer/ReportContent";
import type { AnalysisReport } from "@/types";

interface ViewerPageProps {
  report: AnalysisReport;
}

function formatCreated(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(date);
}

export function ViewerPage({ report }: ViewerPageProps) {
  const { analysis } = report;
  const intent = analysis.intent;
  const status = analysis.status;
  const created = analysis.created_at;

  return (
    <div className="flex h-screen flex-col bg-background">
      <div className="shrink-0 border-b border-border bg-background">
        <div className="mx-auto flex max-w-5xl items-center justify-between px-8 py-3">
          <Eyebrow>Shared from Bullpen</Eyebrow>
          <a
            href="https://puemos.github.io/bullpen/"
            target="_blank"
            rel="noreferrer"
            className="font-mono text-[11px] tracking-[0.14em] text-muted-foreground hover:text-foreground"
          >
            puemos.github.io/bullpen →
          </a>
        </div>
      </div>
      <header className="shrink-0 border-b border-border bg-background">
        <div className="mx-auto flex max-w-5xl flex-col gap-6 px-8 pt-10 pb-8">
          <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
            <Eyebrow>Shared report</Eyebrow>
            {intent && (
              <>
                <Dot />
                <Eyebrow>{intent.replace(/_/g, " ")}</Eyebrow>
              </>
            )}
            {status && (
              <>
                <Dot />
                <Eyebrow>{status}</Eyebrow>
              </>
            )}
            {created && (
              <>
                <Dot />
                <Eyebrow>{formatCreated(created)}</Eyebrow>
              </>
            )}
          </div>

          <div className="space-y-4">
            <h1 className="text-[34px] font-semibold leading-[1.05] tracking-[-0.02em]">
              {analysis.title}
            </h1>
            {analysis.user_prompt && (
              <p className="max-w-[62ch] text-[14.5px] leading-[1.55] text-muted-foreground">
                {analysis.user_prompt}
              </p>
            )}
          </div>
        </div>
      </header>

      <div className="min-h-0 flex-1 overflow-auto">
        <ReportContent />
      </div>
    </div>
  );
}
