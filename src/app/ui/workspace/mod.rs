pub(crate) mod history;
pub(crate) mod index;
mod menubar;
mod modal_dialogs;
pub(crate) mod semantic_index;
pub(crate) mod state;

// Re-exports for external callers
pub(super) use state::spawn_ai_tool_check;
pub(crate) use state::{
    FsChangeResult, SecondaryWorkspace, WorkspaceState, init_workspace, open_file_in_ws,
    ws_to_panel_state,
};

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use eframe::egui;

use super::super::build_runner::run_build_check;
use super::super::types::{AppShared, FocusedPanel, Toast, should_emit_save_error_toast};
use super::background::process_background_events;
use super::panels::{render_left_panel, render_toasts};
use super::search_picker::{render_file_picker, render_project_search_dialog};
use super::terminal::right::render_ai_panel;
use super::widgets::command_palette::{execute_command, render_command_palette};
use crate::app::ui::dialogs::confirm::{UnsavedGuardDecision, show_unsaved_close_guard_dialog};
use crate::app::ui::widgets::tab_bar::TabBarAction;
use crate::app::ui::workspace::state::{
    DirtyCloseQueueMode, PendingCloseFlow, PendingCloseMode, build_dirty_close_queue_for_mode,
};
use crate::config;
use crate::settings::SaveMode;
use crate::tr;
pub(crate) use menubar::MenuActions;
use menubar::{process_menu_actions, render_menu_bar};
use modal_dialogs::render_dialogs;

fn should_save_settings_draft_on_ctrl_s(show_settings: bool) -> bool {
    show_settings
}

/// Odešle snapshot signál přes background IO kanál pro ne-binární taby.
/// Volá se po úspěšném uložení souboru (manual save, autosave, unsaved-close-guard).
fn send_snapshot_signal(ws: &WorkspaceState, tab_path: &PathBuf) {
    if let Some(tab) = ws.editor.tabs.iter().find(|t| &t.path == tab_path) {
        if tab.is_binary {
            return;
        }
        if let Ok(rel_path) = tab.path.strip_prefix(&ws.root_path) {
            let _ = ws.background_io_tx.send(FsChangeResult::LocalHistory(
                rel_path.to_path_buf(),
                tab.content.clone(),
            ));
        }
    }
}

fn save_mode_status_key(save_mode: &SaveMode) -> &'static str {
    match save_mode {
        SaveMode::Automatic => "statusbar-save-mode-automatic",
        SaveMode::Manual => "statusbar-save-mode-manual",
    }
}

fn status_bar_runtime_mode_key(runtime_mode: &SaveMode) -> &'static str {
    save_mode_status_key(runtime_mode)
}

fn status_bar_save_mode_key_for_runtime(
    runtime_mode: &SaveMode,
    _settings_draft_mode: Option<&SaveMode>,
) -> &'static str {
    status_bar_runtime_mode_key(runtime_mode)
}

fn consume_close_tab_shortcut(ctx: &egui::Context) -> bool {
    ctx.input_mut(|input| {
        input.consume_shortcut(&egui::KeyboardShortcut::new(
            egui::Modifiers::CTRL,
            egui::Key::W,
        ))
    })
}

fn editor_input_locked(dialog_open_base: bool, pending_close_flow_active: bool) -> bool {
    dialog_open_base || pending_close_flow_active
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ManualSaveRequest {
    SaveSettingsDraft,
    SaveEditorFile,
    ShowAlreadySavedInfo,
    NoActiveTab,
}

fn manual_save_request(show_settings: bool, active_modified: Option<bool>) -> ManualSaveRequest {
    if should_save_settings_draft_on_ctrl_s(show_settings) {
        ManualSaveRequest::SaveSettingsDraft
    } else {
        match active_modified {
            Some(true) => ManualSaveRequest::SaveEditorFile,
            Some(false) => ManualSaveRequest::ShowAlreadySavedInfo,
            None => ManualSaveRequest::NoActiveTab,
        }
    }
}

fn manual_save_request_for_shortcut(
    show_settings: bool,
    active_modified: Option<bool>,
) -> ManualSaveRequest {
    manual_save_request(show_settings, active_modified)
}

pub(super) fn handle_manual_save_action(
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &crate::i18n::I18n,
) {
    match manual_save_request_for_shortcut(
        ws.show_settings,
        ws.editor.active().map(|tab| tab.modified),
    ) {
        ManualSaveRequest::SaveSettingsDraft => {
            modal_dialogs::save_settings_draft(ws, shared, i18n)
        }
        ManualSaveRequest::SaveEditorFile => {
            let saved_path = ws.editor.active_path().cloned();
            let internal_save = Arc::clone(&shared.lock().expect("lock").is_internal_save);
            if let Some(err) = ws.editor.save(i18n, &internal_save)
                && should_emit_save_error_toast(&err)
            {
                ws.toasts.push(Toast::error(err));
            } else {
                if let Some(path) = &saved_path {
                    send_snapshot_signal(ws, path);
                }
                ws.refresh_profiles_if_active_path();
            }
        }
        ManualSaveRequest::ShowAlreadySavedInfo => {
            ws.toasts
                .push(Toast::info(i18n.get("info-file-already-saved")));
        }
        ManualSaveRequest::NoActiveTab => {}
    }
}

/// Guard-aware entry point for closing the active editor tab.
///
/// - If there is no active tab, nothing happens.
/// - If there are no dirty tabs, the active tab is closed immediately.
/// - If a guard flow is already active, new close requests are ignored.
/// - Otherwise, a new `PendingCloseFlow` is created with a stable queue of dirty tabs.
fn request_close_tab_target(ws: &mut WorkspaceState, target_path: PathBuf) {
    // Re-entrancy guard: ignore new requests while a flow is in progress.
    if ws.pending_close_flow.is_some() {
        return;
    }

    // Snapshot dirty tabs for queue building.
    let tabs_snapshot: Vec<(PathBuf, bool)> = ws
        .editor
        .tabs
        .iter()
        .map(|t| (t.path.clone(), t.modified))
        .collect();

    let queue = build_dirty_close_queue_for_mode(
        DirtyCloseQueueMode::SingleTab(&target_path),
        &tabs_snapshot,
    );

    if queue.is_empty() {
        // Target tab is clean (or already gone) — close it without guard dialog.
        ws.editor.close_tabs_for_path(&target_path);
        return;
    }

    ws.pending_close_flow = Some(PendingCloseFlow {
        mode: PendingCloseMode::SingleTab,
        queue,
        current_index: 0,
        inline_error: None,
    });
}

pub(crate) fn request_close_active_tab(ws: &mut WorkspaceState) {
    let Some(active_path) = ws.editor.active_path().cloned() else {
        return;
    };
    request_close_tab_target(ws, active_path);
}

pub(crate) fn tabbar_close_target_path(tabs: &[(PathBuf, bool)], idx: usize) -> Option<PathBuf> {
    tabs.get(idx).map(|(path, _)| path.clone())
}

pub(crate) fn open_guard_queue_item_without_focus(
    editor: &mut crate::app::ui::editor::Editor,
    path: &PathBuf,
) {
    editor.open_file_without_focus(path);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnsavedCloseOutcome {
    Continue,
    Finished,
    Cancelled,
}

fn apply_unsaved_close_decision(
    flow: &mut PendingCloseFlow,
    decision: UnsavedGuardDecision,
    save_result: Result<(), String>,
) -> UnsavedCloseOutcome {
    if flow.queue.is_empty() || flow.current_index >= flow.queue.len() {
        flow.inline_error = None;
        return UnsavedCloseOutcome::Cancelled;
    }

    match decision {
        UnsavedGuardDecision::Cancel => {
            flow.inline_error = None;
            UnsavedCloseOutcome::Cancelled
        }
        UnsavedGuardDecision::Discard => {
            flow.inline_error = None;
            if flow.current_index + 1 < flow.queue.len() {
                flow.current_index += 1;
                UnsavedCloseOutcome::Continue
            } else {
                UnsavedCloseOutcome::Finished
            }
        }
        UnsavedGuardDecision::Save => match save_result {
            Ok(()) => {
                flow.inline_error = None;
                if flow.current_index + 1 < flow.queue.len() {
                    flow.current_index += 1;
                    UnsavedCloseOutcome::Continue
                } else {
                    UnsavedCloseOutcome::Finished
                }
            }
            Err(msg) => {
                flow.inline_error = Some(msg);
                UnsavedCloseOutcome::Continue
            }
        },
        UnsavedGuardDecision::Pending => UnsavedCloseOutcome::Continue,
    }
}

fn process_guard_save_failure_feedback(
    flow: &mut PendingCloseFlow,
    toasts: &mut Vec<Toast>,
    message: &str,
    emit_toast: bool,
) -> UnsavedCloseOutcome {
    if emit_toast {
        toasts.push(Toast::error(message.to_string()));
    }
    apply_unsaved_close_decision(flow, UnsavedGuardDecision::Save, Err(message.to_string()))
}

fn should_close_tabs_after_guard_decision(
    decision: UnsavedGuardDecision,
    save_result: &Result<(), String>,
) -> bool {
    match decision {
        UnsavedGuardDecision::Discard => true,
        UnsavedGuardDecision::Save => save_result.is_ok(),
        UnsavedGuardDecision::Cancel | UnsavedGuardDecision::Pending => false,
    }
}

fn process_unsaved_close_guard_dialog(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &crate::i18n::I18n,
) {
    let mut should_refresh_profiles = false;

    {
        let Some(flow) = ws.pending_close_flow.as_mut() else {
            return;
        };

        if flow.queue.is_empty() || flow.current_index >= flow.queue.len() {
            // An empty or exhausted queue means the guard flow is effectively done.
            // Treat this as a cancelled flow from the workspace perspective.
            ws.last_unsaved_close_cancelled = true;
            ws.pending_close_flow = None;
            return;
        }

        let current_path = flow.queue[flow.current_index].clone();

        // Ensure the editor is focused on the current item in the queue.
        if ws.editor.tabs.iter().all(|t| t.path != current_path) {
            // Tab no longer exists; treat as discarded and advance the queue.
            let outcome = apply_unsaved_close_decision(flow, UnsavedGuardDecision::Discard, Ok(()));
            match outcome {
                UnsavedCloseOutcome::Cancelled => {
                    ws.last_unsaved_close_cancelled = true;
                    ws.pending_close_flow = None;
                }
                UnsavedCloseOutcome::Continue => {
                    // Nothing else to do this frame; next item will be handled on the next call.
                }
                UnsavedCloseOutcome::Finished => {
                    ws.last_unsaved_close_cancelled = false;
                    ws.pending_close_flow = None;
                }
            }
            return;
        }

        // Keep queue alignment without requesting editor focus while modal is active.
        open_guard_queue_item_without_focus(&mut ws.editor, &current_path);

        let file_name = current_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let file_path = current_path.to_string_lossy().to_string();

        let decision = show_unsaved_close_guard_dialog(
            ctx,
            i18n,
            file_name,
            &file_path,
            flow.inline_error.as_deref(),
        );

        if decision == UnsavedGuardDecision::Pending {
            return;
        }

        let mut precomputed_outcome = None;
        // Only attempt a save when the user explicitly chose Save.
        let save_result: Result<(), String> = if matches!(decision, UnsavedGuardDecision::Save) {
            let internal_save = Arc::clone(&shared.lock().expect("lock").is_internal_save);
            if let Some(err) = ws.editor.save(i18n, &internal_save) {
                let mut args = fluent_bundle::FluentArgs::new();
                args.set("name", file_name);
                args.set("reason", err.as_str());
                let message = i18n.get_args("unsaved_close_guard_save_failed", &args);
                precomputed_outcome = Some(process_guard_save_failure_feedback(
                    flow,
                    &mut ws.toasts,
                    &message,
                    should_emit_save_error_toast(&message),
                ));
                Err(message.clone())
            } else {
                // Odeslat snapshot signál po úspěšném save v unsaved-close-guard
                if let Some(tab) = ws.editor.tabs.iter().find(|t| t.path == current_path)
                    && !tab.is_binary
                    && let Ok(rel_path) = tab.path.strip_prefix(&ws.root_path)
                {
                    let _ = ws.background_io_tx.send(FsChangeResult::LocalHistory(
                        rel_path.to_path_buf(),
                        tab.content.clone(),
                    ));
                }
                should_refresh_profiles = true;
                Ok(())
            }
        } else {
            Ok(())
        };

        let outcome = precomputed_outcome
            .unwrap_or_else(|| apply_unsaved_close_decision(flow, decision, save_result.clone()));

        // Close tabs only when the decision allows it and the save (if any) succeeded.
        let should_close_tabs = should_close_tabs_after_guard_decision(decision, &save_result);

        if should_close_tabs {
            ws.editor.close_tabs_for_path(&current_path);
        }

        match outcome {
            UnsavedCloseOutcome::Continue => {
                // Keep the flow active; next item (or same on save-fail) will be handled on the next frame.
            }
            UnsavedCloseOutcome::Finished | UnsavedCloseOutcome::Cancelled => {
                ws.last_unsaved_close_cancelled = matches!(outcome, UnsavedCloseOutcome::Cancelled);
                ws.pending_close_flow = None;
            }
        }
    }

    if should_refresh_profiles {
        ws.refresh_profiles_if_active_path();
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
        let id = ws.next_claude_tab_id;
        ws.next_claude_tab_id += 1;
        ws.claude_tabs.push(crate::app::ui::terminal::Terminal::new(
            id,
            ctx,
            &ws.root_path,
            None,
        ));
    }
    if ws.show_build_terminal && ws.build_terminal.is_none() {
        ws.build_terminal = Some(crate::app::ui::terminal::Terminal::new(
            1,
            ctx,
            &ws.root_path,
            None,
        ));
    }

    // Background events
    ws.tick_retired_terminals();
    process_background_events(ws, shared, i18n);

    // --- REPAINT THROTTLING (Focus-aware) ---
    let is_focused = ctx.input(|i| i.viewport().focused.unwrap_or(true));
    let is_minimized = ctx.input(|i| i.viewport().minimized.unwrap_or(false));

    if !is_focused || is_minimized {
        // Unfocused or minimized: VERY slow repaint (2s interval)
        ctx.request_repaint_after(std::time::Duration::from_secs(2));
    } else {
        // --- TYPING FPS CAP ---
        // If user is actively typing, cap at ~30 FPS (33ms) to prevent repaint storm
        let has_kb_input = ctx.input(|i| {
            i.events.iter().any(|e| {
                matches!(
                    e,
                    egui::Event::Key { .. } | egui::Event::Text(_) | egui::Event::Ime(_)
                )
            })
        });
        if has_kb_input {
            ws.last_keystroke_time = Some(std::time::Instant::now());
        }

        let is_typing = ws
            .last_keystroke_time
            .map(|t| t.elapsed().as_millis() < 500)
            .unwrap_or(false);

        if is_typing {
            ctx.request_repaint_after(std::time::Duration::from_millis(33));
        }

        // Focused: Podmíněný repaint — pouze pokud běží aktivní operace na pozadí.
        let has_active_work = ws.build_error_rx.is_some()
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
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
        handle_manual_save_action(ws, shared, i18n);
    }
    if consume_close_tab_shortcut(ctx) {
        request_close_active_tab(ws);
    }
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::B)) {
        if let Some(t) = &mut ws.build_terminal {
            t.send_command("cargo build 2>&1");
        }
        let build_path = ws.root_path.clone();
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
    let mut open_here_path = process_menu_actions(ctx, ws, shared, actions, i18n);

    // --- 2. GLOBAL DIALOGS (Highest priority for input) ---
    let dialog_open_base = ws.file_tree.has_open_dialog()
        || ws.command_palette.is_some()
        || ws.show_settings
        || ws.show_new_project
        || ws.show_about
        || ws.show_semantic_indexing_modal;
    let editor_locked = editor_input_locked(dialog_open_base, ws.pending_close_flow.is_some());

    let dialogs_interacted = render_dialogs(ctx, ws, shared, i18n);
    // Unsaved close guard dialog is rendered after generic dialogs so that it
    // can safely own the decision flow for pending close operations.
    process_unsaved_close_guard_dialog(ctx, ws, shared, i18n);
    render_semantic_indexing_modal(ctx, ws, i18n);
    ws.dep_wizard.render(ctx, i18n);
    if let Some(path) = render_file_picker(ctx, ws, i18n) {
        open_file_in_ws(ws, path);
    }
    render_project_search_dialog(ctx, ws, i18n);

    if let Some(cmd_id) = render_command_palette(ctx, ws, shared, i18n) {
        let mut actions = MenuActions::default();
        if let Some(plugin_res) = execute_command(cmd_id, &mut actions, shared) {
            ws.toasts.push(crate::app::types::Toast::info(plugin_res));
        }
        if let Some(path) = process_menu_actions(ctx, ws, shared, actions, i18n) {
            open_here_path = Some(path);
        }
    }

    // Viewport window
    let mut ai_viewport_clicked = false;
    if ws.ai_viewport_open {
        let viewport_id =
            egui::ViewportId::from_hash_of(format!("ai_viewport_{}", ws.root_path.display()));
        let ai_label = "Terminal".to_string();
        ctx.show_viewport_immediate(
            viewport_id,
            egui::ViewportBuilder::default()
                .with_title(format!("{} — {}", i18n.get("ai-panel-title"), ai_label))
                .with_inner_size([600.0, 500.0]),
            |ctx, _| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let config = crate::app::ui::terminal::right::PanelDisplayConfig {
                        dialog_open: false,
                        focused: ws.focused_panel,
                        font_size: config::EDITOR_FONT_SIZE * ws.ai_panel.font_scale as f32 / 100.0,
                        is_float: false,
                        is_viewport: true,
                    };
                    if crate::app::ui::terminal::right::render_ai_panel_content(
                        ui, ws, shared, i18n, &config,
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
    let should_render_info_separator = ws.lsp_binary_missing || ws.lsp_install_rx.is_some();

    if should_render_info_separator {
        egui::TopBottomPanel::top("info_bar_separator")
            .exact_height(1.0)
            .show(ctx, |ui| {
                ui.separator();
            });
    }

    render_lsp_setup_bar(ctx, ws, i18n);

    egui::TopBottomPanel::bottom("footer_separator")
        .exact_height(1.0)
        .show(ctx, |ui| {
            ui.separator();
        });

    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(config::STATUS_BAR_HEIGHT)
        .show(ctx, |ui| {
            let runtime_save_mode = shared.lock().expect("lock").settings.save_mode.clone();
            let settings_draft_mode = ws.settings_draft.as_ref().map(|draft| &draft.save_mode);
            ui.horizontal(|ui| {
                ws.editor
                    .status_bar(ui, ws.git_branch.as_deref(), i18n, ws.lsp_client.as_ref());
                ui.separator();
                ui.label(
                    egui::RichText::new(i18n.get(status_bar_save_mode_key_for_runtime(
                        &runtime_save_mode,
                        settings_draft_mode,
                    )))
                    .small(),
                );
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

                // Support Button (Heart)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    let support_btn = ui
                        .selectable_label(false, "❤️")
                        .on_hover_text(i18n.get("menu-help-support"));
                    if support_btn.clicked() {
                        ws.show_support = true;
                    }
                });
            });
        });
    // --- 4. PANELS (Side & Bottom) ---
    let bottom_clicked =
        crate::app::ui::terminal::bottom::render_bottom_panel(ctx, ws, dialog_open_base, i18n);
    let ai_clicked = render_ai_panel(ctx, ws, shared, dialog_open_base, i18n);
    let left_clicked = render_left_panel(ctx, ws, shared, dialog_open_base, i18n);

    // --- 5. CENTRAL PANEL (Editor) ---
    let prev_active_path = ws.editor.active_path().cloned();
    egui::CentralPanel::default().show(ctx, |ui| {
        let settings = Arc::clone(&shared.lock().expect("lock").settings);
        let editor_res = ws
            .editor
            .ui(ui, editor_locked, i18n, ws.lsp_client.as_ref(), &settings);
        if editor_res.clicked {
            ws.focused_panel = FocusedPanel::Editor;
        }

        if let Some(action) = editor_res.tab_action {
            match action {
                TabBarAction::Close(idx) => {
                    let tabs_snapshot: Vec<(PathBuf, bool)> = ws
                        .editor
                        .tabs
                        .iter()
                        .map(|tab| (tab.path.clone(), tab.modified))
                        .collect();
                    if let Some(target_path) = tabbar_close_target_path(&tabs_snapshot, idx) {
                        request_close_tab_target(ws, target_path);
                    }
                }
                TabBarAction::ShowHistory(idx) => {
                    if let Some(tab) = ws.editor.tabs.get(idx)
                        && let Ok(rel_path) = tab.path.strip_prefix(&ws.root_path)
                    {
                        let entries = ws.local_history.get_history(rel_path);
                        if entries.is_empty() {
                            ws.toasts
                                .push(Toast::info(i18n.get("history-panel-no-versions")));
                        } else {
                            ws.history_view = Some(history::HistoryViewState {
                                file_path: tab.path.clone(),
                                relative_path: rel_path.to_path_buf(),
                                entries,
                                selected_index: None,
                                preview_content: None,
                                scroll_to_selected: false,
                            });
                        }
                    }
                }
                TabBarAction::Switch(_) | TabBarAction::New => {
                    // Other actions are already handled inside the editor UI.
                }
            }
        }

        if let Some((path_str, action, _new_text)) = editor_res.diff_action
            && action == crate::app::ui::editor::DiffAction::Accepted
        {
            let path = PathBuf::from(&path_str);
            if !ws.editor.tabs.iter().any(|t| t.path == path) {
                open_file_in_ws(ws, path);
            }
        }

        // History panel overlay — zobrazí se místo editoru pokud je aktivní
        if ws.history_view.is_some() {
            // Vytvořit nový scope pro borrow split:
            // potřebujeme &mut history_view a &local_history současně.
            let hv = ws.history_view.as_mut().unwrap();
            let close = history::render_history_panel(hv, &ws.local_history, ui, i18n);
            if close {
                ws.history_view = None;
            }
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

    // Reset focus to editor only when the user explicitly clicks outside all panels.
    // Do NOT reset just because the mouse drifted away from the terminal area —
    // that would make keyboard input impossible after clicking the terminal.
    let any_panel_interacted =
        ai_clicked || left_clicked || ai_viewport_clicked || bottom_clicked || dialogs_interacted;
    let guard_active = ws.pending_close_flow.is_some();
    if !any_panel_interacted && !guard_active {
        let in_terminal =
            ws.focused_panel == FocusedPanel::Claude || ws.focused_panel == FocusedPanel::Build;
        let explicit_click_elsewhere = ctx.input(|i| i.pointer.any_click());
        if in_terminal && explicit_click_elsewhere {
            ws.focused_panel = FocusedPanel::Editor;
            ws.editor.request_editor_focus();
        }
    }

    render_toasts(ctx, ws, i18n);
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

fn render_semantic_indexing_modal(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    i18n: &crate::i18n::I18n,
) {
    if !ws.show_semantic_indexing_modal {
        return;
    }
    let res = {
        if let Ok(si) = ws.semantic_index.try_lock() {
            Some((
                si.is_indexing.load(std::sync::atomic::Ordering::SeqCst),
                si.files_processed.load(std::sync::atomic::Ordering::SeqCst),
                si.files_total.load(std::sync::atomic::Ordering::SeqCst),
                si.current_file.lock().unwrap().clone(),
                si.error.lock().unwrap().clone(),
                Arc::clone(&si.stop_requested),
            ))
        } else {
            None
        }
    };

    let Some((is_indexing, processed, total, current_file, error, stop_signal)) = res else {
        // If locked, we don't render the modal content this frame to keep UI responsive
        return;
    };
    if !is_indexing && error.is_none() && (processed >= total || total == 0) {
        ws.show_semantic_indexing_modal = false;
        return;
    }

    egui::Window::new(i18n.get("semantic-indexing-title"))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_width(450.0);
            ui.add_space(8.0);

            if let Some(err) = &error {
                ui.colored_label(egui::Color32::RED, err);
                ui.add_space(16.0);
                if ui.button(i18n.get("btn-close")).clicked() {
                    ws.show_semantic_indexing_modal = false;
                }
            } else if is_indexing {
                ui.vertical_centered(|ui| {
                    let progress = if total > 0 {
                        processed as f32 / total as f32
                    } else {
                        0.0
                    };

                    ui.label(tr!(
                        i18n,
                        "semantic-indexing-processing",
                        processed = processed,
                        total = total
                    ));
                    ui.add(
                        egui::ProgressBar::new(progress)
                            .show_percentage()
                            .animate(true),
                    );

                    // Show current file name, but truncate if too long
                    let display_path = if current_file.len() > 50 {
                        format!("...{}", &current_file[current_file.len() - 47..])
                    } else {
                        current_file.clone()
                    };
                    ui.small(display_path);

                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        if ui.button(i18n.get("semantic-indexing-btn-stop")).clicked() {
                            stop_signal.store(true, std::sync::atomic::Ordering::SeqCst);
                        }
                        if ui.button(i18n.get("semantic-indexing-btn-bg")).clicked() {
                            ws.show_semantic_indexing_modal = false;
                        }
                    });
                });
            } else {
                ui.vertical_centered(|ui| {
                    ui.label(i18n.get("btn-done"));
                    ui.add_space(16.0);
                    if ui.button(i18n.get("btn-close")).clicked() {
                        ws.show_semantic_indexing_modal = false;
                    }
                });
            }
            ui.add_space(8.0);
        });
}

#[cfg(test)]
mod tests {
    use super::ManualSaveRequest;
    use super::manual_save_request;
    use super::manual_save_request_for_shortcut;
    use super::save_mode_status_key;
    use super::should_save_settings_draft_on_ctrl_s;
    use super::status_bar_save_mode_key_for_runtime;
    use crate::settings::SaveMode;

    mod save_mode;
    mod unsaved_close_guard;

    #[test]
    fn ctrl_s_is_routed_to_settings_when_settings_modal_is_open() {
        assert!(should_save_settings_draft_on_ctrl_s(true));
        assert!(!should_save_settings_draft_on_ctrl_s(false));
    }

    #[test]
    fn status_bar_uses_mode_specific_save_mode_key() {
        assert_eq!(
            save_mode_status_key(&SaveMode::Automatic),
            "statusbar-save-mode-automatic"
        );
        assert_eq!(
            save_mode_status_key(&SaveMode::Manual),
            "statusbar-save-mode-manual"
        );
    }

    #[test]
    fn save_mode_status_ignores_settings_draft_before_apply() {
        let runtime_mode = SaveMode::Manual;
        let settings_draft_mode = Some(&SaveMode::Automatic);
        assert_eq!(
            status_bar_save_mode_key_for_runtime(&runtime_mode, settings_draft_mode),
            "statusbar-save-mode-manual"
        );
    }

    #[test]
    fn save_mode_status_tracks_runtime_after_apply() {
        let runtime_mode = SaveMode::Automatic;
        let settings_draft_mode = Some(&SaveMode::Manual);
        assert_eq!(
            status_bar_save_mode_key_for_runtime(&runtime_mode, settings_draft_mode),
            "statusbar-save-mode-automatic"
        );
    }

    #[test]
    fn manual_save_request_routes_to_settings_when_modal_is_open() {
        assert_eq!(
            manual_save_request(true, Some(true)),
            ManualSaveRequest::SaveSettingsDraft
        );
    }

    #[test]
    fn manual_save_request_routes_modified_tab_to_file_save() {
        assert_eq!(
            manual_save_request(false, Some(true)),
            ManualSaveRequest::SaveEditorFile
        );
    }

    #[test]
    fn manual_save_request_routes_clean_tab_to_info_toast() {
        assert_eq!(
            manual_save_request(false, Some(false)),
            ManualSaveRequest::ShowAlreadySavedInfo
        );
    }

    #[test]
    fn manual_save_request_routes_no_active_tab_to_noop() {
        assert_eq!(
            manual_save_request(false, None),
            ManualSaveRequest::NoActiveTab
        );
    }

    #[test]
    fn manual_save_request_shortcut_helper_respects_settings_priority() {
        assert_eq!(
            manual_save_request_for_shortcut(true, Some(true)),
            ManualSaveRequest::SaveSettingsDraft
        );
        assert_eq!(
            manual_save_request_for_shortcut(false, Some(true)),
            ManualSaveRequest::SaveEditorFile
        );
        assert_eq!(
            manual_save_request_for_shortcut(false, Some(false)),
            ManualSaveRequest::ShowAlreadySavedInfo
        );
        assert_eq!(
            manual_save_request_for_shortcut(false, None),
            ManualSaveRequest::NoActiveTab
        );
    }
}
