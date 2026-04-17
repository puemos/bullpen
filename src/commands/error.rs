use crate::infra::acp::analysis_generator::{AcpCancelled, AcpTimeout};
use serde::Serialize;

/// Error returned by Tauri IPC commands.
///
/// Serializes as `{ "message": String, "kind": CommandErrorKind }` so the
/// frontend can branch on `err.kind` instead of substring-matching
/// `err.message`.
#[derive(Debug)]
pub struct CommandError {
    message: String,
    kind: CommandErrorKind,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandErrorKind {
    Internal,
    Validation,
    NotFound,
    Cancelled,
    Timeout,
}

impl CommandError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: CommandErrorKind::Internal,
        }
    }

    #[must_use]
    pub fn with_kind(message: impl Into<String>, kind: CommandErrorKind) -> Self {
        Self {
            message: message.into(),
            kind,
        }
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn kind(&self) -> CommandErrorKind {
        self.kind
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CommandError {}

impl Serialize for CommandError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("CommandError", 2)?;
        state.serialize_field("message", &self.message)?;
        state.serialize_field("kind", &self.kind)?;
        state.end()
    }
}

impl From<anyhow::Error> for CommandError {
    fn from(err: anyhow::Error) -> Self {
        log::error!("command error: {err:#}");
        let kind = if err.downcast_ref::<AcpCancelled>().is_some() {
            CommandErrorKind::Cancelled
        } else if err.downcast_ref::<AcpTimeout>().is_some() {
            CommandErrorKind::Timeout
        } else {
            CommandErrorKind::Internal
        };
        Self {
            message: err.to_string(),
            kind,
        }
    }
}

impl From<String> for CommandError {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for CommandError {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        Self::new(err.to_string())
    }
}

impl<T> From<std::sync::PoisonError<T>> for CommandError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        Self::new(format!("lock poisoned: {err}"))
    }
}

impl From<serde_json::Error> for CommandError {
    fn from(err: serde_json::Error) -> Self {
        Self::new(err.to_string())
    }
}

impl From<reqwest::Error> for CommandError {
    fn from(err: reqwest::Error) -> Self {
        Self::new(err.to_string())
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for CommandError {
    fn from(err: tokio::sync::oneshot::error::RecvError) -> Self {
        Self::new(err.to_string())
    }
}
