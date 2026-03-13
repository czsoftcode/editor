use crate::app::ui::editor::Editor;
use eframe::egui;

pub fn completion_kind_label(
    kind: Option<async_lsp::lsp_types::CompletionItemKind>,
) -> &'static str {
    use async_lsp::lsp_types::CompletionItemKind;
    match kind {
        Some(CompletionItemKind::FUNCTION) | Some(CompletionItemKind::METHOD) => "fn",
        Some(CompletionItemKind::CONSTRUCTOR) => "fn",
        Some(CompletionItemKind::STRUCT) | Some(CompletionItemKind::CLASS) => "st",
        Some(CompletionItemKind::INTERFACE) => "if",
        Some(CompletionItemKind::ENUM) => "en",
        Some(CompletionItemKind::ENUM_MEMBER) => "ev",
        Some(CompletionItemKind::CONSTANT) => "co",
        Some(CompletionItemKind::VARIABLE)
        | Some(CompletionItemKind::FIELD)
        | Some(CompletionItemKind::PROPERTY) => "va",
        Some(CompletionItemKind::KEYWORD) => "kw",
        Some(CompletionItemKind::MODULE) => "mo",
        Some(CompletionItemKind::TYPE_PARAMETER) => "ty",
        Some(CompletionItemKind::SNIPPET) => "sn",
        _ => "  ",
    }
}

pub fn render_completion_popup(
    ui: &egui::Ui,
    state: &crate::app::ui::editor::LspCompletionState,
) -> Option<usize> {
    let mut accepted = None;

    let font_size = Editor::current_editor_font_size(ui);
    let popup_pos = state.screen_pos + egui::vec2(0.0, font_size + 4.0);

    egui::Area::new(egui::Id::new("lsp_completion_popup"))
        .fixed_pos(popup_pos)
        .order(egui::Order::Foreground)
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(38, 41, 50))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgb(80, 100, 130),
                ))
                .inner_margin(egui::Margin::same(4))
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.set_max_width(380.0);
                    egui::ScrollArea::vertical()
                        .id_salt("lsp_completion_scroll")
                        .max_height(220.0)
                        .show(ui, |ui| {
                            for (i, item) in state.items.iter().enumerate() {
                                let is_selected = i == state.selected;
                                let kind_label = completion_kind_label(item.kind);
                                let label = format!(
                                    "{} {}",
                                    kind_label,
                                    item.detail
                                        .as_deref()
                                        .map(|d| format!("{} — {}", item.label, d))
                                        .unwrap_or_else(|| item.label.clone()),
                                );
                                let text = egui::RichText::new(&label)
                                    .monospace()
                                    .size(font_size - 1.0);
                                let response = ui.selectable_label(is_selected, text);
                                if is_selected {
                                    response.scroll_to_me(None);
                                }
                                if response.clicked() {
                                    accepted = Some(i);
                                }
                            }
                        });
                });
        });

    accepted
}
