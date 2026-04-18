import { useEffect, useMemo, useState } from "react";
import { Eyebrow, HairlineDivider, SectionHeader } from "@/components/ui/editorial";
import { Input } from "@/components/ui/input";
import {
  clearSourceKey,
  listSources,
  refreshSourceKeyStatus,
  setEnabledSources,
  setSourceKey,
  testSourceKey,
} from "@/shared/api/commands";
import type { SourceCategoryId, SourceDescriptor } from "@/types";

const CATEGORY_ORDER: SourceCategoryId[] = [
  "web_search",
  "filings",
  "fundamentals",
  "market_data",
  "news",
  "forums",
  "screener",
];

const CATEGORY_LABEL: Record<SourceCategoryId, string> = {
  web_search: "Web Search",
  filings: "Filings",
  fundamentals: "Fundamentals",
  market_data: "Market Data",
  news: "News",
  forums: "Forums",
  screener: "Screener",
};

type TestState = { status: string; message: string } | null;

export function DataSourcesSection() {
  const [sources, setSources] = useState<SourceDescriptor[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [draftKey, setDraftKey] = useState<Record<string, string>>({});
  const [pending, setPending] = useState<Record<string, boolean>>({});
  const [testResults, setTestResults] = useState<Record<string, TestState>>({});

  const load = async () => {
    try {
      const list = await listSources();
      setSources(list);
    } catch (err) {
      setError(String(err));
    }
  };

  useEffect(() => {
    void load();
  }, []);

  const grouped = useMemo(() => {
    const out: Record<string, SourceDescriptor[]> = {};
    if (!sources) return out;
    for (const category of CATEGORY_ORDER) out[category] = [];
    for (const src of sources) {
      (out[src.category] ||= []).push(src);
    }
    return out;
  }, [sources]);

  const activeCategories = useMemo(
    () => CATEGORY_ORDER.filter((c) => (grouped[c] ?? []).length > 0),
    [grouped],
  );

  const setBusy = (id: string, busy: boolean) =>
    setPending((prev) => ({ ...prev, [id]: busy }));

  const onSaveKey = async (id: string) => {
    const key = draftKey[id]?.trim();
    if (!key) return;
    setBusy(id, true);
    try {
      await setSourceKey(id, key);
      setDraftKey((prev) => ({ ...prev, [id]: "" }));
      await refreshSourceKeyStatus();
      await load();
    } catch (err) {
      setTestResults((prev) => ({
        ...prev,
        [id]: { status: "error", message: String(err) },
      }));
    } finally {
      setBusy(id, false);
    }
  };

  const onClearKey = async (id: string) => {
    setBusy(id, true);
    try {
      await clearSourceKey(id);
      await load();
    } catch (err) {
      setTestResults((prev) => ({
        ...prev,
        [id]: { status: "error", message: String(err) },
      }));
    } finally {
      setBusy(id, false);
    }
  };

  const onTestKey = async (id: string) => {
    setBusy(id, true);
    try {
      const result = await testSourceKey(id);
      setTestResults((prev) => ({ ...prev, [id]: result }));
      setTimeout(
        () => setTestResults((prev) => ({ ...prev, [id]: null })),
        4000,
      );
    } catch (err) {
      setTestResults((prev) => ({
        ...prev,
        [id]: { status: "error", message: String(err) },
      }));
    } finally {
      setBusy(id, false);
    }
  };

  const onToggleEnabled = async (id: string, enabled: boolean) => {
    if (!sources) return;
    const next = sources
      .filter((s) => (s.id === id ? enabled : s.enabled))
      .map((s) => s.id);
    setBusy(id, true);
    try {
      await setEnabledSources(next);
      await load();
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(id, false);
    }
  };

  const onRefresh = async () => {
    try {
      const updated = await refreshSourceKeyStatus();
      setSources(updated);
    } catch (err) {
      setError(String(err));
    }
  };

  const enabledCount = sources?.filter((s) => s.enabled).length ?? 0;
  const keyedCount = sources?.filter((s) => s.has_key).length ?? 0;

  return (
    <section className="space-y-10">
      <SectionHeader
        number="03"
        label="Data Sources"
        title="Provider registry"
        meta={
          sources && (
            <span className="flex items-center gap-3 font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground">
              <span className="tabular-nums">
                {String(enabledCount).padStart(2, "0")} enabled
              </span>
              <span aria-hidden className="h-3 w-px bg-border" />
              <span className="tabular-nums">
                {String(keyedCount).padStart(2, "0")} keyed
              </span>
              <span aria-hidden className="h-3 w-px bg-border" />
              <button
                type="button"
                onClick={onRefresh}
                className="hover:text-foreground"
              >
                Refresh
              </button>
            </span>
          )
        }
      />
      <p className="max-w-[60ch] text-[14px] leading-[1.6] text-muted-foreground">
        Enable the providers the agent may call during a research run. Paid-tier keys
        live in your OS keychain — Bullpen never writes them to disk. Disable a
        provider globally here, or flip it off for a single run from the composer.
      </p>
      {error && <div className="text-sm text-destructive">{error}</div>}
      {!sources ? (
        <div className="text-sm text-muted-foreground">Loading…</div>
      ) : (
        <div className="space-y-10">
          {activeCategories.map((category, idx) => {
            const rows = grouped[category] ?? [];
            const enabledInCat = rows.filter((s) => s.enabled).length;
            return (
              <div key={category} className="space-y-4">
                {idx > 0 && <HairlineDivider />}
                <header className="flex items-baseline justify-between gap-4 pt-2">
                  <div className="flex items-baseline gap-3">
                    <span className="font-mono text-[10.5px] font-medium tabular-nums text-muted-foreground">
                      {String(idx + 1).padStart(2, "0")}
                    </span>
                    <Eyebrow>{CATEGORY_LABEL[category]}</Eyebrow>
                  </div>
                  <span className="font-mono text-[10.5px] uppercase tracking-[0.14em] tabular-nums text-muted-foreground">
                    {String(enabledInCat).padStart(2, "0")} / {String(rows.length).padStart(2, "0")}
                  </span>
                </header>
                <div className="divide-y divide-border border-t border-border">
                  {rows.map((src) => (
                    <ProviderRow
                      key={src.id}
                      src={src}
                      busy={!!pending[src.id]}
                      draft={draftKey[src.id] ?? ""}
                      testResult={testResults[src.id]}
                      onDraftChange={(value) =>
                        setDraftKey((prev) => ({ ...prev, [src.id]: value }))
                      }
                      onSave={() => void onSaveKey(src.id)}
                      onClear={() => void onClearKey(src.id)}
                      onTest={() => void onTestKey(src.id)}
                      onToggleEnabled={(enabled) =>
                        void onToggleEnabled(src.id, enabled)
                      }
                    />
                  ))}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </section>
  );
}

interface ProviderRowProps {
  src: SourceDescriptor;
  busy: boolean;
  draft: string;
  testResult: TestState;
  onDraftChange: (value: string) => void;
  onSave: () => void;
  onClear: () => void;
  onTest: () => void;
  onToggleEnabled: (enabled: boolean) => void;
}

function ProviderRow({
  src,
  busy,
  draft,
  testResult,
  onDraftChange,
  onSave,
  onClear,
  onTest,
  onToggleEnabled,
}: ProviderRowProps) {
  const hasDraft = draft.trim().length > 0;
  const canTest = !src.requires_key || src.has_key;

  return (
    <div className="grid grid-cols-[1fr_auto] items-start gap-x-8 gap-y-3 py-5">
      <div className="min-w-0 space-y-1">
        <div className="flex items-baseline gap-3">
          <Eyebrow className="tracking-[0.14em]">
            {src.requires_key ? (src.has_key ? "Key stored" : "Key required") : "No key"}
          </Eyebrow>
          {src.rate_limit_hint && (
            <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground/80">
              {src.rate_limit_hint}
            </span>
          )}
        </div>
        <h3 className="text-[18px] font-semibold leading-[1.2] tracking-[-0.01em]">
          {src.display_name}
        </h3>
        <p className="max-w-[60ch] text-[13px] leading-[1.55] text-muted-foreground">
          {src.description}
        </p>
      </div>
      <EnabledToggle
        enabled={src.enabled}
        disabled={busy}
        onChange={onToggleEnabled}
      />
      <div className="col-span-2 space-y-3 pt-1">
        <div className="flex flex-wrap items-center gap-x-3 gap-y-2 font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground">
          <a
            href={src.docs_url}
            target="_blank"
            rel="noreferrer"
            className="underline-offset-[3px] hover:text-foreground hover:underline"
          >
            Docs
          </a>
          {src.key_acquisition_url && (
            <>
              <span aria-hidden className="h-3 w-px bg-border" />
              <a
                href={src.key_acquisition_url}
                target="_blank"
                rel="noreferrer"
                className="underline-offset-[3px] hover:text-foreground hover:underline"
              >
                Get key
              </a>
            </>
          )}
          <span aria-hidden className="h-3 w-px bg-border" />
          <span>{src.id}</span>
        </div>

        {src.requires_key ? (
          <div className="flex flex-wrap items-center gap-3">
            <Input
              type="password"
              autoComplete="off"
              spellCheck={false}
              value={draft}
              placeholder={src.has_key ? "Replace stored key" : "Paste API key"}
              onChange={(event) => onDraftChange(event.target.value)}
              className="h-9 max-w-sm flex-1 bg-transparent font-mono text-[13px]"
            />
            <button
              type="button"
              disabled={busy || !hasDraft}
              onClick={onSave}
              className="inline-flex h-9 items-center border border-foreground bg-foreground px-4 font-mono text-[10.5px] uppercase tracking-[0.14em] text-background transition-colors hover:bg-background hover:text-foreground disabled:cursor-not-allowed disabled:border-border disabled:bg-transparent disabled:text-muted-foreground/50"
            >
              {src.has_key ? "Replace" : "Save"}
            </button>
            <span aria-hidden className="h-4 w-px bg-border" />
            <button
              type="button"
              disabled={busy || !canTest}
              onClick={onTest}
              className="font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground transition-colors hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
            >
              Test
            </button>
            {src.has_key && (
              <>
                <span aria-hidden className="h-4 w-px bg-border" />
                <button
                  type="button"
                  disabled={busy}
                  onClick={onClear}
                  className="font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground transition-colors hover:text-destructive disabled:cursor-not-allowed disabled:opacity-40"
                >
                  Clear
                </button>
              </>
            )}
            {testResult && (
              <span
                className={
                  "ml-auto font-mono text-[10.5px] uppercase tracking-[0.14em] tabular-nums " +
                  (testResult.status === "ok"
                    ? "text-foreground"
                    : "text-destructive")
                }
              >
                {testResult.status === "ok"
                  ? "OK"
                  : `FAIL · ${testResult.message.slice(0, 40).toUpperCase()}`}
              </span>
            )}
          </div>
        ) : (
          <div className="flex items-center gap-3">
            <button
              type="button"
              disabled={busy}
              onClick={onTest}
              className="font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground transition-colors hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
            >
              Test reachability
            </button>
            {testResult && (
              <span
                className={
                  "font-mono text-[10.5px] uppercase tracking-[0.14em] tabular-nums " +
                  (testResult.status === "ok"
                    ? "text-foreground"
                    : "text-destructive")
                }
              >
                {testResult.status === "ok"
                  ? "OK"
                  : `FAIL · ${testResult.message.slice(0, 40).toUpperCase()}`}
              </span>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

interface EnabledToggleProps {
  enabled: boolean;
  disabled: boolean;
  onChange: (enabled: boolean) => void;
}

function EnabledToggle({ enabled, disabled, onChange }: EnabledToggleProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={enabled}
      disabled={disabled}
      onClick={() => onChange(!enabled)}
      className={
        "inline-flex h-7 items-center gap-2 border px-2.5 font-mono text-[10.5px] uppercase tracking-[0.14em] transition-colors disabled:cursor-not-allowed disabled:opacity-50 " +
        (enabled
          ? "border-foreground bg-foreground text-background hover:bg-background hover:text-foreground"
          : "border-border text-muted-foreground hover:border-foreground hover:text-foreground")
      }
    >
      <span
        aria-hidden
        className={
          "h-1.5 w-1.5 " + (enabled ? "bg-background" : "bg-muted-foreground/60")
        }
      />
      <span>{enabled ? "Enabled" : "Disabled"}</span>
    </button>
  );
}
