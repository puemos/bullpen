use crate::infra::db::Database;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;

pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub active_runs: Arc<Mutex<HashMap<String, CancellationToken>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            db: Arc::new(Mutex::new(
                Database::open().expect("failed to open CrazyLines database"),
            )),
            active_runs: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
