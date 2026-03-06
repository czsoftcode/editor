use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use eframe::egui;

use super::super::types::{AppShared, FocusedPanel};
use super::terminal::Terminal;
use super::widgets::tab_bar::{TabBarAction, TabItem, render_compact_tab_bar};
use super::workspace::{WorkspaceState, spawn_ai_tool_check};
use crate::app::ai::{AiContextPayload, AiManager};
use crate::app::registry::Agent;
use crate::config;

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

pub(crate) fn format_context_for_terminal(ctx: &AiContextPayload) -> String {
    let mut s = String::new();
    s.push_str("Context info (paths are relative to current working directory):\n");

    if !ctx.open_files.is_empty() {
        s.push_str("Open files:\n");
        for file in &ctx.open_files {
            let active = if file.is_active { " (active)" } else { "" };
            s.push_str(&format!("- {}{}\n", file.path, active));
        }
    }

    if !ctx.build_errors.is_empty() {
        s.push_str("\nBuild errors:\n");
        for err in &ctx.build_errors {
            let level = if err.is_warning { "Warning" } else { "Error" };
            s.push_str(&format!(
                "[{}] {}:{}: {}\n",
                level, err.file, err.line, err.message
            ));
        }
    } else {
        s.push_str("\nBuild is clean.\n");
    }
    s
}

fn ai_tool_is_available(available: &HashMap<String, bool>, id: &str) -> bool {
    available.get(id).copied().unwrap_or(false)
}

fn ai_tool_status_label(
    agent: &Agent,
    available: &HashMap<String, bool>,
    checking: bool,
    i18n: &crate::i18n::I18n,
) -> String {
    let mut args = fluent_bundle::FluentArgs::new();
    args.set("tool", agent.label.clone());
    if checking {
        i18n.get_args("ai-tool-status-checking", &args)
    } else if ai_tool_is_available(available, &agent.id) {
        i18n.get_args("ai-tool-status-available", &args)
    } else {
        i18n.get_args("ai-tool-status-missing", &args)
    }
}

fn render_ai_tool_picker(
    ui: &mut egui::Ui,
    id_salt: &'static str,
    selected_id: &mut String,
    available: &HashMap<String, bool>,
    agents: &[Agent],
    checking: bool,
    i18n: &crate::i18n::I18n,
) {
    let selected_agent = agents.iter().find(|a| a.id == *selected_id);
    let label = if let Some(agent) = selected_agent {
        ai_tool_status_label(agent, available, checking, i18n)
    } else {
        i18n.get("plugins-unknown-agent")
    };

    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(label)
        .width(190.0)
        .show_ui(ui, |ui| {
            for agent in agents {
                ui.selectable_value(
                    selected_id,
                    agent.id.clone(),
                    ai_tool_status_label(agent, available, checking, i18n),
                );
            }
        });
}

fn render_ai_tool_controls(
    ui: &mut egui::Ui,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    combo_id: &'static str,
    i18n: &crate::i18n::I18n,
) {
    let checking = ws.ai_tool_check_rx.is_some();
    let agents = {
        let sh = shared.lock().expect("lock");
        sh.registry.agents.get_all().to_vec()
    };

    ui.label(i18n.get("ai-label-assistant"));
    render_ai_tool_picker(
        ui,
        combo_id,
        &mut ws.selected_agent_id,
        &ws.ai_tool_available,
        &agents,
        checking,
        i18n,
    );

    let hover_reverify = if checking {
        i18n.get("ai-hover-checking")
    } else {
        i18n.get("ai-hover-reverify")
    };
    if ui.small_button("↻").on_hover_text(hover_reverify).clicked() && ws.ai_tool_check_rx.is_none()
    {
        let check_list: Vec<(String, String)> = agents
            .iter()
            .map(|a| (a.id.clone(), a.command.clone()))
            .collect();
        ws.ai_tool_check_rx = Some(spawn_ai_tool_check(check_list));
    }

    let current_agent = agents.iter().find(|a| a.id == ws.selected_agent_id);
    let installed = ai_tool_is_available(&ws.ai_tool_available, &ws.selected_agent_id);
    let can_start = installed && !checking && current_agent.is_some();

    let hover_text = if checking {
        i18n.get("ai-hover-checking")
    } else if let Some(agent) = current_agent {
        if installed {
            let mut args = fluent_bundle::FluentArgs::new();
            args.set("tool", agent.label.clone());
            args.set("cmd", agent.command.clone());
            i18n.get_args("ai-hover-start", &args)
        } else {
            let mut args = fluent_bundle::FluentArgs::new();
            args.set("cmd", agent.command.clone());
            i18n.get_args("ai-hover-missing", &args)
        }
    } else {
        String::new()
    };

    let start_response = ui
        .add_enabled(can_start, egui::Button::new(i18n.get("ai-btn-start")))
        .on_hover_text(hover_text);
    if start_response.clicked()
        && let Some(agent) = current_agent
    {
        // Start agent immediately
        let cmd = agent.command.clone();
        let active = ws.claude_active_tab;
        let context = format_context_for_terminal(&AiManager::generate_context(ws, shared, None, Vec::new()));
        if let Some(terminal) = ws.claude_tabs.get_mut(active) {
            terminal.send_command(&cmd);
            if agent.context_aware {
                terminal.send_command(&context);
            }
        }
    }

    // Sync context button
    let active_tab = ws.claude_tabs.get(ws.claude_active_tab);
    let can_sync = active_tab.map(|t| !t.is_exited()).unwrap_or(false);

    if ui
        .add_enabled(can_sync, egui::Button::new(i18n.get("ai-btn-sync")))
        .on_hover_text(i18n.get("ai-hover-sync"))
        .clicked()
        && let Some(agent) = current_agent
        && agent.context_aware
    {
        let context = format_context_for_terminal(&AiManager::generate_context(ws, shared, None, Vec::new()));
        if let Some(terminal) = ws.claude_tabs.get_mut(ws.claude_active_tab) {
            terminal.send_command(&context);
        }
    }
}

/// Renders terminal tabs and returns the requested action.
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
            let root = ws.root_path.clone();
            ws.claude_tabs.push(Terminal::new(id, ctx, &root, None));
            ws.claude_active_tab = ws.claude_tabs.len() - 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Public function
// ---------------------------------------------------------------------------

/// Renders the right panel with the AI terminal. Returns true if the terminal was clicked.
pub(super) fn render_ai_panel(
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
    let focused = ws.focused_panel;
    let font_size = config::EDITOR_FONT_SIZE * ws.ai.settings.font_scale as f32 / 100.0;

    if ws.claude_float {
        let mut is_open = true;
        egui::Window::new(i18n.get("ai-panel-title"))
            .id(egui::Id::new("claude_float_win"))
            .default_size([520.0, 420.0])
            .min_size([300.0, 200.0])
            .resizable(true)
            .collapsible(false)
            .open(&mut is_open)
            .show(ctx, |ui| {
                any_clicked |= render_ai_panel_content(
                    ui,
                    ws,
                    shared,
                    dialog_open,
                    focused,
                    font_size,
                    i18n,
                    true,  // is_float
                    false, // is_viewport
                );
            });
        if !is_open {
            ws.show_right_panel = false;
        }
    } else {
        egui::SidePanel::right("claude_panel")
            .default_width(config::AI_PANEL_DEFAULT_WIDTH)
            .width_range(config::AI_PANEL_MIN_WIDTH..=config::AI_PANEL_MAX_WIDTH)
            .resizable(true)
            .show(ctx, |ui| {
                any_clicked |= render_ai_panel_content(
                    ui,
                    ws,
                    shared,
                    dialog_open,
                    focused,
                    font_size,
                    i18n,
                    false, // is_float
                    false, // is_viewport
                );
            });
    }
    any_clicked
}

/// Actual content of the AI panel, shared between docked, floating, and separate window.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_ai_panel_content(
    ui: &mut egui::Ui,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    dialog_open: bool,
    focused: FocusedPanel,
    font_size: f32,
    i18n: &crate::i18n::I18n,
    is_float: bool,
    is_viewport: bool,
) -> bool {
    let mut any_clicked = false;

    ui.horizontal(|ui| {
        if !is_viewport {
            ui.heading(i18n.get("ai-panel-title"));
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if is_viewport {
                // In separate window, button to dock back
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
                // In main window, buttons to undock/viewport
                if ui
                    .small_button("↗")
                    .on_hover_text(i18n.get("ai-viewport-open"))
                    .clicked()
                {
                    ws.ai_viewport_open = true;
                }

                if is_float {
                    if ui
                        .small_button("⊟")
                        .on_hover_text(i18n.get("ai-float-dock"))
                        .clicked()
                    {
                        ws.claude_float = false;
                    }
                } else if ui
                    .small_button("⧉")
                    .on_hover_text(i18n.get("ai-float-undock"))
                    .clicked()
                {
                    ws.claude_float = true;
                }
            }
        });
    });

    ui.horizontal(|ui| {
        let combo_id = if is_viewport {
            "ai_tool_combo_viewport"
        } else if is_float {
            "ai_tool_combo_float"
        } else {
            "ai_tool_combo_docked"
        };
        render_ai_tool_controls(ui, ws, shared, combo_id, i18n);
    });

    let items: Vec<TabItem> = (0..ws.claude_tabs.len())
        .map(|i| TabItem {
            label: (i + 1).to_string(),
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

    if let Some(terminal) = ws.claude_tabs.get_mut(ws.claude_active_tab) {
        let terminal_action = terminal.ui(
            ui,
            focused == FocusedPanel::Claude && !dialog_open,
            font_size,
            i18n,
        );
        match terminal_action {
            Some(super::terminal::TerminalAction::Clicked)
            | Some(super::terminal::TerminalAction::Hovered) => {
                ws.focused_panel = FocusedPanel::Claude;
                any_clicked = true;
            }
            Some(super::terminal::TerminalAction::Navigate(path, line, col)) => {
                let abs_path = if path.is_absolute() {
                    path
                } else {
                    ws.root_path.join(path)
                };
                super::workspace::open_file_in_ws(ws, abs_path);
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
