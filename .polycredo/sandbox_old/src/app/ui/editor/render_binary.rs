use super::*;
use eframe::egui;

impl Editor {
    pub(super) fn ui_binary(&mut self, ui: &mut egui::Ui, _i18n: &crate::i18n::I18n) -> bool {
        let idx = match self.active_tab {
            Some(i) => i,
            None => return false,
        };
        let ext = self.extension();

        let is_image = matches!(
            ext.as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" | "svg"
        );

        if is_image {
            self.render_image_preview(ui, idx);
        } else {
            ui.centered_and_justified(|ui| {
                ui.vertical(|ui| {
                    ui.heading(format!("{} File", ext.to_uppercase()));
                    if let Some(data) = &self.tabs[idx].binary_data {
                        ui.label(format!("Size: {} B", data.len()));
                    }
                });
            });
        }
        false
    }

    fn render_image_preview(&mut self, ui: &mut egui::Ui, idx: usize) {
        let tab = &mut self.tabs[idx];
        let bytes = match &tab.binary_data {
            Some(b) => b,
            None => {
                ui.label("No data to display.");
                return;
            }
        };

        egui::ScrollArea::both()
            .id_salt("image_preview_scroll")
            .show(ui, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.add(
                        egui::Image::from_bytes(
                            format!("bytes://{}", tab.path.display()),
                            bytes.clone(),
                        )
                        .shrink_to_fit(),
                    );
                });
            });
    }
}
