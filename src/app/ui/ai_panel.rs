use std::collections::HashMap;

use eframe::egui;

use super::super::types::{AiTool, FocusedPanel};
use super::terminal::Terminal;
use super::widgets::tab_bar::{TabBarAction, TabItem, render_compact_tab_bar};
use super::workspace::{WorkspaceState, spawn_ai_tool_check};
use crate::config;

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

fn ai_tool_is_available(available: &HashMap<AiTool, bool>, tool: AiTool) -> bool {
    available.get(&tool).copied().unwrap_or(false)
}

fn ai_tool_status_label(
    tool: AiTool,
    available: &HashMap<AiTool, bool>,
    checking: bool,
    i18n: &crate::i18n::I18n,
) -> String {
    let mut args = fluent_bundle::FluentArgs::new();
    args.set("tool", tool.label());
    if checking {
        i18n.get_args("ai-tool-status-checking", &args)
    } else if ai_tool_is_available(available, tool) {
        i18n.get_args("ai-tool-status-available", &args)
    } else {
        i18n.get_args("ai-tool-status-missing", &args)
    }
}

fn render_ai_tool_picker(
    ui: &mut egui::Ui,
    id_salt: &'static str,
    selected: &mut AiTool,
    available: &HashMap<AiTool, bool>,
    checking: bool,
    i18n: &crate::i18n::I18n,
) {
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(ai_tool_status_label(*selected, available, checking, i18n))
        .width(190.0)
        .show_ui(ui, |ui| {
            for tool in AiTool::ALL {
                ui.selectable_value(
                    selected,
                    tool,
                    ai_tool_status_label(tool, available, checking, i18n),
                );
            }
        });
}

fn render_ai_tool_controls(
    ui: &mut egui::Ui,
    ws: &mut WorkspaceState,
    combo_id: &'static str,
    i18n: &crate::i18n::I18n,
) {
    let checking = ws.ai_tool_check_rx.is_some();

    ui.label(i18n.get("ai-label-assistant"));
    render_ai_tool_picker(
        ui,
        combo_id,
        &mut ws.claude_tool,
        &ws.ai_tool_available,
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
        ws.ai_tool_check_rx = Some(spawn_ai_tool_check());
    }

    let checking = ws.ai_tool_check_rx.is_some();
    let installed = ai_tool_is_available(&ws.ai_tool_available, ws.claude_tool);
    let can_start = installed && !checking;

    let hover_text = if checking {
        i18n.get("ai-hover-checking")
    } else if installed {
        let mut args = fluent_bundle::FluentArgs::new();
        args.set("tool", ws.claude_tool.label());
        args.set("cmd", ws.claude_tool.command());
        i18n.get_args("ai-hover-start", &args)
    } else {
        let mut args = fluent_bundle::FluentArgs::new();
        args.set("cmd", ws.claude_tool.command());
        i18n.get_args("ai-hover-missing", &args)
    };

    let start_response = ui
        .add_enabled(can_start, egui::Button::new(i18n.get("ai-btn-start")))
        .on_hover_text(hover_text);
    if start_response.clicked() {
        let cmd = ws.claude_tool.command().to_owned();
        let active = ws.claude_active_tab;
        if let Some(terminal) = ws.claude_tabs.get_mut(active) {
            terminal.send_command(&cmd);
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
            ws.claude_tabs.remove(idx);
            if ws.claude_active_tab >= ws.claude_tabs.len() {
                ws.claude_active_tab = ws.claude_tabs.len().saturating_sub(1);
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
    dialog_open: bool,
    i18n: &crate::i18n::I18n,
) -> bool {
    if !ws.show_right_panel {
        return false;
    }
    let mut any_clicked = false;
    let focused = ws.focused_panel;
    let font_size = config::EDITOR_FONT_SIZE * ws.ai_font_scale as f32 / 100.0;

    if ws.claude_float {
        let mut is_open = true;
        let mut tab_action: Option<TabBarAction> = None;
        egui::Window::new(i18n.get("ai-panel-title"))
            .id(egui::Id::new("claude_float_win"))
            .default_size([520.0, 420.0])
            .min_size([300.0, 200.0])
            .resizable(true)
            .collapsible(false)
            .open(&mut is_open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    render_ai_tool_controls(ui, ws, "ai_tool_combo_float", i18n);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button("⊟")
                            .on_hover_text(i18n.get("ai-float-dock"))
                            .clicked()
                        {
                            ws.claude_float = false;
                        }
                    });
                });
                let items: Vec<TabItem> = (0..ws.claude_tabs.len())
                    .map(|i| TabItem {
                        label: (i + 1).to_string(),
                        closable: ws.claude_tabs.len() > 1,
                    })
                    .collect();
                tab_action = render_compact_tab_bar(
                    ui,
                    &items,
                    ws.claude_active_tab,
                    true,
                    &i18n.get("ai-tab-close-hover"),
                    &i18n.get("ai-tab-new-hover"),
                );
                ui.separator();
                if !dialog_open
                    && let Some(terminal) = ws.claude_tabs.get_mut(ws.claude_active_tab)
                    && terminal.ui(ui, focused == FocusedPanel::Claude, font_size, i18n)
                {
                    ws.focused_panel = FocusedPanel::Claude;
                    any_clicked = true;
                }
            });
        if let Some(action) = tab_action {
            apply_tab_action(ws, action, ctx);
        }
        if !is_open {
            ws.show_right_panel = false;
        }
    } else {
        let mut tab_action: Option<TabBarAction> = None;
        egui::SidePanel::right("claude_panel")
            .default_width(config::AI_PANEL_DEFAULT_WIDTH)
            .width_range(config::AI_PANEL_MIN_WIDTH..=config::AI_PANEL_MAX_WIDTH)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(i18n.get("ai-panel-title"));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button("⧉")
                            .on_hover_text(i18n.get("ai-float-undock"))
                            .clicked()
                        {
                            ws.claude_float = true;
                        }
                    });
                });
                ui.horizontal(|ui| {
                    render_ai_tool_controls(ui, ws, "ai_tool_combo_docked", i18n);
                });
                let items: Vec<TabItem> = (0..ws.claude_tabs.len())
                    .map(|i| TabItem {
                        label: (i + 1).to_string(),
                        closable: ws.claude_tabs.len() > 1,
                    })
                    .collect();
                tab_action = render_compact_tab_bar(
                    ui,
                    &items,
                    ws.claude_active_tab,
                    true,
                    &i18n.get("ai-tab-close-hover"),
                    &i18n.get("ai-tab-new-hover"),
                );
                ui.separator();
                if !dialog_open
                    && let Some(terminal) = ws.claude_tabs.get_mut(ws.claude_active_tab)
                    && terminal.ui(ui, focused == FocusedPanel::Claude, font_size, i18n)
                {
                    ws.focused_panel = FocusedPanel::Claude;
                    any_clicked = true;
                }
            });
        if let Some(action) = tab_action {
            apply_tab_action(ws, action, ctx);
        }
    }
    any_clicked
}
