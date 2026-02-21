use super::*;
use crate::app::ui::widgets::tab_bar::TabBarAction;
use async_lsp::lsp_types::Url;
use eframe::egui;

impl Editor {
    // --- UI entry point ---

    /// Returns `true` if the user clicked in the editor.
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        dialog_open: bool,
        i18n: &crate::i18n::I18n,
        lsp_client: Option<&crate::app::lsp::LspClient>,
        settings: &crate::settings::Settings,
    ) -> bool {
        if self.tabs.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(i18n.get("editor-empty-hint"));
            });
            return false;
        }

        let mut tab_action = None;
        self.tab_bar(ui, &mut tab_action);
        match tab_action {
            Some(TabBarAction::Switch(idx)) => {
                self.active_tab = Some(idx);
                self.focus_editor_requested = true;
                self.update_search();
            }
            Some(TabBarAction::Close(idx)) => {
                if let Some(lsp) = lsp_client
                    && let Some(tab) = self.tabs.get(idx)
                    && tab.lsp_version > 0
                    && let Ok(uri) = Url::from_file_path(&tab.path)
                {
                    lsp.notify_did_close(uri);
                }
                self.close_tab(idx);
            }
            Some(TabBarAction::New) | None => {}
        }

        if self.tabs.is_empty() {
            return false;
        }

        let ctrl_f = ui
            .ctx()
            .input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::F));
        let ctrl_h = ui
            .ctx()
            .input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::H));
        let ctrl_g = ui
            .ctx()
            .input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::G));
        let escape = ui.ctx().input(|i| i.key_pressed(egui::Key::Escape));

        if ctrl_f {
            self.show_search = true;
            self.show_replace = false;
            self.search_focus_requested = true;
            self.show_goto_line = false;
            self.goto_line_focus_requested = false;
            self.update_search();
        }
        if ctrl_h {
            self.show_search = true;
            self.show_replace = true;
            self.search_focus_requested = true;
            self.show_goto_line = false;
            self.goto_line_focus_requested = false;
            self.update_search();
        }
        if ctrl_g {
            self.show_goto_line = !self.show_goto_line;
            if self.show_goto_line {
                self.goto_line_input.clear();
                self.show_search = false;
                self.goto_line_focus_requested = true;
            } else {
                self.goto_line_focus_requested = false;
            }
        }
        if escape {
            if self.show_search {
                self.show_search = false;
                self.show_replace = false;
                self.search_matches.clear();
                self.current_match = None;
            } else if self.show_goto_line {
                self.show_goto_line = false;
                self.goto_line_focus_requested = false;
            }
        }

        if self.show_search {
            self.search_bar(ui, i18n);
        }
        if self.show_goto_line {
            self.goto_line_bar(ui, i18n);
        }

        // Send didOpen for the active tab — only after LSP handshake is complete
        // For now, we only support Rust LSP (rust-analyzer), so we only notify for .rs files.
        if let (Some(idx), Some(lsp)) = (self.active_tab, lsp_client) {
            let tab = &mut self.tabs[idx];
            if tab.lsp_version == 0
                && !tab.is_binary
                && lsp.is_initialized()
                && let Ok(uri) = Url::from_file_path(&tab.path)
            {
                let lang_id = lang_id_from_path(&tab.path);
                if lang_id == "rust" {
                    lsp.notify_did_open(uri, lang_id, 1, tab.content.clone());
                    tab.lsp_version = 1;
                }
            }
        }

        let is_binary = self.active().is_some_and(|t| t.is_binary);

        let current_diagnostics: Option<Vec<async_lsp::lsp_types::Diagnostic>> = lsp_client
            .and_then(|lsp| {
                let tab = self.active()?;
                if tab.lsp_version == 0 {
                    return None;
                }
                Url::from_file_path(&tab.path)
                    .ok()
                    .and_then(|uri| lsp.diagnostics().lock().unwrap().get(&uri).cloned())
            });

        let clicked = if is_binary {
            self.ui_binary(ui, i18n)
        } else if self.is_markdown() {
            self.ui_markdown_split(
                ui,
                dialog_open,
                i18n,
                current_diagnostics.as_ref(),
                lsp_client,
            )
        } else {
            if self.is_svg() {
                if let Some(idx) = self.active_tab
                    && !self.tabs[idx].svg_modal_shown
                {
                    let path = self.tabs[idx].path.clone();
                    let fname = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    let mut open_external = false;
                    let mut edit_as_text = false;

                    let modal = egui::Modal::new(egui::Id::new(("svg_modal", &path)));
                    modal.show(ui.ctx(), |ui| {
                        ui.heading(i18n.get("svg-modal-title"));
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(&fname).strong());
                        ui.add_space(8.0);
                        ui.label(i18n.get("svg-modal-body"));
                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            if ui.button(i18n.get("svg-open-external")).clicked() {
                                open_external = true;
                            }
                            if ui.button(i18n.get("svg-modal-edit")).clicked() {
                                edit_as_text = true;
                            }
                        });
                    });

                    if open_external {
                        let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
                        self.tabs[idx].svg_modal_shown = true;
                    }
                    if edit_as_text {
                        self.tabs[idx].svg_modal_shown = true;
                    }
                }

                // Button always visible — user can use it anytime during editing
                if let Some(path) = self.active_path().cloned() {
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(i18n.get("svg-open-external")).clicked() {
                                let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
                            }
                        });
                    });
                    ui.separator();
                }
            }
            self.ui_normal(
                ui,
                dialog_open,
                i18n,
                current_diagnostics.as_ref(),
                lsp_client,
            )
        };

        if let Some((path, old_text, new_text)) = self.pending_ai_diff.clone() {
            let font_size = Self::current_editor_font_size(ui);
            let diff_res = super::diff_view::render_diff_modal(
                ui.ctx(),
                i18n,
                &path,
                &old_text,
                &new_text,
                font_size,
                settings.diff_side_by_side,
            );
            if diff_res.accepted || diff_res.rejected {
                if diff_res.accepted
                    && let Some(tab) = self
                        .tabs
                        .iter_mut()
                        .find(|t| t.path.to_string_lossy() == path)
                {
                    tab.content = new_text;
                    tab.modified = true;
                    tab.last_edit = Some(std::time::Instant::now());
                    tab.save_status = super::SaveStatus::Modified;
                }
                self.pending_ai_diff = None;
            }
        }

        clicked
    }

    pub fn status_bar(
        &self,
        ui: &mut egui::Ui,
        git_branch: Option<&str>,
        i18n: &crate::i18n::I18n,
        lsp_client: Option<&crate::app::lsp::LspClient>,
    ) {
        let tab = match self.active() {
            Some(t) => t,
            None => return,
        };
        let primary_color = egui::Color32::from_rgb(235, 240, 248);
        let secondary_color = egui::Color32::from_rgb(195, 205, 220);
        let status_warn_color = egui::Color32::from_rgb(255, 200, 120);
        let status_ok_color = egui::Color32::from_rgb(170, 230, 185);

        let cursor_text = tab.last_cursor_range.map(|cr| {
            let rc = cr.primary.rcursor;
            format!("{}:{}", rc.row + 1, rc.column + 1)
        });

        let file_type = ext_to_file_type(&self.extension());

        // Diagnostic counts for active file
        let (error_count, warning_count) = 'diag: {
            let Some(lsp) = lsp_client else {
                break 'diag (0usize, 0usize);
            };
            let Some(path) = self.active_path() else {
                break 'diag (0, 0);
            };
            let Ok(uri) = Url::from_file_path(path) else {
                break 'diag (0, 0);
            };
            let Ok(diags) = lsp.diagnostics().lock() else {
                break 'diag (0, 0);
            };
            let Some(file_diags) = diags.get(&uri) else {
                break 'diag (0, 0);
            };
            let errors = file_diags
                .iter()
                .filter(|d| d.severity == Some(async_lsp::lsp_types::DiagnosticSeverity::ERROR))
                .count();
            let warnings = file_diags
                .iter()
                .filter(|d| d.severity == Some(async_lsp::lsp_types::DiagnosticSeverity::WARNING))
                .count();
            (errors, warnings)
        };

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(tab.path.to_string_lossy().to_string()).color(primary_color),
            );
            ui.separator();

            if let Some((msg, time)) = &self.lsp_status
                && time.elapsed().as_secs() < 3
            {
                ui.label(egui::RichText::new(msg).color(status_ok_color));
                ui.separator();
            }

            match tab.save_status {
                SaveStatus::None => {}
                SaveStatus::Modified => {
                    ui.label(
                        egui::RichText::new(i18n.get("statusbar-unsaved")).color(status_warn_color),
                    );
                }
                SaveStatus::Saving => {
                    ui.label(
                        egui::RichText::new(i18n.get("statusbar-saving")).color(secondary_color),
                    );
                }
                SaveStatus::Saved => {
                    ui.label(
                        egui::RichText::new(i18n.get("statusbar-saved")).color(status_ok_color),
                    );
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(pos) = cursor_text {
                    ui.label(egui::RichText::new(pos).monospace().color(primary_color));
                    ui.separator();
                }
                if let Some(branch) = git_branch {
                    ui.label(
                        egui::RichText::new(format!("\u{2387} {}", branch)).color(status_ok_color),
                    );
                    ui.separator();
                }
                // Diagnostic counts — right-to-left: warnings first, then errors
                if warning_count > 0 {
                    ui.label(
                        egui::RichText::new(format!("\u{26A0} {}", warning_count))
                            .color(egui::Color32::from_rgb(255, 180, 0)),
                    );
                }
                if error_count > 0 {
                    if warning_count > 0 {
                        ui.separator();
                    }
                    ui.label(
                        egui::RichText::new(format!("\u{2715} {}", error_count))
                            .color(egui::Color32::from_rgb(255, 80, 80)),
                    );
                }
                if error_count > 0 || warning_count > 0 {
                    ui.separator();
                }
                ui.label(
                    egui::RichText::new(i18n.get("statusbar-encoding")).color(secondary_color),
                );
                ui.separator();
                ui.label(egui::RichText::new(file_type).color(secondary_color));
            });
        });
        ui.separator();
    }
}
