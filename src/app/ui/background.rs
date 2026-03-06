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

use super::super::types::{AppShared, Toast};
use super::git_status::{GitVisualStatus, parse_porcelain_status};
use super::workspace::{FsChangeResult, WorkspaceState, spawn_ai_tool_check};
use super::workspace::state::OllamaConnectionStatus;
use crate::app::ai::{OllamaStatus, spawn_ollama_check};
use crate::watcher::{FileEvent, FsChange};
use std::sync::Mutex;

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

    // --- 4b. Ollama polling ---
    if let Some(rx) = &ws.ollama_check_rx
        && let Ok(status) = rx.try_recv()
    {
        match status {
            OllamaStatus::Available(models) => {
                ws.ollama_status = OllamaConnectionStatus::Connected;
                if ws.ollama_selected_model.is_empty()
                    || !models.contains(&ws.ollama_selected_model)
                {
                    if let Some(first) = models.first() {
                        ws.ollama_selected_model = first.clone();
                    }
                }
                ws.ollama_models = models;
            }
            OllamaStatus::Unavailable => {
                ws.ollama_status = OllamaConnectionStatus::Disconnected;
                ws.ollama_models.clear();
            }
        }
        ws.ollama_check_rx = None;
        ws.ollama_last_check = std::time::Instant::now();
    }
    if ws.ollama_last_check.elapsed().as_secs() >= crate::config::OLLAMA_CHECK_INTERVAL_SECS
        && ws.ollama_check_rx.is_none()
        && !ws.ai_loading
    {
        ws.ollama_check_rx = Some(spawn_ollama_check(ws.ollama_base_url.clone()));
    }

    // --- 5. Async results ---
    if let Some(rx) = &ws.build_error_rx
        && let Ok(errors) = rx.try_recv()
    {
        ws.build_errors = errors;
        ws.build_error_rx = None;
    }

    if ws.external_change_conflict.is_none() {
        if let Some(err) = ws.editor.try_autosave(
            i18n,
            &shared.lock().expect("lock").is_internal_save,
        ) {
            ws.toasts.push(Toast::error(err));
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
