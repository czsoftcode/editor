use crate::app::ui::workspace::state::WorkspaceState;
use eframe::egui;

pub fn render_approval_ui(
    ui: &mut egui::Ui,
    id: String,
    _action_name: String,
    details: String,
    sender: std::sync::mpsc::Sender<crate::app::types::PluginApprovalResponse>,
    ws: &mut WorkspaceState,
) {
    egui::Frame::new()
        .stroke(egui::Stroke::new(1.5, egui::Color32::YELLOW))
        .inner_margin(16.0)
        .corner_radius(8.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("⚠️").size(24.0));
                    ui.label(
                        egui::RichText::new(format!("Agent '{}' vyžaduje schválení akce", id))
                            .strong()
                            .size(20.0)
                            .color(egui::Color32::YELLOW),
                    );
                });
                ui.add_space(12.0);

                egui::ScrollArea::vertical()
                    .max_height(250.0)
                    .id_salt("approval_details_scroll")
                    .show(ui, |ui| {
                        if details.contains("```diff") {
                            // Specialized diff rendering for the approval dialog
                            let parts: Vec<&str> = details.split("```diff").collect();
                            if parts.len() > 1 {
                                // Render header (markdown before diff)
                                egui_commonmark::CommonMarkViewer::new().show(
                                    ui,
                                    &mut ws.markdown_cache,
                                    parts[0],
                                );

                                // Render the diff block with background colors
                                let diff_content = parts[1].split("```").next().unwrap_or("");
                                egui::Frame::new()
                                    .fill(egui::Color32::from_rgb(30, 30, 35))
                                    .inner_margin(8.0)
                                    .show(ui, |ui| {
                                        ui.set_width(ui.available_width());
                                        for line in diff_content.lines() {
                                            if line.trim().is_empty() {
                                                continue;
                                            }
                                            // Detect tag (+, -, or space) at index 5 (after 4 digits and space)
                                            let tag = if line.len() > 5 {
                                                line.as_bytes()[5] as char
                                            } else {
                                                ' '
                                            };

                                            let (bg, fg) = if tag == '+' {
                                                (
                                                    egui::Color32::from_rgba_unmultiplied(
                                                        40, 100, 40, 100,
                                                    ),
                                                    egui::Color32::from_rgb(150, 255, 150),
                                                )
                                            } else if tag == '-' {
                                                (
                                                    egui::Color32::from_rgba_unmultiplied(
                                                        120, 30, 30, 100,
                                                    ),
                                                    egui::Color32::from_rgb(255, 150, 150),
                                                )
                                            } else {
                                                (
                                                    egui::Color32::TRANSPARENT,
                                                    egui::Color32::from_gray(180),
                                                )
                                            };

                                            egui::Frame::new().fill(bg).show(ui, |ui| {
                                                ui.set_width(ui.available_width());
                                                ui.label(
                                                    egui::RichText::new(line).monospace().color(fg),
                                                );
                                            });
                                        }
                                    });

                                // Render footer (markdown after diff if any)
                                if let Some(footer) = parts[1].split("```").nth(1) {
                                    egui_commonmark::CommonMarkViewer::new().show(
                                        ui,
                                        &mut ws.markdown_cache,
                                        footer,
                                    );
                                }
                            } else {
                                egui_commonmark::CommonMarkViewer::new().show(
                                    ui,
                                    &mut ws.markdown_cache,
                                    &details,
                                );
                            }
                        } else {
                            egui_commonmark::CommonMarkViewer::new().show(
                                ui,
                                &mut ws.markdown_cache,
                                &details,
                            );
                        }
                    });

                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    let btn_size = egui::vec2(150.0, 32.0);
                    let mut handled = false;

                    if ui
                        .add_sized(
                            btn_size,
                            egui::Button::new(egui::RichText::new("1 - Provést").strong()),
                        )
                        .clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Num1))
                    {
                        // Add the approved change details to the conversation history for permanence
                        if let Some(last) = ws.gemini_conversation.last_mut()
                            && !last.1.contains(&details)
                        {
                            if !last.1.is_empty() {
                                last.1.push_str(
                                    "

",
                                );
                            }
                            last.1.push_str(&details);
                        }
                        let _ = sender.send(crate::app::types::PluginApprovalResponse::Approve);
                        handled = true;
                    }
                    ui.add_space(12.0);
                    if ui
                        .add_sized(btn_size, egui::Button::new("2 - Schvalovat vždy"))
                        .clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Num2))
                    {
                        // Add to history as well
                        if let Some(last) = ws.gemini_conversation.last_mut()
                            && !last.1.contains(&details)
                        {
                            if !last.1.is_empty() {
                                last.1.push_str(
                                    "

",
                                );
                            }
                            last.1.push_str(&details);
                        }
                        let _ =
                            sender.send(crate::app::types::PluginApprovalResponse::ApproveAlways);
                        handled = true;
                    }
                    ui.add_space(12.0);
                    if ui
                        .add_sized(btn_size, egui::Button::new("3/Esc - Zamítnout"))
                        .clicked()
                        || ui.input(|i| {
                            i.key_pressed(egui::Key::Num3) || i.key_pressed(egui::Key::Escape)
                        })
                    {
                        let _ = sender.send(crate::app::types::PluginApprovalResponse::Deny);
                        ws.gemini_cancellation_token
                            .store(true, std::sync::atomic::Ordering::Relaxed);
                        handled = true;
                    }

                    if !handled {
                        ws.pending_plugin_approval = Some((id, _action_name, details, sender));
                    } else {
                        ui.ctx().request_repaint();
                    }
                });
            });
        });
}
