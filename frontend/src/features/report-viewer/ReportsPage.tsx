import { ReportContent } from "./ReportContent";

interface ReportsPageProps {
  onRefresh: () => Promise<void>;
}

export function ReportsPage({ onRefresh }: ReportsPageProps) {
  void onRefresh;
  return <ReportContent />;
}
