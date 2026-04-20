import type { Projection, ProjectionScenario } from "@/types";

export function projectionUsesPercentPoints(projection: Projection): boolean {
  const unit = projection.unit.trim().toLowerCase();
  return (
    unit === "%" || unit === "percent" || unit === "percentage" || projection.metric.includes("(%)")
  );
}

export function formatProjectionTargetLabel(
  projection: Projection,
  scenario: ProjectionScenario,
): string {
  const label = scenario.target_label.trim();
  if (/\d/.test(label) && (!projectionUsesPercentPoints(projection) || label.includes("%"))) {
    return label;
  }
  return formatProjectionValue(projection, scenario.target_value);
}

export function formatProjectionMovement(
  projection: Projection,
  scenario: ProjectionScenario,
): string | null {
  if (projectionUsesPercentPoints(projection)) return null;
  if (!Number.isFinite(projection.current_value) || Math.abs(projection.current_value) < 1e-9) {
    return null;
  }
  const pct = ((scenario.target_value - projection.current_value) / projection.current_value) * 100;
  const sign = pct >= 0 ? "+" : "";
  return `${sign}${pct.toFixed(1)}%`;
}

function formatProjectionValue(projection: Projection, value: number): string {
  if (projectionUsesPercentPoints(projection)) {
    const sign = signedPercentMetric(projection.metric) && value >= 0 ? "+" : "";
    return `${sign}${value.toFixed(1)}%`;
  }

  const unit = projection.unit.trim();
  if (unit.toUpperCase() === "USD" || unit === "$") return `$${formatNumeric(value)}`;
  if (!unit) return formatNumeric(value);
  return `${formatNumeric(value)} ${unit}`;
}

function signedPercentMetric(metric: string): boolean {
  const normalized = metric.toLowerCase();
  return (
    normalized.includes("return") ||
    normalized.includes("change") ||
    normalized.includes("upside") ||
    normalized.includes("downside") ||
    normalized.includes("impact")
  );
}

function formatNumeric(value: number): string {
  const abs = Math.abs(value);
  const maximumFractionDigits = abs >= 1000 ? 0 : abs >= 10 ? 1 : 2;
  return new Intl.NumberFormat(undefined, {
    maximumFractionDigits,
  }).format(value);
}
