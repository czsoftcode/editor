use crate::app::ui::widgets::modal::StandardModal;
use crate::app::ui::workspace::state::{SearchResult, WorkspaceState};
use crate::i18n::I18n;
use eframe::egui;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, mpsc};

pub fn fuzzy_match(query: &str, text: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    // Simple contains for now, could be improved to real fuzzy
    text_lower.contains(&query_lower)
}

pub fn render_file_picker(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    i18n: &I18n,
) -> Option<PathBuf> {
    let picker = ws.file_picker.as_mut()?;

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
    let mut show_flag = true;
    let mut close = key_esc;

    if key_enter && !picker.filtered.is_empty() {
        let idx = picker.filtered[picker.selected];
        selected_file = Some(ws.root_path.join(&picker.files[idx]));
        close = true;
    }

    let modal = StandardModal::new(i18n.get("file-picker-heading"), "file_picker_modal")
        .with_size(600.0, 450.0);

    modal.show(ctx, &mut show_flag, |ui| {
        // FOOTER
        if let Some(c) = modal.ui_footer_actions(ui, i18n, |f| {
            if f.close() || f.cancel() {
                return Some(true);
            }
            None
        }) {
            close = c;
        }

        // BODY
        modal.ui_body(ui, |ui| {
            ui.add_space(4.0);
            let resp = ui.add(
                egui::TextEdit::singleline(&mut picker.query)
                    .hint_text(i18n.get("file-picker-placeholder"))
                    .desired_width(ui.available_width())
                    .id(egui::Id::new("file_picker_input")),
            );
            if picker.focus_requested {
                resp.request_focus();
                picker.focus_requested = false;
            }
            if resp.changed() {
                picker.update_filter();
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let mut args = fluent_bundle::FluentArgs::new();
                args.set("total", picker.files.len() as i64);
                args.set("filtered", picker.filtered.len() as i64);
                ui.label(i18n.get_args("file-picker-count-filtered", &args));
            });
            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .id_salt("file_picker_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    for (disp_idx, &file_idx) in picker.filtered.iter().enumerate() {
                        let path = &picker.files[file_idx];
                        let is_sel = disp_idx == picker.selected;
                        let text = path.to_string_lossy();
                        let r = ui.selectable_label(is_sel, text);
                        if is_sel
                            && ctx.input(|i| {
                                i.key_pressed(egui::Key::ArrowUp)
                                    || i.key_pressed(egui::Key::ArrowDown)
                            })
                        {
                            // r.scroll_to_me(None); // Handled by scroll_area implicitly often, but can be explicit
                        }
                        if r.clicked() {
                            selected_file = Some(ws.root_path.join(path));
                            close = true;
                        }
                    }
                });
        });
    });

    if close || !show_flag {
        ws.file_picker = None;
    }

    selected_file
}

pub fn render_project_search_dialog(ctx: &egui::Context, ws: &mut WorkspaceState, i18n: &I18n) {
    if !ws.project_search.show_input {
        return;
    }

    let focus_req = ws.project_search.focus_requested;
    let mut start_search = false;
    let mut close = false;
    let mut show_flag = true;

    let modal = StandardModal::new(i18n.get("project-search-heading"), "project_search_modal")
        .with_size(500.0, 250.0);

    modal.show(ctx, &mut show_flag, |ui| {
        // FOOTER
        if let Some((start, cl)) = modal.ui_footer_actions(ui, i18n, |f| {
            if f.close() || f.cancel() {
                return Some((false, true));
            }
            if f.button("project-search-btn").clicked() {
                return Some((true, false));
            }
            None
        }) {
            start_search = start;
            close = cl;
        }

        // BODY
        modal.ui_body(ui, |ui| {
            ui.add_space(8.0);
            let resp = ui.add(
                egui::TextEdit::singleline(&mut ws.project_search.query)
                    .hint_text(i18n.get("project-search-hint"))
                    .desired_width(ui.available_width())
                    .id(egui::Id::new("project_search_input")),
            );
            if focus_req {
                resp.request_focus();
            }
            if resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                start_search = true;
            }
            ui.add_space(16.0);
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
    if close || !show_flag {
        ws.project_search
            .cancel_epoch
            .fetch_add(1, Ordering::Relaxed);
        ws.project_search.rx = None;
        ws.project_search.show_input = false;
    }
}

pub fn run_project_search(
    root: PathBuf,
    files: Arc<Vec<PathBuf>>,
    query: String,
    epoch: u64,
    cancel_epoch: Arc<std::sync::atomic::AtomicU64>,
) -> mpsc::Receiver<Vec<SearchResult>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut results = Vec::new();
        let q = query.to_lowercase();

        for path in files.iter() {
            if cancel_epoch.load(Ordering::Relaxed) > epoch {
                return;
            }
            let full_path = root.join(path);
            if let Ok(content) = std::fs::read_to_string(&full_path) {
                for (idx, line) in content.lines().enumerate() {
                    if line.to_lowercase().contains(&q) {
                        results.push(SearchResult {
                            file: path.clone(),
                            line: idx + 1,
                            text: line.trim().to_string(),
                        });
                        if results.len() > 1000 {
                            break;
                        }
                    }
                }
            }
            if results.len() > 1000 {
                break;
            }
        }
        let _ = tx.send(results);
    });
    rx
}

pub fn collect_project_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let mut dirs = vec![root.to_path_buf()];

    while let Some(dir) = dirs.pop() {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }
                if path.is_dir() {
                    dirs.push(path);
                } else if let Ok(rel) = path.strip_prefix(root) {
                    files.push(rel.to_path_buf());
                }
            }
        }
        if files.len() > 5000 {
            break;
        }
    }
    files
}
