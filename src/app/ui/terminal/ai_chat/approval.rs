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
                    ui.label(egui::RichText::new("\u{26A0}\u{FE0F}").size(24.0));
                    ui.label(
                        egui::RichText::new(format!("Agent '{}' vyžaduje schválení akce", id))
                            .strong()
                            .size(20.0)
                            .color(egui::Color32::YELLOW),
                    );
                });
                ui.add_space(12.0);

                egui::ScrollArea::both()
                    .max_height(400.0)
                    .id_salt("ai_chat_terminal_approval_scroll")
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        render_diff_or_markdown(ui, ws, &details);
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
                        append_to_last_conversation(ws, &details);
                        let _ = sender.send(crate::app::types::PluginApprovalResponse::Approve);
                        handled = true;
                    }
                    ui.add_space(12.0);
                    if ui
                        .add_sized(btn_size, egui::Button::new("2 - Schvalovat vždy"))
                        .clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Num2))
                    {
                        append_to_last_conversation(ws, &details);
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
                        ws.ai.cancellation_token
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

/// Renders the ask_user dialog when the agent is blocking for user input.
pub fn render_ask_user_ui(
    ui: &mut egui::Ui,
    id: String,
    question: String,
    options: Vec<String>,
    input_buffer: &mut String,
    sender: std::sync::mpsc::Sender<String>,
    ws: &mut WorkspaceState,
) {
    egui::Frame::new()
        .stroke(egui::Stroke::new(
            1.5,
            egui::Color32::from_rgb(100, 160, 255),
        ))
        .inner_margin(16.0)
        .corner_radius(8.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("❓").size(24.0));
                    ui.label(
                        egui::RichText::new(format!("Agent '{}' se ptá:", id))
                            .strong()
                            .size(18.0)
                            .color(egui::Color32::from_rgb(100, 160, 255)),
                    );
                });
                ui.add_space(8.0);
                ui.label(egui::RichText::new(&question).size(15.0));
                ui.add_space(12.0);

                // Option buttons
                if !options.is_empty() {
                    ui.label(egui::RichText::new("Rychlé možnosti:").weak());
                    ui.add_space(4.0);
                    let mut chosen: Option<String> = None;
                    ui.horizontal_wrapped(|ui| {
                        for opt in &options {
                            if ui.button(opt).clicked() {
                                chosen = Some(opt.clone());
                            }
                        }
                    });
                    if let Some(answer) = chosen {
                        let _ = sender.send(answer.clone());
                        append_to_last_conversation(ws, &format!("**Odpověď:** {}", answer));
                        ws.pending_ask_user = None;
                        ui.ctx().request_repaint();
                        return;
                    }
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Nebo napište vlastní:").weak());
                }

                // Free-text input
                let response = ui.add(
                    egui::TextEdit::singleline(input_buffer)
                        .desired_width(ui.available_width())
                        .hint_text("Vaše odpověď…"),
                );
                let submitted =
                    response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                ui.add_space(8.0);
                let mut handled = false;
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            egui::vec2(120.0, 28.0),
                            egui::Button::new(egui::RichText::new("Odeslat").strong()),
                        )
                        .clicked()
                        || submitted
                    {
                        let answer = std::mem::take(input_buffer);
                        let _ = sender.send(answer.clone());
                        append_to_last_conversation(ws, &format!("**Odpověď:** {}", answer));
                        handled = true;
                    }
                    ui.add_space(8.0);
                    if ui
                        .add_sized(egui::vec2(100.0, 28.0), egui::Button::new("Zrušit"))
                        .clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Escape))
                    {
                        let _ = sender.send(String::new());
                        ws.ai.cancellation_token
                            .store(true, std::sync::atomic::Ordering::Relaxed);
                        handled = true;
                    }
                });

                if !handled {
                    ws.pending_ask_user =
                        Some((id, question, options, input_buffer.clone(), sender));
                } else {
                    ws.pending_ask_user = None;
                    ui.ctx().request_repaint();
                }
            });
        });
}

fn append_to_last_conversation(ws: &mut WorkspaceState, details: &str) {
    if let Some(last) = ws.ai.chat.conversation.last_mut()
        && !last.1.contains(details)
    {
        if !last.1.is_empty() {
            last.1.push_str("\n\n");
        }
        last.1.push_str(details);
    }
}

fn render_diff_or_markdown(ui: &mut egui::Ui, ws: &mut WorkspaceState, details: &str) {
    if !details.contains("```diff") {
        egui_commonmark::CommonMarkViewer::new().show(ui, &mut ws.ai.markdown_cache, details);
        return;
    }

    let parts: Vec<&str> = details.split("```diff").collect();
    if parts.len() < 2 {
        egui_commonmark::CommonMarkViewer::new().show(ui, &mut ws.ai.markdown_cache, details);
        return;
    }

    egui_commonmark::CommonMarkViewer::new().show(ui, &mut ws.ai.markdown_cache, parts[0]);

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
                let tag = line.as_bytes().get(5).copied().unwrap_or(b' ') as char;
                let (bg, fg) = match tag {
                    '+' => (
                        egui::Color32::from_rgba_unmultiplied(40, 100, 40, 100),
                        egui::Color32::from_rgb(150, 255, 150),
                    ),
                    '-' => (
                        egui::Color32::from_rgba_unmultiplied(120, 30, 30, 100),
                        egui::Color32::from_rgb(255, 150, 150),
                    ),
                    _ => (egui::Color32::TRANSPARENT, egui::Color32::from_gray(180)),
                };
                egui::Frame::new().fill(bg).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.add(
                        egui::Label::new(egui::RichText::new(line).monospace().color(fg)).wrap(),
                    );
                });
            }
        });

    if let Some(footer) = parts[1].split("```").nth(1) {
        egui_commonmark::CommonMarkViewer::new().show(ui, &mut ws.ai.markdown_cache, footer);
    }
}
