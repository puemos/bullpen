use super::ProgressTx;
use crate::infra::progress::{FrontendPlan, FrontendPlanEntry, ProgressEventPayload};
use agent_client_protocol::{
    ContentBlock, ExtNotification, ExtRequest, ExtResponse, Meta, PermissionOptionKind,
    RequestPermissionOutcome, RequestPermissionRequest, RequestPermissionResponse,
    SelectedPermissionOutcome, SessionNotification, SessionUpdate,
};
use async_trait::async_trait;
use serde_json::value::RawValue;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const BULLPEN_TOOLS: &[&str] = &[
    "submit_research_plan",
    "submit_entity_resolution",
    "submit_source",
    "submit_metric_snapshot",
    "submit_structured_artifact",
    "submit_analysis_block",
    "submit_final_stance",
    "submit_projection",
    "submit_counter_thesis",
    "submit_uncertainty_ledger",
    "submit_methodology_note",
    "submit_decision_criterion_answer",
    "submit_holding_review",
    "submit_allocation_review",
    "submit_portfolio_risk",
    "submit_rebalancing_suggestion",
    "submit_portfolio_scenario_analysis",
    "submit_portfolio_expected_return_model",
    "finalize_analysis",
];

type PendingToolCallMap = HashMap<String, (String, String, Option<serde_json::Value>)>;

pub(super) struct BullpenClient {
    pub(super) messages: Arc<Mutex<Vec<String>>>,
    pub(super) thoughts: Arc<Mutex<Vec<String>>>,
    pub(super) finalization_received: Arc<Mutex<bool>>,
    progress: Option<ProgressTx>,
    tool_call_names: Arc<Mutex<HashMap<String, String>>>,
    pending_tool_calls: Arc<Mutex<PendingToolCallMap>>,
    last_message_id: Arc<Mutex<Option<String>>>,
    last_thought_id: Arc<Mutex<Option<String>>>,
    last_sent_lengths: Arc<Mutex<HashMap<String, usize>>>,
}

impl BullpenClient {
    pub(super) fn new(progress: Option<ProgressTx>) -> Self {
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
        BULLPEN_TOOLS
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
            if let Ok(mut lengths) = self.last_sent_lengths.lock() {
                lengths.clear();
            }
            if let Some(tx) = &self.progress {
                let _ = tx.send(ProgressEventPayload::Completed);
            }
        }
    }

    fn emit_tool_completion(
        &self,
        tool_id: &str,
        status: &str,
        title: String,
        raw_input: Option<serde_json::Value>,
        raw_output: Option<serde_json::Value>,
        tool_name: Option<&str>,
    ) {
        if let Some(tx) = &self.progress {
            let _ = tx.send(ProgressEventPayload::ToolCallComplete {
                tool_call_id: tool_id.to_string(),
                status: status.to_string(),
                title,
                raw_input,
                raw_output,
            });
        }

        if status == "completed" {
            if let Some(extra) = submitted_event_for(tool_name) {
                if let Some(tx) = &self.progress {
                    let _ = tx.send(extra);
                }
            } else if matches!(tool_name, Some("finalize_analysis")) {
                self.mark_finalization_received();
            }
        }

        self.tool_call_names.lock().unwrap().remove(tool_id);
    }

    fn handle_extension_payload(&self, method: &str) -> bool {
        if matches!(method, "bullpen/finalize_analysis" | "finalize_analysis") {
            self.mark_finalization_received();
            return true;
        }
        BULLPEN_TOOLS.iter().any(|tool| method.contains(tool))
    }
}

fn submitted_event_for(tool_name: Option<&str>) -> Option<ProgressEventPayload> {
    match tool_name? {
        "submit_research_plan" => Some(ProgressEventPayload::PlanSubmitted),
        "submit_source" => Some(ProgressEventPayload::SourceSubmitted),
        "submit_metric_snapshot" => Some(ProgressEventPayload::MetricSubmitted),
        "submit_structured_artifact" => Some(ProgressEventPayload::ArtifactSubmitted),
        "submit_analysis_block" => Some(ProgressEventPayload::BlockSubmitted),
        "submit_final_stance" => Some(ProgressEventPayload::StanceSubmitted),
        "submit_projection" => Some(ProgressEventPayload::ProjectionSubmitted),
        "submit_portfolio_scenario_analysis" => Some(ProgressEventPayload::ArtifactSubmitted),
        "submit_portfolio_expected_return_model" => Some(ProgressEventPayload::ArtifactSubmitted),
        _ => None,
    }
}

#[async_trait(?Send)]
impl agent_client_protocol::Client for BullpenClient {
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

        let outcome = allow_option.map_or(RequestPermissionOutcome::Cancelled, |option| {
            RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(
                option.option_id.clone(),
            ))
        });

        Ok(RequestPermissionResponse::new(outcome))
    }

    async fn session_notification(
        &self,
        notification: SessionNotification,
    ) -> agent_client_protocol::Result<()> {
        match notification.update {
            SessionUpdate::AgentMessageChunk(chunk) => {
                if let ContentBlock::Text(text) = &chunk.content {
                    let prev_id = self.last_message_id.lock().unwrap().clone();
                    let full = Self::append_streamed_content(
                        &self.messages,
                        &self.last_message_id,
                        chunk.meta.as_ref(),
                        &text.text,
                    );
                    let msg_id = Self::extract_chunk_id(chunk.meta.as_ref())
                        .unwrap_or_else(|| "default_message".to_string());
                    let delta = {
                        let mut lengths = self.last_sent_lengths.lock().unwrap();
                        if let Some(prev) = prev_id.as_deref()
                            && prev != msg_id.as_str()
                        {
                            lengths.remove(prev);
                        }
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
                        let _ = tx.send(ProgressEventPayload::MessageDelta { id: msg_id, delta });
                    }
                }
            }
            SessionUpdate::AgentThoughtChunk(chunk) => {
                if let ContentBlock::Text(text) = &chunk.content {
                    let prev_id = self.last_thought_id.lock().unwrap().clone();
                    let full = Self::append_streamed_content(
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
                        if let Some(prev) = prev_id.as_deref()
                            && prev != thought_id.as_str()
                        {
                            lengths.remove(&format!("thought_{prev}"));
                        }
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
                        let _ = tx.send(ProgressEventPayload::ThoughtDelta {
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
                let is_completed = matches!(
                    call.status,
                    agent_client_protocol::ToolCallStatus::Completed
                );
                let is_failed =
                    matches!(call.status, agent_client_protocol::ToolCallStatus::Failed);

                if is_completed || is_failed {
                    // The agent sent a terminal ToolCall without a separate
                    // ToolCallUpdate — merge with anything we've already stored
                    // and emit completion directly (no duplicate Started).
                    let stored = self.pending_tool_calls.lock().unwrap().remove(&tool_id);
                    let (stored_title, _stored_kind, stored_input) =
                        stored.unwrap_or_else(|| (String::new(), String::new(), None));
                    let final_title = if call.title.is_empty() {
                        stored_title
                    } else {
                        call.title
                    };
                    let final_input = call.raw_input.or(stored_input);
                    let final_output = call.raw_output;
                    let status = if is_completed { "completed" } else { "failed" };
                    self.emit_tool_completion(
                        &tool_id,
                        status,
                        final_title,
                        final_input,
                        final_output,
                        tool_name.as_deref(),
                    );
                } else {
                    // Non-terminal ToolCall: emit Started once per id, and keep
                    // the stored state fresh for any re-emissions or later
                    // ToolCallUpdates.
                    let already_pending = self
                        .pending_tool_calls
                        .lock()
                        .unwrap()
                        .contains_key(&tool_id);
                    if !already_pending && let Some(tx) = &self.progress {
                        let _ = tx.send(ProgressEventPayload::ToolCallStarted {
                            tool_call_id: tool_id.clone(),
                            title: call.title.clone(),
                            kind: kind.clone(),
                        });
                    }
                    self.pending_tool_calls
                        .lock()
                        .unwrap()
                        .insert(tool_id, (call.title, kind, call.raw_input));
                }
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
                if let Some(name) = &tool_name {
                    self.tool_call_names
                        .lock()
                        .unwrap()
                        .insert(tool_id.to_string(), name.clone());
                }

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
                    let (stored_title, _stored_kind, stored_input) =
                        pending.unwrap_or_else(|| (String::new(), String::new(), None));
                    let final_title = update.fields.title.clone().unwrap_or(stored_title);
                    let final_input = update.fields.raw_input.clone().or(stored_input);
                    let final_output = update.fields.raw_output.clone();
                    let status = if is_completed { "completed" } else { "failed" };
                    self.emit_tool_completion(
                        tool_id,
                        status,
                        final_title,
                        final_input,
                        final_output,
                        tool_name.as_deref(),
                    );
                } else {
                    // Intermediate update — merge into stored state so the
                    // eventual completion carries the latest title/input even
                    // when the terminal event omits them.
                    let mut pending = self.pending_tool_calls.lock().unwrap();
                    let entry = pending
                        .entry(tool_id.to_string())
                        .or_insert_with(|| (String::new(), String::new(), None));
                    if let Some(title) = update.fields.title.clone() {
                        entry.0 = title;
                    }
                    if let Some(kind) = update.fields.kind {
                        entry.1 = format!("{kind:?}").to_lowercase();
                    }
                    if let Some(raw_input) = update.fields.raw_input.clone() {
                        entry.2 = Some(raw_input);
                    }
                }
            }
            SessionUpdate::Plan(plan) => {
                if let Some(tx) = &self.progress {
                    let _ = tx.send(ProgressEventPayload::Plan(FrontendPlan {
                        entries: plan
                            .entries
                            .into_iter()
                            .map(|entry| FrontendPlanEntry {
                                content: entry.content,
                                priority: format!("{:?}", entry.priority),
                                status: format!("{:?}", entry.status),
                            })
                            .collect(),
                    }));
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
        let raw = RawValue::from_string(response_value.to_string()).map_or_else(
            |_| Arc::from(RawValue::from_string("null".into()).unwrap()),
            Arc::from,
        );
        Ok(ExtResponse::new(raw))
    }

    async fn ext_notification(&self, args: ExtNotification) -> agent_client_protocol::Result<()> {
        self.handle_extension_payload(&args.method);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_client_protocol::{
        Client, SessionId, ToolCall, ToolCallStatus, ToolCallUpdate, ToolCallUpdateFields, ToolKind,
    };
    use std::matches;
    use tokio::sync::mpsc;

    fn make_client() -> (BullpenClient, mpsc::UnboundedReceiver<ProgressEventPayload>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (BullpenClient::new(Some(tx)), rx)
    }

    fn notify(update: SessionUpdate) -> SessionNotification {
        SessionNotification::new(SessionId::new("session-1"), update)
    }

    fn pending_call(id: &str, title: &str) -> SessionUpdate {
        SessionUpdate::ToolCall(
            ToolCall::new(id.to_string(), title)
                .kind(ToolKind::Fetch)
                .status(ToolCallStatus::Pending),
        )
    }

    fn completed_update_with(id: &str, fields: ToolCallUpdateFields) -> SessionUpdate {
        SessionUpdate::ToolCallUpdate(ToolCallUpdate::new(
            id.to_string(),
            fields.status(ToolCallStatus::Completed),
        ))
    }

    fn drain(rx: &mut mpsc::UnboundedReceiver<ProgressEventPayload>) -> Vec<ProgressEventPayload> {
        let mut out = Vec::new();
        while let Ok(event) = rx.try_recv() {
            out.push(event);
        }
        out
    }

    /// Baseline: a single pending → completed lifecycle must emit exactly
    /// one Started and one Complete for the same tool_call_id.
    #[tokio::test]
    async fn normal_lifecycle_emits_one_started_then_one_complete() {
        let (client, mut rx) = make_client();

        client
            .session_notification(notify(pending_call("call-1", "fetch")))
            .await
            .unwrap();
        client
            .session_notification(notify(completed_update_with(
                "call-1",
                ToolCallUpdateFields::new()
                    .title("fetch \"QBTS\"".to_string())
                    .raw_output(serde_json::json!({"ok": true})),
            )))
            .await
            .unwrap();

        let events = drain(&mut rx);
        assert_eq!(
            events.len(),
            2,
            "expected Started + Complete, got {events:?}"
        );

        match &events[0] {
            ProgressEventPayload::ToolCallStarted {
                tool_call_id,
                title,
                kind,
            } => {
                assert_eq!(tool_call_id, "call-1");
                assert_eq!(title, "fetch");
                assert_eq!(kind, "fetch");
            }
            other => panic!("expected ToolCallStarted, got {other:?}"),
        }
        match &events[1] {
            ProgressEventPayload::ToolCallComplete {
                tool_call_id,
                status,
                title,
                raw_output,
                ..
            } => {
                assert_eq!(tool_call_id, "call-1");
                assert_eq!(status, "completed");
                assert_eq!(title, "fetch \"QBTS\"");
                assert_eq!(
                    raw_output.as_ref().unwrap(),
                    &serde_json::json!({"ok": true})
                );
            }
            other => panic!("expected ToolCallComplete, got {other:?}"),
        }
    }

    /// Bug #1 regression: the agent re-emits `SessionUpdate::ToolCall` for the
    /// same id (e.g. pending → in-progress via full re-send). The client used
    /// to fire Started every time, which orphaned every earlier block in the
    /// UI. After the fix, Started is only emitted on the first arrival for
    /// that id.
    #[tokio::test]
    async fn duplicate_toolcall_events_only_emit_one_started() {
        let (client, mut rx) = make_client();

        client
            .session_notification(notify(pending_call("call-7", "fetch")))
            .await
            .unwrap();
        // Agent re-sends the same ToolCall with status=InProgress — this used
        // to generate another Started and orphan the first block.
        client
            .session_notification(notify(SessionUpdate::ToolCall(
                ToolCall::new("call-7".to_string(), "fetch \"ionq\"")
                    .kind(ToolKind::Fetch)
                    .status(ToolCallStatus::InProgress),
            )))
            .await
            .unwrap();
        client
            .session_notification(notify(completed_update_with(
                "call-7",
                ToolCallUpdateFields::new(),
            )))
            .await
            .unwrap();

        let events = drain(&mut rx);
        let started_count = events
            .iter()
            .filter(|e| matches!(e, ProgressEventPayload::ToolCallStarted { .. }))
            .count();
        let complete_count = events
            .iter()
            .filter(|e| matches!(e, ProgressEventPayload::ToolCallComplete { .. }))
            .count();
        assert_eq!(started_count, 1, "started should dedupe, got {events:?}");
        assert_eq!(complete_count, 1, "one completion expected, got {events:?}");
    }

    /// Bug #2 regression: some agents send a single `ToolCall` with status
    /// already Completed instead of a pending → update sequence. The client
    /// used to emit Started and never Complete, leaving the UI stuck on
    /// "running...". The fix routes terminal ToolCall events straight to
    /// completion with no Started.
    #[tokio::test]
    async fn terminal_toolcall_skips_started_and_emits_complete() {
        let (client, mut rx) = make_client();

        client
            .session_notification(notify(SessionUpdate::ToolCall(
                ToolCall::new("call-inline".to_string(), "fetch \"dwave\"")
                    .kind(ToolKind::Fetch)
                    .status(ToolCallStatus::Completed)
                    .raw_input(serde_json::json!({"query": "dwave"}))
                    .raw_output(serde_json::json!({"results": []})),
            )))
            .await
            .unwrap();

        let events = drain(&mut rx);
        assert_eq!(events.len(), 1, "expected only Complete, got {events:?}");
        match &events[0] {
            ProgressEventPayload::ToolCallComplete {
                tool_call_id,
                status,
                title,
                raw_input,
                raw_output,
            } => {
                assert_eq!(tool_call_id, "call-inline");
                assert_eq!(status, "completed");
                assert_eq!(title, "fetch \"dwave\"");
                assert_eq!(
                    raw_input.as_ref().unwrap(),
                    &serde_json::json!({"query": "dwave"}),
                );
                assert_eq!(
                    raw_output.as_ref().unwrap(),
                    &serde_json::json!({"results": []}),
                );
            }
            other => panic!("expected ToolCallComplete, got {other:?}"),
        }
    }

    /// A terminal `ToolCall` with status=Failed should emit a single Complete
    /// with status="failed" and no Started.
    #[tokio::test]
    async fn terminal_failed_toolcall_emits_failed_complete() {
        let (client, mut rx) = make_client();

        client
            .session_notification(notify(SessionUpdate::ToolCall(
                ToolCall::new("call-bad".to_string(), "fetch")
                    .kind(ToolKind::Fetch)
                    .status(ToolCallStatus::Failed),
            )))
            .await
            .unwrap();

        let events = drain(&mut rx);
        assert_eq!(events.len(), 1);
        match &events[0] {
            ProgressEventPayload::ToolCallComplete { status, .. } => assert_eq!(status, "failed"),
            other => panic!("expected failed ToolCallComplete, got {other:?}"),
        }
    }

    /// Bug #3 regression: title and raw_input delivered in an intermediate
    /// ToolCallUpdate were dropped, so the final Complete fell back to the
    /// stale pending title. The fix merges intermediate updates into stored
    /// state.
    #[tokio::test]
    async fn intermediate_update_title_and_input_survive_to_completion() {
        let (client, mut rx) = make_client();

        client
            .session_notification(notify(pending_call("call-42", "fetch")))
            .await
            .unwrap();
        // Intermediate update with title + raw_input, still InProgress.
        client
            .session_notification(notify(SessionUpdate::ToolCallUpdate(ToolCallUpdate::new(
                "call-42".to_string(),
                ToolCallUpdateFields::new()
                    .status(ToolCallStatus::InProgress)
                    .title("fetch \"quantum outlook\"".to_string())
                    .raw_input(serde_json::json!({"query": "quantum outlook"})),
            ))))
            .await
            .unwrap();
        // Terminal update carries only the status and raw_output; the
        // fix must surface the intermediate title/input.
        client
            .session_notification(notify(SessionUpdate::ToolCallUpdate(ToolCallUpdate::new(
                "call-42".to_string(),
                ToolCallUpdateFields::new()
                    .status(ToolCallStatus::Completed)
                    .raw_output(serde_json::json!({"results": ["a", "b"]})),
            ))))
            .await
            .unwrap();

        let events = drain(&mut rx);
        // Only Started and Complete — the intermediate update must not leak
        // its own Started event.
        assert_eq!(events.len(), 2, "unexpected events: {events:?}");
        match &events[1] {
            ProgressEventPayload::ToolCallComplete {
                title, raw_input, ..
            } => {
                assert_eq!(title, "fetch \"quantum outlook\"");
                assert_eq!(
                    raw_input.as_ref().unwrap(),
                    &serde_json::json!({"query": "quantum outlook"}),
                );
            }
            other => panic!("expected ToolCallComplete, got {other:?}"),
        }
    }

    /// When the agent skips the initial `ToolCall` entirely and only ever
    /// sends `ToolCallUpdate`s (some agents do this for fast tools), no
    /// Started is expected, but the terminal update must still produce a
    /// well-formed Complete.
    #[tokio::test]
    async fn update_only_lifecycle_without_started_still_completes() {
        let (client, mut rx) = make_client();

        client
            .session_notification(notify(SessionUpdate::ToolCallUpdate(ToolCallUpdate::new(
                "call-update-only".to_string(),
                ToolCallUpdateFields::new()
                    .title("fetch \"late\"".to_string())
                    .raw_input(serde_json::json!({"query": "late"})),
            ))))
            .await
            .unwrap();
        client
            .session_notification(notify(completed_update_with(
                "call-update-only",
                ToolCallUpdateFields::new(),
            )))
            .await
            .unwrap();

        let events = drain(&mut rx);
        assert_eq!(events.len(), 1, "expected only Complete, got {events:?}");
        match &events[0] {
            ProgressEventPayload::ToolCallComplete {
                title, raw_input, ..
            } => {
                assert_eq!(title, "fetch \"late\"");
                assert_eq!(
                    raw_input.as_ref().unwrap(),
                    &serde_json::json!({"query": "late"}),
                );
            }
            other => panic!("expected ToolCallComplete, got {other:?}"),
        }
    }

    /// Regression-guard for the MCP submission side-effects: tool name
    /// recognition must still fire the matching `*Submitted` event on the
    /// terminal-ToolCall path (previously only reachable through
    /// ToolCallUpdate completion).
    #[tokio::test]
    async fn terminal_toolcall_for_submit_source_fires_source_submitted() {
        let (client, mut rx) = make_client();

        client
            .session_notification(notify(SessionUpdate::ToolCall(
                ToolCall::new("call-submit".to_string(), "mcp: bullpen submit_source(...)")
                    .kind(ToolKind::Other)
                    .status(ToolCallStatus::Completed),
            )))
            .await
            .unwrap();

        let events = drain(&mut rx);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, ProgressEventPayload::ToolCallComplete { .. })),
            "missing ToolCallComplete in {events:?}",
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, ProgressEventPayload::SourceSubmitted)),
            "missing SourceSubmitted in {events:?}",
        );
    }

    /// Each tool_call_id must complete independently — a completion on one id
    /// must not disturb another pending call.
    #[tokio::test]
    async fn concurrent_tool_calls_do_not_interfere() {
        let (client, mut rx) = make_client();

        client
            .session_notification(notify(pending_call("call-a", "fetch")))
            .await
            .unwrap();
        client
            .session_notification(notify(pending_call("call-b", "fetch")))
            .await
            .unwrap();
        client
            .session_notification(notify(completed_update_with(
                "call-a",
                ToolCallUpdateFields::new().title("fetch \"a\"".to_string()),
            )))
            .await
            .unwrap();
        client
            .session_notification(notify(completed_update_with(
                "call-b",
                ToolCallUpdateFields::new().title("fetch \"b\"".to_string()),
            )))
            .await
            .unwrap();

        let events = drain(&mut rx);
        let completions: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                ProgressEventPayload::ToolCallComplete {
                    tool_call_id,
                    title,
                    ..
                } => Some((tool_call_id.clone(), title.clone())),
                _ => None,
            })
            .collect();
        assert_eq!(
            completions,
            vec![
                ("call-a".to_string(), "fetch \"a\"".to_string()),
                ("call-b".to_string(), "fetch \"b\"".to_string()),
            ],
        );
    }
}
