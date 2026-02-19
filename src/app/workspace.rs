use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

/// Výsledek asynchronního výběru složky.
/// bool = true → otevřít v novém okně; false → nahradit aktuální workspace.
type FolderPickResult = (Option<PathBuf>, bool);

use eframe::egui;

use super::build_runner::{BuildError, run_build_check};
use super::dialogs::{WizardState, show_project_wizard};
use super::modules::editor::Editor;
use super::modules::file_tree::FileTree;
use super::modules::terminal::Terminal;
use super::types::{AiTool, AppAction, AppShared, FocusedPanel, PersistentState, Toast, default_wizard_path};
use crate::config;
use crate::watcher::{FileEvent, FileWatcher, FsChange, ProjectWatcher};

// ---------------------------------------------------------------------------
// WorkspaceState — stav jednoho pracovního prostoru (okna projektu)
// ---------------------------------------------------------------------------

pub(crate) struct WorkspaceState {
    pub file_tree: FileTree,
    pub editor: Editor,
    pub watcher: FileWatcher,
    pub project_watcher: ProjectWatcher,
    pub claude_terminal: Option<Terminal>,
    pub build_terminal: Option<Terminal>,
    pub focused_panel: FocusedPanel,
    pub root_path: PathBuf,
    pub show_left_panel: bool,
    pub show_right_panel: bool,
    pub show_build_terminal: bool,
    pub show_about: bool,
    pub show_settings: bool,
    pub ai_font_scale: u32,
    pub build_errors: Vec<BuildError>,
    pub build_error_rx: Option<mpsc::Receiver<Vec<BuildError>>>,
    pub claude_tool: AiTool,
    pub claude_float: bool,
    // Wizard nového projektu (pro toto okno)
    pub show_new_project: bool,
    pub wizard: WizardState,
    pub toasts: Vec<Toast>,
    /// Kanál pro výsledek asynchronního file dialogu (výběr složky).
    pub folder_pick_rx: Option<mpsc::Receiver<FolderPickResult>>,
}

// ---------------------------------------------------------------------------
// SecondaryWorkspace — sekundární viewport (jeden projekt v novém okně)
// ---------------------------------------------------------------------------

pub(crate) struct SecondaryWorkspace {
    pub viewport_id: egui::ViewportId,
    pub state: Arc<Mutex<WorkspaceState>>,
}

// ---------------------------------------------------------------------------
// Pomocné funkce
// ---------------------------------------------------------------------------

pub(crate) fn ws_to_panel_state(ws: &WorkspaceState) -> PersistentState {
    PersistentState {
        show_left_panel: ws.show_left_panel,
        show_right_panel: ws.show_right_panel,
        show_build_terminal: ws.show_build_terminal,
        claude_float: ws.claude_float,
        ai_font_scale: ws.ai_font_scale,
    }
}

pub(crate) fn init_workspace(root_path: PathBuf, panel_state: &PersistentState) -> WorkspaceState {
    let mut file_tree = FileTree::new();
    file_tree.load(&root_path);
    let project_watcher = ProjectWatcher::new(&root_path);

    WorkspaceState {
        file_tree,
        editor: Editor::new(),
        watcher: FileWatcher::new(),
        project_watcher,
        claude_terminal: None,
        build_terminal: None,
        focused_panel: FocusedPanel::Editor,
        root_path,
        show_left_panel: panel_state.show_left_panel,
        show_right_panel: panel_state.show_right_panel,
        show_build_terminal: panel_state.show_build_terminal,
        show_about: false,
        show_settings: false,
        ai_font_scale: panel_state.ai_font_scale,
        build_errors: Vec::new(),
        build_error_rx: None,
        claude_tool: AiTool::ClaudeCode,
        claude_float: panel_state.claude_float,
        show_new_project: false,
        wizard: WizardState { path: default_wizard_path(), ..WizardState::default() },
        toasts: Vec::new(),
        folder_pick_rx: None,
    }
}

pub(crate) fn open_file_in_ws(ws: &mut WorkspaceState, path: PathBuf) {
    ws.editor.open_file(&path);
    if let Some(parent) = path.parent() {
        ws.watcher.watch(parent);
    }
}

// ---------------------------------------------------------------------------
// Pomocné datové typy pro render_workspace
// ---------------------------------------------------------------------------

/// Akce vzniklé z menu baru — zpracovávají se po vykreslení menu.
#[derive(Default)]
struct MenuActions {
    open_folder: bool,
    save: bool,
    close_file: bool,
    quit: bool,
    new_project: bool,
    open_project: bool,
    open_recent: Option<PathBuf>,
    toggle_left: bool,
    toggle_right: bool,
    toggle_build: bool,
    toggle_float: bool,
    about: bool,
    settings: bool,
}

// ---------------------------------------------------------------------------
// Pomocné funkce pro render_workspace
// ---------------------------------------------------------------------------

/// Zpracuje události z watcherů, build výsledky a autosave.
fn process_background_events(ws: &mut WorkspaceState) {
    for event in ws.watcher.try_recv() {
        match event {
            FileEvent::Changed(changed_path) => {
                if let Some(editor_path) = ws.editor.active_path() {
                    if let (Ok(a), Ok(b)) = (changed_path.canonicalize(), editor_path.canonicalize()) {
                        if a == b && !ws.editor.is_modified() {
                            ws.editor.reload_from_disk();
                        }
                    }
                }
            }
            FileEvent::Removed(removed_path) => {
                ws.editor.notify_file_deleted(&removed_path);
                let name = removed_path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| removed_path.to_string_lossy().into_owned());
                ws.toasts.push(Toast::error(format!("Soubor byl smazán z disku: {name}")));
            }
        }
    }

    let fs_changes = ws.project_watcher.poll();
    if !fs_changes.is_empty() {
        let mut need_reload = false;
        let mut open_file: Option<PathBuf> = None;
        for change in &fs_changes {
            match change {
                FsChange::Created(path) => {
                    need_reload = true;
                    if path.is_file() { open_file = Some(path.clone()); }
                }
                FsChange::Removed(path) => {
                    need_reload = true;
                    ws.editor.close_tabs_for_path(path);
                }
                FsChange::Modified => { need_reload = true; }
            }
        }
        if need_reload {
            if let Some(ref path) = open_file {
                ws.file_tree.request_reload_and_expand(path);
            } else {
                ws.file_tree.request_reload();
            }
        }
        if let Some(path) = open_file {
            open_file_in_ws(ws, path);
        }
    }

    if let Some(rx) = &ws.build_error_rx {
        if let Ok(errors) = rx.try_recv() {
            ws.build_errors = errors;
            ws.build_error_rx = None;
        }
    }

    ws.editor.try_autosave();
}

/// Vykreslí menu bar a vrátí zaznamenané akce.
fn render_menu_bar(
    ctx: &egui::Context,
    ws: &WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
) -> MenuActions {
    let mut actions = MenuActions::default();
    let recent_snapshot = shared.lock().unwrap().recent_projects.clone();

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("Soubor", |ui| {
                if ui.button("Otevřít složku…").clicked() {
                    actions.open_folder = true;
                    ui.close_menu();
                }
                if ui.add(egui::Button::new("Uložit").shortcut_text("Ctrl+S")).clicked() {
                    actions.save = true;
                    ui.close_menu();
                }
                if ui.add(egui::Button::new("Zavřít soubor").shortcut_text("Ctrl+W")).clicked() {
                    actions.close_file = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Ukončit").clicked() {
                    actions.quit = true;
                    ui.close_menu();
                }
            });

            ui.menu_button("Projekt", |ui| {
                if ui.button("Otevřít projekt…").clicked() {
                    actions.open_project = true;
                    ui.close_menu();
                }
                if ui.button("Nový projekt…").clicked() {
                    actions.new_project = true;
                    ui.close_menu();
                }
                if !recent_snapshot.is_empty() {
                    ui.separator();
                    ui.menu_button("Nedávné projekty", |ui| {
                        for path in &recent_snapshot {
                            let name = path.file_name()
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_else(|| path.to_string_lossy().into_owned());
                            if ui.button(&name).on_hover_text(path.to_string_lossy()).clicked() {
                                actions.open_recent = Some(path.clone());
                                ui.close_menu();
                            }
                        }
                    });
                }
            });

            ui.menu_button("Upravit", |ui| {
                ui.add_enabled(false, egui::Button::new("Kopírovat").shortcut_text("Ctrl+C"));
                ui.add_enabled(false, egui::Button::new("Vložit").shortcut_text("Ctrl+V"));
                ui.add_enabled(false, egui::Button::new("Vybrat vše").shortcut_text("Ctrl+A"));
                ui.separator();
                if ui.add(egui::Button::new("Hledat…").shortcut_text("Ctrl+F")).clicked() {
                    ui.close_menu();
                }
                if ui.add(egui::Button::new("Hledat a nahradit…").shortcut_text("Ctrl+H")).clicked() {
                    ui.close_menu();
                }
            });

            ui.menu_button("Zobrazit", |ui| {
                let left_label = if ws.show_left_panel { "✓ Soubory" } else { "  Soubory" };
                if ui.button(left_label).clicked() { actions.toggle_left = true; ui.close_menu(); }
                let build_label = if ws.show_build_terminal { "✓ Build terminál" } else { "  Build terminál" };
                if ui.button(build_label).clicked() { actions.toggle_build = true; ui.close_menu(); }
                let right_label = if ws.show_right_panel { "✓ AI terminál" } else { "  AI terminál" };
                if ui.button(right_label).clicked() { actions.toggle_right = true; ui.close_menu(); }
                let float_label = if ws.claude_float { "✓ Plovoucí AI terminál" } else { "  Plovoucí AI terminál" };
                if ui.button(float_label).clicked() { actions.toggle_float = true; ui.close_menu(); }
            });

            ui.menu_button("Nápověda", |ui| {
                if ui.button("Nastavení…").clicked() { actions.settings = true; ui.close_menu(); }
                ui.separator();
                if ui.button("O aplikaci").clicked() { actions.about = true; ui.close_menu(); }
            });
        });
    });

    actions
}

/// Aplikuje menu akce na stav workspace. Vrací cestu pro reinicializaci (pokud byla vybrána složka).
fn process_menu_actions(
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    actions: MenuActions,
) -> Option<PathBuf> {
    if actions.quit { shared.lock().unwrap().actions.push(AppAction::QuitAll); }
    if actions.save { ws.editor.save(); }
    if actions.close_file { ws.editor.clear(); }
    if actions.toggle_left { ws.show_left_panel = !ws.show_left_panel; }
    if actions.toggle_right { ws.show_right_panel = !ws.show_right_panel; }
    if actions.toggle_float { ws.claude_float = !ws.claude_float; }
    if actions.toggle_build { ws.show_build_terminal = !ws.show_build_terminal; }
    if actions.about { ws.show_about = true; }
    if actions.settings { ws.show_settings = true; }
    if actions.new_project { ws.show_new_project = true; }

    if let Some(path) = actions.open_recent {
        if path.is_dir() {
            let mut sh = shared.lock().unwrap();
            sh.actions.push(AppAction::AddRecent(path.clone()));
            sh.actions.push(AppAction::OpenInNewWindow(path));
        }
    }

    // Výsledek předchozího async file dialogu
    let mut open_here_path: Option<PathBuf> = None;
    if let Some(rx) = &ws.folder_pick_rx {
        if let Ok((maybe_path, in_new_window)) = rx.try_recv() {
            ws.folder_pick_rx = None;
            if let Some(dir) = maybe_path {
                let path = dir.canonicalize().unwrap_or(dir);
                if in_new_window {
                    let mut sh = shared.lock().unwrap();
                    sh.actions.push(AppAction::AddRecent(path.clone()));
                    sh.actions.push(AppAction::OpenInNewWindow(path));
                } else {
                    open_here_path = Some(path);
                }
            }
        }
    }

    // Spuštění async file dialogu (neblokuje UI vlákno)
    if actions.open_project && ws.folder_pick_rx.is_none() {
        let projects_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")).join("MyProject");
        let _ = std::fs::create_dir_all(&projects_dir);
        let (tx, rx) = mpsc::channel();
        ws.folder_pick_rx = Some(rx);
        std::thread::spawn(move || {
            let _ = tx.send((rfd::FileDialog::new().set_directory(&projects_dir).pick_folder(), true));
        });
    }
    if actions.open_folder && ws.folder_pick_rx.is_none() {
        let start_dir = ws.root_path.clone();
        let (tx, rx) = mpsc::channel();
        ws.folder_pick_rx = Some(rx);
        std::thread::spawn(move || {
            let _ = tx.send((rfd::FileDialog::new().set_directory(&start_dir).pick_folder(), false));
        });
    }

    open_here_path
}

/// Vykreslí modální dialogy (O aplikaci, Nastavení, Nový projekt).
fn render_dialogs(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
) {
    if ws.show_about {
        let modal = egui::Modal::new(egui::Id::new("about_modal"));
        modal.show(ctx, |ui| {
            ui.heading("Rust Editor");
            ui.add_space(8.0);
            ui.label(format!("Verze: {}", env!("BUILD_VERSION")));
            ui.add_space(8.0);
            ui.label("Jednoduchý textový editor napsaný v Rustu s eframe/egui.");
            ui.add_space(12.0);
            if ui.button("Zavřít").clicked() { ws.show_about = false; }
        });
    }

    if ws.show_settings {
        let modal = egui::Modal::new(egui::Id::new("settings_modal"));
        modal.show(ctx, |ui| {
            ui.heading("Nastavení");
            ui.add_space(12.0);
            ui.strong("AI terminál");
            ui.add_space(4.0);
            ui.label("Velikost fontu:");
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                for &scale in &[100u32, 125, 150, 200] {
                    ui.radio_value(&mut ws.ai_font_scale, scale, format!("{}%", scale));
                }
            });
            ui.add_space(16.0);
            if ui.button("Zavřít").clicked() { ws.show_settings = false; }
        });
    }

    if ws.show_new_project {
        show_project_wizard(ctx, &mut ws.wizard, &mut ws.show_new_project, "ws_new_project_modal", shared, |path, sh| {
            let mut sh = sh.lock().unwrap();
            sh.actions.push(AppAction::AddRecent(path.clone()));
            sh.actions.push(AppAction::OpenInNewWindow(path));
        });
    }
}

/// Vykreslí pravý panel s AI terminálem. Vrací true pokud bylo kliknuto do terminálu.
fn render_ai_panel(ctx: &egui::Context, ws: &mut WorkspaceState, dialog_open: bool) -> bool {
    if !ws.show_right_panel {
        return false;
    }
    let mut any_clicked = false;
    let focused = ws.focused_panel;
    let font_size = config::EDITOR_FONT_SIZE * ws.ai_font_scale as f32 / 100.0;

    if ws.claude_float {
        let mut is_open = true;
        egui::Window::new("AI terminál")
            .id(egui::Id::new("claude_float_win"))
            .default_size([520.0, 420.0])
            .min_size([300.0, 200.0])
            .resizable(true)
            .collapsible(false)
            .open(&mut is_open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let prev_tool = ws.claude_tool;
                    ui.radio_value(&mut ws.claude_tool, AiTool::ClaudeCode, "Claude Code");
                    ui.radio_value(&mut ws.claude_tool, AiTool::Codex, "Codex");
                    if ws.claude_tool != prev_tool {
                        if let Some(terminal) = &mut ws.claude_terminal {
                            terminal.restart_with_command(ui.ctx(), Some(ws.claude_tool.command()));
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("⊟").on_hover_text("Přikovat do panelu").clicked() {
                            ws.claude_float = false;
                        }
                    });
                });
                ui.separator();
                if !dialog_open {
                    if let Some(terminal) = &mut ws.claude_terminal {
                        if terminal.ui(ui, focused == FocusedPanel::Claude, font_size) {
                            ws.focused_panel = FocusedPanel::Claude;
                            any_clicked = true;
                        }
                    }
                }
            });
        if !is_open { ws.show_right_panel = false; }
    } else {
        egui::SidePanel::right("claude_panel")
            .default_width(config::AI_PANEL_DEFAULT_WIDTH)
            .width_range(config::AI_PANEL_MIN_WIDTH..=config::AI_PANEL_MAX_WIDTH)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("AI terminál");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("⧉").on_hover_text("Odpojit do plovoucího okna").clicked() {
                            ws.claude_float = true;
                        }
                    });
                });
                let prev_tool = ws.claude_tool;
                ui.horizontal(|ui| {
                    ui.radio_value(&mut ws.claude_tool, AiTool::ClaudeCode, "Claude Code");
                    ui.radio_value(&mut ws.claude_tool, AiTool::Codex, "Codex");
                });
                if ws.claude_tool != prev_tool {
                    if let Some(terminal) = &mut ws.claude_terminal {
                        terminal.restart_with_command(ui.ctx(), Some(ws.claude_tool.command()));
                    }
                }
                ui.separator();
                if !dialog_open {
                    if let Some(terminal) = &mut ws.claude_terminal {
                        if terminal.ui(ui, focused == FocusedPanel::Claude, font_size) {
                            ws.focused_panel = FocusedPanel::Claude;
                            any_clicked = true;
                        }
                    }
                }
            });
    }
    any_clicked
}

/// Vykreslí levý panel (strom souborů + build terminál). Vrací true pokud bylo kliknuto do terminálu.
fn render_left_panel(ctx: &egui::Context, ws: &mut WorkspaceState, dialog_open: bool) -> bool {
    if !ws.show_left_panel {
        return false;
    }
    let mut any_clicked = false;
    let focused = ws.focused_panel;

    egui::SidePanel::left("left_panel")
        .default_width(config::LEFT_PANEL_DEFAULT_WIDTH)
        .width_range(config::LEFT_PANEL_MIN_WIDTH..=config::LEFT_PANEL_MAX_WIDTH)
        .resizable(true)
        .show(ctx, |ui| {
            let total_height = ui.available_height();
            let tree_height = if ws.show_build_terminal {
                (total_height * 0.55).max(100.0)
            } else {
                total_height
            };

            egui::Frame::NONE.show(ui, |ui| {
                ui.set_max_height(tree_height);
                ui.heading("Soubory");
                ui.separator();
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let result = ws.file_tree.ui(ui);
                        if let Some(err) = ws.file_tree.take_error() {
                            ws.toasts.push(Toast::error(err));
                        }
                        if let Some(path) = result.selected { open_file_in_ws(ws, path); }
                        if let Some(path) = result.created_file { open_file_in_ws(ws, path); }
                        if let Some(deleted) = result.deleted {
                            ws.editor.close_tabs_for_path(&deleted);
                        }
                    });
            });

            if ws.show_build_terminal {
                ui.separator();
                render_build_panel(ui, ws, dialog_open, focused, &mut any_clicked);
            }
        });
    any_clicked
}

/// Vykreslí build terminál a error list uvnitř levého panelu.
fn render_build_panel(
    ui: &mut egui::Ui,
    ws: &mut WorkspaceState,
    dialog_open: bool,
    focused: FocusedPanel,
    any_clicked: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.strong("Build");
        ui.separator();
        if ui.small_button("\u{25B6} Build").clicked() {
            if let Some(t) = &mut ws.build_terminal { t.send_command("cargo build 2>&1"); }
            ws.build_error_rx = Some(run_build_check(ws.root_path.clone()));
            ws.build_errors.clear();
        }
        if ui.small_button("\u{25B6} Run").clicked() {
            if let Some(t) = &mut ws.build_terminal { t.send_command("cargo run 2>&1"); }
        }
        if ui.small_button("\u{25B6} Test").clicked() {
            if let Some(t) = &mut ws.build_terminal { t.send_command("cargo test 2>&1"); }
        }
        if ui.small_button("\u{2716} Clean").clicked() {
            if let Some(t) = &mut ws.build_terminal { t.send_command("cargo clean"); }
        }
    });
    ui.separator();

    if !dialog_open {
        if let Some(terminal) = &mut ws.build_terminal {
            if terminal.ui(ui, focused == FocusedPanel::Build, config::EDITOR_FONT_SIZE) {
                ws.focused_panel = FocusedPanel::Build;
                *any_clicked = true;
            }
        }
    }

    if !ws.build_errors.is_empty() {
        ui.separator();
        ui.label(egui::RichText::new(format!("Chyby ({})", ws.build_errors.len())).strong().size(12.0));
        let mut open_error_file: Option<(PathBuf, usize)> = None;
        egui::ScrollArea::vertical()
            .id_salt("build_errors_scroll")
            .max_height(config::BUILD_ERROR_LIST_MAX_HEIGHT)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for error in &ws.build_errors {
                    let color = if error.is_warning {
                        egui::Color32::from_rgb(230, 180, 60)
                    } else {
                        egui::Color32::from_rgb(230, 80, 80)
                    };
                    let text = format!("{}:{}  {}", error.file.display(), error.line, error.message);
                    let r = ui.add(
                        egui::Label::new(egui::RichText::new(&text).size(11.0).color(color))
                            .sense(egui::Sense::click()),
                    );
                    if r.clicked() {
                        open_error_file = Some((ws.root_path.join(&error.file), error.line));
                    }
                }
            });
        if let Some((path, _line)) = open_error_file {
            open_file_in_ws(ws, path);
        }
    }
}

/// Vykreslí toast notifikace v pravém dolním rohu. Odstraní vypršelé toasty.
fn render_toasts(ctx: &egui::Context, ws: &mut WorkspaceState) {
    ws.toasts.retain(|t| !t.is_expired());
    if ws.toasts.is_empty() { return; }

    let screen = ctx.screen_rect();
    let toast_w = 340.0_f32;
    let toast_h = 40.0_f32;
    let padding = 12.0_f32;
    let start_y = screen.max.y - padding - (toast_h + padding) * ws.toasts.len() as f32;

    egui::Area::new(egui::Id::new("toasts_area"))
        .fixed_pos(egui::pos2(screen.max.x - toast_w - padding, start_y))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            for toast in &ws.toasts {
                let (bg, fg) = if toast.is_error {
                    (egui::Color32::from_rgb(90, 30, 30), egui::Color32::from_rgb(255, 180, 170))
                } else {
                    (egui::Color32::from_rgb(30, 60, 45), egui::Color32::from_rgb(160, 230, 180))
                };
                egui::Frame::new()
                    .fill(bg)
                    .corner_radius(6.0)
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.set_min_width(toast_w);
                        ui.label(egui::RichText::new(&toast.message).color(fg).size(config::UI_FONT_SIZE));
                    });
                ui.add_space(padding);
            }
        });
}

// ---------------------------------------------------------------------------
// render_workspace — orchestrátor vykreslení jednoho pracovního prostoru
// Vrací Some(path) pokud má být workspace reinicializován s novou cestou.
// ---------------------------------------------------------------------------

pub(crate) fn render_workspace(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
) -> Option<PathBuf> {
    // Lazy init terminálů
    if ws.claude_terminal.is_none() {
        ws.claude_terminal = Some(Terminal::new(0, ctx, &ws.root_path, Some(ws.claude_tool.command())));
    }
    if ws.build_terminal.is_none() {
        ws.build_terminal = Some(Terminal::new(1, ctx, &ws.root_path, None));
    }

    // Události na pozadí (watcher, build, autosave)
    process_background_events(ws);

    // Pravidelné překreslování pro autosave a watcher
    ctx.request_repaint_after(std::time::Duration::from_millis(config::REPAINT_INTERVAL_MS));

    // Klávesové zkratky
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) { ws.editor.save(); }
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::W)) { ws.editor.clear(); }

    // Menu bar + zpracování akcí
    let actions = render_menu_bar(ctx, ws, shared);
    let open_here_path = process_menu_actions(ws, shared, actions);

    // Modální dialogy
    render_dialogs(ctx, ws, shared);

    // Status bar (musí být před SidePanel)
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(config::STATUS_BAR_HEIGHT)
        .show(ctx, |ui| { ws.editor.status_bar(ui); });

    let dialog_open = ws.file_tree.has_open_dialog();

    // Panely (pořadí: pravý, levý, centrální)
    let ai_clicked = render_ai_panel(ctx, ws, dialog_open);
    let left_clicked = render_left_panel(ctx, ws, dialog_open);

    egui::CentralPanel::default().show(ctx, |ui| {
        if ws.editor.ui(ui, dialog_open) {
            ws.focused_panel = FocusedPanel::Editor;
        }
    });

    // Focus follows mouse — vrátit fokus editoru pokud terminál nebyl aktivně kliknut
    if !ai_clicked && !left_clicked {
        let in_terminal = ws.focused_panel == FocusedPanel::Claude
            || ws.focused_panel == FocusedPanel::Build;
        if in_terminal { ws.focused_panel = FocusedPanel::Editor; }
    }

    // Toast notifikace
    render_toasts(ctx, ws);

    open_here_path
}
