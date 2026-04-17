use serde::Serialize;

/// Error type returned by Tauri IPC commands.
///
/// Preserves the existing wire format (a JSON string) so the frontend does not
/// need to change, while providing `From` conversions from the common error
/// types flowing through the commands layer. This replaces the scattered
/// `.map_err(|err| err.to_string())` boilerplate with plain `?`.
///
/// The full error chain is preserved for logging via `Debug`/`Display`, but
/// only the top-level message reaches the frontend — callers that need
/// structured errors should log the chain themselves before returning.
#[derive(Debug)]
pub struct CommandError(String);

impl CommandError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CommandError {}

impl Serialize for CommandError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl From<anyhow::Error> for CommandError {
    fn from(err: anyhow::Error) -> Self {
        // Include the full context chain when logging, but only the top-level
        // message goes to the frontend.
        log::error!("command error: {err:#}");
        Self(err.to_string())
    }
}

impl From<String> for CommandError {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for CommandError {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        Self(err.to_string())
    }
}

impl<T> From<std::sync::PoisonError<T>> for CommandError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        Self(format!("lock poisoned: {err}"))
    }
}

impl From<serde_json::Error> for CommandError {
    fn from(err: serde_json::Error) -> Self {
        Self(err.to_string())
    }
}

impl From<reqwest::Error> for CommandError {
    fn from(err: reqwest::Error) -> Self {
        Self(err.to_string())
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for CommandError {
    fn from(err: tokio::sync::oneshot::error::RecvError) -> Self {
        Self(err.to_string())
    }
}
