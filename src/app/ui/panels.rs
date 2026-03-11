use super::super::types::{AppShared, FocusedPanel, Toast};
use super::workspace::open_file_in_ws;
use super::workspace::state::WorkspaceState;
use crate::config;
use eframe::egui;
use std::sync::{Arc, Mutex};

/// Renders the left panel (file tree + plugin bar + build terminal). Returns true if the terminal was clicked.
pub(super) fn render_left_panel(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    dialog_open: bool,
    i18n: &crate::i18n::I18n,
) -> bool {
    if !ws.show_left_panel {
        return false;
    }
    let mut left_clicked = false;

    egui::SidePanel::left("left_panel")
        .default_width(config::LEFT_PANEL_DEFAULT_WIDTH)
        .width_range(config::LEFT_PANEL_MIN_WIDTH..=config::LEFT_PANEL_MAX_WIDTH)
        .resizable(true)
        .show(ctx, |ui| {
            let total_height = ui.available_height();
            let show_terminal_in_panel = ws.show_build_terminal && !ws.build_terminal_float;

            let tree_height = if show_terminal_in_panel {
                (total_height * ws.left_panel_split).max(100.0)
            } else {
                total_height
            };

            // 1. FILE TREE AREA
            let tree_resp = ui.scope(|ui| {
                egui::Frame::NONE.show(ui, |ui| {
                    ui.set_max_height(tree_height);
                    ui.set_min_height(tree_height);

                    ui.heading(egui::RichText::new(i18n.get("panel-files")).strong());

                    ui.separator();
                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .id_salt("file_tree_scroll")
                        .show(ui, |ui| {
                            let result = ws.file_tree.ui(ui, i18n);
                            if let Some(err) = ws.file_tree.take_error() {
                                ws.toasts.push(Toast::error(err));
                            }
                            if let Some(path) = result.selected {
                                open_file_in_ws(ws, path);
                            }
                            if let Some(path) = result.created_file {
                                open_file_in_ws(ws, path);
                            }
                            if let Some(deleted) = result.deleted {
                                ws.editor.close_tabs_for_path(&deleted);
                            }
                        });
                });
            });

            if tree_resp.response.clicked() || tree_resp.response.dragged() {
                ws.focused_panel = FocusedPanel::Files;
                left_clicked = true;
            }

            // 2. AI PLUGIN BAR
            render_plugin_bar(ui, ws, shared, i18n);

            // 3. RESIZE SPLITTER & TERMINAL
            if show_terminal_in_panel {
                // Render interactive separator (splitter)
                let sep_rect = ui.separator().rect;
                let interact_rect = sep_rect.expand2(egui::vec2(0.0, 4.0)); // Make it easier to grab
                let response = ui.interact(
                    interact_rect,
                    ui.id().with("tree_splitter"),
                    egui::Sense::drag(),
                );

                if response.dragged() {
                    let delta_y = response.drag_delta().y;
                    let total_h = ui.available_height() + tree_height;
                    if total_h > 0.0 {
                        ws.left_panel_split =
                            (ws.left_panel_split + delta_y / total_h).clamp(0.1, 0.9);
                    }
                }

                if response.hovered() || response.dragged() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
                }

                if crate::app::ui::terminal::bottom::render_bottom_content(
                    ui,
                    ws,
                    dialog_open,
                    i18n,
                ) {
                    left_clicked = true;
                }
            }
        });
    left_clicked
}

/// Renders the AI quick-launch bar (Start + Settings).
fn render_plugin_bar(
    ui: &mut egui::Ui,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &crate::i18n::I18n,
) {
    ui.separator();

    ui.horizontal(|ui| {
        ui.label(i18n.get("cli-bar-label"));

        if ui
            .button(i18n.get("ai-btn-start"))
            .on_hover_text(i18n.get("cli-bar-start-hover"))
            .clicked()
        {
            let agents = {
                let sh = shared.lock().expect("lock");
                sh.registry.agents.get_all().to_vec()
            };
            if let Some(agent) = agents.iter().find(|a| a.id == ws.selected_agent_id) {
                let cmd = agent.command.clone();
                if let Some(terminal) = ws.claude_tabs.get_mut(ws.claude_active_tab) {
                    terminal.send_command(&cmd);
                }
            }
        }

        if ui
            .button(i18n.get("cli-bar-settings"))
            .on_hover_text(i18n.get("cli-bar-settings-hover"))
            .clicked()
        {
            ws.show_settings = true;
        }
    });
}

/// Renders toast notifications in the bottom right corner. Removes expired toasts.
pub(super) fn render_toasts(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    _i18n: &crate::i18n::I18n,
) {
    ws.toasts
        .retain(|t| !t.is_expired());
    if ws.toasts.is_empty() {
        return;
    }

    let screen = ctx.screen_rect();
    let toast_w = 340.0_f32;
    let toast_h = 40.0_f32;
    let padding = 12.0_f32;
    let start_y = screen.max.y - padding - (toast_h + padding) * ws.toasts.len() as f32;

    egui::Area::new(egui::Id::new("toasts_area"))
        .fixed_pos(egui::pos2(screen.max.x - toast_w - padding, start_y))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            for toast in ws.toasts.iter() {
                let (bg, fg) = if toast.is_error {
                    (
                        egui::Color32::from_rgb(90, 30, 30),
                        egui::Color32::from_rgb(255, 180, 170),
                    )
                } else {
                    (
                        egui::Color32::from_rgb(30, 60, 45),
                        egui::Color32::from_rgb(160, 230, 180),
                    )
                };
                egui::Frame::new()
                    .fill(bg)
                    .corner_radius(6.0)
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.set_min_width(toast_w);
                        ui.label(
                            egui::RichText::new(&toast.message)
                                .color(fg)
                                .size(config::UI_FONT_SIZE),
                        );
                    });
                ui.add_space(padding);
            }
        });
}
