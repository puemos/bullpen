import { describe, expect, it } from "vitest";
import type { Projection, ProjectionScenario } from "@/types";
import {
  formatProjectionMovement,
  formatProjectionTargetLabel,
  projectionUsesPercentPoints,
} from "./projection-format";

function projection(overrides: Partial<Projection> = {}): Projection {
  return {
    id: "projection-1",
    run_id: "run-1",
    entity_id: "VGWE",
    horizon: "3-5 years",
    metric: "Annualized Total Return (%)",
    current_value: 1,
    current_value_label: "Index Level",
    unit: "%",
    scenarios: [],
    methodology: "Return bands",
    key_assumptions: [],
    evidence_ids: [],
    confidence: 0.7,
    disclaimer: "",
    created_at: "",
    ...overrides,
  };
}

function scenario(overrides: Partial<ProjectionScenario> = {}): ProjectionScenario {
  return {
    label: "bull",
    target_value: 12,
    target_label: "Target Return",
    probability: 0.25,
    rationale: "",
    catalysts: [],
    risks: [],
    ...overrides,
  };
}

describe("projection formatting", () => {
  it("treats percent-unit targets as percent points, not relative upside", () => {
    const p = projection();
    const s = scenario();

    expect(projectionUsesPercentPoints(p)).toBe(true);
    expect(formatProjectionTargetLabel(p, s)).toBe("+12.0%");
    expect(formatProjectionMovement(p, s)).toBeNull();
  });

  it("keeps relative movement for absolute projections", () => {
    const p = projection({
      metric: "Data Center Revenue",
      current_value: 16.6,
      current_value_label: "$16.6B",
      unit: "$B",
    });
    const s = scenario({ target_value: 26, target_label: "$26B" });

    expect(projectionUsesPercentPoints(p)).toBe(false);
    expect(formatProjectionTargetLabel(p, s)).toBe("$26B");
    expect(formatProjectionMovement(p, s)).toBe("+56.6%");
  });

  it("allows explicit 800 percent targets without collapsing units", () => {
    const p = projection();
    const s = scenario({ target_value: 800, target_label: "" });

    expect(formatProjectionTargetLabel(p, s)).toBe("+800.0%");
  });

  it("regenerates numeric percent labels that omit the percent sign", () => {
    const p = projection();
    const s = scenario({ target_value: 12, target_label: "12" });

    expect(formatProjectionTargetLabel(p, s)).toBe("+12.0%");
  });
});
