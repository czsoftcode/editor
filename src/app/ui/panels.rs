use super::super::types::{FocusedPanel, Toast};
use super::workspace::open_file_in_ws;
use super::workspace::state::WorkspaceState;
use crate::config;
use eframe::egui;

/// Renders the left panel (file tree + build terminal). Returns true if the terminal was clicked.
pub(super) fn render_left_panel(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
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

                    ui.horizontal(|ui| {
                        let title = if ws.file_tree_in_sandbox {
                            egui::RichText::new(i18n.get("panel-files-sandbox"))
                                .color(egui::Color32::from_rgb(255, 230, 100))
                                .strong()
                        } else {
                            egui::RichText::new(i18n.get("panel-files")).strong()
                        };
                        ui.heading(title);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let prev_in_sandbox = ws.file_tree_in_sandbox;
                            if ui
                                .selectable_label(
                                    !ws.file_tree_in_sandbox,
                                    i18n.get("btn-tree-project"),
                                )
                                .clicked()
                            {
                                ws.file_tree_in_sandbox = false;
                            }
                            if ui
                                .selectable_label(
                                    ws.file_tree_in_sandbox,
                                    i18n.get("btn-tree-sandbox"),
                                )
                                .clicked()
                            {
                                ws.file_tree_in_sandbox = true;
                            }

                            if ws.file_tree_in_sandbox != prev_in_sandbox {
                                let target_dir = if ws.file_tree_in_sandbox {
                                    &ws.sandbox.root
                                } else {
                                    &ws.root_path
                                };
                                ws.file_tree.load(target_dir);
                            }
                        });
                    });

                    ui.separator();
                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .id_salt("file_tree_scroll")
                        .show(ui, |ui| {
                            let result = ws.file_tree.ui(ui, i18n, ws.file_tree_in_sandbox);
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

            // 2. RESIZE SPLITTER & TERMINAL
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

/// Renders toast notifications in the bottom right corner. Removes expired toasts.
pub(super) fn render_toasts(ctx: &egui::Context, ws: &mut WorkspaceState) {
    ws.toasts.retain(|t: &Toast| !t.is_expired());
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
            for toast in &ws.toasts {
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
