use crate::app::project_config::trash_meta_path;
use crate::app::trash::{TrashEntryMeta, move_path_to_trash};
use crate::app::ui::background::spawn_task;
use crate::app::ui::file_tree::{DeleteJobResult, FileTree, RestoreJobResult};
use crate::app::ui::widgets::modal::{ModalResult, show_modal};
use crate::app::validation::is_safe_filename;
use std::fs;
use std::path::{Path, PathBuf};

// phase36-delete-scope-guard-enabled: this module stays delete-flow only.
fn map_delete_error_reason_key(engine_error: &str) -> &'static str {
    let normalized = engine_error.to_ascii_lowercase();
    if normalized.contains("interni `.polycredo/trash`") {
        "file-tree-delete-move-failed-reason-internal-trash"
    } else if normalized.contains("permission denied")
        || normalized.contains("opravnen")
        || normalized.contains("prava")
    {
        "file-tree-delete-move-failed-reason-permission"
    } else if normalized.contains("device or resource busy")
        || normalized.contains("used by another process")
        || normalized.contains("in use")
    {
        "file-tree-delete-move-failed-reason-locked"
    } else if normalized.contains("no such file")
        || normalized.contains("not found")
        || normalized.contains("uz neexistuje")
    {
        "file-tree-delete-move-failed-reason-missing"
    } else {
        "file-tree-delete-move-failed-reason-generic"
    }
}

pub(crate) fn format_delete_toast_error(i18n: &crate::i18n::I18n, engine_error: &str) -> String {
    let mut args = fluent_bundle::FluentArgs::new();
    args.set(
        "reason",
        i18n.get(map_delete_error_reason_key(engine_error)),
    );
    let reason = i18n.get_args("file-tree-delete-move-failed-reason", &args);
    format!(
        "{reason} {}",
        i18n.get("file-tree-delete-move-failed-guidance")
    )
}

impl FileTree {
    pub fn show_dialogs(&mut self, ui: &mut eframe::egui::Ui, i18n: &crate::i18n::I18n) {
        self.show_new_item_dialog(ui, i18n);
        self.show_rename_dialog(ui, i18n);
        self.show_delete_dialog(ui, i18n);
        self.show_restore_conflict_dialog(ui, i18n);
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
                                let detail = format!("trash move failed: {err}");
                                DeleteJobResult::Error(detail)
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

    fn show_restore_conflict_dialog(
        &mut self,
        ui: &mut eframe::egui::Ui,
        i18n: &crate::i18n::I18n,
    ) {
        let Some(conflict_path) = self.restore_conflict.clone() else {
            return;
        };
        let result = show_modal(
            ui.ctx(),
            "restore_conflict_modal",
            &i18n.get("file-tree-restore-conflict-title"),
            &i18n.get("file-tree-restore-as-copy"),
            &i18n.get("btn-cancel"),
            |ui| {
                ui.label(i18n.get("file-tree-restore-conflict-message"));
                Some(conflict_path.clone())
            },
        );

        match result {
            ModalResult::Confirmed(path) => {
                let root = self.root_path.clone();
                self.restore_rx = Some(spawn_task(move || {
                    match restore_from_trash_as_copy(&root, &path) {
                        Ok(restored_to) => RestoreJobResult::Restored(restored_to),
                        Err(err) => RestoreJobResult::Error(err),
                    }
                }));
                self.restore_conflict = None;
            }
            ModalResult::Cancelled => {
                self.restore_conflict = None;
            }
            ModalResult::Pending => {}
        }
    }
}

fn restore_from_trash_as_copy(
    project_root: &Path,
    trash_entry_path: &Path,
) -> Result<PathBuf, String> {
    let project_abs = project_root
        .canonicalize()
        .map_err(|e| format!("restore selhal: nelze kanonizovat root projektu: {e}"))?;
    let source_abs = trash_entry_path
        .canonicalize()
        .map_err(|e| format!("restore selhal: nelze kanonizovat trash source: {e}"))?;
    let source_name = source_abs
        .file_name()
        .ok_or_else(|| "restore selhal: trash source nema validni nazev".to_string())?;
    if !source_abs.starts_with(project_abs.join(".polycredo").join("trash")) {
        return Err(
            "restore selhal: source neni uvnitr `.polycredo/trash`; operace byla zastavena"
                .to_string(),
        );
    }

    let meta_path = trash_meta_path(&source_abs);
    let raw = fs::read_to_string(&meta_path)
        .map_err(|e| format!("restore selhal: nelze nacist metadata sidecar: {e}"))?;
    let meta: TrashEntryMeta = serde_json::from_str(&raw)
        .map_err(|e| format!("restore selhal: metadata nejsou validni JSON: {e}"))?;
    if meta.trash_name != source_name.to_string_lossy() {
        return Err(
            "restore selhal: metadata trash_name neodpovida nazvu trash polozky".to_string(),
        );
    }
    if meta.original_relative_path.is_absolute()
        || meta
            .original_relative_path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(
            "restore selhal: metadata original_relative_path je mimo bezpecny kontrakt".to_string(),
        );
    }

    let original_target = project_abs.join(&meta.original_relative_path);
    let restore_target = resolve_restore_copy_target(&original_target)?;
    if let Some(parent) = restore_target.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("restore selhal: nelze vytvorit parent adresare: {e}"))?;
    }

    fs::rename(&source_abs, &restore_target).map_err(|e| {
        format!("restore selhal: {e}; trash polozka zustava beze zmeny, zkontrolujte prava a zkuste znovu")
    })?;
    if let Err(cleanup_err) = fs::remove_file(&meta_path) {
        return match fs::rename(&restore_target, &source_abs) {
            Ok(_) => Err(format!(
                "restore selhal: operace byla vracena zpet, metadata cleanup selhal ({cleanup_err})"
            )),
            Err(rollback_err) => Err(format!(
                "restore selhal: metadata cleanup selhal ({cleanup_err}); rollback selhal ({rollback_err})"
            )),
        };
    }

    Ok(restore_target)
}

fn resolve_restore_copy_target(original_target: &Path) -> Result<PathBuf, String> {
    if !original_target.exists() {
        return Ok(original_target.to_path_buf());
    }
    let parent = original_target
        .parent()
        .ok_or_else(|| "restore selhal: cilova cesta nema parent adresar".to_string())?;
    let stem = original_target
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "restore selhal: cilovy nazev nema validni stem".to_string())?;
    let ext = original_target.extension().and_then(|s| s.to_str());
    for attempt in 1..=1000_u32 {
        let candidate_name = if let Some(ext) = ext {
            format!("{stem}-restored-copy-{attempt}.{ext}")
        } else {
            format!("{stem}-restored-copy-{attempt}")
        };
        let candidate = parent.join(candidate_name);
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err("restore selhal: nelze najit volny nazev pro obnovu jako kopii".to_string())
}
