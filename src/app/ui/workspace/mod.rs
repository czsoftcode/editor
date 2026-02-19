mod menubar;
mod modal_dialogs;
pub(crate) mod state;

// Re-exporty pro vnější volající (panels.rs, ai_panel.rs, background.rs, app/mod.rs, …)
pub(crate) use state::{
    FilePicker, SearchResult, SecondaryWorkspace, WorkspaceState,
    init_workspace, open_and_jump, open_file_in_ws, ws_to_panel_state,
};
// Viditelné pro sourozence v ui/ (background.rs, ai_panel.rs)
pub(super) use state::{spawn_ai_tool_check, spawn_file_index_scan};

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use eframe::egui;

use super::super::build_runner::run_build_check;
use super::super::types::{AppShared, FocusedPanel, Toast};
use super::background::{fetch_git_status, process_background_events};
use super::panels::{render_left_panel, render_toasts};
use super::ai_panel::render_ai_panel;
use super::search_picker::{render_file_picker, render_project_search_dialog};
use menubar::{render_menu_bar, process_menu_actions};
use modal_dialogs::render_dialogs;
use crate::config;

// ---------------------------------------------------------------------------
// render_workspace — orchestrátor vykreslení jednoho pracovního prostoru
// Vrací Some(path) pokud má být workspace reinicializován s novou cestou.
// ---------------------------------------------------------------------------

pub(crate) fn render_workspace(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
) -> Option<PathBuf> {
    // Extrahujeme i18n z shared (krátkodobý lock, poté pracujeme jen s Arc)
    let i18n_arc = { std::sync::Arc::clone(&shared.lock().unwrap().i18n) };
    let i18n = &*i18n_arc;

    // Lazy init terminálů
    if ws.claude_tabs.is_empty() {
        let root = ws.root_path.clone();
        let id = ws.next_claude_tab_id;
        ws.next_claude_tab_id += 1;
        ws.claude_tabs.push(super::terminal::Terminal::new(id, ctx, &root, None));
    }
    if ws.build_terminal.is_none() {
        ws.build_terminal = Some(super::terminal::Terminal::new(1, ctx, &ws.root_path, None));
    }

    // Události na pozadí (watcher, build, autosave)
    process_background_events(ws, i18n);

    // Pravidelné překreslování pro autosave a watcher
    ctx.request_repaint_after(std::time::Duration::from_millis(
        config::REPAINT_INTERVAL_MS,
    ));

    // Klávesové zkratky
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
        if let Some(err) = ws.editor.save(i18n) {
            ws.toasts.push(Toast::error(err));
        }
        // Po uložení okamžitě aktualizujeme git status
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
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::R)) {
        if let Some(t) = &mut ws.build_terminal {
            t.send_command("cargo run 2>&1");
        }
    }
    // Ctrl+P — fuzzy file picker
    if ctx.input(|i| i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::P)) {
        if ws.file_picker.is_none() {
            if ws.file_index_rx.is_none() {
                ws.file_index_rx = Some(spawn_file_index_scan(ws.root_path.clone()));
            }
            let files = ws.file_index_cache.clone();
            ws.file_picker = Some(FilePicker::new(files));
        } else {
            ws.file_picker = None;
        }
    }
    // Ctrl+Shift+F — hledání napříč projektem
    if ctx.input(|i| i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::F)) {
        ws.project_search.show_input = true;
        ws.project_search.focus_requested = true;
    }

    // Menu bar + zpracování akcí
    let actions = render_menu_bar(ctx, ws, shared, i18n);
    let open_here_path = process_menu_actions(ws, shared, actions, i18n);

    // Modální dialogy
    render_dialogs(ctx, ws, shared, i18n);

    // File picker (Ctrl+P)
    if let Some(path) = render_file_picker(ctx, ws, i18n) {
        open_file_in_ws(ws, path);
    }

    // Hledání napříč projektem
    render_project_search_dialog(ctx, ws, i18n);

    // Status bar (musí být před SidePanel)
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(config::STATUS_BAR_HEIGHT)
        .show(ctx, |ui| {
            ws.editor.status_bar(ui, ws.git_branch.as_deref(), i18n);
        });

    let dialog_open = ws.file_tree.has_open_dialog();

    // Panely (pořadí: pravý, levý, centrální)
    let ai_clicked = render_ai_panel(ctx, ws, dialog_open, i18n);
    let left_clicked = render_left_panel(ctx, ws, dialog_open, i18n);

    // Zapamatovat aktivní záložku před renderem — kvůli detekci přepnutí tabu
    let prev_active_path = ws.editor.active_path().cloned();

    egui::CentralPanel::default().show(ctx, |ui| {
        if ws.editor.ui(ui, dialog_open, i18n) {
            ws.focused_panel = FocusedPanel::Editor;
        }
    });

    // Pokud se přepnula záložka, přepnout FileWatcher na adresář nové záložky.
    let new_active_path = ws.editor.active_path().cloned();
    if new_active_path != prev_active_path {
        if let Some(path) = &new_active_path {
            if let Some(parent) = path.parent() {
                ws.watcher.watch(parent);
            }
        }
    }

    // Focus follows mouse — vrátit fokus editoru pokud terminál nebyl aktivně kliknut
    if !ai_clicked && !left_clicked {
        let in_terminal =
            ws.focused_panel == FocusedPanel::Claude || ws.focused_panel == FocusedPanel::Build;
        if in_terminal {
            ws.focused_panel = FocusedPanel::Editor;
            ws.editor.request_editor_focus();
        }
    }

    // Toast notifikace
    render_toasts(ctx, ws);

    open_here_path
}
