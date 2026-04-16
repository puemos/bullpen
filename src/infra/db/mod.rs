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
                required_blocks TEXT NOT NULL,
                required_artifacts TEXT NOT NULL DEFAULT '[]',
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
                as_of TEXT,
                reliability TEXT NOT NULL,
                summary TEXT NOT NULL,
                FOREIGN KEY(run_id) REFERENCES analysis_runs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS metrics (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL,
                entity_id TEXT,
                metric TEXT NOT NULL,
                value TEXT NOT NULL,
                numeric_value REAL,
                unit TEXT,
                period TEXT,
                as_of TEXT NOT NULL,
                source_id TEXT NOT NULL,
                notes TEXT,
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
                entity_ids TEXT NOT NULL,
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
                entity_ids TEXT NOT NULL,
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
                watch_items TEXT NOT NULL,
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
            CREATE INDEX IF NOT EXISTS idx_run_progress_run_id ON run_progress(run_id);
            "#,
        )?;
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
            (id, analysis_id, agent_id, prompt_text, status, started_at, completed_at, error)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                run.id,
                run.analysis_id,
                run.agent_id,
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
            "SELECT id, analysis_id, agent_id, prompt_text, status, started_at, completed_at, error
             FROM analysis_runs WHERE analysis_id = ?1 ORDER BY started_at DESC",
        )?;
        let rows = stmt.query_map([analysis_id], |row| {
            Ok(AnalysisRun {
                id: row.get(0)?,
                analysis_id: row.get(1)?,
                agent_id: row.get(2)?,
                prompt_text: row.get(3)?,
                status: AnalysisStatus::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                started_at: row.get(5)?,
                completed_at: row.get(6)?,
                error: row.get(7)?,
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
            (id, run_id, intent, summary, decision_criteria, planned_checks, required_blocks, required_artifacts, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                plan.id,
                plan.run_id,
                plan.intent.to_string(),
                plan.summary,
                serde_json::to_string(&plan.decision_criteria)?,
                serde_json::to_string(&plan.planned_checks)?,
                serde_json::to_string(&plan.required_blocks)?,
                serde_json::to_string(&plan.required_artifacts)?,
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
            (id, run_id, title, url, publisher, source_type, retrieved_at, as_of, reliability, summary)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                source.id,
                source.run_id,
                source.title,
                source.url,
                source.publisher,
                source.source_type,
                source.retrieved_at,
                source.as_of,
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
            (id, run_id, entity_id, metric, value, numeric_value, unit, period, as_of, source_id, notes, prior_value, change_pct)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                metric.id,
                metric.run_id,
                metric.entity_id,
                metric.metric,
                metric.value,
                metric.numeric_value,
                metric.unit,
                metric.period,
                metric.as_of,
                metric.source_id,
                metric.notes,
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
            (id, run_id, kind, title, summary, columns, rows, series, evidence_ids, entity_ids, display_order, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
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
                serde_json::to_string(&artifact.entity_ids)?,
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
            (id, run_id, kind, title, body, evidence_ids, entity_ids, confidence, importance, display_order, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                block.id,
                block.run_id,
                block.kind.to_string(),
                block.title,
                block.body,
                serde_json::to_string(&block.evidence_ids)?,
                serde_json::to_string(&block.entity_ids)?,
                block.confidence,
                block.importance,
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
            (id, run_id, stance, horizon, confidence, summary, key_reasons, watch_items, what_would_change, disclaimer, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                stance.id,
                stance.run_id,
                stance.stance.to_string(),
                stance.horizon,
                stance.confidence,
                stance.summary,
                serde_json::to_string(&stance.key_reasons)?,
                serde_json::to_string(&stance.watch_items)?,
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
        let metrics = self.get_metrics(run_id)?;
        let artifacts = self.get_structured_artifacts(run_id)?;
        let final_stance = self.get_final_stance(run_id)?;
        let projections = self.get_projections(run_id)?;
        let mut errors = Vec::new();
        let source_ids: HashSet<&str> = sources.iter().map(|source| source.id.as_str()).collect();
        let block_kinds: HashSet<String> =
            blocks.iter().map(|block| block.kind.to_string()).collect();
        let artifact_kinds: HashSet<String> = artifacts
            .iter()
            .map(|artifact| artifact.kind.to_string())
            .collect();

        if research_plan.is_none() {
            errors.push("missing research plan".to_string());
        }

        if !blocks.iter().any(|b| b.kind == BlockKind::Thesis) {
            errors.push("missing required thesis block".to_string());
        }
        if !blocks.iter().any(|b| b.kind == BlockKind::Risks) {
            errors.push("missing required risks block".to_string());
        }
        if sources.is_empty() {
            errors.push("missing sources; submit at least one source".to_string());
        }
        if final_stance.is_none() {
            errors.push("missing final stance".to_string());
        }

        if let Some(plan) = &research_plan {
            for required in &plan.required_blocks {
                if !block_kinds.contains(required) {
                    errors.push(format!("missing required {required} block"));
                }
            }
            for required in &plan.required_artifacts {
                if !artifact_kinds.contains(required) {
                    errors.push(format!("missing required {required} artifact"));
                }
            }
            if matches!(
                plan.intent,
                AnalysisIntent::CompareEquities | AnalysisIntent::Watchlist
            ) {
                if entities.len() < 2 {
                    errors
                        .push("comparison reports must resolve at least two entities".to_string());
                }
                if !artifacts
                    .iter()
                    .any(|artifact| artifact.kind == ArtifactKind::ComparisonMatrix)
                {
                    errors.push(
                        "comparison reports require a comparison_matrix artifact".to_string(),
                    );
                }
            }
        }

        for metric in &metrics {
            if metric.numeric_value.is_none() {
                errors.push(format!(
                    "metric '{}' must include numeric_value",
                    metric.metric
                ));
            }
            if !source_ids.contains(metric.source_id.as_str()) {
                errors.push(format!(
                    "metric '{}' references unknown source_id '{}'",
                    metric.metric, metric.source_id
                ));
            }
        }

        for block in &blocks {
            if block.kind != BlockKind::OpenQuestions && block.evidence_ids.is_empty() {
                errors.push(format!(
                    "material block '{}' must include evidence_ids",
                    block.title
                ));
            }
            for evidence_id in &block.evidence_ids {
                if !source_ids.contains(evidence_id.as_str()) {
                    errors.push(format!(
                        "block '{}' references unknown evidence_id '{}'",
                        block.title, evidence_id
                    ));
                }
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
                ArtifactKind::BarChart | ArtifactKind::LineChart
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
            for evidence_id in &artifact.evidence_ids {
                if !source_ids.contains(evidence_id.as_str()) {
                    errors.push(format!(
                        "artifact '{}' references unknown evidence_id '{}'",
                        artifact.title, evidence_id
                    ));
                }
            }
        }

        if let Some(stance) = &final_stance {
            if stance.key_reasons.is_empty() {
                errors.push("final stance must include key_reasons".to_string());
            }
            if stance.watch_items.is_empty() {
                errors.push("final stance must include watch_items".to_string());
            }
            if stance.what_would_change.is_empty() {
                errors.push("final stance must include what_would_change".to_string());
            }
        }

        if let Some(plan) = &research_plan {
            let projection_intent = matches!(
                plan.intent,
                AnalysisIntent::SingleEquity | AnalysisIntent::CompareEquities
            );
            if projection_intent && projections.is_empty() {
                errors.push(
                    "missing forward-looking projection; submit_projection is required for single_equity and compare_equities"
                        .to_string(),
                );
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
            for evidence_id in &projection.evidence_ids {
                if !source_ids.contains(evidence_id.as_str()) {
                    errors.push(format!(
                        "projection '{}' references unknown evidence_id '{}'",
                        projection.metric, evidence_id
                    ));
                }
            }

            if projection.scenarios.len() < 3 {
                errors.push(format!(
                    "projection '{}' must include at least 3 scenarios (bull/base/bear)",
                    projection.metric
                ));
            }
            let labels: HashSet<String> = projection
                .scenarios
                .iter()
                .map(|scenario| scenario.label.to_ascii_lowercase())
                .collect();
            for required in ["bull", "base", "bear"] {
                if !labels.contains(required) {
                    errors.push(format!(
                        "projection '{}' missing required '{required}' scenario",
                        projection.metric
                    ));
                }
            }
            let prob_sum: f64 = projection
                .scenarios
                .iter()
                .map(|scenario| scenario.probability)
                .sum();
            if (prob_sum - 1.0).abs() > 0.05 {
                errors.push(format!(
                    "projection '{}' scenario probabilities sum to {:.2}; must sum to ~1.0",
                    projection.metric, prob_sum
                ));
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
            "SELECT id, run_id, intent, summary, decision_criteria, planned_checks, required_blocks, required_artifacts, created_at FROM research_plans WHERE run_id = ?1",
            [run_id],
            |row| {
                let criteria: String = row.get(4)?;
                let planned: String = row.get(5)?;
                let required_blocks: String = row.get(6)?;
                let required_artifacts: String = row.get(7)?;
                Ok(ResearchPlan {
                    id: row.get(0)?,
                    run_id: row.get(1)?,
                    intent: AnalysisIntent::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                    summary: row.get(3)?,
                    decision_criteria: serde_json::from_str(&criteria).unwrap_or_default(),
                    planned_checks: serde_json::from_str(&planned).unwrap_or_default(),
                    required_blocks: serde_json::from_str(&required_blocks).unwrap_or_default(),
                    required_artifacts: serde_json::from_str(&required_artifacts)
                        .unwrap_or_default(),
                    created_at: row.get(8)?,
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
            "SELECT id, run_id, title, url, publisher, source_type, retrieved_at, as_of, reliability, summary
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
                as_of: row.get(7)?,
                reliability: SourceReliability::from_str(&row.get::<_, String>(8)?)
                    .unwrap_or_default(),
                summary: row.get(9)?,
            })
        })?;
        collect_rows(rows)
    }

    fn get_metrics(&self, run_id: &str) -> Result<Vec<MetricSnapshot>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, run_id, entity_id, metric, value, numeric_value, unit, period, as_of, source_id, notes, prior_value, change_pct
             FROM metrics WHERE run_id = ?1 ORDER BY metric",
        )?;
        let rows = stmt.query_map([run_id], |row| {
            Ok(MetricSnapshot {
                id: row.get(0)?,
                run_id: row.get(1)?,
                entity_id: row.get(2)?,
                metric: row.get(3)?,
                value: row.get(4)?,
                numeric_value: row.get(5)?,
                unit: row.get(6)?,
                period: row.get(7)?,
                as_of: row.get(8)?,
                source_id: row.get(9)?,
                notes: row.get(10)?,
                prior_value: row.get(11)?,
                change_pct: row.get(12)?,
            })
        })?;
        collect_rows(rows)
    }

    fn get_structured_artifacts(&self, run_id: &str) -> Result<Vec<StructuredArtifact>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, run_id, kind, title, summary, columns, rows, series, evidence_ids, entity_ids, display_order, created_at
             FROM structured_artifacts WHERE run_id = ?1 ORDER BY display_order, created_at",
        )?;
        let rows = stmt.query_map([run_id], |row| {
            let columns: String = row.get(5)?;
            let rows_json: String = row.get(6)?;
            let series: String = row.get(7)?;
            let evidence: String = row.get(8)?;
            let entities: String = row.get(9)?;
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
                entity_ids: serde_json::from_str(&entities).unwrap_or_default(),
                display_order: row.get(10)?,
                created_at: row.get(11)?,
            })
        })?;
        collect_rows(rows)
    }

    fn get_blocks(&self, run_id: &str) -> Result<Vec<AnalysisBlock>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, run_id, kind, title, body, evidence_ids, entity_ids, confidence, importance, display_order, created_at
             FROM analysis_blocks WHERE run_id = ?1 ORDER BY display_order, created_at",
        )?;
        let rows = stmt.query_map([run_id], |row| {
            let evidence: String = row.get(5)?;
            let entities: String = row.get(6)?;
            Ok(AnalysisBlock {
                id: row.get(0)?,
                run_id: row.get(1)?,
                kind: BlockKind::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                title: row.get(3)?,
                body: row.get(4)?,
                evidence_ids: serde_json::from_str(&evidence).unwrap_or_default(),
                entity_ids: serde_json::from_str(&entities).unwrap_or_default(),
                confidence: row.get(7)?,
                importance: row.get(8)?,
                display_order: row.get(9)?,
                created_at: row.get(10)?,
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

    fn get_final_stance(&self, run_id: &str) -> Result<Option<FinalStance>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.query_row(
            "SELECT id, run_id, stance, horizon, confidence, summary, key_reasons, watch_items, what_would_change, disclaimer, created_at
             FROM final_stances WHERE run_id = ?1",
            [run_id],
            |row| {
                let reasons: String = row.get(6)?;
                let watch: String = row.get(7)?;
                let changes: String = row.get(8)?;
                Ok(FinalStance {
                    id: row.get(0)?,
                    run_id: row.get(1)?,
                    stance: StanceKind::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                    horizon: row.get(3)?,
                    confidence: row.get(4)?,
                    summary: row.get(5)?,
                    key_reasons: serde_json::from_str(&reasons).unwrap_or_default(),
                    watch_items: serde_json::from_str(&watch).unwrap_or_default(),
                    what_would_change: serde_json::from_str(&changes).unwrap_or_default(),
                    disclaimer: row.get(9)?,
                    created_at: row.get(10)?,
                })
            },
        )
        .optional()
        .map_err(Into::into)
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
            prompt_text: prompt.into(),
            status: AnalysisStatus::Running,
            started_at: now,
            completed_at: None,
            error: None,
        })
        .unwrap();
        run_id
    }

    fn save_plan(
        db: &Database,
        run_id: &str,
        intent: AnalysisIntent,
        required_artifacts: Vec<String>,
    ) {
        db.save_research_plan(&ResearchPlan {
            id: "plan-1".into(),
            run_id: run_id.into(),
            intent,
            summary: "Assess the research question.".into(),
            decision_criteria: vec!["valuation".into(), "risk".into()],
            planned_checks: vec!["Check primary sources.".into()],
            required_blocks: vec!["thesis".into(), "risks".into()],
            required_artifacts,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .unwrap();
    }

    fn save_source(db: &Database, run_id: &str) -> String {
        let source_id = "source-1".to_string();
        db.save_source(&Source {
            id: source_id.clone(),
            run_id: run_id.into(),
            title: "Company filing".into(),
            url: Some("https://example.com/filing".into()),
            publisher: Some("Example".into()),
            source_type: "filing".into(),
            retrieved_at: chrono::Utc::now().to_rfc3339(),
            as_of: Some("2026-04-16".into()),
            reliability: SourceReliability::Primary,
            summary: "Primary source.".into(),
        })
        .unwrap();
        source_id
    }

    fn save_block(db: &Database, run_id: &str, kind: BlockKind, source_id: &str) {
        db.save_block(&AnalysisBlock {
            id: format!("block-{kind}"),
            run_id: run_id.into(),
            kind,
            title: kind.to_string(),
            body: "Evidence-backed block.".into(),
            evidence_ids: vec![source_id.into()],
            entity_ids: Vec::new(),
            confidence: 0.8,
            importance: "high".into(),
            display_order: 10,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .unwrap();
    }

    fn save_stance(db: &Database, run_id: &str) {
        db.save_final_stance(&FinalStance {
            id: "stance-1".into(),
            run_id: run_id.into(),
            stance: StanceKind::Neutral,
            horizon: "12 months".into(),
            confidence: 0.7,
            summary: "Balanced evidence.".into(),
            key_reasons: vec!["Valuation is mixed.".into()],
            watch_items: vec!["Margin trend.".into()],
            what_would_change: vec!["Better growth visibility.".into()],
            disclaimer: "Research only. Not investment advice.".into(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .unwrap();
    }

    fn valid_scenarios() -> Vec<ProjectionScenario> {
        vec![
            ProjectionScenario {
                label: "bull".into(),
                target_value: 230.0,
                target_label: "$230".into(),
                upside_pct: 0.26,
                probability: 0.25,
                rationale: "Upside thesis pans out.".into(),
                catalysts: vec!["Product launch".into()],
                risks: vec!["Execution slip".into()],
            },
            ProjectionScenario {
                label: "base".into(),
                target_value: 205.0,
                target_label: "$205".into(),
                upside_pct: 0.13,
                probability: 0.55,
                rationale: "Consensus path holds.".into(),
                catalysts: vec!["Buyback cadence".into()],
                risks: vec!["Margin pressure".into()],
            },
            ProjectionScenario {
                label: "bear".into(),
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
            disclaimer: "Research only. Not investment advice.".into(),
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
            entity_ids: Vec::new(),
            display_order: 10,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .unwrap();
    }

    #[test]
    fn finalization_requires_core_report_parts() {
        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let run_id = seed_run(&db, "Analyze AAPL", AnalysisIntent::SingleEquity);
        let errors = db.validate_finalization(&run_id).unwrap();
        assert!(errors.iter().any(|e| e.contains("research plan")));
        assert!(errors.iter().any(|e| e.contains("thesis")));
        assert!(errors.iter().any(|e| e.contains("sources")));
    }

    #[test]
    fn finalization_rejects_missing_required_artifact() {
        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let run_id = seed_run(&db, "Analyze AAPL", AnalysisIntent::SingleEquity);
        let source_id = save_source(&db, &run_id);
        save_plan(
            &db,
            &run_id,
            AnalysisIntent::SingleEquity,
            vec!["metric_table".into()],
        );
        save_block(&db, &run_id, BlockKind::Thesis, &source_id);
        save_block(&db, &run_id, BlockKind::Risks, &source_id);
        save_stance(&db, &run_id);

        let errors = db.validate_finalization(&run_id).unwrap();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("missing required metric_table artifact"))
        );
    }

    #[test]
    fn finalization_rejects_invalid_evidence_and_metrics() {
        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let run_id = seed_run(&db, "Analyze AAPL", AnalysisIntent::SingleEquity);
        let source_id = save_source(&db, &run_id);
        save_plan(&db, &run_id, AnalysisIntent::SingleEquity, Vec::new());
        save_block(&db, &run_id, BlockKind::Thesis, "missing-source");
        save_block(&db, &run_id, BlockKind::Risks, &source_id);
        save_stance(&db, &run_id);
        db.save_metric(&MetricSnapshot {
            id: "metric-1".into(),
            run_id: run_id.clone(),
            entity_id: None,
            metric: "revenue".into(),
            value: "$10".into(),
            numeric_value: None,
            unit: Some("USD".into()),
            period: Some("FY2025".into()),
            as_of: "2026-04-16".into(),
            source_id,
            notes: None,
            prior_value: None,
            change_pct: None,
        })
        .unwrap();

        let errors = db.validate_finalization(&run_id).unwrap();
        assert!(errors.iter().any(|error| error.contains("numeric_value")));
        assert!(
            errors
                .iter()
                .any(|error| error.contains("unknown evidence_id 'missing-source'"))
        );
    }

    #[test]
    fn valid_single_equity_report_can_finalize() {
        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let run_id = seed_run(&db, "Analyze AAPL", AnalysisIntent::SingleEquity);
        let source_id = save_source(&db, &run_id);
        save_plan(
            &db,
            &run_id,
            AnalysisIntent::SingleEquity,
            vec!["metric_table".into()],
        );
        save_block(&db, &run_id, BlockKind::Thesis, &source_id);
        save_block(&db, &run_id, BlockKind::Risks, &source_id);
        save_artifact(&db, &run_id, ArtifactKind::MetricTable, &source_id);
        save_stance(&db, &run_id);
        save_projection(&db, &run_id, "AAPL", &source_id, valid_scenarios());

        let errors = db.validate_finalization(&run_id).unwrap();
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn valid_comparison_report_requires_entities_and_matrix() {
        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let run_id = seed_run(&db, "Compare NVDA to AMD", AnalysisIntent::CompareEquities);
        let source_id = save_source(&db, &run_id);
        save_plan(
            &db,
            &run_id,
            AnalysisIntent::CompareEquities,
            vec!["comparison_matrix".into()],
        );
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
        save_artifact(&db, &run_id, ArtifactKind::ComparisonMatrix, &source_id);
        save_stance(&db, &run_id);

        let errors = db.validate_finalization(&run_id).unwrap();
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn finalization_requires_projection_for_single_equity() {
        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let run_id = seed_run(&db, "Analyze AAPL", AnalysisIntent::SingleEquity);
        let source_id = save_source(&db, &run_id);
        save_plan(
            &db,
            &run_id,
            AnalysisIntent::SingleEquity,
            vec!["metric_table".into()],
        );
        save_block(&db, &run_id, BlockKind::Thesis, &source_id);
        save_block(&db, &run_id, BlockKind::Risks, &source_id);
        save_artifact(&db, &run_id, ArtifactKind::MetricTable, &source_id);
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
        save_plan(
            &db,
            &run_id,
            AnalysisIntent::CompareEquities,
            vec!["comparison_matrix".into()],
        );
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
        save_artifact(&db, &run_id, ArtifactKind::ComparisonMatrix, &source_id);
        save_stance(&db, &run_id);
        save_projection(&db, &run_id, "NVDA", &source_id, valid_scenarios());

        let errors = db.validate_finalization(&run_id).unwrap();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("missing projection for 'Advanced Micro Devices'")),
            "expected missing-per-entity projection error, got {errors:?}",
        );
    }

    #[test]
    fn projection_scenarios_must_sum_to_one() {
        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let run_id = seed_run(&db, "Analyze AAPL", AnalysisIntent::SingleEquity);
        let source_id = save_source(&db, &run_id);
        save_plan(
            &db,
            &run_id,
            AnalysisIntent::SingleEquity,
            vec!["metric_table".into()],
        );
        save_block(&db, &run_id, BlockKind::Thesis, &source_id);
        save_block(&db, &run_id, BlockKind::Risks, &source_id);
        save_artifact(&db, &run_id, ArtifactKind::MetricTable, &source_id);
        save_stance(&db, &run_id);

        let mut scenarios = valid_scenarios();
        for scenario in &mut scenarios {
            scenario.probability = 0.30;
        }
        save_projection(&db, &run_id, "AAPL", &source_id, scenarios);

        let errors = db.validate_finalization(&run_id).unwrap();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("probabilities sum")),
            "expected probabilities-sum error, got {errors:?}",
        );
    }

    #[test]
    fn projection_requires_all_scenario_labels() {
        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let run_id = seed_run(&db, "Analyze AAPL", AnalysisIntent::SingleEquity);
        let source_id = save_source(&db, &run_id);
        save_plan(
            &db,
            &run_id,
            AnalysisIntent::SingleEquity,
            vec!["metric_table".into()],
        );
        save_block(&db, &run_id, BlockKind::Thesis, &source_id);
        save_block(&db, &run_id, BlockKind::Risks, &source_id);
        save_artifact(&db, &run_id, ArtifactKind::MetricTable, &source_id);
        save_stance(&db, &run_id);

        let mut scenarios = valid_scenarios();
        scenarios.retain(|scenario| scenario.label != "bear");
        save_projection(&db, &run_id, "AAPL", &source_id, scenarios);

        let errors = db.validate_finalization(&run_id).unwrap();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("missing required 'bear' scenario")),
            "expected missing-bear error, got {errors:?}",
        );
    }
}
