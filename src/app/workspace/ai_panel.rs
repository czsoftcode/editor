use std::collections::HashMap;

use eframe::egui;

use super::{AiTool, FocusedPanel, WorkspaceState, spawn_ai_tool_check};
use crate::config;

fn ai_tool_is_available(available: &HashMap<AiTool, bool>, tool: AiTool) -> bool {
    available.get(&tool).copied().unwrap_or(false)
}

fn ai_tool_status_label(tool: AiTool, available: &HashMap<AiTool, bool>, checking: bool) -> String {
    if checking {
        format!("{} (ověřuji…)", tool.label())
    } else if ai_tool_is_available(available, tool) {
        format!("{} (nainstalováno)", tool.label())
    } else {
        format!("{} (není v PATH)", tool.label())
    }
}

fn render_ai_tool_picker(
    ui: &mut egui::Ui,
    id_salt: &'static str,
    selected: &mut AiTool,
    available: &HashMap<AiTool, bool>,
    checking: bool,
) {
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(ai_tool_status_label(*selected, available, checking))
        .width(190.0)
        .show_ui(ui, |ui| {
            for tool in AiTool::ALL {
                ui.selectable_value(
                    selected,
                    tool,
                    ai_tool_status_label(tool, available, checking),
                );
            }
        });
}

fn render_ai_tool_controls(ui: &mut egui::Ui, ws: &mut WorkspaceState, combo_id: &'static str) {
    let checking = ws.ai_tool_check_rx.is_some();

    ui.label("Asistent:");
    render_ai_tool_picker(
        ui,
        combo_id,
        &mut ws.claude_tool,
        &ws.ai_tool_available,
        checking,
    );

    if ui
        .small_button("↻")
        .on_hover_text("Znovu ověřit dostupnost AI CLI nástrojů")
        .clicked()
        && ws.ai_tool_check_rx.is_none()
    {
        ws.ai_tool_check_rx = Some(spawn_ai_tool_check());
    }

    let checking = ws.ai_tool_check_rx.is_some();
    let installed = ai_tool_is_available(&ws.ai_tool_available, ws.claude_tool);
    let can_start = installed && !checking;

    let hover_text = if checking {
        "Ověřuji dostupnost AI CLI nástrojů…".to_string()
    } else if installed {
        format!(
            "Spustí {} (`{}`) v terminálu",
            ws.claude_tool.label(),
            ws.claude_tool.command()
        )
    } else {
        format!(
            "Příkaz `{}` nebyl nalezen v PATH. Nainstaluj nástroj a klikni na ↻.",
            ws.claude_tool.command()
        )
    };

    let start_response = ui
        .add_enabled(can_start, egui::Button::new("\u{25B6} Spustit"))
        .on_hover_text(hover_text);
    if start_response.clicked() {
        if let Some(terminal) = &mut ws.claude_terminal {
            terminal.send_command(ws.claude_tool.command());
        }
    }
}

/// Vykreslí pravý panel s AI terminálem. Vrací true pokud bylo kliknuto do terminálu.
pub(super) fn render_ai_panel(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    dialog_open: bool,
) -> bool {
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
                    render_ai_tool_controls(ui, ws, "ai_tool_combo_float");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button("⊟")
                            .on_hover_text("Přikovat do panelu")
                            .clicked()
                        {
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
        if !is_open {
            ws.show_right_panel = false;
        }
    } else {
        egui::SidePanel::right("claude_panel")
            .default_width(config::AI_PANEL_DEFAULT_WIDTH)
            .width_range(config::AI_PANEL_MIN_WIDTH..=config::AI_PANEL_MAX_WIDTH)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("AI terminál");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button("⧉")
                            .on_hover_text("Odpojit do plovoucího okna")
                            .clicked()
                        {
                            ws.claude_float = true;
                        }
                    });
                });
                ui.horizontal(|ui| {
                    render_ai_tool_controls(ui, ws, "ai_tool_combo_docked");
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
    }
    any_clicked
}
