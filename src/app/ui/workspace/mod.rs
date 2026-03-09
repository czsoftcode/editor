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
use super::super::types::{AppShared, FocusedPanel, Toast};
use super::background::process_background_events;
use super::panels::{render_left_panel, render_toasts};
use super::search_picker::{render_file_picker, render_project_search_dialog};
use super::terminal::right::render_ai_panel;
use super::widgets::command_palette::{execute_command, render_command_palette};
use crate::config;
use crate::tr;
pub(crate) use menubar::MenuActions;
use menubar::{process_menu_actions, render_menu_bar};
use modal_dialogs::render_dialogs;

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
        // Unfocused or minimized: VERY slow repaint (2s fallback)
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
        let has_active_work = ws.ai.chat.loading
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
        ws.show_ai_chat = true;
        ws.ai.chat.focus_requested = true;
        ws.focused_panel = FocusedPanel::AiChat;
    }

    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
        let internal_save = Arc::clone(&shared.lock().expect("lock").is_internal_save);
        if let Some(err) = ws
            .editor
            .save(i18n, &internal_save)
        {
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
        || ws.show_semantic_indexing_modal
        || ws.show_ai_chat;

    let dialogs_interacted = render_dialogs(ctx, ws, shared, i18n);
    render_semantic_indexing_modal(ctx, ws, i18n);
    ws.dep_wizard.render(ctx, i18n);
    if let Some(path) = render_file_picker(ctx, ws, i18n) {
        open_file_in_ws(ws, path);
    }
    render_project_search_dialog(ctx, ws, i18n);

    if let Some(cmd_id) = render_command_palette(ctx, ws, shared, i18n) {
        let mut actions = MenuActions::default();
        if let Some(plugin_res) = execute_command(cmd_id, &mut actions, shared) {
            if plugin_res == "OPEN_AI_CHAT_MODAL" {
                ws.show_ai_chat = true;
                ws.ai.chat.focus_requested = true;
                ws.ai.chat.response = None;
            } else {
                ws.toasts.push(crate::app::types::Toast::info(plugin_res));
            }
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
                        font_size: config::EDITOR_FONT_SIZE * ws.ai.settings.font_scale as f32 / 100.0,
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
    let should_render_info_separator =
        ws.lsp_binary_missing || ws.lsp_install_rx.is_some();

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
    let ai_chat_clicked = crate::app::ui::terminal::ai_chat::show(ctx, ws, shared, i18n);
    let bottom_clicked =
        crate::app::ui::terminal::bottom::render_bottom_panel(ctx, ws, dialog_open_base, i18n);
    let ai_clicked = render_ai_panel(ctx, ws, shared, dialog_open_base, i18n);
    let left_clicked = render_left_panel(ctx, ws, shared, dialog_open_base, i18n);

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
            if !ws.editor.tabs.iter().any(|t| t.path == path) {
                open_file_in_ws(ws, path);
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
    let any_panel_interacted = ai_chat_clicked
        || ai_clicked
        || left_clicked
        || ai_viewport_clicked
        || ws.show_ai_chat
        || bottom_clicked
        || dialogs_interacted;
    if !any_panel_interacted {
        let in_terminal = ws.focused_panel == FocusedPanel::Claude
            || ws.focused_panel == FocusedPanel::Build
            || ws.focused_panel == FocusedPanel::AiChat;
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
    #[test]
    fn ctrl_s_is_routed_to_settings_when_settings_modal_is_open() {
        assert!(should_save_settings_draft_on_ctrl_s(true));
        assert!(!should_save_settings_draft_on_ctrl_s(false));
    }
}
