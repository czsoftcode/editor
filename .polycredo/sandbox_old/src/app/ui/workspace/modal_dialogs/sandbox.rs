use crate::app::sandbox::Sandbox;
use crate::app::ui::background::spawn_task;
use crate::app::ui::widgets::modal::StandardModal;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use eframe::egui;

pub fn show(ctx: &egui::Context, ws: &mut WorkspaceState, i18n: &I18n) {
    let Some(plan) = ws.sandbox_sync_confirmation.clone() else {
        return;
    };

    let mut do_sync = false;
    let mut do_skip = false;
    let mut local_show = true;
    let mut close_req = false;
    let sync_in_progress = ws.sandbox_sync_rx.is_some();

    let modal = StandardModal::new(i18n.get("sandbox-sync-title"), "sandbox_sync_modal")
        .with_size(520.0, 300.0);

    modal.show(ctx, &mut local_show, |ui| {
        if let Some((sync, skip, close)) = modal.ui_footer(ui, |ui| {
            let mut sync_clicked = false;
            ui.add_enabled_ui(!sync_in_progress, |ui| {
                if ui
                    .button(egui::RichText::new(i18n.get("sandbox-sync-btn-sync")).strong())
                    .clicked()
                {
                    sync_clicked = true;
                }
            });
            if sync_clicked {
                return Some((true, false, true));
            }

            if ui.button(i18n.get("sandbox-sync-btn-skip")).clicked() {
                return Some((false, true, true));
            }
            None
        }) {
            do_sync = sync;
            do_skip = skip;
            close_req = close;
        }

        modal.ui_body(ui, |ui| {
            ui.label(egui::RichText::new(i18n.get("sandbox-sync-msg")).size(14.0));
            ui.add_space(8.0);

            if !plan.to_sandbox.is_empty() {
                let mut args = fluent_bundle::FluentArgs::new();
                args.set("count", plan.to_sandbox.len());
                ui.label(
                    egui::RichText::new(format!(
                        "\u{2193} {}",
                        i18n.get_args("sandbox-sync-to-sandbox", &args)
                    ))
                    .color(egui::Color32::from_rgb(100, 150, 255)),
                );
            } else {
                ui.label(i18n.get("sandbox-sync-nothing"));
            }
        });
    });

    if do_sync {
        let project_root = ws.root_path.clone();
        let sandbox_root = ws.sandbox.root.clone();
        let plan_clone = plan.clone();
        ws.sandbox_sync_rx = Some(spawn_task(move || {
            let sandbox = Sandbox::new_with_roots(project_root, sandbox_root);
            sandbox.sync_plan_to_sandbox(&plan_clone)
        }));
    }

    if do_sync || do_skip || !local_show || close_req {
        ws.sandbox_sync_confirmation = None;
    }
}
