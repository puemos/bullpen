use crate::domain::analysis::{
    AnalysisBlock, AnalysisReport, CounterThesis, DecisionCriterionAnswer, FinalStance,
    MetricSnapshot, Projection, StanceKind, StructuredArtifact,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

/// Buckets used by the UI to colour-grade freshness chips. These are display
/// thresholds — the finalize gate uses a separate, much looser cap
/// (`STANCE_MAX_METRIC_AGE_DAYS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreshnessBucket {
    Fresh,
    Aging,
    Stale,
    VeryStale,
}

impl FreshnessBucket {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Aging => "aging",
            Self::Stale => "stale",
            Self::VeryStale => "very_stale",
        }
    }
}

/// Verification status for a source URL, populated by `verify_source_accessibility`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Ok,
    Redirect,
    Dead,
    Timeout,
    Forbidden,
}

impl fmt::Display for VerificationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Ok => "ok",
            Self::Redirect => "redirect",
            Self::Dead => "dead",
            Self::Timeout => "timeout",
            Self::Forbidden => "forbidden",
        };
        write!(f, "{value}")
    }
}

impl FromStr for VerificationStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "ok" => Ok(Self::Ok),
            "redirect" => Ok(Self::Redirect),
            "dead" => Ok(Self::Dead),
            "timeout" => Ok(Self::Timeout),
            "forbidden" => Ok(Self::Forbidden),
            other => Err(format!("unknown verification status: {other}")),
        }
    }
}

/// Parse an ISO-8601 / RFC 3339 timestamp string into a UTC `DateTime`. Falls
/// back to a date-only (`YYYY-MM-DD`) parse so metric `as_of` values written
/// without a time component still bucket correctly.
#[must_use]
pub fn parse_iso(stamp: &str) -> Option<DateTime<Utc>> {
    let trimmed = stamp.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(date) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
        && let Some(noon) = date.and_hms_opt(12, 0, 0)
    {
        return Some(DateTime::<Utc>::from_naive_utc_and_offset(noon, Utc));
    }
    None
}

/// Returns the whole-day age of `stamp` relative to `now`. Negative durations
/// (stamps in the future, e.g. a filing dated tomorrow) clamp to 0 — we never
/// want to treat a future timestamp as "extra stale."
#[must_use]
pub fn age_days(stamp: &str, now: DateTime<Utc>) -> Option<i64> {
    let dt = parse_iso(stamp)?;
    let delta = now - dt;
    Some(delta.num_days().max(0))
}

/// Bucket an age in days into a display tier. See [`FreshnessBucket`] for the
/// rationale.
#[must_use]
pub fn freshness_bucket(days: i64) -> FreshnessBucket {
    match days {
        ..=7 => FreshnessBucket::Fresh,
        8..=30 => FreshnessBucket::Aging,
        31..=180 => FreshnessBucket::Stale,
        _ => FreshnessBucket::VeryStale,
    }
}

/// Environment-overridable finalize-gate threshold. Intentionally generous
/// (12 months) in v1 to avoid blocking existing runs; per-metric-kind tuning
/// is v2.
pub const DEFAULT_STANCE_MAX_METRIC_AGE_DAYS: i64 = 365;

#[must_use]
pub fn stance_max_metric_age_days() -> i64 {
    std::env::var("BULLPEN_MAX_METRIC_AGE_DAYS")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(DEFAULT_STANCE_MAX_METRIC_AGE_DAYS)
}

/// One flagged metric cited by the stance's evidence graph with an `as_of`
/// older than [`stance_max_metric_age_days`]. `age_days` and `max_days` are
/// kept with the entry so the finalize gate can surface them verbatim in its
/// error message.
#[derive(Debug, Clone)]
pub struct StaleStanceMetric {
    pub metric: String,
    pub source_id: String,
    pub age_days: i64,
    pub max_days: i64,
}

/// Inputs to the stance-freshness walk. Grouped so callers don't wire up a
/// 7-argument function and so v2 extensions (research-plan-level citations,
/// etc.) can extend the struct without touching every call site.
pub struct StanceFreshnessInputs<'a> {
    pub stance: Option<&'a FinalStance>,
    pub blocks: &'a [AnalysisBlock],
    pub projections: &'a [Projection],
    pub artifacts: &'a [StructuredArtifact],
    pub counter_theses: &'a [CounterThesis],
    pub criterion_answers: &'a [DecisionCriterionAnswer],
    pub metrics: &'a [MetricSnapshot],
}

impl<'a> StanceFreshnessInputs<'a> {
    #[must_use]
    pub fn from_report(report: &'a AnalysisReport) -> Self {
        Self {
            stance: report.final_stance.as_ref(),
            blocks: &report.blocks,
            projections: &report.projections,
            artifacts: &report.artifacts,
            counter_theses: &report.counter_theses,
            criterion_answers: &report.decision_criterion_answers,
            metrics: &report.metrics,
        }
    }
}

/// Stance-cited metrics older than the configured freshness cap. Returns an
/// empty vec for neutral / insufficient-data stances (those are not making a
/// call, so stale data is not a hazard).
///
/// Shared by the finalize gate (rejects the run) and the report viewer
/// (loud banner). Keeping the evidence-graph walk in one place prevents
/// Rust/TS from drifting as the graph grows.
#[must_use]
pub fn stale_stance_metrics(
    inputs: &StanceFreshnessInputs<'_>,
    now: DateTime<Utc>,
) -> Vec<StaleStanceMetric> {
    let Some(stance) = inputs.stance else {
        return Vec::new();
    };
    if matches!(
        stance.stance,
        StanceKind::Neutral | StanceKind::InsufficientData
    ) {
        return Vec::new();
    }

    let mut cited: HashSet<&str> = HashSet::new();
    for block in inputs.blocks {
        cited.extend(block.evidence_ids.iter().map(String::as_str));
    }
    for projection in inputs.projections {
        cited.extend(projection.evidence_ids.iter().map(String::as_str));
    }
    for artifact in inputs.artifacts {
        cited.extend(artifact.evidence_ids.iter().map(String::as_str));
    }
    for counter in inputs.counter_theses {
        cited.extend(counter.supporting_evidence_ids.iter().map(String::as_str));
    }
    for answer in inputs.criterion_answers {
        cited.extend(answer.supporting_evidence_ids.iter().map(String::as_str));
    }

    let max_days = stance_max_metric_age_days();
    let mut out = Vec::new();
    for metric in inputs.metrics {
        if !cited.contains(metric.source_id.as_str()) {
            continue;
        }
        let Some(age) = age_days(&metric.as_of, now) else {
            continue;
        };
        if age > max_days {
            out.push(StaleStanceMetric {
                metric: metric.metric.clone(),
                source_id: metric.source_id.clone(),
                age_days: age,
                max_days,
            });
        }
    }
    out
}

/// Report-level convenience — returns only the metric names for UI banners.
#[must_use]
pub fn stance_stale_metric_names(report: &AnalysisReport, now: DateTime<Utc>) -> Vec<String> {
    stale_stance_metrics(&StanceFreshnessInputs::from_report(report), now)
        .into_iter()
        .map(|m| m.metric)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn utc(y: i32, m: u32, d: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, 12, 0, 0).unwrap()
    }

    #[test]
    fn parse_rfc3339_with_offset() {
        let dt = parse_iso("2026-01-15T08:00:00-05:00").unwrap();
        // 08:00 EST == 13:00 UTC
        assert_eq!(dt.to_rfc3339(), "2026-01-15T13:00:00+00:00");
    }

    #[test]
    fn parse_date_only_falls_back_to_noon_utc() {
        let dt = parse_iso("2026-02-01").unwrap();
        assert_eq!(dt, utc(2026, 2, 1));
    }

    #[test]
    fn parse_malformed_returns_none() {
        assert!(parse_iso("").is_none());
        assert!(parse_iso("not-a-date").is_none());
        assert!(parse_iso("2026/02/01").is_none());
    }

    #[test]
    fn age_days_future_stamp_clamps_to_zero() {
        let now = utc(2026, 4, 1);
        // Stamp five days in the future.
        let future = utc(2026, 4, 6).to_rfc3339();
        assert_eq!(age_days(&future, now), Some(0));
    }

    #[test]
    fn age_days_whole_days_past() {
        let now = utc(2026, 4, 18);
        let past = utc(2026, 4, 10).to_rfc3339();
        assert_eq!(age_days(&past, now), Some(8));
    }

    #[test]
    fn buckets_track_thresholds() {
        assert_eq!(freshness_bucket(0), FreshnessBucket::Fresh);
        assert_eq!(freshness_bucket(7), FreshnessBucket::Fresh);
        assert_eq!(freshness_bucket(8), FreshnessBucket::Aging);
        assert_eq!(freshness_bucket(30), FreshnessBucket::Aging);
        assert_eq!(freshness_bucket(31), FreshnessBucket::Stale);
        assert_eq!(freshness_bucket(180), FreshnessBucket::Stale);
        assert_eq!(freshness_bucket(181), FreshnessBucket::VeryStale);
        assert_eq!(freshness_bucket(10_000), FreshnessBucket::VeryStale);
    }

    #[test]
    fn verification_status_round_trips() {
        for status in [
            VerificationStatus::Ok,
            VerificationStatus::Redirect,
            VerificationStatus::Dead,
            VerificationStatus::Timeout,
            VerificationStatus::Forbidden,
        ] {
            let parsed = VerificationStatus::from_str(&status.to_string()).unwrap();
            assert_eq!(parsed, status);
        }
    }
}
