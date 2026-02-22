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

use eframe::egui;

use super::super::types::{AppShared, Toast};
use super::workspace::{FsChangeResult, WorkspaceState, spawn_ai_tool_check};
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
        let sandbox_root = &ws.sandbox.root;

        for change in &fs_changes {
            ws.project_index.handle_change(change.clone());
            ws.sandbox_staged_dirty = true;
            ws.sandbox_staged_last_dirty = std::time::Instant::now();

            match change {
                FsChange::Created(path) => {
                    need_reload = true;
                    if path.is_file() {
                        created_file = Some(path.clone());
                        if path.starts_with(sandbox_root) {
                            // Created in sandbox
                            if let Ok(rel_path) = path.strip_prefix(sandbox_root) {
                                let real_path_str =
                                    ws.root_path.join(rel_path).to_string_lossy().to_string();
                                let auto_show = shared
                                    .lock()
                                    .expect("lock shared")
                                    .settings
                                    .auto_show_ai_diff;
                                if auto_show {
                                    let (tx, rx) = mpsc::channel();
                                    ws.background_io_rx = Some(rx);
                                    let p = path.clone();
                                    std::thread::spawn(move || {
                                        if let Ok(new_content) = std::fs::read_to_string(p) {
                                            let _ = tx.send(FsChangeResult::AiDiff(
                                                real_path_str,
                                                String::new(),
                                                new_content,
                                            ));
                                        }
                                    });
                                }
                            }
                        } else if path.starts_with(&ws.root_path) && !path.starts_with(sandbox_root)
                        {
                            // Created in REAL PROJECT -> Auto-sync TO SANDBOX
                            if let Ok(rel_path) = path.strip_prefix(&ws.root_path) {
                                // Skip if the relative path is inside .polycredo itself
                                if !rel_path.starts_with(".polycredo") {
                                    let target = sandbox_root.join(rel_path);
                                    if let Some(parent) = target.parent() {
                                        let _ = std::fs::create_dir_all(parent);
                                    }
                                    let _ = std::fs::copy(path, target);
                                }
                            }
                        }
                    }
                }
                FsChange::Removed(path) => {
                    need_reload = true;
                    ws.editor.close_tabs_for_path(path);

                    if path.starts_with(sandbox_root) {
                        // DELETED IN SANDBOX
                        if let Ok(rel_path) = path.strip_prefix(sandbox_root) {
                            let project_path = ws.root_path.join(rel_path);
                            if project_path.exists() && ws.sandbox_deletion_sync.is_none() {
                                // File was deleted in sandbox but exists in project -> show modal
                                ws.sandbox_deletion_sync = Some(rel_path.to_path_buf());
                            }
                        }
                    }
                }
                FsChange::Modified(path) => {
                    need_reload = true;
                    if path.starts_with(sandbox_root) {
                        // Modified in sandbox
                        if let Ok(rel_path) = path.strip_prefix(sandbox_root) {
                            let auto_show = shared
                                .lock()
                                .expect("lock shared")
                                .settings
                                .auto_show_ai_diff;
                            if auto_show {
                                let project_path = ws.root_path.join(rel_path);
                                let (tx, rx) = mpsc::channel();
                                ws.background_io_rx = Some(rx);
                                let p_sandbox = path.clone();
                                let p_project = project_path.clone();
                                let path_str = project_path.to_string_lossy().to_string();
                                std::thread::spawn(move || {
                                    let original =
                                        std::fs::read_to_string(p_project).unwrap_or_default();
                                    if let Ok(new_content) = std::fs::read_to_string(p_sandbox) {
                                        let _ = tx.send(FsChangeResult::AiDiff(
                                            path_str,
                                            original,
                                            new_content,
                                        ));
                                    }
                                });
                            }
                        }
                    } else if path.starts_with(&ws.root_path) && !path.starts_with(sandbox_root) {
                        // Modified in REAL PROJECT -> Auto-sync TO SANDBOX
                        if let Ok(rel_path) = path.strip_prefix(&ws.root_path) {
                            // Skip if the relative path is inside .polycredo itself
                            if !rel_path.starts_with(".polycredo") {
                                let target = sandbox_root.join(rel_path);
                                // Avoid infinite sync loop by checking if files differ
                                let should_sync = if !target.exists() {
                                    true
                                } else {
                                    let m_proj = match path.metadata() {
                                        Ok(m) => m,
                                        Err(_) => return, // Skip this change if metadata fails
                                    };
                                    let m_sand = match target.metadata() {
                                        Ok(m) => m,
                                        Err(_) => {
                                            // If sandbox file metadata fails, we sync just to be safe
                                            let _ = std::fs::copy(path, target);
                                            return;
                                        }
                                    };
                                    m_proj
                                        .modified()
                                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                                        > m_sand
                                            .modified()
                                            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                                };
                                if should_sync {
                                    let _ = std::fs::copy(path, target);
                                }
                            }
                        }
                    }
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
        ws.file_tree.set_git_colors(status);
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
        ws.ai_tool_check_rx = Some(spawn_ai_tool_check());
    }

    // --- 5. Async results ---
    if let Some(rx) = &ws.build_error_rx
        && let Ok(errors) = rx.try_recv()
    {
        ws.build_errors = errors;
        ws.build_error_rx = None;
    }

    if let Some(rx) = &ws.sandbox_staged_rx
        && let Ok(files) = rx.try_recv()
    {
        ws.sandbox_staged_files = files;
        ws.sandbox_staged_rx = None;
        ws.sandbox_staged_last_refresh = std::time::Instant::now();
    }

    if ws.external_change_conflict.is_none()
        && let Some(err) = ws
            .editor
            .try_autosave(i18n, &shared.lock().expect("lock").is_internal_save)
    {
        ws.toasts.push(Toast::error(err));
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

fn parse_git_status(root: &std::path::Path, raw: &[u8]) -> HashMap<PathBuf, egui::Color32> {
    let mut colors = HashMap::new();
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
        let color = match (x, y) {
            ('?', '?') => egui::Color32::from_rgb(120, 190, 255),
            ('D', _) | (_, 'D') => egui::Color32::from_rgb(210, 80, 80),
            ('A', _) => egui::Color32::from_rgb(100, 200, 110),
            _ => egui::Color32::from_rgb(220, 180, 60),
        };
        colors.insert(root.join(rel.as_ref()), color);
        i += 1;
    }
    colors
}

pub(crate) fn fetch_git_status(
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
