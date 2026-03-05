use super::*;
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
        sandbox_mode_enabled: bool,
    ) -> EditorUiResult {
        let mut diff_action = None;

        if let Some((path, old_text, new_text)) = self.pending_ai_diff.clone() {
            let font_size = Self::current_editor_font_size(ui);
            if let Some(action) = super::diff_view::render_diff_modal(
                ui.ctx(),
                i18n,
                &path,
                &old_text,
                &new_text,
                font_size,
                settings.diff_side_by_side,
            ) {
                if action == super::DiffAction::Accepted
                    && let Some(tab) = self
                        .tabs
                        .iter_mut()
                        .find(|t| t.path.to_string_lossy() == path)
                {
                    tab.content = new_text.clone();
                    tab.modified = true;
                    tab.last_edit = Some(std::time::Instant::now());
                    tab.save_status = super::SaveStatus::Modified;
                }
                diff_action = Some((path, action, new_text));
                self.pending_ai_diff = None;
            }
        }

        if self.tabs.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(i18n.get("editor-empty-hint"));
            });
            return EditorUiResult {
                clicked: false,
                diff_action: None,
            };
        }

        // --- Tabs and bars ---
        use crate::app::ui::widgets::tab_bar::TabBarAction;
        let mut tab_action = None;
        self.tab_bar(ui, &mut tab_action);

        if let Some(action) = tab_action {
            match action {
                TabBarAction::Switch(idx) => {
                    self.active_tab = Some(idx);
                    self.focus_editor_requested = true;
                    self.update_search();
                }
                TabBarAction::Close(idx) => {
                    self.close_tab(idx);
                }
                TabBarAction::New => {}
            }
        }

        if self.show_goto_line {
            self.goto_line_bar(ui, i18n);
        }

        let current_diagnostics = lsp_client.and_then(|lsp| {
            if let Ok(uri) = async_lsp::lsp_types::Url::from_file_path(self.active_path()?) {
                let diag_map = lsp.diagnostics().lock().ok()?;
                diag_map.get(&uri).cloned()
            } else {
                None
            }
        });

        let is_readonly = if sandbox_mode_enabled {
            if let Some(path) = self.active_path() {
                !path.to_string_lossy().contains(".polycredo/sandbox")
            } else {
                false
            }
        } else {
            false
        };
        let theme_name = settings.syntect_theme_name();

        let clicked = if self.is_markdown() {
            self.ui_markdown_split(
                ui,
                dialog_open,
                i18n,
                current_diagnostics.as_ref(),
                lsp_client,
                is_readonly,
                theme_name,
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
                    let mut show_flag = true;

                    use crate::app::ui::widgets::modal::StandardModal;
                    let modal = StandardModal::new(i18n.get("svg-modal-title"), "svg_modal")
                        .with_size(450.0, 280.0);

                    modal.show(ui.ctx(), &mut show_flag, |ui| {
                        // FOOTER
                        if let Some((ext, edit)) = modal.ui_footer_actions(ui, i18n, |f| {
                            if f.close() || f.cancel() {
                                return Some((false, false));
                            }
                            let mut local_ext = false;
                            let mut local_edit = false;
                            if f.button("svg-modal-edit").clicked() {
                                local_edit = true;
                            }
                            if f.button("md-open-external").clicked() {
                                local_ext = true;
                            }
                            if local_ext || local_edit {
                                Some((local_ext, local_edit))
                            } else {
                                None
                            }
                        }) {
                            open_external = ext;
                            edit_as_text = edit;
                        }

                        // BODY
                        modal.ui_body(ui, |ui| {
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new(&fname).strong());
                            ui.add_space(8.0);
                            ui.label(i18n.get("svg-modal-body"));
                            ui.add_space(16.0);
                        });
                    });

                    if !show_flag {
                        edit_as_text = true; // Dismiss is effectively "stay in editor"
                    }

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
                is_readonly,
                theme_name,
            )
        };

        EditorUiResult {
            clicked,
            diff_action,
        }
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
        let visuals = ui.visuals();
        // Theme-aware status bar palette closes UAT gap for low contrast in light mode.
        let primary_color = visuals.text_color();
        let secondary_color = visuals.weak_text_color();
        let (diag_error_color, diag_warning_color, status_warn_color, status_ok_color) =
            if visuals.dark_mode {
                (
                    egui::Color32::from_rgb(240, 100, 100),
                    egui::Color32::from_rgb(250, 180, 80),
                    egui::Color32::from_rgb(255, 200, 120),
                    egui::Color32::from_rgb(170, 230, 185),
                )
            } else {
                (
                    egui::Color32::from_rgb(170, 45, 45),
                    egui::Color32::from_rgb(155, 95, 20),
                    egui::Color32::from_rgb(155, 95, 20),
                    egui::Color32::from_rgb(25, 120, 70),
                )
            };

        let cursor_text = tab.last_cursor_range.map(|cr| {
            let rc = cr.primary.rcursor;
            format!("{}:{}", rc.row + 1, rc.column + 1)
        });

        let file_type = ext_to_file_type(&self.extension());

        // Diagnostic counts for active file
        let (error_count, warning_count) = 'diag: {
            let Some(lsp) = lsp_client else {
                break 'diag (0, 0);
            };
            let Ok(uri) = async_lsp::lsp_types::Url::from_file_path(&tab.path) else {
                break 'diag (0, 0);
            };
            let diag_map = lsp.diagnostics().lock().expect("lock diagnostics");
            let Some(diags) = diag_map.get(&uri) else {
                break 'diag (0, 0);
            };
            let errors = diags
                .iter()
                .filter(|d| d.severity == Some(async_lsp::lsp_types::DiagnosticSeverity::ERROR))
                .count();
            let warnings = diags
                .iter()
                .filter(|d| d.severity == Some(async_lsp::lsp_types::DiagnosticSeverity::WARNING))
                .count();
            (errors, warnings)
        };

        egui::TopBottomPanel::bottom("status_bar")
            .resizable(false)
            .default_height(24.0)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);

                    // Left side: Branch & Path
                    if let Some(branch) = git_branch {
                        ui.label(
                            egui::RichText::new(format!(" {}", branch)).color(secondary_color),
                        );
                        ui.add_space(8.0);
                    }

                    ui.label(egui::RichText::new(tab.path.to_string_lossy()).color(primary_color));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);

                        // Right side: Encoding, Type, Cursor
                        ui.label(
                            egui::RichText::new(i18n.get("statusbar-encoding"))
                                .color(secondary_color),
                        );
                        ui.add_space(12.0);

                        ui.label(egui::RichText::new(file_type).color(secondary_color));
                        ui.add_space(12.0);

                        if let Some(pos) = cursor_text {
                            let mut args = fluent_bundle::FluentArgs::new();
                            let parts: Vec<&str> = pos.split(':').collect();
                            args.set("line", parts[0]);
                            args.set("col", parts[1]);
                            ui.label(
                                egui::RichText::new(i18n.get_args("statusbar-line-col", &args))
                                    .color(primary_color),
                            );
                        }

                        // Diagnostics
                        if error_count > 0 {
                            ui.add_space(12.0);
                            ui.label(
                                egui::RichText::new(format!("✕ {}", error_count))
                                    .color(diag_error_color),
                            );
                        }
                        if warning_count > 0 {
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(format!("⚠ {}", warning_count))
                                    .color(diag_warning_color),
                            );
                        }

                        // LSP status
                        if let Some(lsp) = lsp_client
                            && !lsp.is_initialized()
                        {
                            ui.add_space(12.0);
                            ui.label(
                                egui::RichText::new(i18n.get("statusbar-lsp-initializing"))
                                    .color(status_warn_color),
                            );
                        }

                        // Save status
                        match tab.save_status {
                            super::SaveStatus::None => {}
                            super::SaveStatus::Modified => {
                                ui.add_space(12.0);
                                ui.label(
                                    egui::RichText::new(i18n.get("statusbar-unsaved"))
                                        .color(status_warn_color),
                                );
                            }
                            super::SaveStatus::Saving => {
                                ui.add_space(12.0);
                                ui.label(
                                    egui::RichText::new(i18n.get("statusbar-saving"))
                                        .color(status_warn_color),
                                );
                            }
                            super::SaveStatus::Saved => {
                                ui.add_space(12.0);
                                ui.label(
                                    egui::RichText::new(i18n.get("statusbar-saved"))
                                        .color(status_ok_color),
                                );
                            }
                        }
                    });
                });
            });
    }
}
