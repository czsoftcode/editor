use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use crate::tr;
use eframe::egui;

/// Renders the native tool approval UI for PendingToolApproval (Phase 16).
pub fn render_tool_approval_ui(ui: &mut egui::Ui, ws: &mut WorkspaceState, i18n: &I18n) {
    let pending = match ws.pending_tool_approval.take() {
        Some(p) => p,
        None => return,
    };

    let border_color = if pending.is_network {
        egui::Color32::from_rgb(220, 60, 60)
    } else {
        egui::Color32::YELLOW
    };

    egui::Frame::new()
        .stroke(egui::Stroke::new(1.5, border_color))
        .inner_margin(16.0)
        .corner_radius(8.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                // Header
                let icon = tool_icon(&pending.tool_name);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(icon).size(22.0));
                    ui.label(
                        egui::RichText::new(tr!(i18n, "cli-tool-tool-approval-heading", tool = pending.tool_name.as_str()))
                            .strong()
                            .size(18.0)
                            .color(egui::Color32::YELLOW),
                    );
                });
                ui.add_space(4.0);
                ui.label(&pending.description);

                // Warnings
                if pending.is_network {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(i18n.get("cli-tool-network-warning"))
                            .color(egui::Color32::from_rgb(255, 80, 80))
                            .strong(),
                    );
                }
                if pending.is_new_file {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(i18n.get("cli-tool-new-file-hint"))
                            .color(egui::Color32::from_rgb(80, 200, 80)),
                    );
                }

                ui.add_space(8.0);

                // Details (diff or command preview)
                egui::ScrollArea::both()
                    .max_height(300.0)
                    .id_salt("tool_approval_details_scroll")
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        render_diff_or_markdown(ui, ws, &pending.details);
                    });

                ui.add_space(16.0);

                // Buttons
                let mut handled = false;
                let btn_size = egui::vec2(140.0, 30.0);
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(btn_size, egui::Button::new(
                            egui::RichText::new(i18n.get("cli-tool-approve")).strong(),
                        ))
                        .clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Num1))
                    {
                        let _ = pending.response_tx.send(true);
                        handled = true;
                    }
                    ui.add_space(8.0);
                    if ui
                        .add_sized(btn_size, egui::Button::new(i18n.get("cli-tool-approve-always")))
                        .clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Num2))
                    {
                        ws.tool_always_approved.insert(pending.tool_name.clone());
                        let _ = pending.response_tx.send(true);
                        handled = true;
                    }
                    ui.add_space(8.0);
                    if ui
                        .add_sized(btn_size, egui::Button::new(i18n.get("cli-tool-deny")))
                        .clicked()
                        || ui.input(|i| {
                            i.key_pressed(egui::Key::Num3) || i.key_pressed(egui::Key::Escape)
                        })
                    {
                        let _ = pending.response_tx.send(false);
                        handled = true;
                    }
                });

                if !handled {
                    // Put it back — not handled yet
                    ws.pending_tool_approval = Some(pending);
                } else {
                    ui.ctx().request_repaint();
                }
            });
        });
}

/// Renders the native ask_user UI for PendingToolAsk (Phase 16).
pub fn render_tool_ask_ui(ui: &mut egui::Ui, ws: &mut WorkspaceState, i18n: &I18n) {
    let mut pending = match ws.pending_tool_ask.take() {
        Some(p) => p,
        None => return,
    };

    egui::Frame::new()
        .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 160, 255)))
        .inner_margin(16.0)
        .corner_radius(8.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("\u{2753}").size(22.0));
                    let mut ask_args = fluent_bundle::FluentArgs::new();
                    ask_args.set("agent", "AI");
                    ui.label(
                        egui::RichText::new(i18n.get_args("cli-tool-ask-heading", &ask_args))
                            .strong()
                            .size(18.0)
                            .color(egui::Color32::from_rgb(100, 160, 255)),
                    );
                });
                ui.add_space(8.0);
                ui.label(egui::RichText::new(&pending.question).size(15.0));
                ui.add_space(12.0);

                // Option buttons
                if !pending.options.is_empty() {
                    ui.label(egui::RichText::new(i18n.get("cli-tool-quick-options")).weak());
                    ui.add_space(4.0);
                    let mut chosen: Option<String> = None;
                    ui.horizontal_wrapped(|ui| {
                        for opt in &pending.options {
                            if ui.button(opt).clicked() {
                                chosen = Some(opt.clone());
                            }
                        }
                    });
                    if let Some(answer) = chosen {
                        let _ = pending.response_tx.send(answer);
                        ui.ctx().request_repaint();
                        return;
                    }
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(i18n.get("cli-tool-custom-input")).weak());
                }

                // Free-text input
                let response = ui.add(
                    egui::TextEdit::singleline(&mut pending.input_buffer)
                        .desired_width(ui.available_width())
                        .hint_text(i18n.get("cli-tool-input-placeholder")),
                );
                let submitted = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                ui.add_space(8.0);
                let mut handled = false;
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(egui::vec2(120.0, 28.0), egui::Button::new(
                            egui::RichText::new(i18n.get("cli-tool-send")).strong(),
                        ))
                        .clicked()
                        || submitted
                    {
                        let answer = std::mem::take(&mut pending.input_buffer);
                        let _ = pending.response_tx.send(answer);
                        handled = true;
                    }
                    ui.add_space(8.0);
                    if ui
                        .add_sized(egui::vec2(100.0, 28.0), egui::Button::new(i18n.get("cli-tool-cancel")))
                        .clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Escape))
                    {
                        let _ = pending.response_tx.send(String::new());
                        handled = true;
                    }
                });

                if !handled {
                    ws.pending_tool_ask = Some(pending);
                } else {
                    ui.ctx().request_repaint();
                }
            });
        });
}

/// Returns a unicode icon for a tool name.
fn tool_icon(tool_name: &str) -> &'static str {
    match tool_name {
        "read_project_file" | "list_project_files" | "search_project" | "semantic_search" => "\u{1F4C4}", // document
        "write_file" | "replace" => "\u{270F}\u{FE0F}", // pencil
        "exec" => "\u{1F4BB}", // terminal/laptop
        "ask_user" => "\u{2753}", // question mark
        "announce_completion" => "\u{2705}", // check
        "store_scratch" | "retrieve_scratch" | "store_fact" | "retrieve_fact" | "list_facts" | "delete_fact" => "\u{1F4DD}", // memo
        _ => "\u{1F527}", // wrench
    }
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
