use crate::app::ui::widgets::modal::StandardModal;
use eframe::egui;
use std::path::PathBuf;

#[derive(Default)]
pub(crate) struct PrivacyState {
    pub cache: egui_commonmark::CommonMarkCache,
    pub content: Option<String>,
}

pub(crate) enum PrivacyResult {
    Accepted,
    LanguageChanged(String),
    None,
}

pub(crate) fn show_privacy_dialog(
    ctx: &egui::Context,
    state: &mut PrivacyState,
    i18n: &crate::i18n::I18n,
) -> PrivacyResult {
    if state.content.is_none() {
        let lang = i18n.lang();
        let mut path = PathBuf::from("privacy").join(format!("Privacy_{}.md", lang));
        if !path.exists() {
            path = PathBuf::from("privacy").join("Privacy_en.md");
        }

        match std::fs::read_to_string(&path) {
            Ok(c) => state.content = Some(c),
            Err(e) => {
                state.content = Some(format!(
                    "Error loading privacy policy from {}: {}",
                    path.display(),
                    e
                ));
            }
        }
    }

    let mut local_show = true;
    let modal =
        StandardModal::new(i18n.get("privacy-title"), "privacy_modal").with_size(700.0, 550.0);

    let res = modal.show(ctx, &mut local_show, |ui| {
        let mut result = PrivacyResult::None;

        // FOOTER
        if let Some(r) = modal.ui_footer_actions(ui, i18n, |f| {
            if f.close() {
                return Some(PrivacyResult::None);
            }
            if f.ui
                .button(egui::RichText::new(i18n.get("btn-accept-privacy")).strong())
                .clicked()
            {
                return Some(PrivacyResult::Accepted);
            }
            if f.button("startup-quit").clicked() {
                std::process::exit(0);
            }
            None
        }) {
            result = r;
        }

        // BODY
        modal.ui_body(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("🌐");
                for &lang_code in crate::i18n::SUPPORTED_LANGS {
                    if ui
                        .selectable_label(i18n.lang() == lang_code, lang_code.to_uppercase())
                        .clicked()
                    {
                        result = PrivacyResult::LanguageChanged(lang_code.to_string());
                    }
                }
            });
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .id_salt("privacy_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if let Some(content) = &state.content {
                        egui_commonmark::CommonMarkViewer::new().show(
                            ui,
                            &mut state.cache,
                            content,
                        );
                    }
                });
        });

        result
    });

    res.unwrap_or(PrivacyResult::None)
}
