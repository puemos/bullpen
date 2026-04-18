import { useEffect, useState } from "react";
import { Eyebrow, SectionHeader } from "@/components/ui/editorial";
import { Input } from "@/components/ui/input";
import { getSettings, updateSettings } from "@/shared/api/commands";
import type { AgentCandidate, AppSettings } from "@/types";
import { AgentStatusList } from "./AgentStatusList";
import { DataSourcesSection } from "./DataSourcesSection";

interface SettingsPageProps {
  agents: AgentCandidate[];
}

export function SettingsPage({ agents }: SettingsPageProps) {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [saved, setSaved] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getSettings()
      .then(setSettings)
      .catch((err) => setError(String(err)));
  }, []);

  const save = async () => {
    if (!settings) return;
    setError(null);
    try {
      const next = await updateSettings(settings);
      setSettings(next);
      setSaved("Saved");
      setTimeout(() => setSaved(null), 1300);
    } catch (err) {
      setError(String(err));
    }
  };

  return (
    <div className="mx-auto h-full max-w-3xl space-y-16 overflow-y-auto px-8 pt-10 pb-32">
      <div>
        <Eyebrow>Settings</Eyebrow>
        <h1 className="mt-3 text-[34px] font-semibold leading-[1.05] tracking-[-0.02em]">
          Configuration
        </h1>
      </div>

      <section className="space-y-6">
        <SectionHeader
          number="01"
          label="Agents"
          title="Local ACP agents"
          meta={
            <span className="font-mono tabular-nums">
              {String(agents.length).padStart(2, "0")} detected
            </span>
          }
        />
        <p className="max-w-[60ch] text-[14px] leading-[1.6] text-muted-foreground">
          Bullpen runs research against ACP-compatible agents on your machine. If an agent is
          marked unavailable, check your PATH or the documented environment overrides (
          <code className="font-mono text-[13px]">CODEX_ACP_BIN</code>,{" "}
          <code className="font-mono text-[13px]">BULLPEN_CUSTOM_AGENT</code>).
        </p>
        <AgentStatusList agents={agents} />
      </section>

      <section className="space-y-6">
        <SectionHeader number="02" label="Preferences" title="Overrides" />
        {error && <div className="text-sm text-destructive">{error}</div>}
        {!settings ? (
          <div className="text-sm text-muted-foreground">Loading…</div>
        ) : (
          <div className="space-y-6">
            <div className="space-y-2">
              <Eyebrow>Custom ACP command</Eyebrow>
              <Input
                className="bg-transparent font-mono text-[13px]"
                value={settings.custom_agent_command || ""}
                onChange={(event) =>
                  setSettings({
                    ...settings,
                    custom_agent_command: event.target.value || null,
                  })
                }
                placeholder="e.g. /usr/local/bin/my-agent"
              />
              <p className="max-w-[60ch] text-[12.5px] leading-relaxed text-muted-foreground">
                Absolute path to a custom ACP agent binary. Leave blank to rely on autodiscovery.
              </p>
            </div>

            <div className="border-t border-border pt-6">
              <button
                type="button"
                onClick={save}
                className="group inline-flex items-center gap-2 border border-foreground bg-foreground px-4 py-2 text-[13px] font-medium text-background transition-colors hover:bg-background hover:text-foreground"
              >
                <span>{saved || "Save settings"}</span>
              </button>
            </div>
          </div>
        )}
      </section>

      <DataSourcesSection />
    </div>
  );
}
