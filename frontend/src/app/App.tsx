import {
  ChartLineUp,
  Gear,
  MagnifyingGlass,
  WarningCircle,
} from '@phosphor-icons/react';
import { useEffect, useState, type ReactNode } from 'react';
import { Button } from '@/components/ui/button';
import { ReportsPage } from '@/features/report-viewer/ReportsPage';
import { ResearchPage } from '@/features/run-analysis/ResearchPage';
import { SettingsPage } from '@/features/settings/SettingsPage';
import {
  getAgents,
  getAllAnalyses,
  getAnalysisReport,
} from '@/shared/api/commands';
import {
  getState,
  setState,
  useAppStore,
} from '@/store';
import type { AgentCandidate } from '@/types';
import type { AppView } from './navigation';

export function App() {
  const view = useAppStore(state => state.view);
  const analyses = useAppStore(state => state.analyses);
  const selectedAnalysisId = useAppStore(state => state.selectedAnalysisId);
  const [agents, setAgents] = useState<AgentCandidate[]>([]);
  const [error, setError] = useState<string | null>(null);

  const refresh = async () => {
    setError(null);
    try {
      const [nextAgents, nextAnalyses] = await Promise.all([getAgents(), getAllAnalyses()]);
      setAgents(nextAgents);
      setState({
        analyses: nextAnalyses,
        agentId:
          getState().agentId ||
          nextAgents.find(agent => agent.available)?.id ||
          nextAgents[0]?.id ||
          '',
      });

      const selected = getState().selectedAnalysisId;
      if (selected) {
        const report = await getAnalysisReport(selected);
        setState({ selectedReport: report });
      }
    } catch (err) {
      setError(String(err));
    }
  };

  useEffect(() => {
    refresh();
  }, []);

  return (
    <div className="flex h-screen w-full flex-col bg-background text-foreground">
      <div className="fixed left-1/2 top-6 z-50 flex -translate-x-1/2 items-center gap-1 rounded-full border border-border bg-background/80 px-2 py-1.5 shadow-sm backdrop-blur-xl">
        <nav className="flex items-center gap-1 text-sm">
          <NavButton
            view="research"
            label="Research"
            icon={<MagnifyingGlass size={16} />}
            currentView={view}
          />
          <NavButton
            view="reports"
            label="Reports"
            icon={<ChartLineUp size={16} />}
            currentView={view}
          />
          <NavButton
            view="settings"
            label="Settings"
            icon={<Gear size={16} />}
            currentView={view}
          />
        </nav>
      </div>

      <div className="fixed right-6 top-6 z-50 flex items-center gap-2">
        {view === 'reports' && analyses.length > 0 && (
          <select
            className="rounded-full border border-border bg-background/80 px-3 py-1.5 text-xs shadow-sm outline-none backdrop-blur-md"
            value={selectedAnalysisId || ''}
            onChange={async event => {
              const id = event.target.value;
              setState({ selectedAnalysisId: id, selectedReport: null });
              if (id) {
                const report = await getAnalysisReport(id);
                setState({ selectedReport: report });
              }
            }}
          >
            <option value="" disabled>
              Select report...
            </option>
            {analyses.map(analysis => (
              <option key={analysis.id} value={analysis.id}>
                {analysis.title}
              </option>
            ))}
          </select>
        )}
      </div>

      <main className="relative flex flex-1 flex-col overflow-hidden pt-20">
        {error && (
          <div className="mx-6 mt-4 flex items-center gap-2 rounded-md bg-destructive/10 px-4 py-2 text-sm text-destructive">
            <WarningCircle size={16} />
            {error}
          </div>
        )}
        <div className="flex-1 overflow-auto">
          {view === 'research' && <ResearchPage agents={agents} onDone={refresh} />}
          {view === 'reports' && <ReportsPage onRefresh={refresh} />}
          {view === 'settings' && <SettingsPage agents={agents} />}
        </div>
      </main>
    </div>
  );
}

interface NavButtonProps {
  view: AppView;
  label: string;
  icon: ReactNode;
  currentView: AppView;
}

function NavButton({ view, label, icon, currentView }: NavButtonProps) {
  const active = currentView === view;

  return (
    <Button
      variant="ghost"
      size="sm"
      className={`flex items-center gap-1.5 rounded-full px-4 transition-colors ${
        active
          ? 'bg-secondary font-medium text-foreground shadow-sm'
          : 'text-muted-foreground hover:bg-secondary/40 hover:text-foreground'
      }`}
      onClick={() => setState({ view })}
    >
      {icon}
      {label}
    </Button>
  );
}
