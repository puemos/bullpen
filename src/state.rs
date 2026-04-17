use crate::infra::db::Database;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;

/// App-wide state shared with every Tauri command.
///
/// `Database` is `Clone` and already wraps the SQLite connection in its own
/// mutex, so we don't need a second outer `Mutex<Database>`. Dropping it lets
/// commands run concurrently right up to the connection acquire.
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub active_runs: Arc<Mutex<HashMap<String, CancellationToken>>>,
}

impl AppState {
    /// Open the default on-disk database and construct the state.
    ///
    /// Fails if the database cannot be opened (permissions, disk full,
    /// migration error). Callers should surface this to the user rather than
    /// panic on startup.
    pub fn try_new() -> Result<Self> {
        Ok(Self {
            db: Database::open()?,
            active_runs: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}
