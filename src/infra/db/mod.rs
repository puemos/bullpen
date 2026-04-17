use crate::commands::ProgressEventPayload;
use crate::domain::*;
use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::HashSet;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    path: PathBuf,
}

impl Database {
    pub fn open() -> Result<Self> {
        Self::open_at(Self::default_path())
    }

    pub fn open_at(path: PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create db parent {}", parent.display()))?;
        }
        let conn = Connection::open(&path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            path,
        };
        db.init()?;
        if std::env::var("CRAZYLINES_DB_PATH").is_err() {
            unsafe {
                std::env::set_var("CRAZYLINES_DB_PATH", db.path.to_string_lossy().to_string());
            }
        }
        Ok(db)
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    fn default_path() -> PathBuf {
        if let Ok(path) = std::env::var("CRAZYLINES_DB_PATH") {
            return PathBuf::from(path);
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(home) = home::home_dir() {
                return home
                    .join("Library")
                    .join("Application Support")
                    .join("Crazylines")
                    .join("db.sqlite");
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Some(appdata) = std::env::var_os("APPDATA") {
                return PathBuf::from(appdata).join("Crazylines").join("db.sqlite");
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
                return PathBuf::from(xdg).join("crazylines").join("db.sqlite");
            }
            if let Some(home) = home::home_dir() {
                return home
                    .join(".local")
                    .join("share")
                    .join("crazylines")
                    .join("db.sqlite");
            }
        }

        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".crazylines")
            .join("db.sqlite")
    }

    fn init(&self) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS analyses (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                user_prompt TEXT NOT NULL,
                intent TEXT NOT NULL,
                status TEXT NOT NULL,
                active_run_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS analysis_runs (
                id TEXT PRIMARY KEY,
                analysis_id TEXT NOT NULL,
                agent_id TEXT NOT NULL,
                model_id TEXT,
                prompt_text TEXT NOT NULL,
                status TEXT NOT NULL,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                error TEXT,
                FOREIGN KEY(analysis_id) REFERENCES analyses(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS research_plans (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL UNIQUE,
                intent TEXT NOT NULL,
                summary TEXT NOT NULL,
                decision_criteria TEXT NOT NULL DEFAULT '[]',
                planned_checks TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(run_id) REFERENCES analysis_runs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS entities (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL,
                symbol TEXT,
                name TEXT NOT NULL,
                exchange TEXT,
                asset_type TEXT NOT NULL,
                sector TEXT,
                country TEXT,
                confidence REAL NOT NULL,
                resolution_notes TEXT,
                FOREIGN KEY(run_id) REFERENCES analysis_runs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS sources (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL,
                title TEXT NOT NULL,
                url TEXT,
                publisher TEXT,
                source_type TEXT NOT NULL,
                retrieved_at TEXT NOT NULL,
                reliability TEXT NOT NULL,
                summary TEXT NOT NULL,
                FOREIGN KEY(run_id) REFERENCES analysis_runs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS metrics (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL,
                entity_id TEXT,
                metric TEXT NOT NULL,
                numeric_value REAL NOT NULL,
                unit TEXT,
                period TEXT,
                as_of TEXT NOT NULL,
                source_id TEXT NOT NULL,
                prior_value REAL,
                change_pct REAL,
                FOREIGN KEY(run_id) REFERENCES analysis_runs(id) ON DELETE CASCADE,
                FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS structured_artifacts (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL,
                kind TEXT NOT NULL,
                title TEXT NOT NULL,
                summary TEXT NOT NULL,
                columns TEXT NOT NULL,
                rows TEXT NOT NULL,
                series TEXT NOT NULL,
                evidence_ids TEXT NOT NULL,
                display_order INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(run_id) REFERENCES analysis_runs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS analysis_blocks (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL,
                kind TEXT NOT NULL,
                title TEXT NOT NULL,
                body TEXT NOT NULL,
                evidence_ids TEXT NOT NULL,
                confidence REAL NOT NULL,
                importance TEXT NOT NULL,
                display_order INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(run_id) REFERENCES analysis_runs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS final_stances (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL UNIQUE,
                stance TEXT NOT NULL,
                horizon TEXT NOT NULL,
                confidence REAL NOT NULL,
                summary TEXT NOT NULL,
                key_reasons TEXT NOT NULL,
                what_would_change TEXT NOT NULL,
                disclaimer TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(run_id) REFERENCES analysis_runs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS projections (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                horizon TEXT NOT NULL,
                metric TEXT NOT NULL,
                current_value REAL NOT NULL,
                current_value_label TEXT NOT NULL,
                unit TEXT NOT NULL,
                scenarios TEXT NOT NULL,
                methodology TEXT NOT NULL,
                key_assumptions TEXT NOT NULL,
                evidence_ids TEXT NOT NULL,
                confidence REAL NOT NULL,
                disclaimer TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(run_id) REFERENCES analysis_runs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS counter_theses (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL,
                stance_against TEXT NOT NULL,
                summary TEXT NOT NULL,
                supporting_evidence_ids TEXT NOT NULL,
                why_we_reject_or_partially_accept TEXT NOT NULL,
                residual_probability REAL NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(run_id) REFERENCES analysis_runs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS uncertainty_entries (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL,
                question TEXT NOT NULL,
                why_it_matters TEXT NOT NULL,
                attempted_resolution TEXT NOT NULL,
                blocking INTEGER NOT NULL,
                related_decision_criterion TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY(run_id) REFERENCES analysis_runs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS methodology_notes (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL UNIQUE,
                approach TEXT NOT NULL,
                frameworks TEXT NOT NULL,
                data_windows TEXT NOT NULL,
                known_limitations TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(run_id) REFERENCES analysis_runs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS decision_criterion_answers (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL,
                criterion TEXT NOT NULL,
                verdict TEXT NOT NULL,
                summary TEXT NOT NULL,
                supporting_block_ids TEXT NOT NULL,
                supporting_evidence_ids TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(run_id) REFERENCES analysis_runs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS run_progress (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                run_id TEXT NOT NULL REFERENCES analysis_runs(id) ON DELETE CASCADE,
                event_type TEXT NOT NULL,
                payload TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_analysis_runs_analysis_id ON analysis_runs(analysis_id);
            CREATE INDEX IF NOT EXISTS idx_entities_run_id ON entities(run_id);
            CREATE INDEX IF NOT EXISTS idx_sources_run_id ON sources(run_id);
            CREATE INDEX IF NOT EXISTS idx_metrics_run_id ON metrics(run_id);
            CREATE INDEX IF NOT EXISTS idx_artifacts_run_id ON structured_artifacts(run_id);
            CREATE INDEX IF NOT EXISTS idx_blocks_run_id ON analysis_blocks(run_id);
            CREATE INDEX IF NOT EXISTS idx_projections_run_id ON projections(run_id);
            CREATE INDEX IF NOT EXISTS idx_counter_theses_run_id ON counter_theses(run_id);
            CREATE INDEX IF NOT EXISTS idx_uncertainty_entries_run_id ON uncertainty_entries(run_id);
            CREATE INDEX IF NOT EXISTS idx_decision_criterion_answers_run_id ON decision_criterion_answers(run_id);
            CREATE INDEX IF NOT EXISTS idx_run_progress_run_id ON run_progress(run_id);
            "#,
        )?;

        // Migrations for pre-existing databases: drop retired columns and
        // rewrite the retired scenario_matrix block kind. DROP COLUMN is a
        // no-op if the column never existed, which keeps this idempotent.
        for (table, column) in [
            ("sources", "as_of"),
            ("metrics", "value"),
            ("metrics", "notes"),
            ("structured_artifacts", "entity_ids"),
            ("analysis_blocks", "entity_ids"),
            ("final_stances", "watch_items"),
            ("research_plans", "required_blocks"),
            ("research_plans", "required_artifacts"),
        ] {
            let _ = conn.execute(&format!("ALTER TABLE {table} DROP COLUMN {column}"), []);
        }
        // Add new columns to pre-existing databases. ADD COLUMN errors on a
        // duplicate column name, which we silently swallow to stay idempotent.
        let _ = conn.execute("ALTER TABLE analysis_runs ADD COLUMN model_id TEXT", []);
        let _ = conn.execute(
            "UPDATE analysis_blocks SET kind = 'other' WHERE kind = 'scenario_matrix'",
            [],
        );

        Ok(())
    }

    pub fn save_analysis(&self, analysis: &Analysis) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT OR REPLACE INTO analyses
            (id, title, user_prompt, intent, status, active_run_id, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                analysis.id,
                analysis.title,
                analysis.user_prompt,
                analysis.intent.to_string(),
                analysis.status.to_string(),
                analysis.active_run_id,
                analysis.created_at,
                analysis.updated_at
            ],
        )?;
        Ok(())
    }

    pub fn save_run(&self, run: &AnalysisRun) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT OR REPLACE INTO analysis_runs
            (id, analysis_id, agent_id, model_id, prompt_text, status, started_at, completed_at, error)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                run.id,
                run.analysis_id,
                run.agent_id,
                run.model_id,
                run.prompt_text,
                run.status.to_string(),
                run.started_at,
                run.completed_at,
                run.error
            ],
        )?;
        Ok(())
    }

    pub fn update_run_status(
        &self,
        run_id: &str,
        status: AnalysisStatus,
        error: Option<&str>,
    ) -> Result<()> {
        let completed_at = if matches!(
            status,
            AnalysisStatus::Completed | AnalysisStatus::Failed | AnalysisStatus::Cancelled
        ) {
            Some(chrono::Utc::now().to_rfc3339())
        } else {
            None
        };
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "UPDATE analysis_runs SET status = ?1, completed_at = COALESCE(?2, completed_at), error = ?3 WHERE id = ?4",
            params![status.to_string(), completed_at, error, run_id],
        )?;
        Ok(())
    }

    pub fn update_analysis_status(&self, analysis_id: &str, status: AnalysisStatus) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "UPDATE analyses SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![
                status.to_string(),
                chrono::Utc::now().to_rfc3339(),
                analysis_id
            ],
        )?;
        Ok(())
    }

    pub fn update_analysis_metadata(
        &self,
        analysis_id: &str,
        title: Option<&str>,
        intent: Option<AnalysisIntent>,
    ) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        if let Some(title) = title {
            conn.execute(
                "UPDATE analyses SET title = ?1, updated_at = ?2 WHERE id = ?3",
                params![title, chrono::Utc::now().to_rfc3339(), analysis_id],
            )?;
        }
        if let Some(intent) = intent {
            conn.execute(
                "UPDATE analyses SET intent = ?1, updated_at = ?2 WHERE id = ?3",
                params![
                    intent.to_string(),
                    chrono::Utc::now().to_rfc3339(),
                    analysis_id
                ],
            )?;
        }
        Ok(())
    }

    pub fn delete_analysis(&self, analysis_id: &str) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute("DELETE FROM analyses WHERE id = ?1", [analysis_id])?;
        Ok(())
    }

    pub fn list_analyses(&self) -> Result<Vec<AnalysisSummary>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            r#"
            SELECT
                a.id, a.title, a.user_prompt, a.intent, a.status, a.active_run_id,
                ar.status,
                (SELECT COUNT(*) FROM analysis_blocks b WHERE b.run_id = a.active_run_id),
                (SELECT COUNT(*) FROM sources s WHERE s.run_id = a.active_run_id),
                a.created_at, a.updated_at
            FROM analyses a
            LEFT JOIN analysis_runs ar ON ar.id = a.active_run_id
            ORDER BY a.updated_at DESC
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            let active_status: Option<String> = row.get(6)?;
            Ok(AnalysisSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                user_prompt: row.get(2)?,
                intent: AnalysisIntent::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
                status: AnalysisStatus::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                active_run_id: row.get(5)?,
                active_run_status: active_status
                    .as_deref()
                    .and_then(|s| AnalysisStatus::from_str(s).ok()),
                block_count: row.get::<_, i64>(7)? as usize,
                source_count: row.get::<_, i64>(8)? as usize,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;

        let mut analyses = Vec::new();
        for row in rows {
            analyses.push(row?);
        }
        Ok(analyses)
    }

    pub fn get_report(
        &self,
        analysis_id: &str,
        run_id_override: Option<&str>,
    ) -> Result<Option<AnalysisReport>> {
        let analysis = match self.get_analysis(analysis_id)? {
            Some(analysis) => analysis,
            None => return Ok(None),
        };
        let runs = self.get_runs(analysis_id)?;
        let active_run = run_id_override
            .map(String::from)
            .or_else(|| analysis.active_run_id.clone())
            .or_else(|| runs.first().map(|r| r.id.clone()));

        let Some(run_id) = active_run else {
            return Ok(Some(AnalysisReport {
                analysis,
                runs,
                research_plan: None,
                entities: Vec::new(),
                sources: Vec::new(),
                metrics: Vec::new(),
                artifacts: Vec::new(),
                blocks: Vec::new(),
                final_stance: None,
                projections: Vec::new(),
                counter_theses: Vec::new(),
                uncertainty_entries: Vec::new(),
                methodology_note: None,
                decision_criterion_answers: Vec::new(),
            }));
        };

        Ok(Some(AnalysisReport {
            analysis,
            runs,
            research_plan: self.get_research_plan(&run_id)?,
            entities: self.get_entities(&run_id)?,
            sources: self.get_sources(&run_id)?,
            metrics: self.get_metrics(&run_id)?,
            artifacts: self.get_structured_artifacts(&run_id)?,
            blocks: self.get_blocks(&run_id)?,
            final_stance: self.get_final_stance(&run_id)?,
            projections: self.get_projections(&run_id)?,
            counter_theses: self.get_counter_theses(&run_id)?,
            uncertainty_entries: self.get_uncertainty_entries(&run_id)?,
            methodology_note: self.get_methodology_note(&run_id)?,
            decision_criterion_answers: self.get_decision_criterion_answers(&run_id)?,
        }))
    }

    fn get_analysis(&self, analysis_id: &str) -> Result<Option<Analysis>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.query_row(
            "SELECT id, title, user_prompt, intent, status, active_run_id, created_at, updated_at FROM analyses WHERE id = ?1",
            [analysis_id],
            |row| {
                Ok(Analysis {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    user_prompt: row.get(2)?,
                    intent: AnalysisIntent::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
                    status: AnalysisStatus::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                    active_run_id: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn get_runs(&self, analysis_id: &str) -> Result<Vec<AnalysisRun>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, analysis_id, agent_id, model_id, prompt_text, status, started_at, completed_at, error
             FROM analysis_runs WHERE analysis_id = ?1 ORDER BY started_at DESC",
        )?;
        let rows = stmt.query_map([analysis_id], |row| {
            Ok(AnalysisRun {
                id: row.get(0)?,
                analysis_id: row.get(1)?,
                agent_id: row.get(2)?,
                model_id: row.get(3)?,
                prompt_text: row.get(4)?,
                status: AnalysisStatus::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                started_at: row.get(6)?,
                completed_at: row.get(7)?,
                error: row.get(8)?,
            })
        })?;
        let mut runs = Vec::new();
        for row in rows {
            runs.push(row?);
        }
        Ok(runs)
    }

    pub fn save_research_plan(&self, plan: &ResearchPlan) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT OR REPLACE INTO research_plans
            (id, run_id, intent, summary, decision_criteria, planned_checks, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                plan.id,
                plan.run_id,
                plan.intent.to_string(),
                plan.summary,
                serde_json::to_string(&plan.decision_criteria)?,
                serde_json::to_string(&plan.planned_checks)?,
                plan.created_at
            ],
        )?;
        Ok(())
    }

    pub fn save_entity(&self, entity: &Entity) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT OR REPLACE INTO entities
            (id, run_id, symbol, name, exchange, asset_type, sector, country, confidence, resolution_notes)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                entity.id,
                entity.run_id,
                entity.symbol,
                entity.name,
                entity.exchange,
                entity.asset_type,
                entity.sector,
                entity.country,
                entity.confidence,
                entity.resolution_notes
            ],
        )?;
        Ok(())
    }

    pub fn save_source(&self, source: &Source) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT OR REPLACE INTO sources
            (id, run_id, title, url, publisher, source_type, retrieved_at, reliability, summary)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                source.id,
                source.run_id,
                source.title,
                source.url,
                source.publisher,
                source.source_type,
                source.retrieved_at,
                source.reliability.to_string(),
                source.summary
            ],
        )?;
        Ok(())
    }

    pub fn save_metric(&self, metric: &MetricSnapshot) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT OR REPLACE INTO metrics
            (id, run_id, entity_id, metric, numeric_value, unit, period, as_of, source_id, prior_value, change_pct)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                metric.id,
                metric.run_id,
                metric.entity_id,
                metric.metric,
                metric.numeric_value,
                metric.unit,
                metric.period,
                metric.as_of,
                metric.source_id,
                metric.prior_value,
                metric.change_pct,
            ],
        )?;
        Ok(())
    }

    pub fn save_structured_artifact(&self, artifact: &StructuredArtifact) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT OR REPLACE INTO structured_artifacts
            (id, run_id, kind, title, summary, columns, rows, series, evidence_ids, display_order, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                artifact.id,
                artifact.run_id,
                artifact.kind.to_string(),
                artifact.title,
                artifact.summary,
                serde_json::to_string(&artifact.columns)?,
                serde_json::to_string(&artifact.rows)?,
                serde_json::to_string(&artifact.series)?,
                serde_json::to_string(&artifact.evidence_ids)?,
                artifact.display_order,
                artifact.created_at
            ],
        )?;
        Ok(())
    }

    pub fn save_block(&self, block: &AnalysisBlock) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT OR REPLACE INTO analysis_blocks
            (id, run_id, kind, title, body, evidence_ids, confidence, importance, display_order, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                block.id,
                block.run_id,
                block.kind.to_string(),
                block.title,
                block.body,
                serde_json::to_string(&block.evidence_ids)?,
                block.confidence,
                block.importance.to_string(),
                block.display_order,
                block.created_at
            ],
        )?;
        Ok(())
    }

    pub fn save_projection(&self, projection: &Projection) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT OR REPLACE INTO projections
            (id, run_id, entity_id, horizon, metric, current_value, current_value_label, unit, scenarios, methodology, key_assumptions, evidence_ids, confidence, disclaimer, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                projection.id,
                projection.run_id,
                projection.entity_id,
                projection.horizon,
                projection.metric,
                projection.current_value,
                projection.current_value_label,
                projection.unit,
                serde_json::to_string(&projection.scenarios)?,
                projection.methodology,
                serde_json::to_string(&projection.key_assumptions)?,
                serde_json::to_string(&projection.evidence_ids)?,
                projection.confidence,
                projection.disclaimer,
                projection.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn save_final_stance(&self, stance: &FinalStance) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT OR REPLACE INTO final_stances
            (id, run_id, stance, horizon, confidence, summary, key_reasons, what_would_change, disclaimer, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                stance.id,
                stance.run_id,
                stance.stance.to_string(),
                stance.horizon,
                stance.confidence,
                stance.summary,
                serde_json::to_string(&stance.key_reasons)?,
                serde_json::to_string(&stance.what_would_change)?,
                stance.disclaimer,
                stance.created_at
            ],
        )?;
        Ok(())
    }

    pub fn recompute_analysis_status(&self, analysis_id: &str) -> Result<()> {
        let runs = self.get_runs(analysis_id)?;
        let status = if runs.iter().any(|r| r.status == AnalysisStatus::Running) {
            AnalysisStatus::Running
        } else if runs.iter().any(|r| r.status == AnalysisStatus::Completed) {
            AnalysisStatus::Completed
        } else if runs.iter().all(|r| r.status == AnalysisStatus::Cancelled) {
            AnalysisStatus::Cancelled
        } else {
            AnalysisStatus::Failed
        };
        self.update_analysis_status(analysis_id, status)
    }

    pub fn set_active_run_if_empty(&self, analysis_id: &str, run_id: &str) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "UPDATE analyses SET active_run_id = ?1 WHERE id = ?2 AND active_run_id IS NULL",
            params![run_id, analysis_id],
        )?;
        Ok(())
    }

    pub fn append_progress_event(&self, run_id: &str, event: &ProgressEventPayload) -> Result<()> {
        let event_type = match event {
            ProgressEventPayload::Log(_) => "Log",
            ProgressEventPayload::MessageDelta { .. } => "MessageDelta",
            ProgressEventPayload::ThoughtDelta { .. } => "ThoughtDelta",
            ProgressEventPayload::ToolCallStarted { .. } => "ToolCallStarted",
            ProgressEventPayload::ToolCallComplete { .. } => "ToolCallComplete",
            ProgressEventPayload::Plan(_) => "Plan",
            ProgressEventPayload::PlanSubmitted => "PlanSubmitted",
            ProgressEventPayload::SourceSubmitted => "SourceSubmitted",
            ProgressEventPayload::MetricSubmitted => "MetricSubmitted",
            ProgressEventPayload::ArtifactSubmitted => "ArtifactSubmitted",
            ProgressEventPayload::BlockSubmitted => "BlockSubmitted",
            ProgressEventPayload::StanceSubmitted => "StanceSubmitted",
            ProgressEventPayload::ProjectionSubmitted => "ProjectionSubmitted",
            ProgressEventPayload::Completed => "Completed",
            ProgressEventPayload::Error { .. } => "Error",
        };
        let payload = serde_json::to_string(event)?;
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT INTO run_progress (run_id, event_type, payload) VALUES (?1, ?2, ?3)",
            params![run_id, event_type, payload],
        )?;
        Ok(())
    }

    pub fn get_run_progress(&self, run_id: &str) -> Result<Vec<ProgressEventPayload>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt =
            conn.prepare("SELECT payload FROM run_progress WHERE run_id = ?1 ORDER BY id ASC")?;
        let rows = stmt.query_map([run_id], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })?;
        let mut events = Vec::new();
        for row in rows {
            let json = row?;
            let event: ProgressEventPayload = serde_json::from_str(&json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            events.push(event);
        }
        Ok(events)
    }

    pub fn validate_finalization(&self, run_id: &str) -> Result<Vec<String>> {
        let research_plan = self.get_research_plan(run_id)?;
        let entities = self.get_entities(run_id)?;
        let blocks = self.get_blocks(run_id)?;
        let sources = self.get_sources(run_id)?;
        let artifacts = self.get_structured_artifacts(run_id)?;
        let final_stance = self.get_final_stance(run_id)?;
        let projections = self.get_projections(run_id)?;
        let counter_theses = self.get_counter_theses(run_id)?;
        let uncertainty_entries = self.get_uncertainty_entries(run_id)?;
        let methodology_note = self.get_methodology_note(run_id)?;
        let criterion_answers = self.get_decision_criterion_answers(run_id)?;

        let mut errors = Vec::new();
        let block_kinds: HashSet<BlockKind> = blocks.iter().map(|block| block.kind).collect();
        let artifact_kinds: HashSet<ArtifactKind> =
            artifacts.iter().map(|artifact| artifact.kind).collect();

        if research_plan.is_none() {
            errors.push("missing research plan".to_string());
        }

        if !block_kinds.contains(&BlockKind::Thesis) {
            errors.push("missing required thesis block".to_string());
        }
        if !block_kinds.contains(&BlockKind::Risks) {
            errors.push("missing required risks block".to_string());
        }
        if sources.is_empty() {
            errors.push("missing sources; submit at least one source".to_string());
        } else if sources
            .iter()
            .all(|source| source.reliability == SourceReliability::Low)
        {
            errors.push(
                "every submitted source has low reliability; add a primary/high/medium source before finalizing"
                    .to_string(),
            );
        }
        if final_stance.is_none() {
            errors.push("missing final stance".to_string());
        }
        if methodology_note.is_none() {
            errors
                .push("missing methodology note; submit_methodology_note is required".to_string());
        }

        if let Some(plan) = &research_plan {
            for required in required_blocks_for(plan.intent) {
                if !block_kinds.contains(&required) {
                    errors.push(format!("missing required {required} block"));
                }
            }
            for required in required_artifacts_for(plan.intent) {
                if !artifact_kinds.contains(&required) {
                    errors.push(format!("missing required {required} artifact"));
                }
            }
            if matches!(
                plan.intent,
                AnalysisIntent::CompareEquities | AnalysisIntent::Watchlist
            ) && entities.len() < 2
            {
                errors.push("comparison reports must resolve at least two entities".to_string());
            }

            for answer in &criterion_answers {
                if !plan
                    .decision_criteria
                    .iter()
                    .any(|c| c == &answer.criterion)
                {
                    errors.push(format!(
                        "decision_criterion_answer references unknown criterion '{}'",
                        answer.criterion
                    ));
                }
            }
            for criterion in &plan.decision_criteria {
                let matches: Vec<&DecisionCriterionAnswer> = criterion_answers
                    .iter()
                    .filter(|a| &a.criterion == criterion)
                    .collect();
                match matches.len() {
                    0 => errors.push(format!(
                        "decision criterion '{criterion}' is missing a submit_decision_criterion_answer"
                    )),
                    1 => {
                        if matches[0].verdict == CriterionVerdict::Unresolved
                            && !uncertainty_entries.iter().any(|u| {
                                u.related_decision_criterion.as_deref() == Some(criterion.as_str())
                            })
                        {
                            errors.push(format!(
                                "decision criterion '{criterion}' marked Unresolved but no uncertainty entry references it"
                            ));
                        }
                    }
                    n => errors.push(format!(
                        "decision criterion '{criterion}' has {n} answers; expected exactly one"
                    )),
                }
            }
        }

        for block in &blocks {
            if block.kind != BlockKind::OpenQuestions && block.evidence_ids.is_empty() {
                errors.push(format!(
                    "material block '{}' must include evidence_ids",
                    block.title
                ));
            }
        }

        for artifact in &artifacts {
            if matches!(
                artifact.kind,
                ArtifactKind::MetricTable
                    | ArtifactKind::ComparisonMatrix
                    | ArtifactKind::ScenarioMatrix
            ) && (artifact.columns.is_empty() || artifact.rows.is_empty())
            {
                errors.push(format!(
                    "artifact '{}' must include columns and rows",
                    artifact.title
                ));
            }
            if matches!(
                artifact.kind,
                ArtifactKind::BarChart | ArtifactKind::LineChart | ArtifactKind::AreaChart
            ) && !artifact
                .series
                .iter()
                .any(|series| !series.points.is_empty())
            {
                errors.push(format!(
                    "chart artifact '{}' must include series points",
                    artifact.title
                ));
            }
            if artifact.evidence_ids.is_empty() {
                errors.push(format!(
                    "artifact '{}' must include evidence_ids",
                    artifact.title
                ));
            }
        }

        if let Some(stance) = &final_stance {
            if stance.key_reasons.is_empty() {
                errors.push("final stance must include key_reasons".to_string());
            }
            if stance.what_would_change.is_empty() {
                errors.push("final stance must include what_would_change".to_string());
            }

            let directional = matches!(stance.stance, StanceKind::Bullish | StanceKind::Bearish);
            if directional && counter_theses.is_empty() {
                errors.push(
                    "directional stance requires a submit_counter_thesis call steelmanning the opposing view"
                        .to_string(),
                );
            }
            for counter in &counter_theses {
                if counter.residual_probability < 0.10 {
                    errors.push(format!(
                        "counter_thesis residual_probability {:.2} is below 0.10; if you cannot build a 10%+ steelman, stance must be mixed or insufficient_data",
                        counter.residual_probability
                    ));
                }
            }

            let has_blocking = uncertainty_entries.iter().any(|u| u.blocking);
            if has_blocking && stance.confidence > 0.6 {
                errors.push(format!(
                    "stance confidence {:.2} exceeds 0.6 while blocking uncertainty is unresolved",
                    stance.confidence
                ));
            }
        }

        if let Some(plan) = &research_plan {
            let projection_intent = matches!(
                plan.intent,
                AnalysisIntent::SingleEquity | AnalysisIntent::CompareEquities
            );
            if projection_intent {
                if projections.is_empty() {
                    errors.push(
                        "missing forward-looking projection; submit_projection is required for single_equity and compare_equities"
                            .to_string(),
                    );
                }
                if !uncertainty_entries.is_empty()
                    && !block_kinds.contains(&BlockKind::OpenQuestions)
                {
                    errors.push(
                        "uncertainty entries submitted but no open_questions block summarizes them"
                            .to_string(),
                    );
                }
            }
            if matches!(plan.intent, AnalysisIntent::CompareEquities) && !entities.is_empty() {
                let mut by_entity: std::collections::HashMap<&str, usize> =
                    std::collections::HashMap::new();
                for projection in &projections {
                    *by_entity.entry(projection.entity_id.as_str()).or_insert(0) += 1;
                }
                for entity in &entities {
                    match by_entity.get(entity.id.as_str()) {
                        None => errors.push(format!(
                            "comparison requires one projection per entity; missing projection for '{}'",
                            entity.name
                        )),
                        Some(count) if *count > 1 => errors.push(format!(
                            "comparison must have exactly one projection per entity; entity '{}' has {count}",
                            entity.name
                        )),
                        _ => {}
                    }
                }
            }
            if matches!(plan.intent, AnalysisIntent::SingleEquity) && entities.len() > 1 {
                // single equity with more than one entity is fine, but each projection
                // should still tie back to a resolved entity
                let entity_ids: HashSet<&str> =
                    entities.iter().map(|entity| entity.id.as_str()).collect();
                for projection in &projections {
                    if !entity_ids.contains(projection.entity_id.as_str()) {
                        errors.push(format!(
                            "projection '{}' references unresolved entity '{}'",
                            projection.metric, projection.entity_id
                        ));
                    }
                }
            }
        }

        for projection in &projections {
            if projection.methodology.trim().is_empty() {
                errors.push(format!(
                    "projection '{}' must include methodology",
                    projection.metric
                ));
            }
            if projection.key_assumptions.is_empty() {
                errors.push(format!(
                    "projection '{}' must include key_assumptions",
                    projection.metric
                ));
            }
            if projection.evidence_ids.is_empty() {
                errors.push(format!(
                    "projection '{}' must include evidence_ids",
                    projection.metric
                ));
            }

            let labels: HashSet<ScenarioLabel> = projection
                .scenarios
                .iter()
                .map(|scenario| scenario.label)
                .collect();
            for required in [
                ScenarioLabel::Bull,
                ScenarioLabel::Base,
                ScenarioLabel::Bear,
            ] {
                if !labels.contains(&required) {
                    errors.push(format!(
                        "projection '{}' missing required '{required}' scenario",
                        projection.metric
                    ));
                }
            }
            for scenario in &projection.scenarios {
                if scenario.rationale.trim().is_empty() {
                    errors.push(format!(
                        "projection '{}' scenario '{}' must include rationale",
                        projection.metric, scenario.label
                    ));
                }
                if scenario.catalysts.is_empty() {
                    errors.push(format!(
                        "projection '{}' scenario '{}' must include catalysts",
                        projection.metric, scenario.label
                    ));
                }
                if scenario.risks.is_empty() {
                    errors.push(format!(
                        "projection '{}' scenario '{}' must include risks",
                        projection.metric, scenario.label
                    ));
                }
            }
        }

        Ok(errors)
    }

    fn get_research_plan(&self, run_id: &str) -> Result<Option<ResearchPlan>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.query_row(
            "SELECT id, run_id, intent, summary, decision_criteria, planned_checks, created_at FROM research_plans WHERE run_id = ?1",
            [run_id],
            |row| {
                let criteria: String = row.get(4)?;
                let planned: String = row.get(5)?;
                Ok(ResearchPlan {
                    id: row.get(0)?,
                    run_id: row.get(1)?,
                    intent: AnalysisIntent::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                    summary: row.get(3)?,
                    decision_criteria: serde_json::from_str(&criteria).unwrap_or_default(),
                    planned_checks: serde_json::from_str(&planned).unwrap_or_default(),
                    created_at: row.get(6)?,
                })
            },
        )
        .optional()
        .map_err(Into::into)
    }

    fn get_entities(&self, run_id: &str) -> Result<Vec<Entity>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, run_id, symbol, name, exchange, asset_type, sector, country, confidence, resolution_notes
             FROM entities WHERE run_id = ?1 ORDER BY name",
        )?;
        let rows = stmt.query_map([run_id], |row| {
            Ok(Entity {
                id: row.get(0)?,
                run_id: row.get(1)?,
                symbol: row.get(2)?,
                name: row.get(3)?,
                exchange: row.get(4)?,
                asset_type: row.get(5)?,
                sector: row.get(6)?,
                country: row.get(7)?,
                confidence: row.get(8)?,
                resolution_notes: row.get(9)?,
            })
        })?;
        collect_rows(rows)
    }

    fn get_sources(&self, run_id: &str) -> Result<Vec<Source>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, run_id, title, url, publisher, source_type, retrieved_at, reliability, summary
             FROM sources WHERE run_id = ?1 ORDER BY retrieved_at DESC",
        )?;
        let rows = stmt.query_map([run_id], |row| {
            Ok(Source {
                id: row.get(0)?,
                run_id: row.get(1)?,
                title: row.get(2)?,
                url: row.get(3)?,
                publisher: row.get(4)?,
                source_type: row.get(5)?,
                retrieved_at: row.get(6)?,
                reliability: SourceReliability::from_str(&row.get::<_, String>(7)?)
                    .unwrap_or_default(),
                summary: row.get(8)?,
            })
        })?;
        collect_rows(rows)
    }

    fn get_metrics(&self, run_id: &str) -> Result<Vec<MetricSnapshot>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, run_id, entity_id, metric, numeric_value, unit, period, as_of, source_id, prior_value, change_pct
             FROM metrics WHERE run_id = ?1 ORDER BY metric",
        )?;
        let rows = stmt.query_map([run_id], |row| {
            Ok(MetricSnapshot {
                id: row.get(0)?,
                run_id: row.get(1)?,
                entity_id: row.get(2)?,
                metric: row.get(3)?,
                numeric_value: row.get(4)?,
                unit: row.get(5)?,
                period: row.get(6)?,
                as_of: row.get(7)?,
                source_id: row.get(8)?,
                prior_value: row.get(9)?,
                change_pct: row.get(10)?,
            })
        })?;
        collect_rows(rows)
    }

    fn get_structured_artifacts(&self, run_id: &str) -> Result<Vec<StructuredArtifact>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, run_id, kind, title, summary, columns, rows, series, evidence_ids, display_order, created_at
             FROM structured_artifacts WHERE run_id = ?1 ORDER BY display_order, created_at",
        )?;
        let rows = stmt.query_map([run_id], |row| {
            let columns: String = row.get(5)?;
            let rows_json: String = row.get(6)?;
            let series: String = row.get(7)?;
            let evidence: String = row.get(8)?;
            Ok(StructuredArtifact {
                id: row.get(0)?,
                run_id: row.get(1)?,
                kind: ArtifactKind::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                title: row.get(3)?,
                summary: row.get(4)?,
                columns: serde_json::from_str(&columns).unwrap_or_default(),
                rows: serde_json::from_str(&rows_json).unwrap_or_default(),
                series: serde_json::from_str(&series).unwrap_or_default(),
                evidence_ids: serde_json::from_str(&evidence).unwrap_or_default(),
                display_order: row.get(9)?,
                created_at: row.get(10)?,
            })
        })?;
        collect_rows(rows)
    }

    pub fn get_blocks_for(&self, run_id: &str) -> Result<Vec<AnalysisBlock>> {
        self.get_blocks(run_id)
    }

    fn get_blocks(&self, run_id: &str) -> Result<Vec<AnalysisBlock>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, run_id, kind, title, body, evidence_ids, confidence, importance, display_order, created_at
             FROM analysis_blocks WHERE run_id = ?1 ORDER BY display_order, created_at",
        )?;
        let rows = stmt.query_map([run_id], |row| {
            let evidence: String = row.get(5)?;
            Ok(AnalysisBlock {
                id: row.get(0)?,
                run_id: row.get(1)?,
                kind: BlockKind::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                title: row.get(3)?,
                body: row.get(4)?,
                evidence_ids: serde_json::from_str(&evidence).unwrap_or_default(),
                confidence: row.get(6)?,
                importance: Importance::from_str(&row.get::<_, String>(7)?)
                    .unwrap_or(Importance::Medium),
                display_order: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;
        collect_rows(rows)
    }

    pub fn get_projections(&self, run_id: &str) -> Result<Vec<Projection>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, run_id, entity_id, horizon, metric, current_value, current_value_label, unit, scenarios, methodology, key_assumptions, evidence_ids, confidence, disclaimer, created_at
             FROM projections WHERE run_id = ?1 ORDER BY created_at",
        )?;
        let rows = stmt.query_map([run_id], |row| {
            let scenarios: String = row.get(8)?;
            let assumptions: String = row.get(10)?;
            let evidence: String = row.get(11)?;
            Ok(Projection {
                id: row.get(0)?,
                run_id: row.get(1)?,
                entity_id: row.get(2)?,
                horizon: row.get(3)?,
                metric: row.get(4)?,
                current_value: row.get(5)?,
                current_value_label: row.get(6)?,
                unit: row.get(7)?,
                scenarios: serde_json::from_str(&scenarios).unwrap_or_default(),
                methodology: row.get(9)?,
                key_assumptions: serde_json::from_str(&assumptions).unwrap_or_default(),
                evidence_ids: serde_json::from_str(&evidence).unwrap_or_default(),
                confidence: row.get(12)?,
                disclaimer: row.get(13)?,
                created_at: row.get(14)?,
            })
        })?;
        collect_rows(rows)
    }

    pub fn get_final_stance(&self, run_id: &str) -> Result<Option<FinalStance>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.query_row(
            "SELECT id, run_id, stance, horizon, confidence, summary, key_reasons, what_would_change, disclaimer, created_at
             FROM final_stances WHERE run_id = ?1",
            [run_id],
            |row| {
                let reasons: String = row.get(6)?;
                let changes: String = row.get(7)?;
                Ok(FinalStance {
                    id: row.get(0)?,
                    run_id: row.get(1)?,
                    stance: StanceKind::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                    horizon: row.get(3)?,
                    confidence: row.get(4)?,
                    summary: row.get(5)?,
                    key_reasons: serde_json::from_str(&reasons).unwrap_or_default(),
                    what_would_change: serde_json::from_str(&changes).unwrap_or_default(),
                    disclaimer: row.get(8)?,
                    created_at: row.get(9)?,
                })
            },
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn existing_source_ids(&self, run_id: &str) -> Result<HashSet<String>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare("SELECT id FROM sources WHERE run_id = ?1")?;
        let rows = stmt.query_map([run_id], |row| row.get::<_, String>(0))?;
        let mut out = HashSet::new();
        for row in rows {
            out.insert(row?);
        }
        Ok(out)
    }

    pub fn existing_block_ids(&self, run_id: &str) -> Result<HashSet<String>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare("SELECT id FROM analysis_blocks WHERE run_id = ?1")?;
        let rows = stmt.query_map([run_id], |row| row.get::<_, String>(0))?;
        let mut out = HashSet::new();
        for row in rows {
            out.insert(row?);
        }
        Ok(out)
    }

    pub fn save_counter_thesis(&self, thesis: &CounterThesis) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT OR REPLACE INTO counter_theses
            (id, run_id, stance_against, summary, supporting_evidence_ids, why_we_reject_or_partially_accept, residual_probability, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                thesis.id,
                thesis.run_id,
                thesis.stance_against.to_string(),
                thesis.summary,
                serde_json::to_string(&thesis.supporting_evidence_ids)?,
                thesis.why_we_reject_or_partially_accept,
                thesis.residual_probability,
                thesis.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_counter_theses(&self, run_id: &str) -> Result<Vec<CounterThesis>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, run_id, stance_against, summary, supporting_evidence_ids, why_we_reject_or_partially_accept, residual_probability, created_at
             FROM counter_theses WHERE run_id = ?1 ORDER BY created_at",
        )?;
        let rows = stmt.query_map([run_id], |row| {
            let evidence: String = row.get(4)?;
            Ok(CounterThesis {
                id: row.get(0)?,
                run_id: row.get(1)?,
                stance_against: StanceKind::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                summary: row.get(3)?,
                supporting_evidence_ids: serde_json::from_str(&evidence).unwrap_or_default(),
                why_we_reject_or_partially_accept: row.get(5)?,
                residual_probability: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
        collect_rows(rows)
    }

    pub fn save_uncertainty_entry(&self, entry: &UncertaintyEntry) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT OR REPLACE INTO uncertainty_entries
            (id, run_id, question, why_it_matters, attempted_resolution, blocking, related_decision_criterion, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                entry.id,
                entry.run_id,
                entry.question,
                entry.why_it_matters,
                entry.attempted_resolution,
                entry.blocking as i64,
                entry.related_decision_criterion,
                entry.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_uncertainty_entries(&self, run_id: &str) -> Result<Vec<UncertaintyEntry>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, run_id, question, why_it_matters, attempted_resolution, blocking, related_decision_criterion, created_at
             FROM uncertainty_entries WHERE run_id = ?1 ORDER BY created_at",
        )?;
        let rows = stmt.query_map([run_id], |row| {
            Ok(UncertaintyEntry {
                id: row.get(0)?,
                run_id: row.get(1)?,
                question: row.get(2)?,
                why_it_matters: row.get(3)?,
                attempted_resolution: row.get(4)?,
                blocking: row.get::<_, i64>(5)? != 0,
                related_decision_criterion: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
        collect_rows(rows)
    }

    pub fn save_methodology_note(&self, note: &MethodologyNote) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT OR REPLACE INTO methodology_notes
            (id, run_id, approach, frameworks, data_windows, known_limitations, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                note.id,
                note.run_id,
                note.approach,
                serde_json::to_string(&note.frameworks)?,
                serde_json::to_string(&note.data_windows)?,
                serde_json::to_string(&note.known_limitations)?,
                note.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_methodology_note(&self, run_id: &str) -> Result<Option<MethodologyNote>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.query_row(
            "SELECT id, run_id, approach, frameworks, data_windows, known_limitations, created_at
             FROM methodology_notes WHERE run_id = ?1",
            [run_id],
            |row| {
                let frameworks: String = row.get(3)?;
                let data_windows: String = row.get(4)?;
                let known_limitations: String = row.get(5)?;
                Ok(MethodologyNote {
                    id: row.get(0)?,
                    run_id: row.get(1)?,
                    approach: row.get(2)?,
                    frameworks: serde_json::from_str(&frameworks).unwrap_or_default(),
                    data_windows: serde_json::from_str(&data_windows).unwrap_or_default(),
                    known_limitations: serde_json::from_str(&known_limitations).unwrap_or_default(),
                    created_at: row.get(6)?,
                })
            },
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn save_decision_criterion_answer(&self, answer: &DecisionCriterionAnswer) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT OR REPLACE INTO decision_criterion_answers
            (id, run_id, criterion, verdict, summary, supporting_block_ids, supporting_evidence_ids, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                answer.id,
                answer.run_id,
                answer.criterion,
                answer.verdict.to_string(),
                answer.summary,
                serde_json::to_string(&answer.supporting_block_ids)?,
                serde_json::to_string(&answer.supporting_evidence_ids)?,
                answer.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_decision_criterion_answers(
        &self,
        run_id: &str,
    ) -> Result<Vec<DecisionCriterionAnswer>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, run_id, criterion, verdict, summary, supporting_block_ids, supporting_evidence_ids, created_at
             FROM decision_criterion_answers WHERE run_id = ?1 ORDER BY created_at",
        )?;
        let rows = stmt.query_map([run_id], |row| {
            let blocks: String = row.get(5)?;
            let evidence: String = row.get(6)?;
            Ok(DecisionCriterionAnswer {
                id: row.get(0)?,
                run_id: row.get(1)?,
                criterion: row.get(2)?,
                verdict: CriterionVerdict::from_str(&row.get::<_, String>(3)?)
                    .unwrap_or(CriterionVerdict::Unresolved),
                summary: row.get(4)?,
                supporting_block_ids: serde_json::from_str(&blocks).unwrap_or_default(),
                supporting_evidence_ids: serde_json::from_str(&evidence).unwrap_or_default(),
                created_at: row.get(7)?,
            })
        })?;
        collect_rows(rows)
    }
}

fn required_blocks_for(intent: AnalysisIntent) -> Vec<BlockKind> {
    let mut out = vec![BlockKind::Thesis, BlockKind::Risks];
    match intent {
        AnalysisIntent::SingleEquity => {
            out.extend([
                BlockKind::Financials,
                BlockKind::Valuation,
                BlockKind::Catalysts,
            ]);
        }
        AnalysisIntent::CompareEquities | AnalysisIntent::Watchlist => {
            out.push(BlockKind::PeerComparison);
        }
        AnalysisIntent::SectorAnalysis => {
            out.push(BlockKind::SectorContext);
        }
        AnalysisIntent::MacroTheme | AnalysisIntent::GeneralResearch => {}
    }
    out
}

fn required_artifacts_for(intent: AnalysisIntent) -> Vec<ArtifactKind> {
    match intent {
        AnalysisIntent::CompareEquities | AnalysisIntent::Watchlist => {
            vec![ArtifactKind::ComparisonMatrix]
        }
        _ => Vec::new(),
    }
}

fn collect_rows<T>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>>,
) -> Result<Vec<T>> {
    let mut values = Vec::new();
    for row in rows {
        values.push(row?);
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed_run(db: &Database, prompt: &str, intent: AnalysisIntent) -> String {
        let run_id = "run-1".to_string();
        let now = chrono::Utc::now().to_rfc3339();
        db.save_analysis(&Analysis {
            id: "a".into(),
            title: "Analysis".into(),
            user_prompt: prompt.into(),
            intent,
            status: AnalysisStatus::Running,
            active_run_id: Some(run_id.clone()),
            created_at: now.clone(),
            updated_at: now.clone(),
        })
        .unwrap();
        db.save_run(&AnalysisRun {
            id: run_id.clone(),
            analysis_id: "a".into(),
            agent_id: "fake".into(),
            model_id: None,
            prompt_text: prompt.into(),
            status: AnalysisStatus::Running,
            started_at: now,
            completed_at: None,
            error: None,
        })
        .unwrap();
        run_id
    }

    fn save_plan(db: &Database, run_id: &str, intent: AnalysisIntent) {
        db.save_research_plan(&ResearchPlan {
            id: "plan-1".into(),
            run_id: run_id.into(),
            intent,
            summary: "Assess the research question.".into(),
            decision_criteria: vec!["valuation".into(), "risk".into()],
            planned_checks: vec!["Check primary sources.".into()],
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .unwrap();
    }

    fn save_source(db: &Database, run_id: &str) -> String {
        save_source_with(db, run_id, "source-1", SourceReliability::Primary)
    }

    fn save_source_with(
        db: &Database,
        run_id: &str,
        id: &str,
        reliability: SourceReliability,
    ) -> String {
        db.save_source(&Source {
            id: id.into(),
            run_id: run_id.into(),
            title: format!("Source {id}"),
            url: Some("https://example.com/filing".into()),
            publisher: Some("Example".into()),
            source_type: "filing".into(),
            retrieved_at: chrono::Utc::now().to_rfc3339(),
            reliability,
            summary: "Primary source.".into(),
        })
        .unwrap();
        id.to_string()
    }

    fn save_block(db: &Database, run_id: &str, kind: BlockKind, source_id: &str) {
        db.save_block(&AnalysisBlock {
            id: format!("block-{kind}"),
            run_id: run_id.into(),
            kind,
            title: kind.to_string(),
            body: "Evidence-backed block.".into(),
            evidence_ids: vec![source_id.into()],
            confidence: 0.8,
            importance: Importance::High,
            display_order: 10,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .unwrap();
    }

    fn save_stance(db: &Database, run_id: &str) {
        save_stance_with(db, run_id, StanceKind::Neutral, 0.7);
    }

    fn save_stance_with(db: &Database, run_id: &str, stance: StanceKind, confidence: f64) {
        db.save_final_stance(&FinalStance {
            id: "stance-1".into(),
            run_id: run_id.into(),
            stance,
            horizon: "12 months".into(),
            confidence,
            summary: "Balanced evidence.".into(),
            key_reasons: vec!["Valuation is mixed.".into()],
            what_would_change: vec!["Better growth visibility.".into()],
            disclaimer: RESEARCH_DISCLAIMER.into(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .unwrap();
    }

    fn save_methodology(db: &Database, run_id: &str) {
        db.save_methodology_note(&MethodologyNote {
            id: "methodology-1".into(),
            run_id: run_id.into(),
            approach: "Triangulate filings and consensus.".into(),
            frameworks: vec!["reverse DCF".into()],
            data_windows: vec!["FY24 10-K".into()],
            known_limitations: vec!["Forward guidance withheld.".into()],
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .unwrap();
    }

    fn save_criterion_answers(db: &Database, run_id: &str) {
        for (index, criterion) in ["valuation", "risk"].iter().enumerate() {
            db.save_decision_criterion_answer(&DecisionCriterionAnswer {
                id: format!("answer-{index}"),
                run_id: run_id.into(),
                criterion: (*criterion).into(),
                verdict: CriterionVerdict::Confirmed,
                summary: "Evidence supports this criterion.".into(),
                supporting_block_ids: vec!["block-thesis".into()],
                supporting_evidence_ids: vec!["source-1".into()],
                created_at: chrono::Utc::now().to_rfc3339(),
            })
            .unwrap();
        }
    }

    fn valid_scenarios() -> Vec<ProjectionScenario> {
        vec![
            ProjectionScenario {
                label: ScenarioLabel::Bull,
                target_value: 230.0,
                target_label: "$230".into(),
                upside_pct: 0.26,
                probability: 0.25,
                rationale: "Upside thesis pans out.".into(),
                catalysts: vec!["Product launch".into()],
                risks: vec!["Execution slip".into()],
            },
            ProjectionScenario {
                label: ScenarioLabel::Base,
                target_value: 205.0,
                target_label: "$205".into(),
                upside_pct: 0.13,
                probability: 0.55,
                rationale: "Consensus path holds.".into(),
                catalysts: vec!["Buyback cadence".into()],
                risks: vec!["Margin pressure".into()],
            },
            ProjectionScenario {
                label: ScenarioLabel::Bear,
                target_value: 150.0,
                target_label: "$150".into(),
                upside_pct: -0.18,
                probability: 0.20,
                rationale: "Demand softens.".into(),
                catalysts: vec!["Pricing war".into()],
                risks: vec!["Macro drawdown".into()],
            },
        ]
    }

    fn save_projection(
        db: &Database,
        run_id: &str,
        entity_id: &str,
        source_id: &str,
        scenarios: Vec<ProjectionScenario>,
    ) {
        db.save_projection(&Projection {
            id: format!("projection-{entity_id}"),
            run_id: run_id.into(),
            entity_id: entity_id.into(),
            horizon: "12 months".into(),
            metric: "stock_price".into(),
            current_value: 182.0,
            current_value_label: "$182".into(),
            unit: "USD".into(),
            scenarios,
            methodology: "DCF + multiples".into(),
            key_assumptions: vec!["Steady revenue growth".into()],
            evidence_ids: vec![source_id.into()],
            confidence: 0.7,
            disclaimer: RESEARCH_DISCLAIMER.into(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .unwrap();
    }

    fn save_artifact(db: &Database, run_id: &str, kind: ArtifactKind, source_id: &str) {
        db.save_structured_artifact(&StructuredArtifact {
            id: format!("artifact-{kind}"),
            run_id: run_id.into(),
            kind,
            title: kind.to_string(),
            summary: "Structured evidence.".into(),
            columns: vec![ArtifactColumn {
                key: "metric".into(),
                label: "Metric".into(),
                unit: None,
                description: None,
            }],
            rows: vec![serde_json::json!({ "metric": "example" })],
            series: Vec::new(),
            evidence_ids: vec![source_id.into()],
            display_order: 10,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .unwrap();
    }

    fn seed_full_single_equity(db: &Database) -> (String, String) {
        let run_id = seed_run(db, "Analyze AAPL", AnalysisIntent::SingleEquity);
        let source_id = save_source(db, &run_id);
        save_plan(db, &run_id, AnalysisIntent::SingleEquity);
        save_methodology(db, &run_id);
        for kind in [
            BlockKind::Thesis,
            BlockKind::Risks,
            BlockKind::Financials,
            BlockKind::Valuation,
            BlockKind::Catalysts,
        ] {
            save_block(db, &run_id, kind, &source_id);
        }
        save_stance(db, &run_id);
        save_projection(db, &run_id, "AAPL", &source_id, valid_scenarios());
        save_criterion_answers(db, &run_id);
        (run_id, source_id)
    }

    #[test]
    fn finalization_requires_core_report_parts() {
        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let run_id = seed_run(&db, "Analyze AAPL", AnalysisIntent::SingleEquity);
        let errors = db.validate_finalization(&run_id).unwrap();
        assert!(errors.iter().any(|e| e.contains("research plan")));
        assert!(errors.iter().any(|e| e.contains("thesis")));
        assert!(errors.iter().any(|e| e.contains("sources")));
        assert!(errors.iter().any(|e| e.contains("methodology note")));
    }

    #[test]
    fn finalization_requires_intent_derived_blocks() {
        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let run_id = seed_run(&db, "Analyze AAPL", AnalysisIntent::SingleEquity);
        let source_id = save_source(&db, &run_id);
        save_plan(&db, &run_id, AnalysisIntent::SingleEquity);
        save_methodology(&db, &run_id);
        save_block(&db, &run_id, BlockKind::Thesis, &source_id);
        save_block(&db, &run_id, BlockKind::Risks, &source_id);
        save_stance(&db, &run_id);

        let errors = db.validate_finalization(&run_id).unwrap();
        assert!(
            errors
                .iter()
                .any(|e| e.contains("missing required valuation block"))
        );
        assert!(
            errors
                .iter()
                .any(|e| e.contains("missing required financials block"))
        );
        assert!(
            errors
                .iter()
                .any(|e| e.contains("missing required catalysts block"))
        );
    }

    #[test]
    fn finalization_rejects_when_every_source_is_low_reliability() {
        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let run_id = seed_run(&db, "Analyze AAPL", AnalysisIntent::SingleEquity);
        let source_id = save_source_with(&db, &run_id, "source-low", SourceReliability::Low);
        save_plan(&db, &run_id, AnalysisIntent::SingleEquity);
        save_methodology(&db, &run_id);
        for kind in [
            BlockKind::Thesis,
            BlockKind::Risks,
            BlockKind::Financials,
            BlockKind::Valuation,
            BlockKind::Catalysts,
        ] {
            save_block(&db, &run_id, kind, &source_id);
        }
        save_stance(&db, &run_id);
        save_projection(&db, &run_id, "AAPL", &source_id, valid_scenarios());
        save_criterion_answers(&db, &run_id);

        let errors = db.validate_finalization(&run_id).unwrap();
        assert!(errors.iter().any(|e| e.contains("low reliability")));
    }

    #[test]
    fn valid_single_equity_report_can_finalize() {
        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let (run_id, _) = seed_full_single_equity(&db);
        let errors = db.validate_finalization(&run_id).unwrap();
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn valid_comparison_report_requires_entities_and_matrix() {
        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let run_id = seed_run(&db, "Compare NVDA to AMD", AnalysisIntent::CompareEquities);
        let source_id = save_source(&db, &run_id);
        save_plan(&db, &run_id, AnalysisIntent::CompareEquities);
        save_methodology(&db, &run_id);
        for (id, name) in [("NVDA", "Nvidia"), ("AMD", "Advanced Micro Devices")] {
            db.save_entity(&Entity {
                id: id.into(),
                run_id: run_id.clone(),
                symbol: Some(id.into()),
                name: name.into(),
                exchange: Some("NASDAQ".into()),
                asset_type: "equity".into(),
                sector: Some("Technology".into()),
                country: Some("US".into()),
                confidence: 0.95,
                resolution_notes: None,
            })
            .unwrap();
            save_projection(&db, &run_id, id, &source_id, valid_scenarios());
        }
        save_block(&db, &run_id, BlockKind::Thesis, &source_id);
        save_block(&db, &run_id, BlockKind::Risks, &source_id);
        save_block(&db, &run_id, BlockKind::PeerComparison, &source_id);
        save_artifact(&db, &run_id, ArtifactKind::ComparisonMatrix, &source_id);
        save_stance(&db, &run_id);
        save_criterion_answers(&db, &run_id);

        let errors = db.validate_finalization(&run_id).unwrap();
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn finalization_requires_projection_for_single_equity() {
        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let run_id = seed_run(&db, "Analyze AAPL", AnalysisIntent::SingleEquity);
        let source_id = save_source(&db, &run_id);
        save_plan(&db, &run_id, AnalysisIntent::SingleEquity);
        save_methodology(&db, &run_id);
        for kind in [
            BlockKind::Thesis,
            BlockKind::Risks,
            BlockKind::Financials,
            BlockKind::Valuation,
            BlockKind::Catalysts,
        ] {
            save_block(&db, &run_id, kind, &source_id);
        }
        save_stance(&db, &run_id);

        let errors = db.validate_finalization(&run_id).unwrap();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("forward-looking projection")),
            "expected missing-projection error, got {errors:?}",
        );
    }

    #[test]
    fn finalization_requires_projection_per_entity_for_comparison() {
        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let run_id = seed_run(&db, "Compare NVDA to AMD", AnalysisIntent::CompareEquities);
        let source_id = save_source(&db, &run_id);
        save_plan(&db, &run_id, AnalysisIntent::CompareEquities);
        save_methodology(&db, &run_id);
        for (id, name) in [("NVDA", "Nvidia"), ("AMD", "Advanced Micro Devices")] {
            db.save_entity(&Entity {
                id: id.into(),
                run_id: run_id.clone(),
                symbol: Some(id.into()),
                name: name.into(),
                exchange: Some("NASDAQ".into()),
                asset_type: "equity".into(),
                sector: Some("Technology".into()),
                country: Some("US".into()),
                confidence: 0.95,
                resolution_notes: None,
            })
            .unwrap();
        }
        save_block(&db, &run_id, BlockKind::Thesis, &source_id);
        save_block(&db, &run_id, BlockKind::Risks, &source_id);
        save_block(&db, &run_id, BlockKind::PeerComparison, &source_id);
        save_artifact(&db, &run_id, ArtifactKind::ComparisonMatrix, &source_id);
        save_stance(&db, &run_id);
        save_projection(&db, &run_id, "NVDA", &source_id, valid_scenarios());
        save_criterion_answers(&db, &run_id);

        let errors = db.validate_finalization(&run_id).unwrap();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("missing projection for 'Advanced Micro Devices'")),
            "expected missing-per-entity projection error, got {errors:?}",
        );
    }

    #[test]
    fn projection_requires_all_scenario_labels() {
        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let (run_id, source_id) = seed_full_single_equity(&db);
        // Overwrite projection with only 2 scenarios to check cross-doc validation.
        let mut scenarios = valid_scenarios();
        scenarios.retain(|scenario| scenario.label != ScenarioLabel::Bear);
        save_projection(&db, &run_id, "AAPL", &source_id, scenarios);

        let errors = db.validate_finalization(&run_id).unwrap();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("missing required 'bear' scenario")),
            "expected missing-bear error, got {errors:?}",
        );
    }

    #[test]
    fn directional_stance_requires_counter_thesis() {
        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let (run_id, _) = seed_full_single_equity(&db);
        save_stance_with(&db, &run_id, StanceKind::Bullish, 0.5);

        let errors = db.validate_finalization(&run_id).unwrap();
        assert!(
            errors.iter().any(|e| e.contains("counter_thesis")),
            "expected counter-thesis requirement, got {errors:?}"
        );
    }

    #[test]
    fn blocking_uncertainty_caps_stance_confidence() {
        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let (run_id, _) = seed_full_single_equity(&db);
        save_stance_with(&db, &run_id, StanceKind::Mixed, 0.85);
        save_block(&db, &run_id, BlockKind::OpenQuestions, "source-1");
        db.save_uncertainty_entry(&UncertaintyEntry {
            id: "u-1".into(),
            run_id: run_id.clone(),
            question: "Is the backlog real?".into(),
            why_it_matters: "It drives FY26 revenue.".into(),
            attempted_resolution: "Filing silent.".into(),
            blocking: true,
            related_decision_criterion: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .unwrap();

        let errors = db.validate_finalization(&run_id).unwrap();
        assert!(
            errors.iter().any(|e| e.contains("blocking uncertainty")),
            "expected blocking-uncertainty error, got {errors:?}"
        );
    }

    #[test]
    fn finalize_requires_every_criterion_answered() {
        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let (run_id, _) = seed_full_single_equity(&db);
        // Remove one of the seeded answers to simulate a gap.
        let conn = db.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM decision_criterion_answers WHERE criterion = 'risk' AND run_id = ?1",
            [&run_id],
        )
        .unwrap();
        drop(conn);

        let errors = db.validate_finalization(&run_id).unwrap();
        assert!(
            errors
                .iter()
                .any(|e| e.contains("decision criterion 'risk'")),
            "expected missing-criterion-answer, got {errors:?}"
        );
    }
}
