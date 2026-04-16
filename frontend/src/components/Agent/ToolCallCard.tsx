import { useState } from "react";
import { CaretRight, Lightning } from "@phosphor-icons/react";
import { motion, AnimatePresence } from "framer-motion";
import { cn } from "@/lib/utils";
import PythonCode from "./PythonCode";
import { Button } from "@/components/ui/button";
import { CopyButton } from "@/components/ui/copy-button";

interface ToolCallCardProps {
  title: string;
  toolName: string | null;
  toolKind?: string | null;
  arguments: string | null;
  result: string | null;
  status: "running" | "completed" | "failed";
}



// --- Helpers ---

function cleanTitle(title: string): string {
  return title.replace(/^mcp:\s*\S+\s*/, "").trim();
}

function primaryLabel(
  title: string,
  toolName: string | null,
  toolKind: string | null | undefined,
): string {
  if (toolName) return toolName.replace(/_/g, " ");
  if (toolKind) return toolKind;
  return cleanTitle(title).replace(/\(.*\)$/, "").replace(/_/g, " ").trim();
}

function secondaryLabel(title: string, primary: string): string | null {
  const cleaned = cleanTitle(title);
  if (!cleaned) return null;
  const stripped = cleaned.replace(/^["']|["']$/g, "").trim();
  if (!stripped || stripped.toLowerCase() === "undefined" || stripped.toLowerCase() === "null") {
    return null;
  }
  const p = primary.toLowerCase().trim();
  const c = cleaned.toLowerCase();
  if (!p) return cleaned;
  if (c === p) return null;
  if (c.includes(p)) return null;
  return cleaned;
}

function extractPythonCode(args: string | null): string | null {
  if (!args) return null;
  try {
    const parsed = JSON.parse(args);
    if (typeof parsed.code === "string") return parsed.code;
  } catch {
    /* not JSON */
  }
  return null;
}

interface ParsedResult {
  success: boolean;
  stdout: string | null;
  result: unknown;
  error: string | null;
  functionCalls: { name: string; success: boolean }[] | null;
}

function parseExecutionResult(raw: string | null): ParsedResult | null {
  if (!raw) return null;
  try {
    let parsed = JSON.parse(raw);
    if (Array.isArray(parsed)) {
      const text = parsed.find((c: { type?: string }) => c.type === "text")?.text;
      if (text) {
        try {
          parsed = JSON.parse(text);
        } catch {
          return null;
        }
      } else {
        return null;
      }
    }
    return {
      success: parsed.success ?? true,
      stdout: parsed.stdout ?? null,
      result: parsed.result ?? null,
      error: parsed.error ?? null,
      functionCalls: parsed.function_calls ?? null,
    };
  } catch {
    return null;
  }
}

interface SubWorkerResult {
  status: string;
  run_id: string;
  worker_name: string;
  message?: string | null;
  error?: string | null;
}

function parseSubWorkerResult(result: unknown): SubWorkerResult | null {
  if (!result || typeof result !== "object") return null;
  const obj = result as Record<string, unknown>;
  if (
    typeof obj.run_id === "string" &&
    typeof obj.worker_name === "string" &&
    typeof obj.status === "string"
  ) {
    return obj as unknown as SubWorkerResult;
  }
  return null;
}

function prettyJson(raw: string): string {
  try {
    return JSON.stringify(JSON.parse(raw), null, 2);
  } catch {
    return raw;
  }
}

export default function ToolCallCard({
  title,
  toolName,
  toolKind,
  arguments: args,
  result,
  status,
}: ToolCallCardProps) {
  const [expanded, setExpanded] = useState(false);

  const isExecuteCode = toolName === "execute_code";
  const pythonCode = isExecuteCode ? extractPythonCode(args) : null;
  const execResult = isExecuteCode ? parseExecutionResult(result) : null;
  const subWorkerResult = execResult ? parseSubWorkerResult(execResult.result) : null;
  const hasDetails = !!(args || result);

  const primary = primaryLabel(title, toolName, toolKind);
  const secondary = secondaryLabel(title, primary);

  return (
    <div className="w-full">
      <Button
        type="button"
        variant="ghost"
        onClick={() => hasDetails && setExpanded(!expanded)}
        className={cn(
          "group h-auto w-full justify-start gap-3 px-0 py-1 text-left hover:bg-transparent hover:text-inherit",
          hasDetails ? "cursor-pointer" : "cursor-default",
        )}
      >
        {/* Minimal Status Indicator */}
        <div className="shrink-0 pt-0.5">
          {status === "running" ? (
            <div className="h-2 w-2 rounded-full bg-blue-500 animate-pulse" />
          ) : status === "completed" ? (
            <div className="h-1.5 w-1.5 rounded-full bg-green-500/50" />
          ) : (
            <div className="h-1.5 w-1.5 rounded-full bg-red-500/50" />
          )}
        </div>

        {/* Label */}
        <div className="flex flex-1 items-center gap-2 min-w-0 overflow-hidden">
          <span className="shrink-0 font-mono text-xs text-muted-foreground opacity-70 group-hover:opacity-100 transition-opacity">
            {primary}
          </span>
          {secondary && (
            <span className="flex-1 truncate min-w-0 text-xs text-muted-foreground/50">
              {secondary}
            </span>
          )}
          {status === "running" && (
            <span className="shrink-0 text-xs text-muted-foreground/40 italic">running...</span>
          )}
        </div>

        {/* Toggle Icon */}
        {hasDetails && (
          <motion.div
            animate={{ rotate: expanded ? 90 : 0 }}
            transition={{ duration: 0.2 }}
            className="text-muted-foreground/30 opacity-0 group-hover:opacity-100 transition-opacity"
          >
            <CaretRight weight="bold" size={10} />
          </motion.div>
        )}
      </Button>

      <AnimatePresence>
        {expanded && hasDetails && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.2, ease: "easeOut" }}
            className="overflow-hidden"
          >
            <div className="pl-4 py-2 space-y-4 text-xs">
              {/* Vertical line connector for details */}
              <div className="border-l border-border/40 pl-4 space-y-3">
                {/* Code Input */}
                {isExecuteCode && pythonCode ? (
                  <div className="space-y-1">
                    {/* Clean code block */}
                    <div className="rounded border border-border/30 bg-muted/20 overflow-hidden">
                      <PythonCode code={pythonCode} />
                    </div>
                  </div>
                ) : (
                  args && (
                    <div className="space-y-1">
                      <div className="flex items-center justify-between">
                        <span className="text-[10px] uppercase tracking-wider text-muted-foreground/60 font-medium">
                          Input
                        </span>
                        <CopyButton text={prettyJson(args)} />
                      </div>
                      <pre className="overflow-x-auto font-mono text-[11px] text-muted-foreground whitespace-pre-wrap leading-relaxed">
                        {prettyJson(args)}
                      </pre>
                    </div>
                  )
                )}

                {/* Result / Error */}
                {execResult ? (
                  <>
                    {execResult.stdout ? (
                      <div className="space-y-1">
                        <div className="flex items-center justify-between">
                          <span className="text-[10px] uppercase tracking-wider text-muted-foreground/60 font-medium">
                            Output
                          </span>
                          <CopyButton text={execResult.stdout} />
                        </div>
                        <div className="rounded border border-border/30 bg-muted/20 overflow-hidden">
                          <pre className="overflow-x-auto font-mono text-[11px] text-muted-foreground whitespace-pre-wrap leading-relaxed p-3">
                            {execResult.stdout}
                          </pre>
                        </div>
                      </div>
                    ) : null}
                    {execResult.error ? (
                      <div className="space-y-1">
                        <div className="flex items-center justify-between">
                          <span className="text-[10px] uppercase tracking-wider text-red-500/60 font-medium">
                            Error
                          </span>
                          <CopyButton text={execResult.error} />
                        </div>
                        <div className="rounded border border-red-500/20 bg-red-500/5 overflow-hidden">
                          <pre className="overflow-x-auto text-red-500/80 font-mono text-[11px] whitespace-pre-wrap p-3">
                            {execResult.error}
                          </pre>
                        </div>
                      </div>
                    ) : subWorkerResult ? (
                      <div className="space-y-1">
                        <span className="text-[10px] uppercase tracking-wider text-muted-foreground/60 font-medium">
                          Sub-Worker
                        </span>
                        <div className="flex items-center gap-2 text-[11px] text-muted-foreground">
                          <Lightning size={12} className="shrink-0" />
                          <span className="font-medium">{subWorkerResult.worker_name}</span>
                          <span
                            className={cn(
                              "text-[10px] px-1.5 py-0.5 rounded",
                              subWorkerResult.status === "completed"
                                ? "bg-green-500/10 text-green-500"
                                : subWorkerResult.status === "queued"
                                  ? "bg-blue-500/10 text-blue-500"
                                  : "bg-red-500/10 text-red-500",
                            )}
                          >
                            {subWorkerResult.status}
                          </span>
                        </div>
                        {subWorkerResult.error && (
                          <pre className="overflow-x-auto text-red-500/80 font-mono text-[11px] whitespace-pre-wrap mt-1">
                            {subWorkerResult.error}
                          </pre>
                        )}
                      </div>
                    ) : execResult.result !== null && execResult.result !== undefined ? (
                      <div className="space-y-1">
                        <span className="text-[10px] uppercase tracking-wider text-muted-foreground/60 font-medium">
                          Output
                        </span>
                        <pre className="overflow-x-auto font-mono text-[11px] text-muted-foreground whitespace-pre-wrap leading-relaxed">
                          {typeof execResult.result === "string"
                            ? execResult.result
                            : JSON.stringify(execResult.result, null, 2)}
                        </pre>
                      </div>
                    ) : null}
                  </>
                ) : (
                  result && (
                    <div className="space-y-1">
                      <div className="flex items-center justify-between">
                        <span className="text-[10px] uppercase tracking-wider text-muted-foreground/60 font-medium">
                          Result
                        </span>
                        <CopyButton text={prettyJson(result)} />
                      </div>
                      <pre className="overflow-x-auto font-mono text-[11px] text-muted-foreground whitespace-pre-wrap leading-relaxed">
                        {prettyJson(result)}
                      </pre>
                    </div>
                  )
                )}
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
