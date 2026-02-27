use crate::app::types::AppShared;
use crate::app::ui::widgets::modal::StandardModal;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub fn show(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &I18n,
) {
    // Plugin error dialog
    if let Some(err) = ws.plugin_error.clone() {
        let mut local_show = true;
        let mut close_req = false;
        let modal = StandardModal::new(i18n.get("plugin-error-title"), "plugin_error_modal")
            .with_size(600.0, 400.0);

        modal.show(ctx, &mut local_show, |ui| {
            if let Some(c) = modal.ui_footer(ui, |ui| {
                if ui.button(i18n.get("btn-close")).clicked() {
                    return Some(true);
                }
                None
            }) {
                close_req = c;
            }

            modal.ui_body(ui, |ui| {
                ui.label(
                    egui::RichText::new(i18n.get("plugin-error-heading"))
                        .color(egui::Color32::RED)
                        .strong(),
                );
                ui.add_space(8.0);
                egui::ScrollArea::vertical()
                    .id_salt("plugin_err_scroll")
                    .show(ui, |ui| {
                        let mut err_mut = err.clone();
                        ui.add(
                            egui::TextEdit::multiline(&mut err_mut)
                                .code_editor()
                                .desired_width(f32::INFINITY),
                        );
                    });
            });
        });
        if !local_show || close_req {
            ws.plugin_error = None;
        }
    }

    // Promotion success dialog
    if let Some(path) = ws.promotion_success.clone() {
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());

        let mut local_show = true;
        let mut close_req = false;
        let modal = StandardModal::new(
            i18n.get("ai-promotion-success-title"),
            "promotion_success_modal",
        )
        .with_size(400.0, 250.0);

        modal.show(ctx, &mut local_show, |ui| {
            if let Some(c) = modal.ui_footer(ui, |ui| {
                if ui.button(i18n.get("btn-close")).clicked() {
                    return Some(true);
                }
                if ui.button(i18n.get("btn-ok")).clicked() {
                    return Some(true);
                }
                None
            }) {
                close_req = c;
            }

            modal.ui_body(ui, |ui| {
                ui.vertical_centered(|ui: &mut egui::Ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("\u{2714}")
                            .size(48.0)
                            .color(egui::Color32::from_rgb(100, 200, 100)),
                    );
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new(i18n.get("ai-promotion-success-body")).size(14.0));
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(filename).strong().size(14.0));
                });
            });
        });
        if !local_show || close_req {
            ws.promotion_success = None;
        }
    }

    // Sync confirmation dialog before starting AI
    if let Some(plan) = ws.sync_confirmation.clone() {
        let mut do_sync = false;
        let mut do_skip = false;
        let mut local_show = true;
        let mut close_req = false;

        let modal = StandardModal::new(i18n.get("ai-sync-title"), "ai_sync_confirm_modal")
            .with_size(500.0, 300.0);

        modal.show(ctx, &mut local_show, |ui| {
            if let Some((sync, skip, close)) = modal.ui_footer(ui, |ui| {
                if ui.button(i18n.get("btn-close")).clicked() {
                    return Some((false, false, true));
                }
                if ui
                    .button(egui::RichText::new(i18n.get("ai-sync-btn-sync")).strong())
                    .clicked()
                {
                    return Some((true, false, true));
                }
                if ui.button(i18n.get("ai-sync-btn-skip")).clicked() {
                    return Some((false, true, true));
                }
                if ui.button(i18n.get("btn-cancel")).clicked() {
                    return Some((false, false, true));
                }
                None
            }) {
                do_sync = sync;
                do_skip = skip;
                close_req = close;
            }

            modal.ui_body(ui, |ui| {
                ui.label(egui::RichText::new(i18n.get("ai-sync-msg")).size(14.0));
                ui.add_space(8.0);

                if !plan.to_sandbox.is_empty() {
                    let mut args = fluent_bundle::FluentArgs::new();
                    args.set("count", plan.to_sandbox.len());
                    ui.label(
                        egui::RichText::new(format!(
                            "\u{2193} {}",
                            i18n.get_args("ai-sync-to-sandbox", &args)
                        ))
                        .color(egui::Color32::from_rgb(100, 150, 255)),
                    );
                }

                if !plan.to_project.is_empty() {
                    let mut args = fluent_bundle::FluentArgs::new();
                    args.set("count", plan.to_project.len());
                    ui.label(
                        egui::RichText::new(format!(
                            "\u{2191} {}",
                            i18n.get_args("ai-sync-to-project", &args)
                        ))
                        .color(egui::Color32::from_rgb(255, 200, 100)),
                    );
                }
                ui.add_space(12.0);
            });
        });

        if do_sync {
            // Perform bidirectional sync
            for rel_path in &plan.to_sandbox {
                let src = ws.root_path.join(rel_path);
                let dst = ws.sandbox.root.join(rel_path);
                if let Some(parent) = dst.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::copy(src, dst);
            }
            for rel_path in &plan.to_project {
                let _ = ws.sandbox.promote_file(rel_path);
            }
            ws.sandbox_staged_dirty = true;
            do_skip = true; // Trigger agent start after sync
        }

        if do_skip && let Some(agent_id) = ws.pending_agent_id.take() {
            let agents = {
                let sh = shared.lock().expect("lock");
                sh.registry.agents.get_all().to_vec()
            };
            if let Some(agent) = agents.iter().find(|a| a.id == agent_id) {
                let cmd = agent.command.clone();
                let active = ws.claude_active_tab;
                let context = crate::app::ui::terminal::right::format_context_for_terminal(
                    &crate::app::ai::AiManager::generate_context(ws, shared),
                );
                if let Some(terminal) = ws.claude_tabs.get_mut(active) {
                    terminal.send_command(&cmd);
                    if agent.context_aware {
                        terminal.send_command(&context);
                    }
                }
            }
        }

        if !local_show || close_req {
            ws.sync_confirmation = None;
            ws.pending_agent_id = None;
        }
    }

    // List of all staged files in Sandbox.
    if ws.show_sandbox_staged {
        let staged_files = ws.sandbox_staged_files.clone();
        if staged_files.is_empty() {
            ws.show_sandbox_staged = false;
        } else {
            let mut local_show = ws.show_sandbox_staged;
            let mut close_req = false;
            let modal = StandardModal::new(i18n.get("ai-staged-files"), "ai_sandbox_staged_modal")
                .with_size(600.0, 450.0);

            modal.show(ctx, &mut local_show, |ui| {
                if let Some(c) = modal.ui_footer(ui, |ui| {
                    if ui.button(i18n.get("btn-close")).clicked() {
                        return Some(true);
                    }
                    None
                }) {
                    close_req = c;
                }

                modal.ui_body(ui, |ui| {
                    ui.label(i18n.get("ai-staged-modal-hint"));
                    ui.add_space(8.0);

                    egui::ScrollArea::vertical()
                        .id_salt("staged_files_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            for rel_path in staged_files {
                                let path_str = rel_path.to_string_lossy().to_string();
                                let project_exists = ws.root_path.join(&rel_path).exists();
                                let sandbox_exists = ws.sandbox.root.join(&rel_path).exists();

                                ui.horizontal(|ui| {
                                    if !sandbox_exists {
                                        ui.label(
                                            egui::RichText::new(i18n.get("ai-staged-del"))
                                                .color(egui::Color32::from_rgb(210, 80, 80))
                                                .small(),
                                        );
                                    } else if !project_exists {
                                        ui.label(
                                            egui::RichText::new(i18n.get("ai-staged-new"))
                                                .color(egui::Color32::from_rgb(100, 200, 100))
                                                .small(),
                                        );
                                    } else {
                                        ui.label(
                                            egui::RichText::new(i18n.get("ai-staged-mod"))
                                                .color(egui::Color32::from_rgb(200, 150, 50))
                                                .small(),
                                        );
                                    }

                                    if ui.link(&path_str).clicked() {
                                        let sandbox_path = ws.sandbox.root.join(&rel_path);
                                        let project_path = ws.root_path.join(&rel_path);

                                        let new_content = std::fs::read_to_string(sandbox_path)
                                            .unwrap_or_default();
                                        let old_content = std::fs::read_to_string(project_path)
                                            .unwrap_or_default();

                                        ws.editor.pending_ai_diff = Some((
                                            ws.root_path
                                                .join(rel_path)
                                                .to_string_lossy()
                                                .to_string(),
                                            old_content,
                                            new_content,
                                        ));
                                    }
                                });
                            }
                        });
                });
            });
            ws.show_sandbox_staged = local_show && !close_req;
        }
    }
}
