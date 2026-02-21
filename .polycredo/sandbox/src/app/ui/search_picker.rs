use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;

use eframe::egui;

use super::workspace::{SearchResult, WorkspaceState};

const EXCLUDED_DIRS: &[&str] = &[
    "target",
    ".git",
    "node_modules",
    "vendor",
    ".idea",
    ".vscode",
    ".cache",
];

/// Subsequence fuzzy match — pattern characters must appear in the text in order, but not necessarily adjacent.
pub(crate) fn fuzzy_match(pattern: &str, text: &str) -> bool {
    if pattern.is_empty() {
        return true;
    }
    let mut text_chars = text.chars();
    for pc in pattern.chars() {
        loop {
            match text_chars.next() {
                Some(tc) if tc == pc => break,
                Some(_) => continue,
                None => return false,
            }
        }
    }
    true
}

/// Recursively collects project files (relative paths), skipping insignificant directories.
pub(super) fn collect_project_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut visited = HashSet::new();
    if let Ok(canonical_root) = root.canonicalize() {
        visited.insert(canonical_root);
    }
    collect_files_recursive(root, root, &mut files, &mut visited);
    files.sort();
    files
}

fn collect_files_recursive(
    root: &Path,
    dir: &Path,
    files: &mut Vec<PathBuf>,
    visited: &mut HashSet<PathBuf>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if EXCLUDED_DIRS.contains(&name_str.as_ref()) {
            continue;
        }
        let Ok(meta) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            if let Ok(canonical) = path.canonicalize()
                && !visited.insert(canonical)
            {
                continue;
            }
            collect_files_recursive(root, &path, files, visited);
        } else if meta.is_file()
            && let Ok(rel) = path.strip_prefix(root)
        {
            files.push(rel.to_path_buf());
        }
    }
}

/// render_file_picker — Modal for Ctrl+P
pub(super) fn render_file_picker(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    i18n: &crate::i18n::I18n,
) -> Option<PathBuf> {
    let picker = ws.file_picker.as_mut()?;

    // Global navigation keys (read before rendering to work even when TextEdit has focus)
    let key_up = ctx.input(|i| i.key_pressed(egui::Key::ArrowUp));
    let key_down = ctx.input(|i| i.key_pressed(egui::Key::ArrowDown));
    let key_enter = ctx.input(|i| i.key_pressed(egui::Key::Enter));
    let key_esc = ctx.input(|i| i.key_pressed(egui::Key::Escape));

    let filtered_len = picker.filtered.len();
    if key_up && picker.selected > 0 {
        picker.selected -= 1;
    }
    if key_down && picker.selected + 1 < filtered_len {
        picker.selected += 1;
    }

    let mut selected_file: Option<PathBuf> = None;
    let mut close = key_esc;

    if key_enter && !picker.filtered.is_empty() {
        let idx = picker.filtered[picker.selected];
        selected_file = Some(ws.root_path.join(&picker.files[idx]));
        close = true;
    }

    // Redraw picker only if we still have data
    if let Some(picker) = ws.file_picker.as_mut() {
        let focus_req = picker.focus_requested;
        let total = picker.files.len();
        let _max_show = 14_usize;

        let modal = egui::Modal::new(egui::Id::new("file_picker_modal"));
        modal.show(ctx, |ui| {
            ui.set_min_width(520.0);
            ui.heading(i18n.get("file-picker-heading"));
            ui.add_space(6.0);

            let resp = ui.add(
                egui::TextEdit::singleline(&mut picker.query)
                    .hint_text(i18n.get("file-picker-placeholder"))
                    .desired_width(500.0)
                    .id(egui::Id::new("file_picker_input")),
            );
            if focus_req {
                resp.request_focus();
            }
            if resp.changed() {
                picker.update_filter();
            }

            let count_label = if picker.query.is_empty() {
                let mut args = fluent_bundle::FluentArgs::new();
                args.set("count", total as i64);
                i18n.get_args("file-picker-count", &args)
            } else {
                let mut args = fluent_bundle::FluentArgs::new();
                args.set("filtered", picker.filtered.len() as i64);
                args.set("total", total as i64);
                i18n.get_args("file-picker-count-filtered", &args)
            };
            ui.add_space(2.0);
            ui.label(egui::RichText::new(count_label).weak().size(11.0));
            ui.add_space(4.0);

            egui::ScrollArea::vertical()
                .max_height(320.0)
                .id_salt("fp_scroll")
                .show(ui, |ui| {
                    for (disp_idx, &file_idx) in picker.filtered.iter().enumerate() {
                        let path = &picker.files[file_idx];
                        let is_sel = disp_idx == picker.selected;
                        let text = egui::RichText::new(path.to_string_lossy())
                            .monospace()
                            .size(12.0);
                        let r = ui.selectable_label(is_sel, text);
                        if is_sel {
                            r.scroll_to_me(None);
                        }
                        if r.clicked() {
                            selected_file = Some(ws.root_path.join(path));
                            close = true;
                        }
                    }
                });
        });

        picker.focus_requested = false;
    }

    if close {
        ws.file_picker = None;
    }
    selected_file
}

/// Starts project-wide search in the background (pure Rust, no external tools).
fn run_project_search(
    root: PathBuf,
    files: Vec<PathBuf>,
    query: String,
    epoch: u64,
    cancel_epoch: Arc<AtomicU64>,
) -> mpsc::Receiver<Vec<SearchResult>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        if cancel_epoch.load(Ordering::Relaxed) != epoch {
            return;
        }
        let q = query.to_lowercase();
        let mut results = Vec::new();
        'outer: for rel in files {
            if cancel_epoch.load(Ordering::Relaxed) != epoch {
                return;
            }
            let abs = root.join(&rel);
            let Ok(content) = std::fs::read_to_string(&abs) else {
                continue;
            };
            for (idx, line) in content.lines().enumerate() {
                if line.to_lowercase().contains(&q) {
                    results.push(SearchResult {
                        file: rel.clone(),
                        line: idx + 1,
                        text: line.trim().to_string(),
                    });
                    if results.len() >= 2000 {
                        break 'outer;
                    }
                }
            }
        }
        if cancel_epoch.load(Ordering::Relaxed) == epoch {
            let _ = tx.send(results);
        }
    });
    rx
}

/// Dialog for project-wide search input.
pub(super) fn render_project_search_dialog(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    i18n: &crate::i18n::I18n,
) {
    if !ws.project_search.show_input {
        return;
    }

    let focus_req = ws.project_search.focus_requested;
    let mut start_search = false;
    let mut close = false;

    let modal = egui::Modal::new(egui::Id::new("project_search_modal"));
    modal.show(ctx, |ui| {
        ui.heading(i18n.get("project-search-heading"));
        ui.add_space(8.0);
        let resp = ui.add(
            egui::TextEdit::singleline(&mut ws.project_search.query)
                .hint_text(i18n.get("project-search-hint"))
                .desired_width(380.0)
                .id(egui::Id::new("project_search_input")),
        );
        if focus_req {
            resp.request_focus();
        }
        if resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            start_search = true;
        }
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button(i18n.get("project-search-btn")).clicked() {
                start_search = true;
            }
            if ui.button(i18n.get("btn-cancel")).clicked() {
                close = true;
            }
        });
    });

    ws.project_search.focus_requested = false;

    if start_search && !ws.project_search.query.trim().is_empty() {
        ws.project_search.results.clear();
        let epoch = ws
            .project_search
            .cancel_epoch
            .fetch_add(1, Ordering::Relaxed)
            + 1;
        let cancel_epoch = Arc::clone(&ws.project_search.cancel_epoch);
        let files = ws.project_index.get_files();
        ws.project_search.rx = Some(run_project_search(
            ws.root_path.clone(),
            files,
            ws.project_search.query.trim().to_string(),
            epoch,
            cancel_epoch,
        ));
        ws.project_search.show_input = false;
    }
    if close {
        ws.project_search
            .cancel_epoch
            .fetch_add(1, Ordering::Relaxed);
        ws.project_search.rx = None;
        ws.project_search.show_input = false;
    }
}
