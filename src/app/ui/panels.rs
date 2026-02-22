use std::path::PathBuf;

use eframe::egui;

use super::super::build_runner::run_build_check;
use super::super::types::{FocusedPanel, Toast};
use super::workspace::{WorkspaceState, open_and_jump, open_file_in_ws};
use crate::config;

/// Renders the left panel (file tree + build terminal). Returns true if the terminal was clicked.
pub(super) fn render_left_panel(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    dialog_open: bool,
    i18n: &crate::i18n::I18n,
) -> bool {
    if !ws.show_left_panel {
        return false;
    }
    let mut any_clicked = false;
    let focused = ws.focused_panel;

    egui::SidePanel::left("left_panel")
        .default_width(config::LEFT_PANEL_DEFAULT_WIDTH)
        .width_range(config::LEFT_PANEL_MIN_WIDTH..=config::LEFT_PANEL_MAX_WIDTH)
        .resizable(true)
        .show(ctx, |ui| {
            let total_height = ui.available_height();
            let tree_height = if ws.show_build_terminal {
                (total_height * 0.55).max(100.0)
            } else {
                total_height
            };

            egui::Frame::NONE.show(ui, |ui| {
                ui.set_max_height(tree_height);

                ui.horizontal(|ui| {
                    let title = if ws.file_tree_in_sandbox {
                        egui::RichText::new(i18n.get("panel-files-sandbox"))
                            .color(egui::Color32::from_rgb(255, 230, 100))
                            .strong()
                    } else {
                        egui::RichText::new(i18n.get("panel-files")).strong()
                    };
                    ui.heading(title);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let prev_in_sandbox = ws.file_tree_in_sandbox;
                        if ui
                            .selectable_label(
                                !ws.file_tree_in_sandbox,
                                i18n.get("btn-tree-project"),
                            )
                            .clicked()
                        {
                            ws.file_tree_in_sandbox = false;
                        }
                        if ui
                            .selectable_label(ws.file_tree_in_sandbox, i18n.get("btn-tree-sandbox"))
                            .clicked()
                        {
                            ws.file_tree_in_sandbox = true;
                        }

                        if ws.file_tree_in_sandbox != prev_in_sandbox {
                            let target_dir = if ws.file_tree_in_sandbox {
                                &ws.sandbox.root
                            } else {
                                &ws.root_path
                            };
                            ws.file_tree.load(target_dir);
                        }
                    });
                });

                ui.separator();
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .id_salt("file_tree_scroll")
                    .show(ui, |ui| {
                        let result = ws.file_tree.ui(ui, i18n);
                        if let Some(err) = ws.file_tree.take_error() {
                            ws.toasts.push(Toast::error(err));
                        }
                        if let Some(path) = result.selected {
                            open_file_in_ws(ws, path);
                        }
                        if let Some(path) = result.created_file {
                            open_file_in_ws(ws, path);
                        }
                        if let Some(deleted) = result.deleted {
                            ws.editor.close_tabs_for_path(&deleted);
                        }
                    });
            });

            if ws.show_build_terminal {
                ui.separator();
                render_build_panel(ui, ws, dialog_open, focused, &mut any_clicked, i18n);
            }
        });
    any_clicked
}

/// Renders the build panel and error list inside the left panel.
fn render_build_panel(
    ui: &mut egui::Ui,
    ws: &mut WorkspaceState,
    dialog_open: bool,
    focused: FocusedPanel,
    any_clicked: &mut bool,
    i18n: &crate::i18n::I18n,
) {
    ui.horizontal(|ui| {
        ui.strong(i18n.get("panel-build"));
        ui.separator();

        // Sandbox Mode Toggle
        let prev_in_sandbox = ws.build_in_sandbox;
        let sandbox_label = if ws.build_in_sandbox {
            egui::RichText::new(i18n.get("btn-build-sandbox-on"))
                .color(egui::Color32::from_rgb(255, 230, 100))
                .strong()
        } else {
            egui::RichText::new(i18n.get("btn-build-sandbox-off"))
        };

        if ui
            .selectable_label(ws.build_in_sandbox, sandbox_label)
            .on_hover_text(i18n.get("hover-build-sandbox"))
            .clicked()
        {
            ws.build_in_sandbox = !ws.build_in_sandbox;
        }

        if ws.build_in_sandbox != prev_in_sandbox {
            // Directory changed -> restart terminal
            let target_dir = if ws.build_in_sandbox {
                &ws.sandbox.root
            } else {
                &ws.root_path
            };
            ws.next_terminal_id += 1;
            ws.build_terminal = Some(super::terminal::Terminal::new(
                ws.next_terminal_id,
                ui.ctx(),
                target_dir,
                None,
            ));
        }

        ui.separator();

        // Runner Profile Dropdown
        let combo = egui::ComboBox::from_id_salt("runner_select")
            .selected_text(i18n.get("btn-run-profile"))
            .width(130.0);

        combo.show_ui(ui, |ui| {
            if ws.profiles.runners.is_empty() {
                ui.weak(i18n.get("runner-none"));
            } else {
                let mut run_profile_idx = None;
                for (idx, profile) in ws.profiles.runners.iter().enumerate() {
                    if ui
                        .selectable_label(false, format!("▶ {}", profile.name))
                        .clicked()
                    {
                        run_profile_idx = Some(idx);
                    }
                }

                if let Some(idx) = run_profile_idx {
                    let profile = ws.profiles.runners[idx].clone();
                    let target_dir = if ws.build_in_sandbox {
                        &ws.sandbox.root
                    } else {
                        &ws.root_path
                    };
                    let terminal = crate::app::build_runner::run_profile(
                        ui.ctx(),
                        target_dir,
                        &profile,
                        &mut ws.next_terminal_id,
                    );
                    ws.build_terminal = Some(terminal);
                    ws.show_build_terminal = true;
                    ws.focused_panel = FocusedPanel::Build;
                    // For Rust profiles, also start error check
                    if profile.error_parser == crate::app::types::ErrorParserType::Rust {
                        ws.build_error_rx = Some(run_build_check(target_dir.clone()));
                        ws.build_errors.clear();
                    }
                }
            }
            ui.separator();
            if ui.button(i18n.get("btn-edit-profiles")).clicked() {
                let profiles_path = crate::app::project_config::profiles_path(&ws.root_path);
                open_file_in_ws(ws, profiles_path);
            }
        });

        ui.add_enabled_ui(!ws.build_in_sandbox, |ui| {
            if ui
                .button(i18n.get("btn-create-deb"))
                .on_disabled_hover_text(i18n.get("hover-create-deb-disabled"))
                .clicked()
            {
                let cmd = "./packaging/deb/build-deb.sh";
                ws.next_terminal_id += 1;
                let terminal = super::terminal::Terminal::new(
                    ws.next_terminal_id,
                    ui.ctx(),
                    &ws.root_path,
                    Some(cmd),
                );
                ws.build_terminal = Some(terminal);
                ws.show_build_terminal = true;
                ws.focused_panel = FocusedPanel::Build;
            }
        });
    });

    if !ws.build_in_sandbox {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.strong(i18n.get("panel-git"));
            ui.separator();

            let git_combo = egui::ComboBox::from_id_salt("git_select")
                .selected_text(i18n.get("btn-git-profile"))
                .width(130.0);

            git_combo.show_ui(ui, |ui| {
                let mut git_cmd = None;

                if ui.button(i18n.get("git-status")).clicked() {
                    git_cmd = Some("git status");
                }
                if ui.button(i18n.get("git-diff")).clicked() {
                    git_cmd = Some("git diff");
                }
                ui.separator();
                if ui.button(i18n.get("git-add-all")).clicked() {
                    git_cmd = Some("git add .");
                }
                if ui.button(i18n.get("git-commit")).clicked() {
                    git_cmd = Some("git commit -m '");
                }
                if ui.button(i18n.get("git-push")).clicked() {
                    git_cmd = Some("git push");
                }
                if ui.button(i18n.get("git-pull")).clicked() {
                    git_cmd = Some("git pull");
                }
                ui.separator();
                if ui.button(i18n.get("git-checkout-file")).clicked() {
                    git_cmd = Some("git checkout -- ");
                }
                if ui.button(i18n.get("git-checkout-branch")).clicked() {
                    git_cmd = Some("git checkout ");
                }
                if ui.button(i18n.get("git-reset-hard")).clicked() {
                    git_cmd = Some("git reset --hard HEAD");
                }

                if let Some(cmd) = git_cmd {
                    ws.next_terminal_id += 1;
                    let terminal = super::terminal::Terminal::new(
                        ws.next_terminal_id,
                        ui.ctx(),
                        &ws.root_path,
                        Some(cmd),
                    );
                    ws.build_terminal = Some(terminal);
                    ws.show_build_terminal = true;
                    ws.focused_panel = FocusedPanel::Build;
                    ui.close_menu();
                }
            });
        });
    }
    ui.separator();

    if !dialog_open && let Some(terminal) = &mut ws.build_terminal {
        let terminal_action = terminal.ui(
            ui,
            focused == FocusedPanel::Build,
            config::EDITOR_FONT_SIZE,
            i18n,
        );
        match terminal_action {
            Some(super::terminal::TerminalAction::Clicked)
            | Some(super::terminal::TerminalAction::Hovered) => {
                ws.focused_panel = FocusedPanel::Build;
                *any_clicked = true;
            }
            Some(super::terminal::TerminalAction::Navigate(path, line, col)) => {
                let abs_path = if path.is_absolute() {
                    path
                } else {
                    ws.root_path.join(path)
                };
                open_file_in_ws(ws, abs_path);
                ws.editor.jump_to_location(line, col);
                ws.focused_panel = FocusedPanel::Editor;
                *any_clicked = true;
            }
            None => {}
        }
    }

    if !ws.build_errors.is_empty() {
        ui.separator();
        let mut err_args = fluent_bundle::FluentArgs::new();
        err_args.set("count", ws.build_errors.len() as i64);
        ui.label(
            egui::RichText::new(i18n.get_args("panel-build-errors", &err_args))
                .strong()
                .size(12.0),
        );
        let mut open_error_file: Option<(PathBuf, usize)> = None;
        egui::ScrollArea::vertical()
            .id_salt("build_errors_scroll")
            .max_height(config::BUILD_ERROR_LIST_MAX_HEIGHT)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for error in &ws.build_errors {
                    let color = if error.is_warning {
                        egui::Color32::from_rgb(230, 180, 60)
                    } else {
                        egui::Color32::from_rgb(230, 80, 80)
                    };
                    let text =
                        format!("{}:{}  {}", error.file.display(), error.line, error.message);
                    let r = ui.add(
                        egui::Label::new(egui::RichText::new(&text).size(11.0).color(color))
                            .sense(egui::Sense::click()),
                    );
                    if r.clicked() {
                        open_error_file = Some((ws.root_path.join(&error.file), error.line));
                    }
                }
            });
        if let Some((path, line)) = open_error_file {
            open_and_jump(ws, path, line);
        }
    }

    // Project-wide search results
    let loading = ws.project_search.rx.is_some();
    let has_results = !ws.project_search.results.is_empty();
    if loading || has_results {
        ui.separator();
        ui.horizontal(|ui| {
            if loading {
                ui.label(
                    egui::RichText::new(i18n.get("project-search-loading"))
                        .weak()
                        .size(12.0),
                );
            } else {
                let mut search_args = fluent_bundle::FluentArgs::new();
                search_args.set("query", ws.project_search.query.clone());
                search_args.set("count", ws.project_search.results.len() as i64);
                ui.label(
                    egui::RichText::new(i18n.get_args("project-search-result-label", &search_args))
                        .strong()
                        .size(12.0),
                );
                if ui.small_button("\u{00D7}").clicked() {
                    ws.project_search.results.clear();
                }
            }
        });
        if has_results {
            let mut open_result: Option<(PathBuf, usize)> = None;
            egui::ScrollArea::vertical()
                .id_salt("project_search_scroll")
                .max_height(config::BUILD_ERROR_LIST_MAX_HEIGHT)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for result in &ws.project_search.results {
                        let text =
                            format!("{}:{}  {}", result.file.display(), result.line, result.text);
                        let r = ui.add(
                            egui::Label::new(
                                egui::RichText::new(&text)
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(130, 190, 255)),
                            )
                            .sense(egui::Sense::click()),
                        );
                        if r.clicked() {
                            open_result = Some((ws.root_path.join(&result.file), result.line));
                        }
                    }
                });
            if let Some((path, line)) = open_result {
                open_and_jump(ws, path, line);
            }
        }
    }
}

/// Renders toast notifications in the bottom right corner. Removes expired toasts.
pub(super) fn render_toasts(ctx: &egui::Context, ws: &mut WorkspaceState) {
    ws.toasts.retain(|t| !t.is_expired());
    if ws.toasts.is_empty() {
        return;
    }

    let screen = ctx.screen_rect();
    let toast_w = 340.0_f32;
    let toast_h = 40.0_f32;
    let padding = 12.0_f32;
    let start_y = screen.max.y - padding - (toast_h + padding) * ws.toasts.len() as f32;

    egui::Area::new(egui::Id::new("toasts_area"))
        .fixed_pos(egui::pos2(screen.max.x - toast_w - padding, start_y))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            for toast in &ws.toasts {
                let (bg, fg) = if toast.is_error {
                    (
                        egui::Color32::from_rgb(90, 30, 30),
                        egui::Color32::from_rgb(255, 180, 170),
                    )
                } else {
                    (
                        egui::Color32::from_rgb(30, 60, 45),
                        egui::Color32::from_rgb(160, 230, 180),
                    )
                };
                egui::Frame::new()
                    .fill(bg)
                    .corner_radius(6.0)
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.set_min_width(toast_w);
                        ui.label(
                            egui::RichText::new(&toast.message)
                                .color(fg)
                                .size(config::UI_FONT_SIZE),
                        );
                    });
                ui.add_space(padding);
            }
        });
}
