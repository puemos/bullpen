import { ReportContent } from "@/features/report-viewer/ReportContent";
import { ReportShell } from "@/features/report-viewer/ReportShell";
import type { AnalysisReport } from "@/types";

interface ViewerPageProps {
  report: AnalysisReport;
}

export function ViewerPage({ report }: ViewerPageProps) {
  const { analysis } = report;

  return (
    <div className="h-screen bg-background">
      <ReportShell
        analysis={analysis}
        introLabel="Shared report"
        compactTrailing={
          <a
            href="https://bullpen.sh/"
            target="_blank"
            rel="noreferrer"
            className="whitespace-nowrap font-mono text-[11px] tracking-[0.14em] text-muted-foreground transition-colors hover:text-foreground"
          >
            Bullpen.sh →
          </a>
        }
      >
        <ReportContent />
      </ReportShell>
    </div>
  );
}
