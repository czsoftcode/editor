pub(crate) mod index;
mod menubar;
mod modal_dialogs;
pub(crate) mod semantic_index;
pub(crate) mod state;

// Re-exports for external callers
pub(crate) use index::ProjectIndex;
pub(super) use state::spawn_ai_tool_check;
pub(crate) use state::{
    FsChangeResult, SearchResult, SecondaryWorkspace, WorkspaceState, init_workspace,
    open_and_jump, open_file_in_ws, ws_to_panel_state,
};

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use eframe::egui;

use super::super::build_runner::run_build_check;
use super::super::types::{AppShared, FocusedPanel, Toast};
use super::ai_panel::render_ai_panel;
use super::background::process_background_events;
use super::panels::{render_left_panel, render_toasts};
use super::search_picker::{render_file_picker, render_project_search_dialog};
use super::widgets::command_palette::{execute_command, render_command_palette};
use super::widgets::modal::StandardModal;
use crate::config;
pub(crate) use menubar::MenuActions;
use menubar::{process_menu_actions, render_menu_bar};
use modal_dialogs::render_dialogs;

fn trigger_sandbox_staged_refresh(ws: &mut WorkspaceState) {
    if ws.sandbox_staged_rx.is_some() {
        return; // Already scanning
    }
    let sandbox = ws.sandbox.root.clone();
    let project = ws.root_path.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    ws.sandbox_staged_rx = Some(rx);

    std::thread::spawn(move || {
        let sb = crate::app::sandbox::Sandbox::new_with_roots(project, sandbox);
        let files = sb.get_staged_files();
        let _ = tx.send(files);
    });
}

fn refresh_sandbox_staged_cache_if_due(ws: &mut WorkspaceState) {
    if !ws.sandbox_staged_dirty {
        return;
    }

    // Debounce: Wait at least 1000ms after the LAST change to let things settle.
    // Also wait at least 3000ms between scans to avoid I/O spam.
    let debounce_ms = 1000;
    let min_interval_ms = 3000;

    let time_since_dirty = ws.sandbox_staged_last_dirty.elapsed().as_millis();
    let time_since_refresh = ws.sandbox_staged_last_refresh.elapsed().as_millis();

    if time_since_dirty >= debounce_ms && time_since_refresh >= min_interval_ms {
        trigger_sandbox_staged_refresh(ws);
        ws.sandbox_staged_dirty = false;
    }
}

pub(crate) fn render_workspace(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
) -> Option<PathBuf> {
    let i18n_arc = {
        std::sync::Arc::clone(
            &shared
                .lock()
                .expect("Failed to lock AppShared for i18n in render_workspace")
                .i18n,
        )
    };
    let i18n = &*i18n_arc;

    // Lazy initialization of terminals — only when the respective panel is visible
    if ws.show_right_panel && ws.claude_tabs.is_empty() {
        let root = ws.sandbox.root.clone();
        let id = ws.next_claude_tab_id;
        ws.next_claude_tab_id += 1;
        ws.claude_tabs
            .push(super::terminal::Terminal::new(id, ctx, &root, None));
    }
    if ws.show_build_terminal && ws.build_terminal.is_none() {
        ws.build_terminal = Some(super::terminal::Terminal::new(1, ctx, &ws.root_path, None));
    }

    // Background events
    process_background_events(ws, shared, i18n, ctx);
    refresh_sandbox_staged_cache_if_due(ws);

    // Podmíněný repaint — pouze pokud běží aktivní operace na pozadí.
    let has_active_work = ws.gemini_loading
        || ws.build_error_rx.is_some()
        || ws.git_status_rx.is_some()
        || ws.git_branch_rx.is_some()
        || ws
            .semantic_index
            .lock()
            .map(|si| si.is_indexing.load(std::sync::atomic::Ordering::SeqCst))
            .unwrap_or(false);

    if has_active_work {
        ctx.request_repaint_after(std::time::Duration::from_millis(
            config::REPAINT_INTERVAL_MS,
        ));
    }

    // --- KEYBOARD SHORTCUTS ---
    if ctx.input(|i| i.modifiers.ctrl && i.modifiers.alt && i.key_pressed(egui::Key::E)) {
        ws.focused_panel = FocusedPanel::Editor;
        ws.editor.request_editor_focus();
    }
    if ctx.input(|i| i.modifiers.ctrl && i.modifiers.alt && i.key_pressed(egui::Key::B)) {
        ws.show_build_terminal = true;
        ws.focused_panel = FocusedPanel::Build;
    }
    if ctx.input(|i| i.modifiers.ctrl && i.modifiers.alt && i.key_pressed(egui::Key::A)) {
        ws.show_right_panel = true;
        ws.focused_panel = FocusedPanel::Claude;
    }
    if ctx.input(|i| i.modifiers.ctrl && i.modifiers.alt && i.key_pressed(egui::Key::G)) {
        ws.show_gemini = true;
        ws.gemini_focus_requested = true;
        ws.focused_panel = FocusedPanel::Gemini;
    }

    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
        let settings = Arc::clone(&shared.lock().expect("lock").settings);
        if let Some(err) = ws.editor.save(
            i18n,
            &shared.lock().expect("lock").is_internal_save,
            settings.project_read_only,
        ) {
            ws.toasts.push(Toast::error(err));
        }
    }
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::W)) {
        ws.editor.clear();
    }
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::B)) {
        if let Some(t) = &mut ws.build_terminal {
            t.send_command("cargo build 2>&1");
        }
        let build_path = if ws.build_in_sandbox {
            ws.sandbox.root.clone()
        } else {
            ws.root_path.clone()
        };
        ws.build_error_rx = Some(run_build_check(build_path));
        ws.build_errors.clear();
    }
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::R))
        && let Some(t) = &mut ws.build_terminal
    {
        t.send_command("cargo run 2>&1");
    }

    // --- 1. TOP UI (MenuBar) ---
    let actions = render_menu_bar(ctx, ws, shared, i18n);
    let mut open_here_path = process_menu_actions(ws, shared, actions, i18n);

    // --- 2. GLOBAL DIALOGS (Highest priority for input) ---
    let dialog_open_base = ws.file_tree.has_open_dialog()
        || ws.command_palette.is_some()
        || ws.show_plugins
        || ws.show_settings
        || ws.show_gemini;

    render_dialogs(ctx, ws, shared, i18n);
    render_semantic_indexing_modal(ctx, ws, i18n);
    if let Some(path) = render_file_picker(ctx, ws, i18n) {
        open_file_in_ws(ws, path);
    }
    render_project_search_dialog(ctx, ws, i18n);

    if let Some(cmd_id) = render_command_palette(ctx, ws, shared, i18n) {
        let mut actions = MenuActions::default();
        if let Some(plugin_res) = execute_command(cmd_id, &mut actions, shared) {
            if plugin_res == "OPEN_GEMINI_MODAL" {
                ws.show_gemini = true;
                ws.gemini_focus_requested = true;
                ws.gemini_response = None;
            } else {
                ws.toasts.push(crate::app::types::Toast::info(plugin_res));
            }
        }
        if let Some(path) = process_menu_actions(ws, shared, actions, i18n) {
            open_here_path = Some(path);
        }
    }

    // Viewport window
    let mut ai_viewport_clicked = false;
    if ws.ai_viewport_open {
        let viewport_id =
            egui::ViewportId::from_hash_of(format!("ai_viewport_{}", ws.root_path.display()));
        ctx.show_viewport_immediate(
            viewport_id,
            egui::ViewportBuilder::default()
                .with_title(format!("AI Terminal — {}", ws.root_path.display()))
                .with_inner_size([600.0, 500.0]),
            |ctx, _| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    if crate::app::ui::ai_panel::render_ai_panel_content(
                        ui,
                        ws,
                        shared,
                        false,
                        ws.focused_panel,
                        config::EDITOR_FONT_SIZE * ws.ai_font_scale as f32 / 100.0,
                        i18n,
                        false,
                        true,
                    ) {
                        ai_viewport_clicked = true;
                        ws.focused_panel = FocusedPanel::Claude;
                    }
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    ws.ai_viewport_open = false;
                }
            },
        );
    }

    // --- 3. STATUS BARS ---
    render_sandbox_staged_bar(ctx, ws, i18n);
    render_lsp_setup_bar(ctx, ws, i18n);
    render_sandbox_deletion_sync_dialog(ctx, ws, i18n);

    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(config::STATUS_BAR_HEIGHT)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ws.editor
                    .status_bar(ui, ws.git_branch.as_deref(), i18n, ws.lsp_client.as_ref());
                let is_indexing = ws
                    .semantic_index
                    .lock()
                    .map(|si| si.is_indexing.load(std::sync::atomic::Ordering::SeqCst))
                    .unwrap_or(false);
                if is_indexing {
                    ui.separator();
                    ui.spinner();
                    ui.label(
                        egui::RichText::new(i18n.get("semantic-indexing-status-bar"))
                            .small()
                            .color(egui::Color32::from_rgb(100, 200, 255)),
                    );
                }
            });
        });

    // --- 4. PANELS (Side) ---
    let ai_clicked = render_ai_panel(ctx, ws, shared, dialog_open_base, i18n);
    let left_clicked = render_left_panel(ctx, ws, dialog_open_base, i18n);

    // --- 5. CENTRAL PANEL (Editor) ---
    let prev_active_path = ws.editor.active_path().cloned();
    egui::CentralPanel::default().show(ctx, |ui| {
        let settings = Arc::clone(&shared.lock().expect("lock").settings);
        let editor_res = ws.editor.ui(
            ui,
            dialog_open_base,
            i18n,
            ws.lsp_client.as_ref(),
            &settings,
        );
        if editor_res.clicked {
            ws.focused_panel = FocusedPanel::Editor;
        }

        if let Some((path_str, action, _new_text)) = editor_res.diff_action
            && action == crate::app::ui::editor::DiffAction::Accepted
        {
            let path = PathBuf::from(&path_str);
            let rel_path = path
                .strip_prefix(&ws.root_path)
                .unwrap_or(&path)
                .to_path_buf();
            let _ = ws.sandbox.promote_file(&rel_path);
            if !ws.editor.tabs.iter().any(|t| t.path == path) {
                open_file_in_ws(ws, path.clone());
            }
            ws.promotion_success = Some(path);
            ws.sandbox_staged_dirty = true;
        }
    });

    // --- FINAL SYNC AND FOCUS ---
    if let Some((path, line, col)) = ws.editor.pending_lsp_navigate.take() {
        open_file_in_ws(ws, path);
        ws.editor.jump_to_location(line, col);
    }
    let new_active_path = ws.editor.active_path().cloned();
    if new_active_path != prev_active_path
        && let Some(path) = &new_active_path
        && let Some(parent) = path.parent()
    {
        ws.watcher.watch(parent);
    }

    // Auto-restart LSP
    if !ws.lsp_binary_missing
        && ws.lsp_client.is_none()
        && ws.root_path.join("Cargo.toml").exists()
        && ws.lsp_last_retry.elapsed().as_secs() > 30
    {
        ws.lsp_last_retry = std::time::Instant::now();
        let root_uri =
            async_lsp::lsp_types::Url::from_directory_path(&ws.root_path).expect("valid root path");
        if let Some(client) = crate::app::lsp::LspClient::new(ctx.clone(), root_uri) {
            ws.lsp_client = Some(client);
        }
    }

    // Reset focus to editor if clicking outside any active panel/window
    let any_panel_interacted = ai_clicked || left_clicked || ai_viewport_clicked || ws.show_gemini;
    if !any_panel_interacted {
        let in_terminal = ws.focused_panel == FocusedPanel::Claude
            || ws.focused_panel == FocusedPanel::Build
            || ws.focused_panel == FocusedPanel::Gemini;
        if in_terminal {
            ws.focused_panel = FocusedPanel::Editor;
            ws.editor.request_editor_focus();
        }
    }

    render_toasts(ctx, ws);
    open_here_path
}

fn render_lsp_setup_bar(ctx: &egui::Context, ws: &mut WorkspaceState, i18n: &crate::i18n::I18n) {
    if !ws.lsp_binary_missing && ws.lsp_install_rx.is_none() {
        return;
    }
    egui::TopBottomPanel::top("lsp_setup_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.visuals_mut().widgets.noninteractive.bg_fill = egui::Color32::from_rgb(60, 50, 20);
            ui.label(
                egui::RichText::new(format!("\u{26A0} {}", i18n.get("lsp-missing-msg")))
                    .color(egui::Color32::from_rgb(255, 200, 100)),
            );
            if ui.button(i18n.get("lsp-install-btn")).clicked() {
                ws.toasts.push(Toast::info(i18n.get("lsp-installing")));
                let (tx, rx) = std::sync::mpsc::channel();
                ws.lsp_install_rx = Some(rx);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();
                    let res = rt.block_on(crate::app::lsp::LspClient::install_rust_analyzer());
                    let _ = tx.send(res);
                });
            }
            if ui.button("\u{00D7}").clicked() {
                ws.lsp_binary_missing = false;
            }
        });
    });
}

fn render_sandbox_staged_bar(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    i18n: &crate::i18n::I18n,
) {
    let staged_files = ws.sandbox_staged_files.clone();
    if staged_files.is_empty() {
        return;
    }
    egui::TopBottomPanel::top("sandbox_staged_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.visuals_mut().widgets.noninteractive.bg_fill = egui::Color32::from_rgb(80, 70, 20);
            ui.label(
                egui::RichText::new(format!("\u{26A0} {}", i18n.get("ai-staged-bar-msg")))
                    .color(egui::Color32::from_rgb(255, 230, 100))
                    .strong(),
            );
            ui.label(format!("({})", staged_files.len()));
            if ui.button(i18n.get("ai-staged-bar-review")).clicked() {
                ws.show_sandbox_staged = true;
            }
            if ui.button(i18n.get("ai-staged-bar-promote-all")).clicked() {
                for f in staged_files {
                    let _ = ws.sandbox.promote_file(&f);
                }
                ws.sandbox_staged_dirty = true;
            }
        });
    });
}

fn render_semantic_indexing_modal(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    i18n: &crate::i18n::I18n,
) {
    if !ws.show_semantic_indexing_modal {
        return;
    }
    let (is_indexing, processed, total, current_file, error) = {
        let si = ws.semantic_index.lock().unwrap();
        (
            si.is_indexing.load(std::sync::atomic::Ordering::SeqCst),
            si.files_processed.load(std::sync::atomic::Ordering::SeqCst),
            si.files_total.load(std::sync::atomic::Ordering::SeqCst),
            si.current_file.lock().unwrap().clone(),
            si.error.lock().unwrap().clone(),
        )
    };
    if !is_indexing && error.is_none() && (processed >= total || total == 0) {
        ws.show_semantic_indexing_modal = false;
        return;
    }
    let modal = StandardModal::new(
        i18n.get("semantic-indexing-title"),
        "semantic_indexing_modal",
    )
    .with_size(500.0, 220.0);
    let mut local_show = ws.show_semantic_indexing_modal;
    modal.show(ctx, &mut local_show, |ui| {
        modal.ui_body(ui, |ui| {
            ui.vertical_centered(|ui| {
                if let Some(err) = &error {
                    ui.label(err);
                } else {
                    let progress = if total > 0 {
                        processed as f32 / total as f32
                    } else {
                        0.0
                    };
                    ui.add(egui::ProgressBar::new(progress).show_percentage());
                    ui.label(current_file);
                }
            });
        });
        modal.ui_footer(ui, |ui| {
            if ui.button(i18n.get("btn-close")).clicked() {
                ws.show_semantic_indexing_modal = false;
            }
            None::<()>
        });
    });
}

fn render_sandbox_deletion_sync_dialog(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    i18n: &crate::i18n::I18n,
) {
    if let Some(rel_path) = ws.sandbox_deletion_sync.clone() {
        egui::Window::new(i18n.get("sandbox-delete-title")).show(ctx, |ui| {
            ui.label(format!("File deleted: {}", rel_path.display()));
            if ui.button("Keep in project").clicked() {
                ws.sandbox_deletion_sync = None;
            }
            if ui.button("Delete from project").clicked() {
                let _ = std::fs::remove_file(ws.root_path.join(&rel_path));
                ws.sandbox_deletion_sync = None;
            }
        });
    }
}
