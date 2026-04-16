import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { getSettings, updateSettings } from "@/shared/api/commands";
import type { AgentCandidate, AppSettings } from "@/types";
import { AgentStatusList } from "./AgentStatusList";

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
      setSaved("Saved!");
      setTimeout(() => setSaved(null), 1300);
    } catch (err) {
      setError(String(err));
    }
  };

  return (
    <div className="mx-auto max-w-2xl space-y-10 p-8">
      <div>
        <h2 className="mb-2 text-xl font-semibold">Agents</h2>
        <p className="mb-4 text-xs text-muted-foreground">
          Crazylines uses local ACP agents. Check PATH or ENV overrides.
        </p>
        <AgentStatusList agents={agents} />
      </div>

      <div>
        <h2 className="mb-4 text-xl font-semibold">Preferences</h2>
        {error && <div className="mb-4 text-sm text-destructive">{error}</div>}
        {!settings ? (
          <div className="text-sm">Loading...</div>
        ) : (
          <div className="space-y-4">
            <div className="flex flex-col gap-1.5 text-sm">
              <label className="text-xs font-medium uppercase tracking-wider text-muted-foreground">
                Custom ACP Command
              </label>
              <Input
                className="bg-transparent"
                value={settings.custom_agent_command || ""}
                onChange={(event) =>
                  setSettings({
                    ...settings,
                    custom_agent_command: event.target.value || null,
                  })
                }
              />
            </div>

            <Button className="mt-4 font-semibold" onClick={save}>
              {saved || "Save Settings"}
            </Button>
          </div>
        )}
      </div>
    </div>
  );
}
