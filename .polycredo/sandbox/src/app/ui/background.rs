use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};

/// Spawns a closure in a new thread and returns a Receiver with the result.
/// Replaces the repeating pattern `let (tx, rx) = channel(); thread::spawn(|| tx.send(f())); rx`.
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

use eframe::egui;

use super::super::types::{AppShared, Toast};
use super::workspace::{WorkspaceState, spawn_ai_tool_check};
use crate::watcher::{FileEvent, FsChange};
use std::sync::Mutex;

/// Processes events from watchers, build results, and autosave.
pub(super) fn process_background_events(
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &crate::i18n::I18n,
) {
    for event in ws.watcher.try_recv() {
        match event {
            FileEvent::Changed(changed_path) => {
                // Compare the canonicalized path with all editor tabs.
                if let Ok(changed_canonical) = changed_path.canonicalize()
                    && let Some(tab_path) = ws.editor.tab_path_for_canonical(&changed_canonical)
                {
                    if ws.editor.is_path_modified(&tab_path) {
                        // Tab has unsaved changes → show dialog.
                        // Do not overwrite existing conflict (it might still be pending).
                        if ws.external_change_conflict.is_none() {
                            ws.external_change_conflict = Some(tab_path);
                        }
                    } else {
                        // No unsaved changes → safely reload from disk.
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
                        // If created in sandbox, trigger diff
                        let sandbox_root = &ws.sandbox.root;
                        if path.starts_with(sandbox_root)
                            && let Ok(rel_path) = path.strip_prefix(sandbox_root)
                            && let Ok(new_content) = std::fs::read_to_string(path)
                        {
                            let real_path = ws.root_path.join(rel_path);
                            ws.editor.pending_ai_diff = Some((
                                real_path.to_string_lossy().to_string(),
                                String::new(), // Empty original content
                                new_content,
                            ));
                        }
                    }
                }
                FsChange::Removed(path) => {
                    need_reload = true;
                    ws.editor.close_tabs_for_path(path);
                }
                FsChange::Modified(path) => {
                    need_reload = true;

                    let sandbox_root = &ws.sandbox.root;
                    if path.starts_with(sandbox_root) {
                        // MODIFIED IN SANDBOX (AI agent working)
                        if let Ok(rel_path) = path.strip_prefix(sandbox_root)
                            && let Ok(new_content) = std::fs::read_to_string(path)
                        {
                            // Find the "truth" from the real project or open tabs
                            let real_path = ws.root_path.join(rel_path);
                            let original_content = if let Some(tab) =
                                ws.editor.tabs.iter().find(|t| t.path == real_path)
                            {
                                tab.last_saved_content.clone()
                            } else {
                                std::fs::read_to_string(&real_path).unwrap_or_default()
                            };

                            // Trigger AI Diff modal
                            ws.editor.pending_ai_diff = Some((
                                real_path.to_string_lossy().to_string(),
                                original_content,
                                new_content,
                            ));
                        }
                    } else {
                        // MODIFIED IN REAL PROJECT (Main workspace)
                        let is_internal = shared
                            .lock()
                            .unwrap()
                            .is_internal_save
                            .load(std::sync::atomic::Ordering::SeqCst);
                        if !is_internal {
                            // External modification (e.g. git pull) -> Save to Local History
                            if let Ok(content) = std::fs::read_to_string(path) {
                                let rel_path = path.strip_prefix(&ws.root_path).unwrap_or(path);
                                ws.local_history.take_snapshot(rel_path, &content);
                            }
                        }
                    }
                }
            }
        }
        if need_reload {
            // Expand the tree to the new file, but do not open it in the editor automatically.
            if let Some(ref path) = created_file {
                ws.file_tree.request_reload_and_expand(path);
            } else {
                ws.file_tree.request_reload();
            }
        }

        // Update FilePicker if open
        if let Some(picker) = ws.file_picker.as_mut() {
            picker.files = ws.project_index.get_files();
            picker.update_filter();
        }
    }

    if let Some(rx) = &ws.build_error_rx
        && let Ok(errors) = rx.try_recv()
    {
        ws.build_errors = errors;
        ws.build_error_rx = None;
    }

    if let Some(rx) = &ws.project_search.rx
        && let Ok(results) = rx.try_recv()
    {
        ws.project_search.results = results;
        ws.project_search.rx = None;
    }
    if let Some(rx) = &ws.ai_tool_check_rx
        && let Ok(status) = rx.try_recv()
    {
        ws.ai_tool_available = status;
        ws.ai_tool_check_rx = None;
        ws.ai_tool_last_check = std::time::Instant::now();
    }
    // Periodic re-check of AI CLI tools (claude, aider, …)
    if ws.ai_tool_last_check.elapsed().as_secs() >= crate::config::AI_TOOL_CHECK_INTERVAL_SECS
        && ws.ai_tool_check_rx.is_none()
    {
        ws.ai_tool_check_rx = Some(spawn_ai_tool_check());
    }

    // Git: loading branch
    if let Some(rx) = &ws.git_branch_rx
        && let Ok(branch) = rx.try_recv()
    {
        ws.git_branch = branch;
        ws.git_branch_rx = None;
    }
    // Git: loading file status
    if let Some(rx) = &ws.git_status_rx
        && let Ok(colors) = rx.try_recv()
    {
        ws.file_tree.set_git_colors(colors);
        ws.git_status_rx = None;
    }
    // Git: periodic refresh every 5 seconds
    if ws.git_last_refresh.elapsed().as_secs() >= 5 {
        ws.git_last_refresh = std::time::Instant::now();
        if ws.git_branch_rx.is_none() {
            ws.git_branch_rx = Some(fetch_git_branch(&ws.root_path, Arc::clone(&ws.git_cancel)));
        }
        if ws.git_status_rx.is_none() {
            ws.git_status_rx = Some(fetch_git_status(&ws.root_path, Arc::clone(&ws.git_cancel)));
        }
    }

    // Autosave is paused if an external conflict dialog is pending.
    if ws.external_change_conflict.is_none()
        && let Some(err) = ws
            .editor
            .try_autosave(i18n, &shared.lock().unwrap().is_internal_save)
    {
        ws.toasts.push(Toast::error(err));
    }

    // LSP installation progress
    if let Some(rx) = &ws.lsp_install_rx
        && let Ok(result) = rx.try_recv()
    {
        match result {
            Ok(()) => {
                ws.toasts.push(Toast::info(i18n.get("lsp-install-success")));
                ws.lsp_binary_missing = false;
                // Trigger a re-check/init by making the next frame believe it was just opened?
                // Actually, init_workspace will be called again if we re-open or we can manually
                // trigger it. For MVP, the user might need to re-open the project or we trigger
                // a refresh.
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

/// Polling for subprocess completion (25 ms intervals) with cancellation support.
/// After the process ends, it reads stdout and returns bytes.
/// Returns None if cancelled, error occurred, or non-zero exit code.
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
                if cancel.load(Ordering::Relaxed) || !status.success() {
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

pub(super) fn fetch_git_branch(
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

fn git_status_color(x: char, y: char) -> egui::Color32 {
    match (x, y) {
        ('?', '?') => egui::Color32::from_rgb(120, 190, 255),
        ('D', _) | (_, 'D') => egui::Color32::from_rgb(210, 80, 80),
        ('A', _) => egui::Color32::from_rgb(100, 200, 110),
        _ => egui::Color32::from_rgb(220, 180, 60),
    }
}

fn parse_git_status(root: &std::path::Path, raw: &[u8]) -> HashMap<PathBuf, egui::Color32> {
    let mut colors = HashMap::new();
    let entries: Vec<&[u8]> = raw
        .split(|b| *b == 0)
        .filter(|chunk| !chunk.is_empty())
        .collect();
    let mut i = 0usize;
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
        colors.insert(root.join(rel.as_ref()), git_status_color(x, y));
        i += 1;
    }
    colors
}

pub(super) fn fetch_git_status(
    root: &std::path::Path,
    cancel: Arc<AtomicBool>,
) -> mpsc::Receiver<HashMap<PathBuf, egui::Color32>> {
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
