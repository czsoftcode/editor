use crate::app::trash::move_path_to_trash;
use crate::app::ui::background::spawn_task;
use crate::app::ui::file_tree::{DeleteJobResult, FileTree};
use crate::app::ui::widgets::modal::{ModalResult, show_modal};
use crate::app::validation::is_safe_filename;

impl FileTree {
    pub fn show_dialogs(&mut self, ui: &mut eframe::egui::Ui, i18n: &crate::i18n::I18n) {
        self.show_new_item_dialog(ui, i18n);
        self.show_rename_dialog(ui, i18n);
        self.show_delete_dialog(ui, i18n);
    }

    fn show_new_item_dialog(&mut self, ui: &mut eframe::egui::Ui, i18n: &crate::i18n::I18n) {
        if self.new_item_parent.is_none() {
            return;
        }

        let title = if self.new_item_is_dir {
            i18n.get("file-tree-new-dir")
        } else {
            i18n.get("file-tree-new-file")
        };

        let result = show_modal(
            ui.ctx(),
            "new_item_modal",
            &title,
            &i18n.get("btn-create"),
            &i18n.get("btn-cancel"),
            |ui| {
                ui.horizontal(|ui| {
                    ui.label(i18n.get("btn-name-label"));
                    let response = ui.add(
                        eframe::egui::TextEdit::singleline(&mut self.new_item_buffer)
                            .font(eframe::egui::TextStyle::Body)
                            .desired_width(200.0),
                    );
                    if !response.has_focus() {
                        response.request_focus();
                    }
                });
                let name = self.new_item_buffer.trim().to_string();
                (!name.is_empty() && is_safe_filename(&name)).then_some(name)
            },
        );

        match result {
            ModalResult::Confirmed(name) => {
                if let Some(parent) = &self.new_item_parent {
                    let new_path = parent.join(&name);
                    // Safety check: path must remain within the project root
                    if !new_path.starts_with(&self.root_path) {
                        self.pending_error = Some(i18n.get("file-tree-outside-project"));
                    } else if self.new_item_is_dir {
                        match std::fs::create_dir(&new_path) {
                            Ok(()) => {
                                self.expand_to = Some(new_path);
                            }
                            Err(e) => {
                                let mut args = fluent_bundle::FluentArgs::new();
                                args.set("reason", e.to_string());
                                self.pending_error =
                                    Some(i18n.get_args("file-tree-create-dir-error", &args));
                            }
                        }
                        self.needs_reload = true;
                    } else {
                        match std::fs::write(&new_path, "") {
                            Ok(()) => {
                                self.pending_created_file = Some(new_path.clone());
                                self.expand_to = Some(new_path);
                            }
                            Err(e) => {
                                let mut args = fluent_bundle::FluentArgs::new();
                                args.set("reason", e.to_string());
                                self.pending_error =
                                    Some(i18n.get_args("file-tree-create-file-error", &args));
                            }
                        }
                        self.needs_reload = true;
                    }
                }
                self.new_item_parent = None;
            }
            ModalResult::Cancelled => {
                self.new_item_parent = None;
            }
            ModalResult::Pending => {}
        }
    }

    fn show_rename_dialog(&mut self, ui: &mut eframe::egui::Ui, i18n: &crate::i18n::I18n) {
        if self.rename_target.is_none() {
            return;
        }

        let result = show_modal(
            ui.ctx(),
            "rename_modal",
            &i18n.get("file-tree-rename"),
            &i18n.get("btn-rename"),
            &i18n.get("btn-cancel"),
            |ui| {
                ui.horizontal(|ui| {
                    ui.label(i18n.get("btn-name-label"));
                    let response = ui.add(
                        eframe::egui::TextEdit::singleline(&mut self.rename_buffer)
                            .font(eframe::egui::TextStyle::Body)
                            .desired_width(200.0),
                    );
                    if !response.has_focus() {
                        response.request_focus();
                    }
                });
                let name = self.rename_buffer.trim().to_string();
                (!name.is_empty() && is_safe_filename(&name)).then_some(name)
            },
        );

        match result {
            ModalResult::Confirmed(name) => {
                if let Some(target) = &self.rename_target
                    && let Some(parent) = target.parent()
                {
                    let new_path = parent.join(&name);
                    // Safety check: path must remain within the project root
                    if !new_path.starts_with(&self.root_path) {
                        self.pending_error = Some(i18n.get("file-tree-outside-project"));
                    } else {
                        match std::fs::rename(target, &new_path) {
                            Ok(()) => {
                                self.needs_reload = true;
                            }
                            Err(e) => {
                                let mut args = fluent_bundle::FluentArgs::new();
                                args.set("reason", e.to_string());
                                self.pending_error =
                                    Some(i18n.get_args("file-tree-rename-error", &args));
                            }
                        }
                    }
                }
                self.rename_target = None;
            }
            ModalResult::Cancelled => {
                self.rename_target = None;
            }
            ModalResult::Pending => {}
        }
    }

    fn show_delete_dialog(&mut self, ui: &mut eframe::egui::Ui, i18n: &crate::i18n::I18n) {
        if self.delete_confirm.is_none() {
            return;
        }

        let path_display = self
            .delete_confirm
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let mut args = fluent_bundle::FluentArgs::new();
        args.set("name", path_display.clone());
        let confirm_msg = i18n.get_args("file-tree-confirm-delete", &args);

        let result = show_modal(
            ui.ctx(),
            "delete_modal",
            &confirm_msg,
            &i18n.get("btn-delete"),
            &i18n.get("btn-cancel"),
            |ui| {
                ui.label(&path_display);
                Some(())
            },
        );

        match result {
            ModalResult::Confirmed(()) => {
                if let Some(path) = self.delete_confirm.take() {
                    let root = self.root_path.clone();
                    self.delete_rx =
                        Some(spawn_task(move || match move_path_to_trash(&root, &path) {
                            Ok(outcome) => DeleteJobResult::Deleted(outcome.moved_from),
                            Err(err) => {
                                DeleteJobResult::Error(format!("trash move failed: {err}"))
                            }
                        }));
                }
            }
            ModalResult::Cancelled => {
                self.delete_confirm = None;
            }
            ModalResult::Pending => {}
        }
    }
}
