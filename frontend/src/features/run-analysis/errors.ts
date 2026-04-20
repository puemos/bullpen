export interface NormalizedError {
  message: string;
  kind: string | null;
  details: string | null;
}

export function normalizeError(error: unknown): NormalizedError {
  if (typeof error === "string") {
    return { message: error, kind: null, details: null };
  }

  if (error instanceof Error) {
    return {
      message: error.message || "Unexpected error",
      kind: error.name || null,
      details: error.stack ?? null,
    };
  }

  if (error && typeof error === "object") {
    const record = error as Record<string, unknown>;
    const message = stringValue(record.message) ?? safeJson(error) ?? "Unexpected error";
    return {
      message,
      kind: stringValue(record.kind),
      details: detailsForObject(record, message),
    };
  }

  return { message: String(error), kind: null, details: null };
}

function stringValue(value: unknown) {
  return typeof value === "string" && value.trim() ? value : null;
}

function detailsForObject(record: Record<string, unknown>, message: string) {
  const details = stringValue(record.details) ?? stringValue(record.stack);
  if (details) return details;

  const rest = { ...record };
  delete rest.message;
  delete rest.kind;
  if (Object.keys(rest).length === 0) return null;
  return safeJson(rest) ?? message;
}

function safeJson(value: unknown) {
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return null;
  }
}
