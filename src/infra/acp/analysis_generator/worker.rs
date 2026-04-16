use super::client::CrazylinesClient;
use crate::infra::acp::analysis_mcp_server::RunContext;
use crate::prompts;
use agent_client_protocol::{
    Agent, ClientCapabilities, ClientSideConnection, ContentBlock, FileSystemCapability,
    Implementation, InitializeRequest, McpServer, McpServerStdio, NewSessionRequest, PromptRequest,
    ProtocolVersion, TextContent,
};
use anyhow::{Context, Result};
use futures::future::LocalBoxFuture;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::runtime::Builder;
use tokio::task::LocalSet;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub enum ProgressEvent {
    MessageDelta {
        id: String,
        delta: String,
    },
    ThoughtDelta {
        id: String,
        delta: String,
    },
    ToolCallStarted {
        tool_call_id: String,
        title: String,
        kind: String,
    },
    ToolCallComplete {
        tool_call_id: String,
        status: String,
        title: String,
        raw_input: Option<serde_json::Value>,
        raw_output: Option<serde_json::Value>,
    },
    Plan(agent_client_protocol::Plan),
    LocalLog(String),
    PlanSubmitted,
    SourceSubmitted,
    MetricSubmitted,
    ArtifactSubmitted,
    BlockSubmitted,
    StanceSubmitted,
    ProjectionSubmitted,
    Finalized,
}

pub struct GenerateAnalysisInput {
    pub run_context: RunContext,
    pub agent_command: String,
    pub agent_args: Vec<String>,
    pub progress_tx: Option<tokio::sync::mpsc::UnboundedSender<ProgressEvent>>,
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

pub async fn generate_with_acp(input: GenerateAnalysisInput) -> Result<GenerateAnalysisResult> {
    let (sender, receiver) = futures::channel::oneshot::channel();
    let timeout_secs = input.timeout_secs.unwrap_or(1800);

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
                        ) => result.map_err(|_| anyhow::anyhow!("agent timed out after {timeout_secs}s"))?,
                        _ = async {
                            if let Some(token) = cancel_token {
                                token.cancelled().await;
                            } else {
                                std::future::pending::<()>().await;
                            }
                        } => Err(anyhow::anyhow!("agent generation cancelled by user")),
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
        agent_args,
        progress_tx,
        mcp_server_binary,
        db_path,
        timeout_secs: _,
        cancel_token,
    } = input;

    let logs = Arc::new(Mutex::new(Vec::new()));
    let log_fn = |msg: String| {
        if let Ok(mut guard) = logs.lock() {
            guard.push(msg.clone());
        }
        if let Some(tx) = &progress_tx {
            let _ = tx.send(ProgressEvent::LocalLog(msg));
        }
    };

    log_fn(format!("spawn: {} {}", agent_command, agent_args.join(" ")));

    let mut cmd = Command::new(&agent_command);
    cmd.args(&agent_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

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
                let _ = tx.send(ProgressEvent::LocalLog(msg));
            }
        }
    });

    let client = CrazylinesClient::new(progress_tx.clone());
    let messages = client.messages.clone();
    let thoughts = client.thoughts.clone();
    let finalization_received = client.finalization_received.clone();

    use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
    let stdin_compat = stdin.compat_write();
    let stdout_compat = stdout.compat();
    let spawn_fn = |fut: LocalBoxFuture<'static, ()>| {
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
                    .client_info(Implementation::new("crazylines", env!("CARGO_PKG_VERSION")))
                    .client_capabilities(build_client_capabilities()),
            )
            .await
            .context("ACP initialize failed")?;

        let temp_cwd = tempfile::tempdir().context("create temp working directory")?;
        let cwd = temp_cwd.path().to_path_buf();
        let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("crazylines"));
        let mcp_path = resolve_mcp_server_path(mcp_server_binary.as_ref(), &current_exe);
        let context_file =
            tempfile::NamedTempFile::new().context("create analysis context file")?;
        std::fs::write(
            context_file.path(),
            serde_json::to_string(&run_context).context("serialize run context")?,
        )
        .context("write analysis context file")?;

        let mcp_servers = vec![McpServer::Stdio(
            McpServerStdio::new("crazylines-analysis", mcp_path.clone()).args(vec![
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
                _ = tokio::time::sleep(Duration::from_millis(200)) => {}
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
    if pid != 0 {
        unsafe {
            libc::killpg(pid as i32, libc::SIGKILL);
        }
    }
}

#[cfg(not(unix))]
fn kill_process_group(_pid: u32) {}
