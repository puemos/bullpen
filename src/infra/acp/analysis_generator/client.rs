use super::ProgressEvent;
use agent_client_protocol::{
    ContentBlock, ExtNotification, ExtRequest, ExtResponse, Meta, PermissionOptionKind,
    RequestPermissionOutcome, RequestPermissionRequest, RequestPermissionResponse,
    SelectedPermissionOutcome, SessionNotification, SessionUpdate,
};
use async_trait::async_trait;
use serde_json::value::RawValue;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const CRAZYLINES_TOOLS: &[&str] = &[
    "submit_research_plan",
    "submit_entity_resolution",
    "submit_source",
    "submit_metric_snapshot",
    "submit_analysis_block",
    "submit_final_stance",
    "finalize_analysis",
];

type PendingToolCallMap = HashMap<String, (String, String, Option<serde_json::Value>)>;

pub(super) struct CrazyLinesClient {
    pub(super) messages: Arc<Mutex<Vec<String>>>,
    pub(super) thoughts: Arc<Mutex<Vec<String>>>,
    pub(super) finalization_received: Arc<Mutex<bool>>,
    progress: Option<tokio::sync::mpsc::UnboundedSender<ProgressEvent>>,
    tool_call_names: Arc<Mutex<HashMap<String, String>>>,
    pending_tool_calls: Arc<Mutex<PendingToolCallMap>>,
    last_message_id: Arc<Mutex<Option<String>>>,
    last_thought_id: Arc<Mutex<Option<String>>>,
    last_sent_lengths: Arc<Mutex<HashMap<String, usize>>>,
}

impl CrazyLinesClient {
    pub(super) fn new(progress: Option<tokio::sync::mpsc::UnboundedSender<ProgressEvent>>) -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
            thoughts: Arc::new(Mutex::new(Vec::new())),
            finalization_received: Arc::new(Mutex::new(false)),
            progress,
            tool_call_names: Arc::new(Mutex::new(HashMap::new())),
            pending_tool_calls: Arc::new(Mutex::new(HashMap::new())),
            last_message_id: Arc::new(Mutex::new(None)),
            last_thought_id: Arc::new(Mutex::new(None)),
            last_sent_lengths: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn tool_name_from_title(title: &str) -> Option<&'static str> {
        CRAZYLINES_TOOLS
            .iter()
            .find(|tool| title.contains(*tool))
            .copied()
    }

    fn tool_name_from_payload(payload: &serde_json::Value) -> Option<String> {
        let parsed = if let Some(s) = payload.as_str() {
            serde_json::from_str::<serde_json::Value>(s).ok()
        } else {
            Some(payload.clone())
        }?;

        parsed
            .get("tool")
            .and_then(|value| value.as_str())
            .or_else(|| parsed.get("name").and_then(|value| value.as_str()))
            .map(str::to_string)
    }

    fn extract_chunk_id(meta: Option<&Meta>) -> Option<String> {
        meta.and_then(|meta| {
            ["message_id", "messageId", "id"]
                .iter()
                .find_map(|key| meta.get(*key).and_then(|val| val.as_str()))
                .map(str::to_string)
        })
    }

    fn append_streamed_content(
        &self,
        store: &Arc<Mutex<Vec<String>>>,
        last_id: &Arc<Mutex<Option<String>>>,
        meta: Option<&Meta>,
        text: &str,
    ) -> String {
        let chunk_id = Self::extract_chunk_id(meta);
        let mut id_guard = last_id.lock().unwrap();
        let mut store_guard = store.lock().unwrap();

        if let Some(ref incoming) = chunk_id {
            if id_guard.as_deref() != Some(incoming.as_str()) {
                store_guard.push(String::new());
                *id_guard = Some(incoming.clone());
            }
        } else if store_guard.is_empty() {
            store_guard.push(String::new());
        }

        if store_guard.is_empty() {
            store_guard.push(String::new());
        }

        if id_guard.is_none() {
            *id_guard = chunk_id;
        }

        if let Some(last) = store_guard.last_mut() {
            last.push_str(text);
        }

        store_guard.last().cloned().unwrap_or_default()
    }

    fn mark_finalization_received(&self) {
        if let Ok(mut guard) = self.finalization_received.lock()
            && !*guard
        {
            *guard = true;
            if let Some(tx) = &self.progress {
                let _ = tx.send(ProgressEvent::Finalized);
            }
        }
    }

    fn handle_extension_payload(&self, method: &str) -> bool {
        if matches!(method, "crazylines/finalize_analysis" | "finalize_analysis") {
            self.mark_finalization_received();
            return true;
        }
        CRAZYLINES_TOOLS.iter().any(|tool| method.contains(tool))
    }
}

#[async_trait(?Send)]
impl agent_client_protocol::Client for CrazyLinesClient {
    async fn request_permission(
        &self,
        args: RequestPermissionRequest,
    ) -> agent_client_protocol::Result<RequestPermissionResponse> {
        let allow_option = args.options.iter().find(|opt| {
            matches!(
                opt.kind,
                PermissionOptionKind::AllowOnce | PermissionOptionKind::AllowAlways
            )
        });

        let outcome = allow_option
            .map(|option| {
                RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(
                    option.option_id.clone(),
                ))
            })
            .unwrap_or(RequestPermissionOutcome::Cancelled);

        Ok(RequestPermissionResponse::new(outcome))
    }

    async fn session_notification(
        &self,
        notification: SessionNotification,
    ) -> agent_client_protocol::Result<()> {
        match notification.update {
            SessionUpdate::AgentMessageChunk(chunk) => {
                if let ContentBlock::Text(text) = &chunk.content {
                    let full = self.append_streamed_content(
                        &self.messages,
                        &self.last_message_id,
                        chunk.meta.as_ref(),
                        &text.text,
                    );
                    let msg_id = Self::extract_chunk_id(chunk.meta.as_ref())
                        .unwrap_or_else(|| "default_message".to_string());
                    let delta = {
                        let mut lengths = self.last_sent_lengths.lock().unwrap();
                        let last_len = *lengths.get(&msg_id).unwrap_or(&0);
                        let delta = if full.len() > last_len {
                            full[last_len..].to_string()
                        } else {
                            String::new()
                        };
                        lengths.insert(msg_id.clone(), full.len());
                        delta
                    };
                    if !delta.is_empty()
                        && let Some(tx) = &self.progress
                    {
                        let _ = tx.send(ProgressEvent::MessageDelta { id: msg_id, delta });
                    }
                }
            }
            SessionUpdate::AgentThoughtChunk(chunk) => {
                if let ContentBlock::Text(text) = &chunk.content {
                    let full = self.append_streamed_content(
                        &self.thoughts,
                        &self.last_thought_id,
                        chunk.meta.as_ref(),
                        &text.text,
                    );
                    let thought_id = Self::extract_chunk_id(chunk.meta.as_ref())
                        .unwrap_or_else(|| "default_thought".to_string());
                    let delta = {
                        let mut lengths = self.last_sent_lengths.lock().unwrap();
                        let key = format!("thought_{thought_id}");
                        let last_len = *lengths.get(&key).unwrap_or(&0);
                        let delta = if full.len() > last_len {
                            full[last_len..].to_string()
                        } else {
                            String::new()
                        };
                        lengths.insert(key, full.len());
                        delta
                    };
                    if !delta.is_empty()
                        && let Some(tx) = &self.progress
                    {
                        let _ = tx.send(ProgressEvent::ThoughtDelta {
                            id: thought_id,
                            delta,
                        });
                    }
                }
            }
            SessionUpdate::ToolCall(call) => {
                let tool_id = call.tool_call_id.to_string();
                let tool_name = Self::tool_name_from_title(&call.title)
                    .map(str::to_string)
                    .or_else(|| {
                        call.raw_input
                            .as_ref()
                            .and_then(Self::tool_name_from_payload)
                    });
                if let Some(name) = &tool_name {
                    self.tool_call_names
                        .lock()
                        .unwrap()
                        .insert(tool_id.clone(), name.clone());
                }

                let kind = format!("{:?}", call.kind).to_lowercase();
                if let Some(tx) = &self.progress {
                    let _ = tx.send(ProgressEvent::ToolCallStarted {
                        tool_call_id: tool_id.clone(),
                        title: call.title.clone(),
                        kind: kind.clone(),
                    });
                }
                self.pending_tool_calls
                    .lock()
                    .unwrap()
                    .insert(tool_id, (call.title, kind, call.raw_input.clone()));
            }
            SessionUpdate::ToolCallUpdate(update) => {
                let tool_id: &str = &update.tool_call_id.0;
                let tool_name = update
                    .fields
                    .title
                    .as_deref()
                    .and_then(Self::tool_name_from_title)
                    .map(str::to_string)
                    .or_else(|| {
                        update
                            .fields
                            .raw_input
                            .as_ref()
                            .and_then(Self::tool_name_from_payload)
                    })
                    .or_else(|| self.tool_call_names.lock().unwrap().get(tool_id).cloned());

                let is_completed = matches!(
                    update.fields.status,
                    Some(agent_client_protocol::ToolCallStatus::Completed)
                );
                let is_failed = matches!(
                    update.fields.status,
                    Some(agent_client_protocol::ToolCallStatus::Failed)
                );

                if is_completed || is_failed {
                    let pending = self.pending_tool_calls.lock().unwrap().remove(tool_id);
                    let (title, _kind, stored_input) =
                        pending.unwrap_or_else(|| (String::new(), String::new(), None));
                    let final_title = update.fields.title.clone().unwrap_or(title);
                    let final_input = update.fields.raw_input.clone().or(stored_input);
                    let final_output = update.fields.raw_output.clone();
                    let status = if is_completed { "completed" } else { "failed" }.to_string();
                    if let Some(tx) = &self.progress {
                        let _ = tx.send(ProgressEvent::ToolCallComplete {
                            tool_call_id: tool_id.to_string(),
                            status,
                            title: final_title,
                            raw_input: final_input,
                            raw_output: final_output,
                        });
                    }

                    if is_completed {
                        match tool_name.as_deref() {
                            Some("submit_research_plan") => {
                                if let Some(tx) = &self.progress {
                                    let _ = tx.send(ProgressEvent::PlanSubmitted);
                                }
                            }
                            Some("submit_source") => {
                                if let Some(tx) = &self.progress {
                                    let _ = tx.send(ProgressEvent::SourceSubmitted);
                                }
                            }
                            Some("submit_metric_snapshot") => {
                                if let Some(tx) = &self.progress {
                                    let _ = tx.send(ProgressEvent::MetricSubmitted);
                                }
                            }
                            Some("submit_analysis_block") => {
                                if let Some(tx) = &self.progress {
                                    let _ = tx.send(ProgressEvent::BlockSubmitted);
                                }
                            }
                            Some("submit_final_stance") => {
                                if let Some(tx) = &self.progress {
                                    let _ = tx.send(ProgressEvent::StanceSubmitted);
                                }
                            }
                            Some("finalize_analysis") => self.mark_finalization_received(),
                            _ => {}
                        }
                    }
                    self.tool_call_names.lock().unwrap().remove(tool_id);
                }
            }
            SessionUpdate::Plan(plan) => {
                if let Some(tx) = &self.progress {
                    let _ = tx.send(ProgressEvent::Plan(plan));
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn ext_method(&self, args: ExtRequest) -> agent_client_protocol::Result<ExtResponse> {
        let stored = self.handle_extension_payload(&args.method);
        let response_value = if stored {
            serde_json::json!({ "status": "ok" })
        } else {
            serde_json::json!({ "status": "ignored" })
        };
        let raw = RawValue::from_string(response_value.to_string())
            .map(Arc::from)
            .unwrap_or_else(|_| Arc::from(RawValue::from_string("null".into()).unwrap()));
        Ok(ExtResponse::new(raw))
    }

    async fn ext_notification(&self, args: ExtNotification) -> agent_client_protocol::Result<()> {
        self.handle_extension_payload(&args.method);
        Ok(())
    }
}
