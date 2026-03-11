use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};

/// Spawns a closure in a new thread and returns a Receiver with the result.
pub(crate) fn spawn_task<T, F>(f: F) -> mpsc::Receiver<T>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx
}

use super::super::types::{AppShared, Toast, should_emit_save_error_toast};
use super::git_status::{GitVisualStatus, parse_porcelain_status};
use super::workspace::{FsChangeResult, WorkspaceState, spawn_ai_tool_check};
use crate::app::ai_core::ollama::spawn_model_info_fetch;
use crate::app::ai_core::provider::StreamEvent;
use crate::app::ai_core::state::OllamaConnectionStatus;
use crate::app::ai_core::{OllamaStatus, spawn_ollama_check};
use crate::settings::SaveMode;
use crate::watcher::{FileEvent, FsChange};
use std::sync::Mutex;

fn should_run_autosave(save_mode: SaveMode) -> bool {
    matches!(save_mode, SaveMode::Automatic)
}

/// Processes events from watchers, build results, and autosave.
pub(super) fn process_background_events(
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &crate::i18n::I18n,
) {
    // --- 1. Background I/O results ---
    if let Some(rx) = &ws.background_io_rx
        && let Ok(result) = rx.try_recv()
    {
        match result {
            FsChangeResult::AiDiff(path, original, new) => {
                ws.editor.pending_ai_diff = Some((path, original, new));
            }
            FsChangeResult::LocalHistory(rel_path, content) => {
                ws.local_history.take_snapshot(&rel_path, &content);
            }
        }
    }

    // --- 2. Watcher events (individual files) ---
    for event in ws.watcher.try_recv() {
        match event {
            FileEvent::Changed(changed_path) => {
                if let Ok(changed_canonical) = changed_path.canonicalize()
                    && let Some(tab_path) = ws.editor.tab_path_for_canonical(&changed_canonical)
                {
                    if ws.editor.is_path_modified(&tab_path) {
                        if ws.external_change_conflict.is_none() {
                            ws.external_change_conflict = Some(tab_path);
                        }
                    } else {
                        ws.editor.reload_path_from_disk(&tab_path);
                    }
                }
            }
            FileEvent::Removed(removed_path) => {
                ws.editor.notify_file_deleted(&removed_path);
                let name = removed_path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| removed_path.to_string_lossy().into_owned());
                let mut args = fluent_bundle::FluentArgs::new();
                args.set("path", name);
                ws.toasts
                    .push(Toast::error(i18n.get_args("error-file-deleted", &args)));
            }
        }
    }

    // --- 3. Project watcher events (directory tree) ---
    let fs_changes = ws.project_watcher.poll();
    if !fs_changes.is_empty() {
        let mut need_reload = false;
        let mut created_file: Option<PathBuf> = None;

        for change in &fs_changes {
            ws.project_index.handle_change(change.clone());

            match change {
                FsChange::Created(path) => {
                    need_reload = true;
                    if path.is_file() {
                        created_file = Some(path.clone());
                    }
                }
                FsChange::Removed(path) => {
                    need_reload = true;
                    ws.editor.close_tabs_for_path(path);
                }
                FsChange::Modified(_path) => {
                    need_reload = true;
                }
            }
        }
        if need_reload {
            if let Some(ref path) = created_file {
                ws.file_tree.request_reload_and_expand(path);
            } else {
                ws.file_tree.request_reload();
            }
        }
    }

    // --- 4. Periodic tasks (Git, AI tools) ---
    if ws.git_last_refresh.elapsed().as_secs() > 10 {
        ws.git_last_refresh = std::time::Instant::now();
        if ws.git_status_rx.is_none() {
            ws.git_status_rx = Some(fetch_git_status(&ws.root_path, Arc::clone(&ws.git_cancel)));
        }
        if ws.git_branch_rx.is_none() {
            ws.git_branch_rx = Some(fetch_git_branch(&ws.root_path, Arc::clone(&ws.git_cancel)));
        }
    }

    if let Some(rx) = &ws.git_branch_rx
        && let Ok(branch) = rx.try_recv()
    {
        ws.git_branch = branch;
        ws.git_branch_rx = None;
    }

    if let Some(rx) = &ws.git_status_rx
        && let Ok(status) = rx.try_recv()
    {
        ws.file_tree.set_git_statuses(status);
        ws.git_status_rx = None;
    }

    if let Some(rx) = &ws.ai_tool_check_rx
        && let Ok(status) = rx.try_recv()
    {
        ws.ai_tool_available = status;
        ws.ai_tool_check_rx = None;
        ws.ai_tool_last_check = std::time::Instant::now();
    }
    if ws.ai_tool_last_check.elapsed().as_secs() >= crate::config::AI_TOOL_CHECK_INTERVAL_SECS
        && ws.ai_tool_check_rx.is_none()
    {
        let check_list: Vec<(String, String)> = {
            let sh = shared.lock().expect("lock");
            sh.registry
                .agents
                .get_all()
                .iter()
                .map(|a| (a.id.clone(), a.command.clone()))
                .collect()
        };
        ws.ai_tool_check_rx = Some(spawn_ai_tool_check(check_list));
    }

    if let Some(rx) = &ws.win_tool_check_rx
        && let Ok(status) = rx.try_recv()
    {
        ws.win_tool_available = status;
        ws.win_tool_check_rx = None;
        ws.win_tool_last_check = std::time::Instant::now();
    }
    if ws.win_tool_last_check.elapsed().as_secs() >= 30 // Check every 30 seconds
        && ws.win_tool_check_rx.is_none()
    {
        ws.win_tool_check_rx =
            Some(crate::app::ui::workspace::state::actions::spawn_win_tool_check());
    }

    // --- 4b-sync. Sync Ollama config from Settings ---
    {
        let sh = shared.lock().expect("lock");
        let url_changed = ws.ai.ollama.base_url != sh.settings.ollama_base_url;
        if url_changed {
            ws.ai.ollama.base_url = sh.settings.ollama_base_url.clone();
            ws.ai.ollama.last_check =
                std::time::Instant::now() - std::time::Duration::from_secs(999);
            ws.ai.ollama.status = OllamaConnectionStatus::Checking;
        }
        // Always sync API key (user may change key without changing URL)
        let new_key = if sh.settings.ollama_api_key.is_empty() {
            None
        } else {
            Some(sh.settings.ollama_api_key.clone())
        };
        ws.ai.ollama.api_key = new_key;
        if !sh.settings.ai_default_model.is_empty() && ws.ai.ollama.selected_model.is_empty() {
            ws.ai.ollama.selected_model = sh.settings.ai_default_model.clone();
        }
        ws.ai.settings.expertise = sh.settings.ai_expertise;
        ws.ai.settings.reasoning_depth = sh.settings.ai_reasoning_depth;
        ws.ai.settings.top_p = sh.settings.ollama_top_p;
        ws.ai.settings.top_k = sh.settings.ollama_top_k;
        ws.ai.settings.repeat_penalty = sh.settings.ollama_repeat_penalty;
        ws.ai.settings.seed = sh.settings.ollama_seed;
    }

    // --- 4b. Ollama polling ---
    if let Some(rx) = &ws.ai.ollama.check_rx
        && let Ok(status) = rx.try_recv()
    {
        match status {
            OllamaStatus::Available(models) => {
                ws.ai.ollama.status = OllamaConnectionStatus::Connected;
                // Only auto-select if no model is chosen yet — don't
                // overwrite a custom/cloud model that isn't in the local list
                if ws.ai.ollama.selected_model.is_empty()
                    && let Some(first) = models.first()
                {
                    ws.ai.ollama.selected_model = first.clone();
                }
                ws.ai.ollama.models = models;
            }
            OllamaStatus::Unavailable => {
                ws.ai.ollama.status = OllamaConnectionStatus::Disconnected;
                ws.ai.ollama.models.clear();
            }
        }
        ws.ai.ollama.check_rx = None;
        ws.ai.ollama.last_check = std::time::Instant::now();
    }
    if ws.ai.ollama.last_check.elapsed().as_secs() >= crate::config::OLLAMA_CHECK_INTERVAL_SECS
        && ws.ai.ollama.check_rx.is_none()
        && !ws.ai.chat.loading
    {
        ws.ai.ollama.check_rx = Some(spawn_ollama_check(
            ws.ai.ollama.base_url.clone(),
            ws.ai.ollama.api_key.clone(),
        ));
    }

    // --- 4b2. Model info fetch ---
    // Fetch model info when selected model changes
    if !ws.ai.ollama.selected_model.is_empty()
        && ws.ai.ollama.selected_model != ws.ai.ollama.model_info_for
        && ws.ai.ollama.model_info_rx.is_none()
    {
        ws.ai.ollama.model_info_for = ws.ai.ollama.selected_model.clone();
        ws.ai.ollama.model_info = None;
        ws.ai.ollama.model_info_rx = Some(spawn_model_info_fetch(
            ws.ai.ollama.base_url.clone(),
            ws.ai.ollama.selected_model.clone(),
            ws.ai.ollama.api_key.clone(),
        ));
    }
    // Poll model info result
    if let Some(rx) = &ws.ai.ollama.model_info_rx
        && let Ok(result) = rx.try_recv()
    {
        ws.ai.ollama.model_info = result.ok();
        ws.ai.ollama.model_info_rx = None;
    }

    // --- 4c. Chat streaming ---
    let has_stream = ws.ai.chat.stream_rx.is_some();
    if has_stream {
        let mut events = Vec::new();
        if let Some(ref rx) = ws.ai.chat.stream_rx {
            loop {
                match rx.try_recv() {
                    Ok(evt) => events.push(evt),
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        // Sender dropped — stream thread finished. If we already
                        // have tokens/Done events this is normal completion.
                        // Only report error if no content was received at all.
                        if events.is_empty() && ws.ai.chat.streaming_buffer.is_empty() {
                            events.push(StreamEvent::Error("Stream disconnected".into()));
                        } else if !events.iter().any(|e| matches!(e, StreamEvent::Done { .. })) {
                            // No explicit Done — synthesize one so the stream finalizes cleanly
                            events.push(StreamEvent::Done {
                                model: String::new(),
                                prompt_tokens: 0,
                                completion_tokens: 0,
                            });
                        }
                        break;
                    }
                }
            }
        }
        // Process collected events
        let mut done = false;
        let mut tokens_this_frame = 0u32;
        let mut tool_call_handled = false;
        for evt in events {
            match evt {
                StreamEvent::Token(text) => {
                    ws.ai.chat.streaming_buffer.push_str(&text);
                    tokens_this_frame += 1;
                }
                StreamEvent::Done {
                    prompt_tokens,
                    completion_tokens,
                    ..
                } => {
                    ws.ai.chat.in_tokens += prompt_tokens as u32;
                    ws.ai.chat.out_tokens += completion_tokens as u32;
                    done = true;
                }
                StreamEvent::Error(msg) => {
                    let model = &ws.ai.ollama.selected_model;
                    if !ws.ai.chat.streaming_buffer.is_empty() {
                        ws.ai
                            .chat
                            .streaming_buffer
                            .push_str(&format!("\n\n*[error ({model}): {msg}]*"));
                    } else {
                        ws.toasts.push(Toast::error(format!("AI ({model}): {msg}")));
                    }
                    log_to_stderr_file(&format!("[AI error] model={model}: {msg}"));
                    done = true;
                }
                StreamEvent::Thinking(text) => {
                    // Store thinking text for the current conversation entry
                    let idx = ws.ai.chat.conversation.len().saturating_sub(1);
                    while ws.ai.chat.thinking_history.len() <= idx {
                        ws.ai.chat.thinking_history.push(None);
                    }
                    ws.ai.chat.thinking_history[idx] = Some(text);
                }
                StreamEvent::ContentReplace(clean) => {
                    // Replace entire streaming buffer with post-processed clean content
                    ws.ai.chat.streaming_buffer = clean;
                    tokens_this_frame += 1;
                }
                StreamEvent::ToolCall {
                    id,
                    name,
                    arguments,
                } => {
                    tool_call_handled = true;
                    // Process tool call through executor
                    if let Some(ref mut executor) = ws.tool_executor {
                        let result = executor.execute(&name, &arguments);
                        match result {
                            crate::app::ai_core::executor::ToolResult::Success(output) => {
                                // Auto-approved tool executed — build messages and resume AI
                                let (asst_msg, tool_msg) =
                                    crate::app::ai_core::executor::ToolExecutor::build_approval_messages(
                                        &id, &name, &arguments, &output, false,
                                    );
                                // Append compact tool indicator to conversation display
                                if let Some(last) = ws.ai.chat.conversation.last_mut() {
                                    let size = output.len();
                                    last.1.push_str(&format!("\n\n> **{}** ({} B)", name, size));
                                }
                                // Resume AI with tool result
                                resume_after_tool_call(ws, asst_msg, tool_msg);
                            }
                            crate::app::ai_core::executor::ToolResult::NeedsApproval {
                                tool_name: tn,
                                description,
                                details,
                                is_network,
                                is_new_file,
                            } => {
                                // Check if always-approved at runtime
                                if executor.check_always_approved(&tn) {
                                    let approved_result = executor.process_approval_response(
                                        &tn,
                                        &arguments,
                                        crate::app::ai_core::executor::ApprovalDecision::Always,
                                    );
                                    let (output, is_err) = match &approved_result {
                                        crate::app::ai_core::executor::ToolResult::Success(o) => {
                                            (o.clone(), false)
                                        }
                                        crate::app::ai_core::executor::ToolResult::Error(e) => {
                                            (e.clone(), true)
                                        }
                                        _ => (i18n.get("cli-chat-unexpected-result"), true),
                                    };
                                    let (asst_msg, tool_msg) =
                                        crate::app::ai_core::executor::ToolExecutor::build_approval_messages(
                                            &id, &tn, &arguments, &output, is_err,
                                        );
                                    if let Some(last) = ws.ai.chat.conversation.last_mut() {
                                        let size = output.len();
                                        last.1.push_str(&format!("\n\n> **{}** ({} B)", tn, size));
                                    }
                                    resume_after_tool_call(ws, asst_msg, tool_msg);
                                } else {
                                    // Show approval UI — create channel
                                    let (tx, rx) = std::sync::mpsc::channel();
                                    ws.pending_tool_approval = Some(
                                        crate::app::ui::workspace::state::PendingToolApproval {
                                            tool_call_id: id.clone(),
                                            tool_name: tn,
                                            description,
                                            details,
                                            is_network,
                                            is_new_file,
                                            args: arguments.clone(),
                                            response_tx: tx,
                                        },
                                    );
                                    ws.tool_approval_rx = Some(rx);
                                    // Don't drop stream_rx — approval is pending
                                }
                            }
                            crate::app::ai_core::executor::ToolResult::AskUser {
                                question,
                                options,
                            } => {
                                let (tx, rx) = std::sync::mpsc::channel();
                                ws.pending_tool_ask =
                                    Some(crate::app::ui::workspace::state::PendingToolAsk {
                                        question,
                                        options,
                                        response_tx: tx,
                                        input_buffer: String::new(),
                                    });
                                ws.tool_ask_rx = Some(rx);
                            }
                            crate::app::ai_core::executor::ToolResult::Completion {
                                summary,
                                files_modified: _,
                                follow_up,
                            } => {
                                if let Some(last) = ws.ai.chat.conversation.last_mut() {
                                    let follow = follow_up
                                        .map(|f| format!("\n\n_{}_", f))
                                        .unwrap_or_default();
                                    last.1.push_str(&format!(
                                        "\n\n**Dokonceno:** {}{}",
                                        summary, follow
                                    ));
                                }
                                done = true;
                                ws.toasts.push(Toast::info(summary));
                            }
                            crate::app::ai_core::executor::ToolResult::Error(msg) => {
                                let (asst_msg, tool_msg) =
                                    crate::app::ai_core::executor::ToolExecutor::build_approval_messages(
                                        &id, &name, &arguments, &msg, true,
                                    );
                                if let Some(last) = ws.ai.chat.conversation.last_mut() {
                                    last.1.push_str(&format!("\n\n`{}` chyba: {}", name, msg));
                                }
                                resume_after_tool_call(ws, asst_msg, tool_msg);
                            }
                        }
                    }
                }
            }
        }
        // Update conversation display
        // When a tool call was handled, flush any buffered tokens to conversation
        // before the tool result (model may emit explanation text before tool_calls).
        // But don't overwrite conversation after tool handler already appended results.
        if tool_call_handled {
            // Flush pre-tool text into conversation if any tokens were buffered
            if tokens_this_frame > 0
                && let Some(last) = ws.ai.chat.conversation.last_mut()
            {
                last.1 = ws.ai.chat.streaming_buffer.clone();
            }
            // Don't clear loading/stream — resume_after_tool_call started a new stream
        } else if (tokens_this_frame > 0 || done)
            && let Some(last) = ws.ai.chat.conversation.last_mut()
        {
            last.1 = ws.ai.chat.streaming_buffer.clone();
        }
        if done && !tool_call_handled {
            ws.ai.chat.streaming_buffer.clear();
            ws.ai.chat.loading = false;
            ws.ai.chat.stream_rx = None;
        }
    }

    // --- 4d. Tool approval/ask response processing ---
    if let Some(rx) = &ws.tool_approval_rx
        && let Ok(approved) = rx.try_recv()
    {
        let approval = ws.pending_tool_approval.take();
        ws.tool_approval_rx = None;
        if let Some(pending) = approval
            && let Some(ref mut executor) = ws.tool_executor
        {
            let decision = if approved {
                crate::app::ai_core::executor::ApprovalDecision::Approve
            } else {
                crate::app::ai_core::executor::ApprovalDecision::Deny
            };
            let result =
                executor.process_approval_response(&pending.tool_name, &pending.args, decision);
            let (output, is_err) = match &result {
                crate::app::ai_core::executor::ToolResult::Success(o) => (o.clone(), false),
                crate::app::ai_core::executor::ToolResult::Error(e) => (e.clone(), true),
                _ => (i18n.get("cli-chat-unexpected-result"), true),
            };
            let (asst_msg, tool_msg) =
                crate::app::ai_core::executor::ToolExecutor::build_approval_messages(
                    &pending.tool_call_id,
                    &pending.tool_name,
                    &pending.args,
                    &output,
                    is_err,
                );
            if let Some(last) = ws.ai.chat.conversation.last_mut() {
                let label = if approved { "schvaleno" } else { "zamitnuto" };
                last.1.push_str(&format!(
                    "\n\n`{}` ({}) => {}",
                    pending.tool_name,
                    label,
                    if output.len() > 200 {
                        format!("{}...", &output[..200])
                    } else {
                        output
                    }
                ));
            }
            resume_after_tool_call(ws, asst_msg, tool_msg);
        }
    }

    if let Some(rx) = &ws.tool_ask_rx
        && let Ok(response) = rx.try_recv()
    {
        let ask = ws.pending_tool_ask.take();
        ws.tool_ask_rx = None;
        if let Some(_pending) = ask {
            // Build tool result with user's answer and resume AI
            let tool_msg = crate::app::ai_core::AiMessage {
                role: "tool".to_string(),
                content: response.clone(),
                monologue: Vec::new(),
                timestamp: 0,
                tool_call_name: None,
                tool_call_id: None,
                tool_result_for_id: None,
                tool_is_error: false,
                tool_call_arguments: None,
            };
            let asst_msg = crate::app::ai_core::AiMessage {
                role: "assistant".to_string(),
                content: String::new(),
                monologue: Vec::new(),
                timestamp: 0,
                tool_call_name: Some("ask_user".to_string()),
                tool_call_id: None,
                tool_result_for_id: None,
                tool_is_error: false,
                tool_call_arguments: None,
            };
            if let Some(last) = ws.ai.chat.conversation.last_mut() {
                last.1.push_str(&format!("\n\n**Odpoved:** {}", response));
            }
            resume_after_tool_call(ws, asst_msg, tool_msg);
        }
    }

    // --- 5. Async results ---
    if let Some(rx) = &ws.build_error_rx
        && let Ok(errors) = rx.try_recv()
    {
        ws.build_errors = errors;
        ws.build_error_rx = None;
    }

    // --- 6. Slash command async results ---

    // Poll /build result
    if let Some(rx) = &ws.slash_build_rx
        && let Ok(errors) = rx.try_recv()
    {
        // Only update if conversation wasn't cleared since build started
        if crate::app::ui::terminal::ai_chat::slash::should_apply_async_result(
            ws.slash_conversation_gen,
            ws.slash_build_gen,
        ) {
            let summary = format_slash_build_summary(&errors);
            // Update the last conversation entry that has the "Building..." placeholder
            for entry in ws.ai.chat.conversation.iter_mut().rev() {
                if entry.1.contains("Building...") {
                    entry.1 = format!(
                        "{}{}",
                        crate::app::ui::terminal::ai_chat::slash::SYSTEM_MSG_MARKER,
                        summary
                    );
                    break;
                }
            }
            // Also update ws.build_errors so the editor state reflects latest build
            ws.build_errors = errors;
        }
        ws.slash_build_rx = None;
    }

    // Poll /git result
    if let Some(rx) = &ws.slash_git_rx
        && let Ok(result) = rx.try_recv()
    {
        if crate::app::ui::terminal::ai_chat::slash::should_apply_async_result(
            ws.slash_conversation_gen,
            ws.slash_git_gen,
        ) {
            // Update the last conversation entry that has the "Loading git status..." placeholder
            for entry in ws.ai.chat.conversation.iter_mut().rev() {
                if entry.1.contains("Loading git status...") {
                    entry.1 = format!(
                        "{}{}",
                        crate::app::ui::terminal::ai_chat::slash::SYSTEM_MSG_MARKER,
                        result
                    );
                    break;
                }
            }
        }
        ws.slash_git_rx = None;
    }

    let save_mode = { shared.lock().expect("lock").settings.save_mode.clone() };
    if should_run_autosave(save_mode) && ws.external_change_conflict.is_none() {
        let should_autosave = ws.editor.should_autosave();
        let internal_save = Arc::clone(&shared.lock().expect("lock").is_internal_save);
        if let Some(err) = ws.editor.try_autosave(i18n, &internal_save)
            && should_emit_save_error_toast(&err)
        {
            ws.toasts.push(Toast::error(err));
        } else if should_autosave {
            ws.refresh_profiles_if_active_path();
        }
    }

    if let Some(rx) = &ws.lsp_install_rx
        && let Ok(result) = rx.try_recv()
    {
        match result {
            Ok(()) => {
                ws.toasts.push(Toast::info(i18n.get("lsp-install-success")));
                ws.lsp_binary_missing = false;
            }
            Err(e) => {
                let mut args = fluent_bundle::FluentArgs::new();
                args.set("error", e);
                ws.toasts
                    .push(Toast::error(i18n.get_args("lsp-install-error", &args)));
            }
        }
        ws.lsp_install_rx = None;
    }
}

/// Resumes AI streaming after a tool call by sending the tool result back to Ollama.
/// Replaces the current stream_rx with a new stream from stream_chat.
fn resume_after_tool_call(
    ws: &mut WorkspaceState,
    assistant_msg: crate::app::ai_core::AiMessage,
    tool_result_msg: crate::app::ai_core::AiMessage,
) {
    use crate::app::ai_core::AiMessage;
    use crate::app::ai_core::ollama::OllamaProvider;
    use crate::app::ai_core::provider::{AiProvider, ProviderConfig};
    use crate::app::ai_core::tools::get_standard_tools;

    // Build the full message history including tool call/result
    let mut messages: Vec<AiMessage> = Vec::new();

    // System message (same as in send_query_to_agent)
    let reasoning_mandate = ws.ai.settings.reasoning_depth.get_reasoning_mandate();
    let expertise_mandate = ws.ai.settings.expertise.get_persona_mandate();
    let mut system_parts: Vec<&str> = Vec::new();
    if !ws.ai.chat.system_prompt.is_empty() {
        system_parts.push(&ws.ai.chat.system_prompt);
    }
    if !expertise_mandate.is_empty() {
        system_parts.push(expertise_mandate);
    }
    system_parts.push(reasoning_mandate);
    messages.push(AiMessage {
        role: "system".to_string(),
        content: system_parts.join("\n\n"),
        monologue: Vec::new(),
        timestamp: 0,
        tool_call_name: None,
        tool_call_id: None,
        tool_result_for_id: None,
        tool_is_error: false,
        tool_call_arguments: None,
    });

    // Conversation history
    for (q, _a) in &ws.ai.chat.conversation {
        if !q.is_empty() {
            messages.push(AiMessage {
                role: "user".to_string(),
                content: q.clone(),
                monologue: Vec::new(),
                timestamp: 0,
                tool_call_name: None,
                tool_call_id: None,
                tool_result_for_id: None,
                tool_is_error: false,
                tool_call_arguments: None,
            });
        }
        // Don't push the last assistant response as text — it will be in the tool call message
    }

    // Tool call + result messages
    messages.push(assistant_msg);
    messages.push(tool_result_msg);

    // Start new stream
    let provider = OllamaProvider::new(
        ws.ai.ollama.base_url.clone(),
        ws.ai.ollama.selected_model.clone(),
        ws.ai.ollama.api_key.clone(),
    );
    let config = ProviderConfig {
        base_url: ws.ai.ollama.base_url.clone(),
        model: ws.ai.ollama.selected_model.clone(),
        temperature: ws.ai.settings.temperature,
        num_ctx: ws.ai.settings.num_ctx,
        api_key: ws.ai.ollama.api_key.clone(),
        top_p: ws.ai.settings.top_p,
        top_k: ws.ai.settings.top_k,
        repeat_penalty: ws.ai.settings.repeat_penalty,
        seed: ws.ai.settings.seed,
    };
    let tools = get_standard_tools();
    ws.ai.chat.stream_rx = Some(provider.stream_chat(messages, config, tools));
    ws.ai.chat.loading = true;
}

fn wait_for_child_stdout(
    mut child: std::process::Child,
    cancel: &Arc<AtomicBool>,
) -> Option<Vec<u8>> {
    loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = child.kill();
            return None;
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return None;
                }
                return child.stdout.take().and_then(|mut s| {
                    let mut buf = Vec::new();
                    s.read_to_end(&mut buf).ok()?;
                    Some(buf)
                });
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(25)),
            Err(_) => return None,
        }
    }
}

pub(crate) fn fetch_git_branch(
    root: &std::path::Path,
    cancel: Arc<AtomicBool>,
) -> mpsc::Receiver<Option<String>> {
    let root = root.to_path_buf();
    spawn_task(move || {
        let child = std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&root)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .ok()?;
        let bytes = wait_for_child_stdout(child, &cancel)?;
        Some(String::from_utf8(bytes).ok()?.trim().to_string())
    })
}

fn parse_git_status(root: &std::path::Path, raw: &[u8]) -> HashMap<PathBuf, GitVisualStatus> {
    let mut statuses = HashMap::new();
    let entries: Vec<&[u8]> = raw
        .split(|b| *b == 0)
        .filter(|chunk| !chunk.is_empty())
        .collect();
    let mut i = 0;
    while i < entries.len() {
        let entry = entries[i];
        if entry.len() < 4 {
            i += 1;
            continue;
        }
        let x = entry[0] as char;
        let y = entry[1] as char;
        let mut path_bytes = &entry[3..];
        if matches!(x, 'R' | 'C') && i + 1 < entries.len() {
            i += 1;
            path_bytes = entries[i];
        }
        let rel = String::from_utf8_lossy(path_bytes);
        statuses.insert(root.join(rel.as_ref()), parse_porcelain_status(x, y));
        i += 1;
    }
    statuses
}

/// Append a timestamped message to `/tmp/polycredo-stderr.log`.
fn log_to_stderr_file(msg: &str) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/polycredo-stderr.log")
    {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let _ = writeln!(f, "[{secs}] {msg}");
    }
}

fn format_slash_build_summary(errors: &[crate::app::build_runner::BuildError]) -> String {
    let error_count = errors.iter().filter(|e| !e.is_warning).count();
    let warning_count = errors.iter().filter(|e| e.is_warning).count();

    if errors.is_empty() {
        return "Build OK (0 errors, 0 warnings)".to_string();
    }

    let mut out = format!(
        "**Build complete** ({} errors, {} warnings)\n\n",
        error_count, warning_count
    );

    // List errors first, then warnings
    for err in errors.iter().filter(|e| !e.is_warning) {
        let file = err
            .file
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_else(|| err.file.to_string_lossy().into_owned());
        out.push_str(&format!(
            "- **error** `{}:{}` {}\n",
            file, err.line, err.message
        ));
    }
    for err in errors.iter().filter(|e| e.is_warning) {
        let file = err
            .file
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_else(|| err.file.to_string_lossy().into_owned());
        out.push_str(&format!(
            "- **warning** `{}:{}` {}\n",
            file, err.line, err.message
        ));
    }

    out
}

pub(crate) fn fetch_git_status(
    root: &std::path::Path,
    cancel: Arc<AtomicBool>,
) -> mpsc::Receiver<HashMap<PathBuf, GitVisualStatus>> {
    let root = root.to_path_buf();
    spawn_task(move || {
        let child = std::process::Command::new("git")
            .args(["status", "--porcelain=v1", "-z", "--untracked-files=all"])
            .current_dir(&root)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .ok();
        let raw = child
            .and_then(|c| wait_for_child_stdout(c, &cancel))
            .unwrap_or_default();
        parse_git_status(&root, &raw)
    })
}

#[cfg(test)]
mod tests {
    use super::should_run_autosave;
    use crate::settings::SaveMode;

    #[test]
    fn should_run_autosave_only_in_automatic_mode() {
        assert!(should_run_autosave(SaveMode::Automatic));
        assert!(!should_run_autosave(SaveMode::Manual));
    }
}
