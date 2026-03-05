pub mod ai_bar;

pub use ai_bar::format_context_for_terminal;

use crate::app::types::{AppShared, FocusedPanel};
use crate::app::ui::terminal::StandardTerminalWindow;
use crate::app::ui::terminal::instance::{Terminal, TerminalAction};
use crate::app::ui::widgets::tab_bar::{TabBarAction, TabItem, render_compact_tab_bar};
use crate::app::ui::workspace::state::WorkspaceState;
use crate::config;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct PanelDisplayConfig {
    pub dialog_open: bool,
    pub focused: FocusedPanel,
    pub font_size: f32,
    pub is_float: bool,
    pub is_viewport: bool,
}

pub fn render_ai_panel(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    dialog_open: bool,
    i18n: &crate::i18n::I18n,
) -> bool {
    if !ws.show_right_panel || ws.ai_viewport_open {
        return false;
    }
    let mut any_clicked = false;
    let config = PanelDisplayConfig {
        dialog_open,
        focused: ws.focused_panel,
        font_size: config::EDITOR_FONT_SIZE * ws.ai_font_scale as f32 / 100.0,
        is_float: ws.claude_float,
        is_viewport: false,
    };

    if ws.claude_float {
        let mut is_open = true;
        let label = ws
            .claude_tabs
            .get(ws.claude_active_tab)
            .map(|terminal| {
                crate::app::ui::terminal::terminal_mode_label_for_workdir(
                    &terminal.working_dir,
                    &ws.sandbox.root,
                    &ws.root_path,
                )
            })
            .unwrap_or_else(|| {
                crate::app::ui::terminal::terminal_mode_label(
                    ws.sandbox_mode_enabled,
                    &ws.root_path,
                )
            });
        let float_title = format!("{} — {}", i18n.get("ai-panel-title"), label);
        let win =
            StandardTerminalWindow::new(float_title, "claude_float_win", FocusedPanel::Claude);

        let (interacted, res) = win.show(
            ctx,
            ws,
            &mut is_open,
            |ui, ws_arg| {
                // HEAD: AI Bar + Window Controls
                ui.horizontal(|ui| {
                    let combo_id = "ai_tool_combo_float";
                    ai_bar::render_ai_bar(ui, ws_arg, shared, combo_id, i18n);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button("↗")
                            .on_hover_text(i18n.get("ai-viewport-open"))
                            .clicked()
                        {
                            ws_arg.ai_viewport_open = true;
                        }
                        if ui
                            .small_button("📥")
                            .on_hover_text(i18n.get("ai-float-dock"))
                            .clicked()
                        {
                            ws_arg.claude_float = false;
                        }
                    });
                });
            },
            |ui, ws_arg, _body_h| {
                // BODY: Tabs + Terminal (without duplicating AI bar)
                ui.vertical(|ui| {
                    let items: Vec<TabItem> = ws_arg
                        .claude_tabs
                        .iter()
                        .enumerate()
                        .map(|(i, t)| TabItem {
                            label: if t.has_unread_output {
                                format!("{} \u{2022}", i + 1)
                            } else {
                                (i + 1).to_string()
                            },
                            closable: ws_arg.claude_tabs.len() > 1,
                        })
                        .collect();
                    let tab_action = render_compact_tab_bar(
                        ui,
                        &items,
                        ws_arg.claude_active_tab,
                        true,
                        &i18n.get("ai-tab-close-hover"),
                        &i18n.get("ai-tab-new-hover"),
                    );

                    ui.separator();

                    if let Some(terminal) = ws_arg.claude_tabs.get_mut(ws_arg.claude_active_tab) {
                        let terminal_action = terminal.ui(
                            ui,
                            ws_arg.focused_panel == FocusedPanel::Claude,
                            config.font_size,
                            i18n,
                        );
                        if let Some(act) = terminal_action {
                            match act {
                                TerminalAction::Clicked | TerminalAction::Hovered => {
                                    ws_arg.focused_panel = FocusedPanel::Claude;
                                }
                                TerminalAction::Navigate(path, line, col) => {
                                    let abs_path = if path.is_absolute() {
                                        path
                                    } else {
                                        ws_arg.root_path.join(path)
                                    };
                                    crate::app::ui::workspace::state::open_file_in_ws(
                                        ws_arg, abs_path,
                                    );
                                    ws_arg.editor.jump_to_location(line, col);
                                    ws_arg.focused_panel = FocusedPanel::Editor;
                                }
                            }
                        }
                    }

                    if let Some(action) = tab_action {
                        apply_tab_action(ws_arg, action, ui.ctx());
                    }
                });
                None::<bool>
            },
            |ui, _ws| {
                // FOOTER: Small dock button for consistency
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .small_button("📥")
                        .on_hover_text(i18n.get("ai-float-dock"))
                        .clicked()
                    {
                        return Some(true); // Signal to dock
                    }
                    None
                })
                .inner
            },
        );

        if let Some(true) = res {
            ws.claude_float = false;
        }
        any_clicked |= interacted;

        if !is_open {
            ws.show_right_panel = false;
        }
    } else {
        egui::SidePanel::right("claude_panel")
            .default_width(config::AI_PANEL_DEFAULT_WIDTH)
            .width_range(config::AI_PANEL_MIN_WIDTH..=config::AI_PANEL_MAX_WIDTH)
            .resizable(true)
            .show(ctx, |ui| {
                any_clicked |= render_ai_panel_content(ui, ws, shared, i18n, &config);
            });
    }
    any_clicked
}

pub fn render_ai_panel_content(
    ui: &mut egui::Ui,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &crate::i18n::I18n,
    config: &PanelDisplayConfig,
) -> bool {
    let mut any_clicked = false;

    // Header with AI Bar and window controls
    ui.horizontal(|ui| {
        let combo_id = if config.is_viewport {
            "ai_tool_combo_viewport"
        } else if config.is_float {
            "ai_tool_combo_float"
        } else {
            "ai_tool_combo_docked"
        };
        ai_bar::render_ai_bar(ui, ws, shared, combo_id, i18n);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if config.is_viewport {
                if ui
                    .small_button("📥")
                    .on_hover_text(i18n.get("ai-float-dock"))
                    .clicked()
                {
                    ws.ai_viewport_open = false;
                    ws.show_right_panel = true;
                    ws.claude_float = false;
                }
            } else {
                if ui
                    .small_button("↗")
                    .on_hover_text(i18n.get("ai-viewport-open"))
                    .clicked()
                {
                    ws.ai_viewport_open = true;
                }
                if config.is_float {
                    if ui
                        .small_button("📥")
                        .on_hover_text(i18n.get("ai-float-dock"))
                        .clicked()
                    {
                        ws.claude_float = false;
                    }
                } else if ui
                    .small_button("🗖")
                    .on_hover_text(i18n.get("ai-float-undock"))
                    .clicked()
                {
                    ws.claude_float = true;
                }
            }
        });
    });

    // Tabs
    let items: Vec<TabItem> = ws
        .claude_tabs
        .iter()
        .enumerate()
        .map(|(i, t)| TabItem {
            label: if t.has_unread_output {
                format!("{} \u{2022}", i + 1)
            } else {
                (i + 1).to_string()
            },
            closable: ws.claude_tabs.len() > 1,
        })
        .collect();
    let tab_action = render_compact_tab_bar(
        ui,
        &items,
        ws.claude_active_tab,
        true,
        &i18n.get("ai-tab-close-hover"),
        &i18n.get("ai-tab-new-hover"),
    );

    ui.separator();

    // Active Terminal
    if let Some(terminal) = ws.claude_tabs.get_mut(ws.claude_active_tab) {
        let terminal_action = terminal.ui(
            ui,
            config.focused == FocusedPanel::Claude && !config.dialog_open,
            config.font_size,
            i18n,
        );
        match terminal_action {
            Some(TerminalAction::Clicked) => {
                ws.focused_panel = FocusedPanel::Claude;
                any_clicked = true;
            }
            Some(TerminalAction::Hovered) => {
                if !config.dialog_open {
                    ws.focused_panel = FocusedPanel::Claude;
                }
                any_clicked = true;
            }
            Some(TerminalAction::Navigate(path, line, col)) => {
                let abs_path = if path.is_absolute() {
                    path
                } else {
                    ws.root_path.join(path)
                };
                crate::app::ui::workspace::state::open_file_in_ws(ws, abs_path);
                ws.editor.jump_to_location(line, col);
                ws.focused_panel = FocusedPanel::Editor;
                any_clicked = true;
            }
            None => {}
        }
    }

    if let Some(action) = tab_action {
        apply_tab_action(ws, action, ui.ctx());
    }

    any_clicked
}

fn apply_tab_action(ws: &mut WorkspaceState, action: TabBarAction, ctx: &egui::Context) {
    match action {
        TabBarAction::Switch(i) => {
            ws.claude_active_tab = i;
        }
        TabBarAction::Close(idx) => {
            if let Some(terminal) = ws.claude_tabs.get(idx) {
                if terminal.is_exited() {
                    #[cfg(unix)]
                    terminal.kill_process_group();
                    ws.claude_tabs.remove(idx);
                    if ws.claude_active_tab >= ws.claude_tabs.len() {
                        ws.claude_active_tab = ws.claude_tabs.len().saturating_sub(1);
                    }
                } else {
                    ws.terminal_close_requested = Some(idx);
                }
            }
        }
        TabBarAction::New => {
            let id = ws.next_claude_tab_id;
            ws.next_claude_tab_id += 1;
            let root = crate::app::ui::terminal::terminal_working_dir(
                ws.sandbox_mode_enabled,
                &ws.sandbox.root,
                &ws.root_path,
            )
            .to_path_buf();
            ws.claude_tabs.push(Terminal::new(id, ctx, &root, None));
            ws.claude_active_tab = ws.claude_tabs.len() - 1;
        }
    }
}
