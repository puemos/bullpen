import React from "react";
import ReactDOM from "react-dom/client";
import { setState } from "@/store";
import "../styles.css";
import type { AnalysisReport } from "@/types";
import { ViewerPage } from "./ViewerPage";

/**
 * Placeholder replaced at export time by the Rust `export_analysis_html`
 * command. If this file is opened via `pnpm dev` (the template is unbuilt),
 * the literal string survives and we fall through to the empty state.
 */
declare global {
  interface Window {
    __BULLPEN_REPORT__?: AnalysisReport | string;
  }
}

// biome-ignore lint/suspicious/noExplicitAny: placeholder swapped at build/export time
window.__BULLPEN_REPORT__ = "__BULLPEN_REPORT_JSON__" as any;

function readEmbeddedReport(): AnalysisReport | null {
  const raw = window.__BULLPEN_REPORT__;
  if (!raw || raw === "__BULLPEN_REPORT_JSON__") return null;
  if (typeof raw === "string") {
    try {
      return JSON.parse(raw) as AnalysisReport;
    } catch {
      return null;
    }
  }
  return raw as AnalysisReport;
}

function FullScreen({ title, body }: { title: string; body: string }) {
  return (
    <div className="mx-auto flex min-h-screen max-w-xl flex-col justify-center gap-4 px-8 py-16">
      <p className="font-mono text-[10.5px] uppercase tracking-[0.18em] text-muted-foreground">
        Bullpen · Shared report
      </p>
      <h1 className="text-[34px] font-semibold leading-[1.05] tracking-[-0.02em]">{title}</h1>
      <p className="max-w-[62ch] text-[14.5px] leading-[1.55] text-muted-foreground">{body}</p>
    </div>
  );
}

const rootEl = document.getElementById("root");
if (!rootEl) throw new Error("#root missing");

const report = readEmbeddedReport();
const root = ReactDOM.createRoot(rootEl);

if (report?.analysis?.id) {
  setState({ selectedReport: report, selectedAnalysisId: report.analysis.id });
  root.render(
    <React.StrictMode>
      <ViewerPage report={report} />
    </React.StrictMode>,
  );
} else {
  root.render(
    <React.StrictMode>
      <FullScreen
        title="No report embedded"
        body="This standalone HTML file does not contain a report payload."
      />
    </React.StrictMode>,
  );
}
