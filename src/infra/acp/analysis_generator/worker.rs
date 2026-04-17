use super::client::BullpenClient;
use crate::domain::RunContext;
use crate::infra::progress::ProgressEventPayload;
use crate::prompts;
use agent_client_protocol::{
    Agent, ClientCapabilities, ClientSideConnection, ContentBlock, FileSystemCapability,
    Implementation, InitializeRequest, McpServer, McpServerStdio, NewSessionRequest, PromptRequest,
    ProtocolVersion, TextContent,
};
use anyhow::{Context, Result};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::runtime::Builder;
use tokio::task::LocalSet;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tokio_util::sync::CancellationToken;

pub type ProgressTx = tokio::sync::mpsc::UnboundedSender<ProgressEventPayload>;

pub struct GenerateAnalysisInput {
    pub run_context: RunContext,
    pub agent_command: String,
    pub agent_args: Vec<String>,
    pub model_flag: Option<(String, String)>,
    pub model_env: Option<(String, String)>,
    pub progress_tx: Option<ProgressTx>,
    pub mcp_server_binary: Option<PathBuf>,
    pub db_path: PathBuf,
    pub timeout_secs: Option<u64>,
    pub cancel_token: Option<CancellationToken>,
}

pub struct GenerateAnalysisResult {
    pub messages: Vec<String>,
    pub thoughts: Vec<String>,
    pub logs: Vec<String>,
}

/// Sentinel error returned when the user cancelled a run.
///
/// The caller can detect cancellation with `err.downcast_ref::<AcpCancelled>()`
/// instead of matching on the error message, which would be brittle.
#[derive(Debug, thiserror::Error)]
#[error("agent generation cancelled by user")]
pub struct AcpCancelled;

/// Sentinel error returned when the agent exceeds `timeout_secs`.
#[derive(Debug, thiserror::Error)]
#[error("agent timed out after {0}s")]
pub struct AcpTimeout(pub u64);

/// RAII guard that cancels its token when dropped.
///
/// The worker runs on a detached OS thread whose lifetime is not bound to the
/// parent future. If the parent is dropped (Tauri command cancelled, panic,
/// app shutdown) the detached thread would otherwise keep running and the
/// spawned agent child process would survive — `kill_on_drop` only fires when
/// the `Child` is dropped on the thread that owns it. Cancelling the token
/// forces the worker's `select!` to take the cancellation branch, which kills
/// the child and tears everything down cleanly.
struct CancelOnDrop(CancellationToken);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

pub async fn generate_with_acp(mut input: GenerateAnalysisInput) -> Result<GenerateAnalysisResult> {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let timeout_secs = input.timeout_secs.unwrap_or(1800);

    let cancel_token = input.cancel_token.clone().unwrap_or_default();
    input.cancel_token = Some(cancel_token.clone());
    let _cancel_guard = CancelOnDrop(cancel_token);

    thread::spawn(move || {
        let runtime = Builder::new_current_thread().enable_all().build();
        let result = match runtime {
            Ok(rt) => {
                let local = LocalSet::new();
                local.block_on(&rt, async move {
                    let cancel_token = input.cancel_token.clone();
                    tokio::select! {
                        result = tokio::time::timeout(
                            Duration::from_secs(timeout_secs),
                            generate_with_acp_inner(input),
                        ) => result.map_err(|_| anyhow::Error::new(AcpTimeout(timeout_secs)))?,
                        () = async {
                            if let Some(token) = cancel_token {
                                token.cancelled().await;
                            } else {
                                std::future::pending::<()>().await;
                            }
                        } => Err(anyhow::Error::new(AcpCancelled)),
                    }
                })
            }
            Err(err) => Err(err.into()),
        };
        let _ = sender.send(result);
    });

    receiver
        .await
        .unwrap_or_else(|_| Err(anyhow::anyhow!("ACP worker thread unexpectedly closed")))
}

async fn generate_with_acp_inner(input: GenerateAnalysisInput) -> Result<GenerateAnalysisResult> {
    let GenerateAnalysisInput {
        run_context,
        agent_command,
        mut agent_args,
        model_flag,
        model_env,
        progress_tx,
        mcp_server_binary,
        db_path,
        timeout_secs: _,
        cancel_token,
    } = input;

    if let Some((flag, value)) = &model_flag {
        agent_args.push(flag.clone());
        agent_args.push(value.clone());
    }

    let logs = Arc::new(Mutex::new(Vec::new()));
    let log_fn = |msg: String| {
        if let Ok(mut guard) = logs.lock() {
            guard.push(msg.clone());
        }
        if let Some(tx) = &progress_tx {
            let _ = tx.send(ProgressEventPayload::Log(msg));
        }
    };

    log_fn(format!("spawn: {} {}", agent_command, agent_args.join(" ")));

    let mut cmd = Command::new(&agent_command);
    cmd.args(&agent_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    if let Some((key, value)) = &model_env {
        cmd.env(key, value);
    }

    #[cfg(unix)]
    {
        cmd.process_group(0);
    }

    let mut child = cmd.spawn().with_context(|| {
        format!(
            "failed to spawn agent process: {} {}",
            agent_command,
            agent_args.join(" ")
        )
    })?;
    let child_pid = child.id().unwrap_or(0);
    log_fn(format!("spawned pid: {child_pid}"));

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to get agent stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to get agent stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to get agent stderr"))?;

    let stderr_logs = logs.clone();
    let stderr_tx = progress_tx.clone();
    tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let msg = format!("stderr: {line}");
            if let Ok(mut guard) = stderr_logs.lock() {
                guard.push(msg.clone());
            }
            if let Some(tx) = &stderr_tx {
                let _ = tx.send(ProgressEventPayload::Log(msg));
            }
        }
    });

    let client = BullpenClient::new(progress_tx.clone());
    let messages = client.messages.clone();
    let thoughts = client.thoughts.clone();
    let finalization_received = client.finalization_received.clone();

    let stdin_compat = stdin.compat_write();
    let stdout_compat = stdout.compat();
    let spawn_fn = |fut: Pin<Box<dyn Future<Output = ()> + 'static>>| {
        tokio::task::spawn_local(fut);
    };
    let (connection, io_future) =
        ClientSideConnection::new(client, stdin_compat, stdout_compat, spawn_fn);
    let io_handle = tokio::task::spawn_local(async move {
        let _ = io_future.await;
    });

    let result = async {
        connection
            .initialize(
                InitializeRequest::new(ProtocolVersion::V1)
                    .client_info(Implementation::new("bullpen", env!("CARGO_PKG_VERSION")))
                    .client_capabilities(build_client_capabilities()),
            )
            .await
            .context("ACP initialize failed")?;

        let temp_cwd = tempfile::tempdir().context("create temp working directory")?;
        let cwd = temp_cwd.path().to_path_buf();
        let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("bullpen"));
        let mcp_path = resolve_mcp_server_path(mcp_server_binary.as_ref(), &current_exe);
        let context_file =
            tempfile::NamedTempFile::new().context("create analysis context file")?;
        std::fs::write(
            context_file.path(),
            serde_json::to_string(&run_context).context("serialize run context")?,
        )
        .context("write analysis context file")?;

        let mcp_servers = vec![McpServer::Stdio(
            McpServerStdio::new("bullpen-analysis", mcp_path.clone()).args(vec![
                "--analysis-mcp-server".to_string(),
                "--analysis-context".to_string(),
                context_file.path().to_string_lossy().to_string(),
                "--db-path".to_string(),
                db_path.to_string_lossy().to_string(),
            ]),
        )];

        log_fn(format!(
            "new_session (mcp server: {} --analysis-mcp-server)",
            mcp_path.display()
        ));
        let session = connection
            .new_session(NewSessionRequest::new(cwd).mcp_servers(mcp_servers))
            .await
            .context("ACP new_session failed")?;

        let prompt_text = prompts::build_analysis_prompt(&run_context)?;
        let prompt_result = connection
            .prompt(PromptRequest::new(
                session.session_id,
                vec![ContentBlock::Text(TextContent::new(prompt_text))],
            ))
            .await;

        if let Err(err) = prompt_result
            && !*finalization_received.lock().unwrap()
        {
            return Err(anyhow::anyhow!("ACP prompt failed: {err:?}"));
        }

        loop {
            if *finalization_received.lock().unwrap() {
                log_fn("finalization received; terminating agent".to_string());
                let _ = child.start_kill();
                kill_process_group(child_pid);
                let _ = child.wait().await;
                break;
            }

            if let Some(token) = &cancel_token
                && token.is_cancelled()
            {
                let _ = child.start_kill();
                kill_process_group(child_pid);
                let _ = child.wait().await;
                return Err(anyhow::anyhow!("agent generation cancelled by user"));
            }

            tokio::select! {
                status = child.wait() => {
                    log_fn(format!("agent exited: {}", status?));
                    break;
                }
                () = tokio::time::sleep(Duration::from_millis(200)) => {}
            }
        }

        let _ = io_handle.await;
        if !*finalization_received.lock().unwrap() {
            return Err(anyhow::anyhow!(
                "agent completed but did not call finalize_analysis"
            ));
        }

        Ok(GenerateAnalysisResult {
            messages: messages.lock().unwrap().clone(),
            thoughts: thoughts.lock().unwrap().clone(),
            logs: logs.lock().unwrap().clone(),
        })
    }
    .await;

    if result.is_err() {
        let _ = child.start_kill();
        kill_process_group(child_pid);
        let _ = child.wait().await;
    }

    result
}

fn build_client_capabilities() -> ClientCapabilities {
    ClientCapabilities::new()
        .fs(FileSystemCapability::new()
            .read_text_file(false)
            .write_text_file(false))
        .terminal(false)
}

fn resolve_mcp_server_path(override_path: Option<&PathBuf>, current_exe: &Path) -> PathBuf {
    override_path
        .cloned()
        .unwrap_or_else(|| current_exe.to_path_buf())
}

#[cfg(unix)]
fn kill_process_group(pid: u32) {
    if pid == 0 {
        return;
    }
    let Ok(signed_pid) = i32::try_from(pid) else {
        log::warn!("refusing to kill process group: pid {pid} does not fit in i32");
        return;
    };
    unsafe {
        libc::killpg(signed_pid, libc::SIGKILL);
    }
}

#[cfg(not(unix))]
fn kill_process_group(_pid: u32) {}
