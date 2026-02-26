use crate::app::types::FocusedPanel;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::app::ui::workspace::state::open_file_in_ws;
use eframe::egui;

pub fn render_build_bar(ui: &mut egui::Ui, ws: &mut WorkspaceState, i18n: &crate::i18n::I18n) {
    ui.horizontal(|ui| {
        // 1. LEFT CONTROLS
        ui.strong(i18n.get("panel-build"));
        ui.separator();

        // Sandbox Toggle
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
            let target_dir = if ws.build_in_sandbox {
                &ws.sandbox.root
            } else {
                &ws.root_path
            };
            ws.next_terminal_id += 1;
            ws.build_terminal = Some(crate::app::ui::terminal::instance::Terminal::new(
                ws.next_terminal_id,
                ui.ctx(),
                target_dir,
                None,
            ));
        }

        ui.separator();

        // Profile Dropdown
        let combo = egui::ComboBox::from_id_salt("runner_select_bottom")
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
                    if profile.error_parser == crate::app::types::ErrorParserType::Rust {
                        ws.build_error_rx = Some(crate::app::build_runner::run_build_check(
                            target_dir.clone(),
                        ));
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

        // RESERVE SPACE for the float button
        ui.add_space(28.0);

        // 2. FLOAT BUTTON (Right aligned in the remaining space)
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(4.0); // right margin
            if ws.build_terminal_float {
                if ui
                    .small_button("📥")
                    .on_hover_text(i18n.get("ai-float-dock"))
                    .clicked()
                {
                    ws.build_terminal_float = false;
                }
            } else if ui
                .small_button("⧉")
                .on_hover_text(i18n.get("ai-float-undock"))
                .clicked()
            {
                ws.build_terminal_float = true;
            }
        });
    });
}
