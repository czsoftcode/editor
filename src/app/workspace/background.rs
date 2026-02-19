use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

use eframe::egui;

use super::{Toast, WorkspaceState, spawn_file_index_scan};
use crate::watcher::{FileEvent, FsChange};

/// Zpracuje události z watcherů, build výsledky a autosave.
pub(super) fn process_background_events(ws: &mut WorkspaceState) {
    for event in ws.watcher.try_recv() {
        match event {
            FileEvent::Changed(changed_path) => {
                // Porovnáme kanonizovanou cestu se všemi záložkami editoru.
                if let Ok(changed_canonical) = changed_path.canonicalize() {
                    if let Some(tab_path) =
                        ws.editor.tab_path_for_canonical(&changed_canonical)
                    {
                        if ws.editor.is_path_modified(&tab_path) {
                            // Záložka má neuložené změny → zobrazit dialog.
                            // Nepřepisujeme existující konflikt (mohl by ještě čekat).
                            if ws.external_change_conflict.is_none() {
                                ws.external_change_conflict = Some(tab_path);
                            }
                        } else {
                            // Žádné neuložené změny → bezpečně načíst z disku.
                            ws.editor.reload_path_from_disk(&tab_path);
                        }
                    }
                }
            }
            FileEvent::Removed(removed_path) => {
                ws.editor.notify_file_deleted(&removed_path);
                let name = removed_path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| removed_path.to_string_lossy().into_owned());
                ws.toasts
                    .push(Toast::error(format!("Soubor byl smazán z disku: {name}")));
            }
        }
    }

    let fs_changes = ws.project_watcher.poll();
    if !fs_changes.is_empty() {
        let mut need_reload = false;
        let mut created_file: Option<PathBuf> = None;
        for change in &fs_changes {
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
                FsChange::Modified => {
                    need_reload = true;
                }
            }
        }
        if need_reload {
            // Rozbalit strom na nový soubor, ale neotevírat ho v editoru automaticky.
            if let Some(ref path) = created_file {
                ws.file_tree.request_reload_and_expand(path);
            } else {
                ws.file_tree.request_reload();
            }
        }
        if ws.file_index_rx.is_none() {
            ws.file_index_rx = Some(spawn_file_index_scan(ws.root_path.clone()));
        }
    }

    if let Some(rx) = &ws.file_index_rx {
        if let Ok(files) = rx.try_recv() {
            ws.file_index_cache = files;
            ws.file_index_rx = None;
            if let Some(picker) = ws.file_picker.as_mut() {
                let query = picker.query.clone();
                picker.files = ws.file_index_cache.clone();
                picker.query = query;
                picker.update_filter();
            }
        }
    }

    if let Some(rx) = &ws.build_error_rx {
        if let Ok(errors) = rx.try_recv() {
            ws.build_errors = errors;
            ws.build_error_rx = None;
        }
    }

    if let Some(rx) = &ws.project_search.rx {
        if let Ok(results) = rx.try_recv() {
            ws.project_search.results = results;
            ws.project_search.rx = None;
        }
    }
    if let Some(rx) = &ws.ai_tool_check_rx {
        if let Ok(status) = rx.try_recv() {
            ws.ai_tool_available = status;
            ws.ai_tool_check_rx = None;
            ws.ai_tool_last_check = std::time::Instant::now();
        }
    }
    // Periodický re-check AI CLI nástrojů (claude, aider, …)
    if ws.ai_tool_last_check.elapsed().as_secs() >= crate::config::AI_TOOL_CHECK_INTERVAL_SECS
        && ws.ai_tool_check_rx.is_none()
    {
        ws.ai_tool_check_rx = Some(super::spawn_ai_tool_check());
    }

    // Git: načítání větve
    if let Some(rx) = &ws.git_branch_rx {
        if let Ok(branch) = rx.try_recv() {
            ws.git_branch = branch;
            ws.git_branch_rx = None;
        }
    }
    // Git: načítání stavu souborů
    if let Some(rx) = &ws.git_status_rx {
        if let Ok(colors) = rx.try_recv() {
            ws.file_tree.set_git_colors(colors);
            ws.git_status_rx = None;
        }
    }
    // Git: periodický refresh každých 5 sekund
    if ws.git_last_refresh.elapsed().as_secs() >= 5 {
        ws.git_last_refresh = std::time::Instant::now();
        if ws.git_branch_rx.is_none() {
            ws.git_branch_rx = Some(fetch_git_branch(&ws.root_path, Arc::clone(&ws.git_cancel)));
        }
        if ws.git_status_rx.is_none() {
            ws.git_status_rx = Some(fetch_git_status(&ws.root_path, Arc::clone(&ws.git_cancel)));
        }
    }

    // Autosave je pozastavený, pokud čeká dialog o externím konfliktu.
    if ws.external_change_conflict.is_none() {
        if let Some(err) = ws.editor.try_autosave() {
            ws.toasts.push(Toast::error(err));
        }
    }
}

pub(super) fn fetch_git_branch(
    root: &PathBuf,
    cancel: Arc<AtomicBool>,
) -> mpsc::Receiver<Option<String>> {
    let (tx, rx) = mpsc::channel();
    let root = root.clone();
    std::thread::spawn(move || {
        let Ok(mut child) = std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&root)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
        else {
            let _ = tx.send(None);
            return;
        };
        loop {
            if cancel.load(Ordering::Relaxed) {
                let _ = child.kill();
                return;
            }
            match child.try_wait() {
                Ok(Some(status)) => {
                    if cancel.load(Ordering::Relaxed) {
                        return;
                    }
                    let branch = if status.success() {
                        child.stdout.take().and_then(|mut s| {
                            let mut buf = String::new();
                            s.read_to_string(&mut buf).ok()?;
                            Some(buf.trim().to_string())
                        })
                    } else {
                        None
                    };
                    let _ = tx.send(branch);
                    return;
                }
                Ok(None) => std::thread::sleep(std::time::Duration::from_millis(25)),
                Err(_) => {
                    let _ = tx.send(None);
                    return;
                }
            }
        }
    });
    rx
}

fn git_status_color(x: char, y: char) -> egui::Color32 {
    match (x, y) {
        ('?', '?') => egui::Color32::from_rgb(120, 190, 255),
        ('D', _) | (_, 'D') => egui::Color32::from_rgb(210, 80, 80),
        ('A', _) => egui::Color32::from_rgb(100, 200, 110),
        _ => egui::Color32::from_rgb(220, 180, 60),
    }
}

pub(super) fn fetch_git_status(
    root: &PathBuf,
    cancel: Arc<AtomicBool>,
) -> mpsc::Receiver<HashMap<PathBuf, egui::Color32>> {
    let (tx, rx) = mpsc::channel();
    let root = root.clone();
    std::thread::spawn(move || {
        let Ok(mut child) = std::process::Command::new("git")
            .args(["status", "--porcelain=v1", "-z", "--untracked-files=all"])
            .current_dir(&root)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
        else {
            let _ = tx.send(HashMap::new());
            return;
        };
        loop {
            if cancel.load(Ordering::Relaxed) {
                let _ = child.kill();
                return;
            }
            match child.try_wait() {
                Ok(Some(status)) => {
                    if cancel.load(Ordering::Relaxed) {
                        return;
                    }
                    let mut colors = HashMap::new();
                    if status.success() {
                        if let Some(mut stdout) = child.stdout.take() {
                            let mut raw = Vec::new();
                            if stdout.read_to_end(&mut raw).is_ok() {
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
                            }
                        }
                    }
                    let _ = tx.send(colors);
                    return;
                }
                Ok(None) => std::thread::sleep(std::time::Duration::from_millis(25)),
                Err(_) => {
                    let _ = tx.send(HashMap::new());
                    return;
                }
            }
        }
    });
    rx
}
