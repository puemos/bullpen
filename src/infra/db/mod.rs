use crate::domain::*;
use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, params};
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
                    .join("CrazyLines")
                    .join("db.sqlite");
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Some(appdata) = std::env::var_os("APPDATA") {
                return PathBuf::from(appdata).join("CrazyLines").join("db.sqlite");
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
                planned_checks TEXT NOT NULL,
                required_blocks TEXT NOT NULL,
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
                unit TEXT,
                period TEXT,
                as_of TEXT NOT NULL,
                source_id TEXT NOT NULL,
                notes TEXT,
                FOREIGN KEY(run_id) REFERENCES analysis_runs(id) ON DELETE CASCADE,
                FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE CASCADE
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

            CREATE INDEX IF NOT EXISTS idx_analysis_runs_analysis_id ON analysis_runs(analysis_id);
            CREATE INDEX IF NOT EXISTS idx_entities_run_id ON entities(run_id);
            CREATE INDEX IF NOT EXISTS idx_sources_run_id ON sources(run_id);
            CREATE INDEX IF NOT EXISTS idx_metrics_run_id ON metrics(run_id);
            CREATE INDEX IF NOT EXISTS idx_blocks_run_id ON analysis_blocks(run_id);
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

    pub fn get_report(&self, analysis_id: &str) -> Result<Option<AnalysisReport>> {
        let analysis = match self.get_analysis(analysis_id)? {
            Some(analysis) => analysis,
            None => return Ok(None),
        };
        let runs = self.get_runs(analysis_id)?;
        let active_run = analysis
            .active_run_id
            .clone()
            .or_else(|| runs.first().map(|r| r.id.clone()));

        let Some(run_id) = active_run else {
            return Ok(Some(AnalysisReport {
                analysis,
                runs,
                research_plan: None,
                entities: Vec::new(),
                sources: Vec::new(),
                metrics: Vec::new(),
                blocks: Vec::new(),
                final_stance: None,
            }));
        };

        Ok(Some(AnalysisReport {
            analysis,
            runs,
            research_plan: self.get_research_plan(&run_id)?,
            entities: self.get_entities(&run_id)?,
            sources: self.get_sources(&run_id)?,
            metrics: self.get_metrics(&run_id)?,
            blocks: self.get_blocks(&run_id)?,
            final_stance: self.get_final_stance(&run_id)?,
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

    fn get_runs(&self, analysis_id: &str) -> Result<Vec<AnalysisRun>> {
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
            (id, run_id, intent, summary, planned_checks, required_blocks, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                plan.id,
                plan.run_id,
                plan.intent.to_string(),
                plan.summary,
                serde_json::to_string(&plan.planned_checks)?,
                serde_json::to_string(&plan.required_blocks)?,
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
            (id, run_id, entity_id, metric, value, unit, period, as_of, source_id, notes)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                metric.id,
                metric.run_id,
                metric.entity_id,
                metric.metric,
                metric.value,
                metric.unit,
                metric.period,
                metric.as_of,
                metric.source_id,
                metric.notes
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

    pub fn validate_finalization(&self, run_id: &str) -> Result<Vec<String>> {
        let blocks = self.get_blocks(run_id)?;
        let sources = self.get_sources(run_id)?;
        let final_stance = self.get_final_stance(run_id)?;
        let mut errors = Vec::new();

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
        if blocks
            .iter()
            .any(|b| b.kind != BlockKind::OpenQuestions && b.evidence_ids.is_empty())
        {
            errors.push("material blocks must include evidence_ids".to_string());
        }

        Ok(errors)
    }

    fn get_research_plan(&self, run_id: &str) -> Result<Option<ResearchPlan>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.query_row(
            "SELECT id, run_id, intent, summary, planned_checks, required_blocks, created_at FROM research_plans WHERE run_id = ?1",
            [run_id],
            |row| {
                let planned: String = row.get(4)?;
                let required: String = row.get(5)?;
                Ok(ResearchPlan {
                    id: row.get(0)?,
                    run_id: row.get(1)?,
                    intent: AnalysisIntent::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                    summary: row.get(3)?,
                    planned_checks: serde_json::from_str(&planned).unwrap_or_default(),
                    required_blocks: serde_json::from_str(&required).unwrap_or_default(),
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
            "SELECT id, run_id, entity_id, metric, value, unit, period, as_of, source_id, notes
             FROM metrics WHERE run_id = ?1 ORDER BY metric",
        )?;
        let rows = stmt.query_map([run_id], |row| {
            Ok(MetricSnapshot {
                id: row.get(0)?,
                run_id: row.get(1)?,
                entity_id: row.get(2)?,
                metric: row.get(3)?,
                value: row.get(4)?,
                unit: row.get(5)?,
                period: row.get(6)?,
                as_of: row.get(7)?,
                source_id: row.get(8)?,
                notes: row.get(9)?,
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

    #[test]
    fn finalization_requires_core_report_parts() {
        let db = Database::open_at(PathBuf::from(":memory:")).unwrap();
        let run_id = "run-1".to_string();
        let now = chrono::Utc::now().to_rfc3339();
        db.save_analysis(&Analysis {
            id: "a".into(),
            title: "Analysis".into(),
            user_prompt: "Analyze AAPL".into(),
            intent: AnalysisIntent::SingleEquity,
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
            prompt_text: "Analyze AAPL".into(),
            status: AnalysisStatus::Running,
            started_at: now,
            completed_at: None,
            error: None,
        })
        .unwrap();
        let errors = db.validate_finalization(&run_id).unwrap();
        assert!(errors.iter().any(|e| e.contains("thesis")));
        assert!(errors.iter().any(|e| e.contains("sources")));
    }
}
