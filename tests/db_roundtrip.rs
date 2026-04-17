//! End-to-end tests against a real SQLite database on disk.
//!
//! These complement the `#[cfg(test)]` unit tests in `src/infra/db/mod.rs` by
//! exercising the public `Database` API from outside the crate, catching
//! regressions in the published surface (visibility, `Clone`, `Send`).

use bullpen::domain::{
    Analysis, AnalysisIntent, AnalysisRun, AnalysisStatus, Entity, ResearchPlan, Source,
    SourceReliability,
};
use bullpen::infra::db::Database;
use bullpen::infra::progress::ProgressEventPayload;
use tempfile::TempDir;

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn open_in_tempdir() -> (Database, TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("db.sqlite");
    let db = Database::open_at(path).unwrap();
    (db, dir)
}

fn analysis_fixture(id: &str) -> Analysis {
    Analysis {
        id: id.into(),
        title: "Test analysis".into(),
        user_prompt: "What is NVDA doing?".into(),
        intent: AnalysisIntent::SingleEquity,
        status: AnalysisStatus::Running,
        active_run_id: None,
        created_at: now(),
        updated_at: now(),
    }
}

fn run_fixture(id: &str, analysis_id: &str) -> AnalysisRun {
    AnalysisRun {
        id: id.into(),
        analysis_id: analysis_id.into(),
        agent_id: "codex".into(),
        model_id: None,
        prompt_text: "What is NVDA doing?".into(),
        status: AnalysisStatus::Running,
        started_at: now(),
        completed_at: None,
        error: None,
    }
}

fn source_fixture(id: &str, run_id: &str, reliability: SourceReliability) -> Source {
    Source {
        id: id.into(),
        run_id: run_id.into(),
        title: format!("Source {id}"),
        url: Some("https://example.com".into()),
        publisher: Some("Example".into()),
        source_type: "filing".into(),
        retrieved_at: now(),
        reliability,
        summary: "Primary source".into(),
    }
}

#[test]
fn open_twice_is_idempotent_and_preserves_data() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("db.sqlite");

    {
        let db = Database::open_at(path.clone()).unwrap();
        db.save_analysis(&analysis_fixture("a1")).unwrap();
    }
    // Re-opening runs init() again; the INSERT above must survive.
    let db = Database::open_at(path).unwrap();
    let summaries = db.list_analyses().unwrap();
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].id, "a1");
}

#[test]
fn save_and_list_analyses_roundtrip() {
    let (db, _dir) = open_in_tempdir();
    db.save_analysis(&analysis_fixture("a1")).unwrap();
    db.save_analysis(&analysis_fixture("a2")).unwrap();

    let list = db.list_analyses().unwrap();
    assert_eq!(list.len(), 2);
    let ids: Vec<_> = list.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&"a1"));
    assert!(ids.contains(&"a2"));
}

#[test]
fn run_lifecycle_updates_analysis_status() {
    let (db, _dir) = open_in_tempdir();
    db.save_analysis(&analysis_fixture("a1")).unwrap();
    db.save_run(&run_fixture("r1", "a1")).unwrap();
    db.set_active_run_if_empty("a1", "r1").unwrap();

    db.update_run_status("r1", AnalysisStatus::Completed, None)
        .unwrap();
    db.recompute_analysis_status("a1").unwrap();

    let list = db.list_analyses().unwrap();
    let a1 = list.iter().find(|s| s.id == "a1").unwrap();
    assert_eq!(a1.status, AnalysisStatus::Completed);
}

#[test]
fn delete_analysis_cascades_runs_and_entities() {
    let (db, _dir) = open_in_tempdir();
    db.save_analysis(&analysis_fixture("a1")).unwrap();
    db.save_run(&run_fixture("r1", "a1")).unwrap();
    db.save_entity(&Entity {
        id: "e1".into(),
        run_id: "r1".into(),
        symbol: Some("NVDA".into()),
        name: "Nvidia".into(),
        exchange: Some("NASDAQ".into()),
        asset_type: "equity".into(),
        sector: Some("Technology".into()),
        country: Some("US".into()),
        confidence: 0.9,
        resolution_notes: None,
    })
    .unwrap();

    db.delete_analysis("a1").unwrap();

    assert_eq!(db.list_analyses().unwrap().len(), 0);
    // Getting a report for a deleted analysis returns None, not an error.
    assert!(db.get_report("a1", None).unwrap().is_none());
}

#[test]
fn progress_events_roundtrip_with_insertion_order() {
    let (db, _dir) = open_in_tempdir();
    db.save_analysis(&analysis_fixture("a1")).unwrap();
    db.save_run(&run_fixture("r1", "a1")).unwrap();

    db.append_progress_event("r1", &ProgressEventPayload::Log("one".into()))
        .unwrap();
    db.append_progress_event(
        "r1",
        &ProgressEventPayload::MessageDelta {
            id: "m1".into(),
            delta: "hi".into(),
        },
    )
    .unwrap();
    db.append_progress_event("r1", &ProgressEventPayload::Completed)
        .unwrap();

    let events = db.get_run_progress("r1").unwrap();
    assert_eq!(events.len(), 3);
    assert!(matches!(events[0], ProgressEventPayload::Log(_)));
    assert!(matches!(
        events[1],
        ProgressEventPayload::MessageDelta { .. }
    ));
    assert!(matches!(events[2], ProgressEventPayload::Completed));
}

#[test]
fn research_plan_roundtrip() {
    let (db, _dir) = open_in_tempdir();
    db.save_analysis(&analysis_fixture("a1")).unwrap();
    db.save_run(&run_fixture("r1", "a1")).unwrap();

    let plan = ResearchPlan {
        id: "p1".into(),
        run_id: "r1".into(),
        intent: AnalysisIntent::SingleEquity,
        summary: "Assess NVDA".into(),
        decision_criteria: vec!["Valuation".into(), "Growth".into()],
        planned_checks: vec!["10-Q review".into()],
        created_at: now(),
    };
    db.save_research_plan(&plan).unwrap();
    let report = db.get_report("a1", None).unwrap().unwrap();
    assert_eq!(
        report.research_plan.as_ref().map(|p| p.id.as_str()),
        Some("p1")
    );
}

#[test]
fn database_clone_shares_connection() {
    let (db, _dir) = open_in_tempdir();
    let other = db.clone();

    db.save_analysis(&analysis_fixture("a1")).unwrap();
    // Writes through the clone are visible through the original.
    other.save_analysis(&analysis_fixture("a2")).unwrap();

    let list = db.list_analyses().unwrap();
    assert_eq!(list.len(), 2);
}

#[test]
fn source_with_all_reliability_levels_roundtrips() {
    let (db, _dir) = open_in_tempdir();
    db.save_analysis(&analysis_fixture("a1")).unwrap();
    db.save_run(&run_fixture("r1", "a1")).unwrap();

    for (id, rel) in [
        ("s-primary", SourceReliability::Primary),
        ("s-high", SourceReliability::High),
        ("s-medium", SourceReliability::Medium),
        ("s-low", SourceReliability::Low),
    ] {
        db.save_source(&source_fixture(id, "r1", rel)).unwrap();
    }

    let report = db.get_report("a1", None).unwrap().unwrap();
    assert_eq!(report.sources.len(), 4);
}

fn seed_analysis_with_runs(db: &Database, analysis_id: &str, run_statuses: &[AnalysisStatus]) {
    db.save_analysis(&analysis_fixture(analysis_id)).unwrap();
    for (idx, status) in run_statuses.iter().enumerate() {
        let run_id = format!("{analysis_id}-r{idx}");
        let mut run = run_fixture(&run_id, analysis_id);
        run.status = *status;
        db.save_run(&run).unwrap();
    }
}

fn analysis_status(db: &Database, analysis_id: &str) -> AnalysisStatus {
    db.list_analyses()
        .unwrap()
        .into_iter()
        .find(|s| s.id == analysis_id)
        .map(|s| s.status)
        .unwrap()
}

#[test]
fn recompute_analysis_status_precedence_matrix() {
    use AnalysisStatus::{Cancelled, Completed, Failed, Queued, Running};

    let cases: &[(&str, &[AnalysisStatus], AnalysisStatus)] = &[
        (
            "running-wins-over-completed",
            &[Running, Completed],
            Running,
        ),
        ("running-wins-over-failed", &[Running, Failed], Running),
        (
            "running-wins-over-cancelled",
            &[Running, Cancelled],
            Running,
        ),
        ("failed-wins-over-completed", &[Failed, Completed], Failed),
        ("failed-wins-over-cancelled", &[Failed, Cancelled], Failed),
        ("all-cancelled", &[Cancelled, Cancelled], Cancelled),
        (
            "cancelled-with-completed-is-completed",
            &[Cancelled, Completed],
            Completed,
        ),
        ("completed-with-queued", &[Completed, Queued], Completed),
        ("only-queued", &[Queued, Queued], Queued),
    ];

    for (label, statuses, expected) in cases {
        let (db, _dir) = open_in_tempdir();
        seed_analysis_with_runs(&db, label, statuses);
        db.recompute_analysis_status(label).unwrap();
        assert_eq!(
            analysis_status(&db, label),
            *expected,
            "case {label}: statuses {statuses:?}",
        );
    }
}

#[test]
fn recompute_analysis_status_with_no_runs_is_queued() {
    let (db, _dir) = open_in_tempdir();
    db.save_analysis(&analysis_fixture("no-runs")).unwrap();
    db.recompute_analysis_status("no-runs").unwrap();
    assert_eq!(analysis_status(&db, "no-runs"), AnalysisStatus::Queued);
}

#[test]
fn update_analysis_metadata_is_atomic_across_fields() {
    let (db, _dir) = open_in_tempdir();
    db.save_analysis(&analysis_fixture("a1")).unwrap();

    db.update_analysis_metadata("a1", Some("Renamed"), Some(AnalysisIntent::MacroTheme))
        .unwrap();

    let list = db.list_analyses().unwrap();
    let a1 = list.iter().find(|s| s.id == "a1").unwrap();
    assert_eq!(a1.title, "Renamed");
    assert_eq!(a1.intent, AnalysisIntent::MacroTheme);
}
