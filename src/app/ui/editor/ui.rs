use super::*;
use eframe::egui;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SaveStatusPresentation {
    key: &'static str,
    is_primary: bool,
    tone: SaveStatusTone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SaveStatusTone {
    Warning,
    Success,
    Neutral,
}

fn save_status_presentation(status: &super::SaveStatus) -> Option<SaveStatusPresentation> {
    match status {
        super::SaveStatus::None => None,
        // Dirty stav je primární UX signál.
        super::SaveStatus::Modified => Some(SaveStatusPresentation {
            key: "statusbar-unsaved",
            is_primary: true,
            tone: SaveStatusTone::Warning,
        }),
        super::SaveStatus::Saving => Some(SaveStatusPresentation {
            key: "statusbar-saving",
            is_primary: false,
            tone: SaveStatusTone::Neutral,
        }),
        super::SaveStatus::Saved => Some(SaveStatusPresentation {
            key: "statusbar-saved",
            is_primary: false,
            tone: SaveStatusTone::Success,
        }),
    }
}

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
    ) -> EditorUiResult {
        let mut diff_action = None;
        let mut result_tab_action = None;

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
                tab_action: None,
            };
        }

        // --- Tabs and bars ---
        use crate::app::ui::widgets::tab_bar::TabBarAction;
        let mut tab_action = None;
        self.tab_bar(ui, &mut tab_action, settings, i18n);

        if let Some(action) = tab_action {
            match action {
                TabBarAction::Switch(idx) => {
                    self.active_tab = Some(idx);
                    self.focus_editor_requested = true;
                    self.update_search();
                }
                TabBarAction::Close(idx) => {
                    // Defer handling of tab close to the workspace so that
                    // unsaved close guard can run through a single entry point.
                    result_tab_action = Some(TabBarAction::Close(idx));
                }
                TabBarAction::New => {}
                TabBarAction::ShowHistory(idx) => {
                    result_tab_action = Some(TabBarAction::ShowHistory(idx));
                }
            }
        }

        if self.show_search {
            self.search_bar(ui, i18n);
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

        let is_readonly = false;
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
            tab_action: result_tab_action,
        }
    }

    /// Renderuje obsah status baru do předaného horizontálního `ui`.
    /// Globální prvky (save mode, branch, heart) se zobrazují vždy,
    /// per-soubor prvky (cesta, cursor, typ, encoding, diagnostiky) jen s aktivním tabem.
    /// Vrací `true` pokud uživatel klikl na support tlačítko.
    pub fn status_bar(
        &self,
        ui: &mut egui::Ui,
        git_branch: Option<&str>,
        save_mode_label: Option<&str>,
        is_indexing: bool,
        i18n: &crate::i18n::I18n,
        lsp_client: Option<&crate::app::lsp::LspClient>,
    ) -> bool {
        let dark_mode = ui.visuals().dark_mode;
        let primary_color = ui.visuals().text_color();
        let secondary_color = ui.visuals().weak_text_color();

        let mut support_clicked = false;

        // ── Levá strana (globální): save mode → branch ──
        ui.add_space(8.0);

        if let Some(mode_label) = save_mode_label {
            ui.label(
                egui::RichText::new(mode_label)
                    .small()
                    .color(secondary_color),
            );
            ui.separator();
        }

        if let Some(branch) = git_branch {
            ui.label(egui::RichText::new(format!(" {}", branch)).color(secondary_color));
            ui.add_space(8.0);
        }

        // ── Per-soubor prvky (jen s aktivním tabem) ──
        if let Some(tab) = self.active() {
            let (diag_error_color, diag_warning_color, status_warn_color, status_ok_color) =
                if dark_mode {
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
                    .filter(|d| {
                        d.severity == Some(async_lsp::lsp_types::DiagnosticSeverity::WARNING)
                    })
                    .count();
                (errors, warnings)
            };

            // Cesta k souboru (levá strana, per-soubor)
            ui.label(egui::RichText::new(tab.path.to_string_lossy()).color(primary_color));

            // Pravá strana: ❤️ → encoding → typ → cursor → diagnostiky → LSP → save status → indexing
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);

                let support_btn = ui
                    .selectable_label(false, "❤️")
                    .on_hover_text(i18n.get("menu-help-support"));
                if support_btn.clicked() {
                    support_clicked = true;
                }

                ui.separator();

                ui.label(
                    egui::RichText::new(i18n.get("statusbar-encoding")).color(secondary_color),
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

                if error_count > 0 {
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new(format!("✕ {}", error_count)).color(diag_error_color),
                    );
                }
                if warning_count > 0 {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(format!("⚠ {}", warning_count))
                            .color(diag_warning_color),
                    );
                }

                if let Some(lsp) = lsp_client
                    && !lsp.is_initialized()
                {
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new(i18n.get("statusbar-lsp-initializing"))
                            .color(status_warn_color),
                    );
                }

                if let Some(status) = save_status_presentation(&tab.save_status) {
                    ui.add_space(12.0);
                    let color = match status.tone {
                        SaveStatusTone::Warning => status_warn_color,
                        SaveStatusTone::Success => status_ok_color,
                        SaveStatusTone::Neutral => secondary_color,
                    };
                    let text = egui::RichText::new(i18n.get(status.key)).color(color);
                    ui.label(if status.is_primary {
                        text.strong()
                    } else {
                        text
                    });
                }

                if is_indexing {
                    ui.add_space(8.0);
                    ui.spinner();
                    ui.label(
                        egui::RichText::new(i18n.get("semantic-indexing-status-bar"))
                            .small()
                            .color(egui::Color32::from_rgb(100, 200, 255)),
                    );
                }
            });
        } else {
            // Žádný tab — jen heart vpravo
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                let support_btn = ui
                    .selectable_label(false, "❤️")
                    .on_hover_text(i18n.get("menu-help-support"));
                if support_btn.clicked() {
                    support_clicked = true;
                }
            });
        }

        support_clicked
    }
}

#[cfg(test)]
mod tests {
    use super::super::SaveStatus;
    use super::save_status_presentation;

    #[test]
    fn dirty_state_visual_priority_marks_modified_as_primary_signal() {
        let modified = save_status_presentation(&SaveStatus::Modified);
        let saving = save_status_presentation(&SaveStatus::Saving);

        assert!(modified.is_some());
        assert!(saving.is_some());
        let modified = modified.expect("modified presentation");
        let saving = saving.expect("saving presentation");

        assert!(modified.is_primary);
        assert!(!saving.is_primary);
    }

    #[test]
    fn dirty_state_visual_priority_keeps_mode_status_secondary() {
        let modified =
            save_status_presentation(&SaveStatus::Modified).expect("modified presentation");
        let saved = save_status_presentation(&SaveStatus::Saved).expect("saved presentation");

        assert!(modified.is_primary);
        assert!(!saved.is_primary);
    }
}
