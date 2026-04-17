use crate::infra::shell::find_bin;
use serde::Serialize;
use std::process::Stdio;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize)]
struct UpdateLog {
    stream: &'static str,
    line: String,
}

#[derive(Debug, Clone, Serialize)]
struct UpdateError {
    message: String,
}

fn emit_log(app: &AppHandle, stream: &'static str, line: String) {
    let _ = app.emit("update:log", UpdateLog { stream, line });
}

fn emit_error(app: &AppHandle, message: String) {
    let _ = app.emit("update:error", UpdateError { message });
}

#[tauri::command]
pub async fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub async fn run_self_update(app: AppHandle) -> Result<(), String> {
    let brew = match find_bin("brew") {
        Some(path) => path,
        None => {
            let msg =
                "Homebrew not found in PATH. Open Terminal and run `brew upgrade --cask bullpen`."
                    .to_string();
            emit_error(&app, msg.clone());
            return Err(msg);
        }
    };

    emit_log(
        &app,
        "stdout",
        format!("$ {} upgrade --cask bullpen", brew.display()),
    );

    let mut child = match Command::new(&brew)
        .args(["upgrade", "--cask", "bullpen"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            let msg = format!("Failed to spawn brew: {err}");
            emit_error(&app, msg.clone());
            return Err(msg);
        }
    };

    let mut already_up_to_date = false;

    if let Some(stdout) = child.stdout.take() {
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                emit_log(&app_clone, "stdout", line);
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        let app_clone = app.clone();
        // Brew prints status to stderr. Watch for the "already up-to-date" hint there too.
        tauri::async_runtime::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                emit_log(&app_clone, "stderr", line);
            }
        });
    }

    let status = match child.wait().await {
        Ok(status) => status,
        Err(err) => {
            let msg = format!("Failed waiting for brew: {err}");
            emit_error(&app, msg.clone());
            return Err(msg);
        }
    };

    if !status.success() {
        let msg = format!("brew upgrade failed with status {status}");
        emit_error(&app, msg.clone());
        return Err(msg);
    }

    // Best-effort detection of "already up-to-date" — brew prints this to stderr.
    // We don't currently buffer the stream, so detection happens via a quick second probe.
    let probe = Command::new(&brew)
        .args(["outdated", "--cask", "bullpen"])
        .output()
        .await;
    if let Ok(out) = probe
        && out.status.success()
        && String::from_utf8_lossy(&out.stdout).trim().is_empty()
    {
        already_up_to_date = true;
    }

    let _ = app.emit("update:done", ());

    if already_up_to_date {
        emit_log(
            &app,
            "stdout",
            "Already up to date — skipping restart.".to_string(),
        );
        return Ok(());
    }

    // Give the frontend a beat to render the success state before relaunch.
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;
    app.restart();
}
