use crate::app::trash::{TrashEntryKind, restore_from_trash};
use crate::app::ui::background::spawn_task;
use crate::app::ui::file_tree::{FileTree, RestoreJobResult};
use crate::app::ui::widgets::modal::{ModalResult, show_modal};

impl FileTree {
    pub(crate) fn show_trash_preview_dialog(
        &mut self,
        ui: &mut eframe::egui::Ui,
        i18n: &crate::i18n::I18n,
    ) {
        if !self.trash_preview_open {
            return;
        }

        let preview_title = i18n.get("file-tree-trash-preview-title");

        let result = show_modal(
            ui.ctx(),
            "preview_modal",
            &preview_title,
            &i18n.get("file-tree-trash-preview-restore"),
            &i18n.get("btn-close"),
            |ui| {
                ui.horizontal(|ui| {
                    ui.label(i18n.get("file-tree-trash-preview-filter"));
                    ui.add(
                        eframe::egui::TextEdit::singleline(&mut self.trash_preview_filter)
                            .desired_width(ui.available_width()),
                    );
                });
                ui.add_space(8.0);
                if self.trash_preview_loading {
                    ui.label(i18n.get("file-tree-trash-preview-loading"));
                }

                let filter = self.trash_preview_filter.to_ascii_lowercase();
                let mut visible_count = 0_usize;
                eframe::egui::ScrollArea::vertical()
                    .max_height(320.0)
                    .show(ui, |ui| {
                        for item in &self.trash_preview_items {
                            let original = item
                                .original_relative_path
                                .as_ref()
                                .map(|p| p.display().to_string())
                                .unwrap_or_else(|| "-".to_string());
                            let searchable =
                                format!("{} {}", item.name, original).to_ascii_lowercase();
                            if !filter.is_empty() && !searchable.contains(&filter) {
                                continue;
                            }
                            visible_count += 1;
                            let selected = self
                                .trash_preview_selected
                                .as_ref()
                                .is_some_and(|path| path == &item.trash_path);
                            let kind_key = match item.entry_kind {
                                TrashEntryKind::File => "file-tree-trash-preview-kind-file",
                                TrashEntryKind::Directory => "file-tree-trash-preview-kind-dir",
                            };
                            let deleted_at = item
                                .deleted_at
                                .map(|value| value.to_string())
                                .unwrap_or_else(|| "-".to_string());
                            let label = format!(
                                "{} · {} · {} · {}",
                                item.name,
                                i18n.get(kind_key),
                                deleted_at,
                                original
                            );
                            if ui.selectable_label(selected, label).clicked() {
                                self.trash_preview_selected = Some(item.trash_path.clone());
                            }
                        }
                    });

                if visible_count == 0 && !self.trash_preview_loading {
                    ui.label(i18n.get("file-tree-trash-preview-no-results"));
                }
                self.trash_preview_selected.clone()
            },
        );

        match result {
            ModalResult::Confirmed(selected_path) => {
                if self.restore_rx.is_some() {
                    return;
                }
                let root = self.root_path.clone();
                self.restore_rx = Some(spawn_task(move || {
                    match restore_from_trash(&root, &selected_path) {
                        Ok(outcome) => RestoreJobResult::Restored(outcome.restored_to),
                        Err(err) => {
                            let detail = err.to_string();
                            if detail.contains("konflikt: cilova cesta uz existuje") {
                                RestoreJobResult::Conflict(selected_path)
                            } else {
                                RestoreJobResult::Error(detail)
                            }
                        }
                    }
                }));
            }
            ModalResult::Cancelled => {
                self.trash_preview_open = false;
                self.trash_preview_filter.clear();
                self.trash_preview_selected = None;
            }
            ModalResult::Pending => {}
        }
    }
}
