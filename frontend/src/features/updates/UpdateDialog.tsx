import { ArrowSquareOut, ArrowUp, CircleNotch, Copy, X } from "@phosphor-icons/react";
import { Dialog as DialogPrimitive } from "radix-ui";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import MarkdownMessage from "@/components/Agent/MarkdownMessage";
import { Eyebrow, HairlineDivider } from "@/components/ui/editorial";
import { useBackendEvent } from "@/hooks/useBackendEvent";
import { useCopyToClipboard } from "@/hooks/useCopyToClipboard";
import type { UpdateInfo } from "@/hooks/useUpdateCheck";
import { runSelfUpdate } from "@/shared/api/commands";

const UPDATE_COMMAND = "brew upgrade --cask bullpen";

interface UpdateLog {
  stream: "stdout" | "stderr";
  line: string;
}

type Phase = "idle" | "running" | "error";

interface UpdateDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  currentVersion: string;
  updateInfo: UpdateInfo;
}

export function UpdateDialog({
  open,
  onOpenChange,
  currentVersion,
  updateInfo,
}: UpdateDialogProps) {
  const [phase, setPhase] = useState<Phase>("idle");
  const [latestLog, setLatestLog] = useState<string>("");
  const [errorMessage, setErrorMessage] = useState<string>("");
  const { copied, copy } = useCopyToClipboard();

  useEffect(() => {
    if (!open) {
      setPhase("idle");
      setLatestLog("");
      setErrorMessage("");
    }
  }, [open]);

  useBackendEvent<UpdateLog>("update:log", (payload) => {
    if (payload.line.trim()) setLatestLog(payload.line);
  });

  useBackendEvent<{ message: string }>("update:error", (payload) => {
    setPhase("error");
    setErrorMessage(payload.message);
  });

  useBackendEvent<null>("update:done", () => {
    // App will restart shortly; keep the dialog in running state with last log line.
    setLatestLog("Update complete. Relaunching…");
  });

  const handleInstall = async () => {
    setPhase("running");
    setErrorMessage("");
    setLatestLog("Starting brew upgrade…");
    try {
      await runSelfUpdate();
    } catch (err) {
      setPhase("error");
      setErrorMessage(String(err));
    }
  };

  const handleCopy = async () => {
    try {
      await copy(UPDATE_COMMAND);
      toast("Copied to clipboard", {
        description: "Paste the command in your terminal to update.",
      });
    } catch (err) {
      toast.error("Copy failed", { description: String(err) });
    }
  };

  return (
    <DialogPrimitive.Root open={open} onOpenChange={onOpenChange}>
      <DialogPrimitive.Portal>
        <DialogPrimitive.Overlay className="fixed inset-0 z-50 bg-black/50 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:animate-in data-[state=open]:fade-in-0" />
        <DialogPrimitive.Content
          aria-describedby={undefined}
          className="fixed left-1/2 top-1/2 z-50 flex w-[min(560px,calc(100vw-2rem))] -translate-x-1/2 -translate-y-1/2 flex-col border border-border bg-background text-foreground data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:animate-in data-[state=open]:fade-in-0"
        >
          <header className="flex items-start justify-between gap-4 px-5 pt-5 pb-4">
            <div className="flex flex-col gap-2">
              <Eyebrow>Update available</Eyebrow>
              <DialogPrimitive.Title className="flex items-center gap-2 text-base font-semibold tracking-tight">
                <ArrowUp size={14} aria-hidden />
                <span>{updateInfo.releaseName || `v${updateInfo.latestVersion}`}</span>
              </DialogPrimitive.Title>
              <p className="font-mono text-[11.5px] tabular-nums text-muted-foreground">
                v{currentVersion} → v{updateInfo.latestVersion}
              </p>
            </div>
            <DialogPrimitive.Close
              className="text-muted-foreground transition-colors hover:text-foreground"
              aria-label="Close"
            >
              <X size={16} />
            </DialogPrimitive.Close>
          </header>

          <HairlineDivider />

          <section className="max-h-[42vh] overflow-y-auto px-5 py-4">
            <Eyebrow className="mb-3">What&rsquo;s new</Eyebrow>
            {updateInfo.releaseNotes ? (
              <MarkdownMessage text={updateInfo.releaseNotes} />
            ) : (
              <p className="text-[13px] italic text-muted-foreground">
                No release notes available for this version.
              </p>
            )}
          </section>

          <HairlineDivider />

          <section className="px-5 py-4">
            <Eyebrow className="mb-3">Manual command</Eyebrow>
            <div className="flex items-center justify-between border border-border px-3 py-2">
              <code className="font-mono text-[12px] tabular-nums text-foreground">
                {UPDATE_COMMAND}
              </code>
              <button
                type="button"
                onClick={handleCopy}
                className="flex items-center gap-1.5 font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground transition-colors hover:text-foreground"
              >
                <Copy size={12} aria-hidden />
                {copied ? "Copied" : "Copy"}
              </button>
            </div>
          </section>

          {phase === "running" && (
            <>
              <HairlineDivider />
              <section className="flex items-center gap-2 px-5 py-3">
                <CircleNotch size={12} className="animate-spin text-primary" aria-hidden />
                <span className="truncate font-mono text-[11.5px] text-muted-foreground">
                  {latestLog || "Working…"}
                </span>
              </section>
            </>
          )}

          {phase === "error" && (
            <>
              <HairlineDivider />
              <section className="px-5 py-3">
                <Eyebrow className="mb-2 text-destructive">Update failed</Eyebrow>
                <p className="font-mono text-[11.5px] text-destructive">{errorMessage}</p>
                <p className="mt-2 text-[12px] text-muted-foreground">
                  Run the manual command above in your terminal as a fallback.
                </p>
              </section>
            </>
          )}

          <HairlineDivider />

          <footer className="flex items-center justify-between gap-3 px-5 py-4">
            <a
              href={updateInfo.releaseUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-1.5 font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground transition-colors hover:text-foreground"
            >
              <ArrowSquareOut size={12} aria-hidden />
              View on GitHub
            </a>
            <div className="flex items-center gap-3">
              {phase === "error" && (
                <button
                  type="button"
                  onClick={handleInstall}
                  className="font-mono text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground transition-colors hover:text-foreground"
                >
                  Retry
                </button>
              )}
              <button
                type="button"
                onClick={handleInstall}
                disabled={phase === "running"}
                className="flex items-center gap-2 border border-foreground bg-foreground px-3 py-1.5 text-[12px] font-medium text-background transition-colors hover:bg-background hover:text-foreground disabled:cursor-not-allowed disabled:opacity-60 disabled:hover:bg-foreground disabled:hover:text-background"
              >
                {phase === "running" ? (
                  <>
                    <CircleNotch size={12} className="animate-spin" aria-hidden />
                    Installing…
                  </>
                ) : (
                  <>
                    <ArrowUp size={12} aria-hidden />
                    Install &amp; Restart
                  </>
                )}
              </button>
            </div>
          </footer>
        </DialogPrimitive.Content>
      </DialogPrimitive.Portal>
    </DialogPrimitive.Root>
  );
}
