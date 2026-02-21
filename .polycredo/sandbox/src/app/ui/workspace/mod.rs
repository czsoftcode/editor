pub(crate) mod index;
mod menubar;
mod modal_dialogs;
pub(crate) mod state;

// Re-exports for external callers (panels.rs, ai_panel.rs, background.rs, app/mod.rs, …)
pub(crate) use index::ProjectIndex;
pub(crate) use state::{
    FilePicker, SearchResult, SecondaryWorkspace, WorkspaceState, init_workspace, open_and_jump,
    open_file_in_ws, ws_to_panel_state,
};
// Visible to siblings in ui/ (background.rs, ai_panel.rs)
pub(super) use state::spawn_ai_tool_check;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use eframe::egui;

use super::super::build_runner::run_build_check;
use super::super::types::{AppShared, FocusedPanel, Toast};
use super::ai_panel::render_ai_panel;
use super::background::{fetch_git_status, process_background_events};
use super::panels::{render_left_panel, render_toasts};
use super::search_picker::{render_file_picker, render_project_search_dialog};
use super::widgets::command_palette::{execute_command, render_command_palette};
use crate::config;
pub(crate) use menubar::MenuActions;
use menubar::{process_menu_actions, render_menu_bar};
use modal_dialogs::render_dialogs;

// ---------------------------------------------------------------------------
// render_workspace — Orchestrator for rendering a single workspace
// Returns Some(path) if the workspace should be reinitialized with a new path.
// ---------------------------------------------------------------------------

pub(crate) fn render_workspace(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
) -> Option<PathBuf> {
    // Extract i18n from shared (short-term lock, then work only with Arc)
    let i18n_arc = { std::sync::Arc::clone(&shared.lock().unwrap().i18n) };
    let i18n = &*i18n_arc;

    // Lazy initialization of terminals
    if ws.claude_tabs.is_empty() {
        let root = ws.sandbox.root.clone();
        let id = ws.next_claude_tab_id;
        ws.next_claude_tab_id += 1;
        ws.claude_tabs
            .push(super::terminal::Terminal::new(id, ctx, &root, None));
    }
    if ws.build_terminal.is_none() {
        ws.build_terminal = Some(super::terminal::Terminal::new(1, ctx, &ws.root_path, None));
    }

    // Background events (watcher, build, autosave)
    process_background_events(ws, shared, i18n);

    // Periodic repaint for autosave and watcher
    ctx.request_repaint_after(std::time::Duration::from_millis(
        config::REPAINT_INTERVAL_MS,
    ));

    // Keyboard shortcuts
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
        if let Some(err) = ws
            .editor
            .save(i18n, &shared.lock().unwrap().is_internal_save)
        {
            ws.toasts.push(Toast::error(err));
        }
        // After saving, immediately update git status
        if ws.git_status_rx.is_none() {
            ws.git_status_rx = Some(fetch_git_status(&ws.root_path, Arc::clone(&ws.git_cancel)));
        }
    }
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::W)) {
        ws.editor.clear();
    }
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::B)) {
        if let Some(t) = &mut ws.build_terminal {
            t.send_command("cargo build 2>&1");
        }
        ws.build_error_rx = Some(run_build_check(ws.root_path.clone()));
        ws.build_errors.clear();
    }
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::R))
        && let Some(t) = &mut ws.build_terminal
    {
        t.send_command("cargo run 2>&1");
    }
    // Ctrl+P — fuzzy file picker
    if ctx.input(|i| i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::P)) {
        if ws.file_picker.is_none() {
            let files = ws.project_index.get_files();
            ws.file_picker = Some(FilePicker::new(files));
        } else {
            ws.file_picker = None;
        }
    }
    // Ctrl+Shift+F — project-wide search
    if ctx.input(|i| i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::F)) {
        ws.project_search.show_input = true;
        ws.project_search.focus_requested = true;
    }
    // Ctrl+Shift+P — command palette
    if ctx.input(|i| i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::P)) {
        if ws.command_palette.is_none() {
            ws.command_palette =
                Some(crate::app::ui::widgets::command_palette::CommandPaletteState::new());
        } else {
            ws.command_palette = None;
        }
    }

    // Menu bar + action processing
    let actions = render_menu_bar(ctx, ws, shared, i18n);
    let mut open_here_path = process_menu_actions(ws, shared, actions, i18n);

    // AI Sandbox Staging Bar (Yellow bar if changes are pending)
    render_sandbox_staged_bar(ctx, ws, i18n);

    // LSP Setup Bar (if binary is missing)
    render_lsp_setup_bar(ctx, ws, i18n);

    // Auto-restart LSP if it was just installed
    if !ws.lsp_binary_missing && ws.lsp_client.is_none() && ws.root_path.join("Cargo.toml").exists()
    {
        let root_uri = async_lsp::lsp_types::Url::from_directory_path(&ws.root_path)
            .expect("valid root path for Url");
        if let Some(client) = crate::app::lsp::LspClient::new(ctx.clone(), root_uri) {
            ws.lsp_client = Some(client);
        } else {
            // Failed to start -> mark as missing to avoid retrying every frame
            ws.lsp_binary_missing = true;
        }
    }

    // Modal dialogs
    render_dialogs(ctx, ws, shared, i18n);

    let mut ai_viewport_clicked = false;
    // AI Viewport (separate window)
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
                        false, // dialog_open (dialogs are in main win)
                        ws.focused_panel,
                        config::EDITOR_FONT_SIZE * ws.ai_font_scale as f32 / 100.0,
                        i18n,
                        false, // is_float
                        true,  // is_viewport
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

    // File picker (Ctrl+P)
    if let Some(path) = render_file_picker(ctx, ws, i18n) {
        open_file_in_ws(ws, path);
    }

    // Project-wide search
    render_project_search_dialog(ctx, ws, i18n);

    // Command Palette (Ctrl+Shift+P)
    if let Some(cmd_id) = render_command_palette(ctx, ws, shared, i18n) {
        let mut actions = MenuActions::default();
        execute_command(cmd_id, &mut actions);
        if let Some(path) = process_menu_actions(ws, shared, actions, i18n) {
            open_here_path = Some(path);
        }
    }

    // Status bar (must be before SidePanel)
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(config::STATUS_BAR_HEIGHT)
        .show(ctx, |ui| {
            ws.editor
                .status_bar(ui, ws.git_branch.as_deref(), i18n, ws.lsp_client.as_ref());
        });

    let dialog_open = ws.file_tree.has_open_dialog();

    // Panels (order: right, left, central)
    let ai_clicked = render_ai_panel(ctx, ws, dialog_open, i18n);
    let left_clicked = render_left_panel(ctx, ws, dialog_open, i18n);

    // Remember active tab before render — to detect tab switching
    let prev_active_path = ws.editor.active_path().cloned();

    egui::CentralPanel::default().show(ctx, |ui| {
        // Construct a dummy settings object just for the fields we need, or clone the whole settings.
        // Let's just clone settings.
        let settings = shared.lock().unwrap().settings.clone();
        let editor_res = ws
            .editor
            .ui(ui, dialog_open, i18n, ws.lsp_client.as_ref(), &settings);
        if editor_res.clicked {
            ws.focused_panel = FocusedPanel::Editor;
        }

        if let Some((path_str, action, _new_text)) = editor_res.diff_action {
            if action == crate::app::ui::editor::DiffAction::Accepted {
                let path = PathBuf::from(&path_str);
                let rel_path = path.strip_prefix(&ws.root_path).unwrap_or(&path).to_path_buf();
                let sandbox_path = ws.sandbox.root.join(&rel_path);

                if sandbox_path.exists() {
                    if let Err(e) = ws.sandbox.promote_file(&rel_path) {
                        ws.toasts
                            .push(Toast::error(format!("AI Promotion failed: {}", e)));
                    } else {
                        // If file was not open in editor (e.g. newly created file by AI), open it now.
                        if !ws.editor.tabs.iter().any(|t| t.path == path) {
                            open_file_in_ws(ws, path.clone());
                        }
                        ws.promotion_success = Some(path);
                    }
                }
            }
        }
    });

    // LSP navigation: open file and jump to precise location
    if let Some((path, line, col)) = ws.editor.pending_lsp_navigate.take() {
        open_file_in_ws(ws, path);
        ws.editor.jump_to_location(line, col);
    }

    // If the tab was switched, switch FileWatcher to the directory of the new tab.
    let new_active_path = ws.editor.active_path().cloned();
    if new_active_path != prev_active_path
        && let Some(path) = &new_active_path
        && let Some(parent) = path.parent()
    {
        ws.watcher.watch(parent);
    }

    // Focus follows mouse — return focus to editor if terminal was not actively clicked
    if !ai_clicked && !left_clicked && !ai_viewport_clicked {
        let in_terminal =
            ws.focused_panel == FocusedPanel::Claude || ws.focused_panel == FocusedPanel::Build;
        if in_terminal {
            ws.focused_panel = FocusedPanel::Editor;
            ws.editor.request_editor_focus();
        }
    }

    // Toast notifications
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
            ui.spacing_mut().item_spacing.x = 12.0;

            ui.label(
                egui::RichText::new(format!("\u{26A0} {}", i18n.get("lsp-missing-msg")))
                    .color(egui::Color32::from_rgb(255, 200, 100)),
            );

            if ui.button(i18n.get("lsp-install-btn")).clicked() {
                ws.toasts.push(Toast::info(i18n.get("lsp-installing")));
                let (tx, rx) = std::sync::mpsc::channel();
                ws.lsp_install_rx = Some(rx);

                std::thread::spawn(move || {
                    let runtime = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();
                    let res = runtime.block_on(crate::app::lsp::LspClient::install_rust_analyzer());
                    let _ = tx.send(res);
                });
            }

            if ui.button("\u{00D7}").clicked() {
                ws.lsp_binary_missing = false;
            }
        });
    });
}

fn render_sandbox_staged_bar(ctx: &egui::Context, ws: &mut WorkspaceState, i18n: &crate::i18n::I18n) {
    let staged_files = ws.sandbox.get_staged_files();
    if staged_files.is_empty() {
        return;
    }

    egui::TopBottomPanel::top("sandbox_staged_bar").show(ctx, |ui| {
        ui.vertical(|ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                // Warning/Info colors (Yellowish)
                ui.visuals_mut().widgets.noninteractive.bg_fill = egui::Color32::from_rgb(80, 70, 20);
                ui.spacing_mut().item_spacing.x = 12.0;

                ui.label(
                    egui::RichText::new(format!("\u{26A0} {}", i18n.get("ai-staged-bar-msg")))
                        .color(egui::Color32::from_rgb(255, 230, 100))
                        .strong(),
                );

                ui.label(
                    egui::RichText::new(format!("({})", staged_files.len()))
                        .color(egui::Color32::from_rgb(255, 255, 255)),
                );

                if ui.button(i18n.get("ai-staged-bar-review")).clicked() {
                    // Trigger diff for the first staged file
                    if let Some(rel_path) = staged_files.first() {
                        let sandbox_path = ws.sandbox.root.join(rel_path);
                        let project_path = ws.root_path.join(rel_path);

                        if let Ok(new_content) = std::fs::read_to_string(sandbox_path) {
                            let old_content =
                                std::fs::read_to_string(project_path).unwrap_or_default();

                            ws.editor.pending_ai_diff = Some((
                                ws.root_path.join(rel_path).to_string_lossy().to_string(),
                                old_content,
                                new_content,
                            ));
                        }
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(i18n.get("btn-dismiss")).clicked() {
                        // For now, dismiss just hides it until next change or manual sync
                        // (we don't have a way to "ignore" these files permanently yet)
                    }
                });
            });
            ui.add_space(2.0);
        });
    });
}
