use crate::app::types::{AppShared, ToastAction, ToastActionKind};
use crate::app::ui::background::spawn_task;
use crate::app::ui::widgets::modal::StandardModal;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use crate::settings::LightVariant;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub enum SettingsModalAction {
    Save,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SandboxModeChange {
    None,
    Enabled,
    Disabled,
}

fn sandbox_mode_change(
    original: Option<&crate::settings::Settings>,
    draft: &crate::settings::Settings,
) -> SandboxModeChange {
    let Some(original) = original else {
        return SandboxModeChange::None;
    };

    match (original.sandbox_mode, draft.sandbox_mode) {
        (true, true) | (false, false) => SandboxModeChange::None,
        (false, true) => SandboxModeChange::Enabled,
        (true, false) => SandboxModeChange::Disabled,
    }
}

fn requires_sandbox_off_confirm(change: SandboxModeChange) -> bool {
    matches!(change, SandboxModeChange::Disabled)
}

fn should_block_sandbox_apply(ws: &WorkspaceState) -> bool {
    ws.show_plugins
        || ws.show_new_project
        || ws.show_about
        || ws.show_support
        || ws.show_semantic_indexing_modal
        || ws.sync_confirmation.is_some()
        || ws.show_sandbox_staged
        || ws.confirm_discard_changes.is_some()
        || ws.pending_plugin_approval.is_some()
        || ws.pending_ask_user.is_some()
        || ws.external_change_conflict.is_some()
}

fn show_sandbox_off_confirm(
    ctx: &egui::Context,
    i18n: &I18n,
    id_salt: &std::ffi::OsStr,
) -> Option<bool> {
    let mut show_flag = true;
    let mut decided = None;
    let modal = StandardModal::new(
        i18n.get("settings-sandbox-off-title"),
        format!("sandbox_off_confirm_{}", id_salt.to_string_lossy()),
    )
    .with_size(520.0, 260.0);

    modal.show(ctx, &mut show_flag, |ui| {
        modal.ui_footer_actions(ui, i18n, |f| {
            if f.button("btn-cancel").clicked() || f.close() {
                decided = Some(false);
                return Some(());
            }
            if f.button("btn-confirm").clicked() {
                decided = Some(true);
                return Some(());
            }
            None
        });

        modal.ui_body(ui, |ui| {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(i18n.get("settings-sandbox-off-message"))
                    .size(14.0)
                    .line_height(Some(20.0)),
            );
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(i18n.get("settings-sandbox-off-warning"))
                    .color(egui::Color32::from_rgb(255, 210, 140)),
            );
        });
    });

    if !show_flag && decided.is_none() {
        decided = Some(false);
    }

    decided
}

fn light_variant_label_key(variant: &LightVariant) -> &'static str {
    match variant {
        LightVariant::WarmIvory => "settings-light-variant-warm-ivory",
        LightVariant::CoolGray => "settings-light-variant-cool-gray",
        LightVariant::Sepia => "settings-light-variant-sepia",
    }
}

fn light_variant_swatch(variant: &LightVariant) -> egui::Color32 {
    match variant {
        LightVariant::WarmIvory => egui::Color32::from_rgb(250, 246, 235),
        LightVariant::CoolGray => egui::Color32::from_rgb(236, 236, 236),
        LightVariant::Sepia => egui::Color32::from_rgb(234, 223, 202),
    }
}

fn show_light_variant_card(
    ui: &mut egui::Ui,
    draft: &mut crate::settings::Settings,
    i18n: &I18n,
    variant: LightVariant,
) -> bool {
    let is_selected = draft.light_variant == variant;
    let border_color = if is_selected {
        ui.visuals().selection.stroke.color
    } else {
        ui.visuals().widgets.noninteractive.bg_stroke.color
    };

    let card = egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .stroke(egui::Stroke::new(
            if is_selected { 2.0 } else { 1.0 },
            border_color,
        ))
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_min_size(egui::vec2(180.0, 52.0));
            ui.horizontal(|ui| {
                let (swatch_rect, _) =
                    ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::hover());
                ui.painter()
                    .rect_filled(swatch_rect, 4.0, light_variant_swatch(&variant));
                ui.add_space(8.0);
                ui.label(i18n.get(light_variant_label_key(&variant)));
                if is_selected {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("✓")
                            .strong()
                            .color(ui.visuals().selection.stroke.color),
                    );
                }
            });
        });

    let card_id = ui.id().with((
        "settings-light-variant-card",
        light_variant_label_key(&variant),
    ));
    let response = ui.interact(card.response.rect, card_id, egui::Sense::click());
    if response.clicked() && draft.light_variant != variant {
        draft.light_variant = variant;
        return true;
    }
    false
}

fn theme_fingerprint(settings: &crate::settings::Settings) -> (bool, LightVariant) {
    (settings.dark_theme, settings.light_variant.clone())
}

fn should_persist_settings_change(
    original: Option<&crate::settings::Settings>,
    draft: &crate::settings::Settings,
) -> bool {
    original.map(|snapshot| snapshot != draft).unwrap_or(true)
}

fn apply_theme_preview(shared: &Arc<Mutex<AppShared>>, draft: &crate::settings::Settings) {
    let mut shared_state = shared.lock().expect("lock");
    shared_state.settings = Arc::new(draft.clone());
    shared_state
        .settings_version
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

pub(crate) fn restore_runtime_settings_from_snapshot(
    shared: &Arc<Mutex<AppShared>>,
    snapshot: crate::settings::Settings,
) {
    let mut shared_state = shared.lock().expect("lock");
    let should_bump_version =
        theme_fingerprint(&shared_state.settings) != theme_fingerprint(&snapshot);
    shared_state.settings = Arc::new(snapshot);
    if should_bump_version {
        shared_state
            .settings_version
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

pub(super) fn discard_settings_draft(ws: &mut WorkspaceState, shared: &Arc<Mutex<AppShared>>) {
    if let Some(snapshot) = ws.settings_original.take() {
        restore_runtime_settings_from_snapshot(shared, snapshot);
    }
    ws.settings_draft = None;
}

pub fn show(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &I18n,
    id_salt: &std::ffi::OsStr,
) {
    if !ws.show_settings {
        return;
    }

    if ws.settings_draft.is_none() {
        let current_settings = (*shared.lock().expect("lock").settings).clone();
        ws.settings_original = Some(current_settings.clone());
        ws.settings_draft = Some(current_settings);
    }

    if let Some(rx) = ws.settings_folder_pick_rx.as_ref()
        && let Ok(picked) = rx.try_recv()
    {
        ws.settings_folder_pick_rx = None;
        if let Some(dir) = picked
            && let Some(draft) = ws.settings_draft.as_mut()
        {
            draft.default_project_path = dir.to_string_lossy().to_string();
        }
    }

    let mut save_requested = false;
    let mut cancel_requested = false;
    let mut browse_requested = false;
    let mut selected_cat = ws
        .selected_settings_category
        .clone()
        .unwrap_or_else(|| "general".to_string());
    let mut show_flag = ws.show_settings;

    let modal = StandardModal::new(i18n.get("settings-title"), "main_settings");

    modal.show(ctx, &mut show_flag, |ui| {
        // FOOTER
        let _footer_action = modal.ui_footer_actions(ui, i18n, |f| {
            if f.confirm_cancel(ws) {
                cancel_requested = true;
                return Some(SettingsModalAction::Cancel);
            }
            if f.save() {
                save_requested = true;
                return Some(SettingsModalAction::Save);
            }
            None
        });

        // BODY
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let draft = ws.settings_draft.as_mut().unwrap();

            // 1. LEFT PANEL (Tree/Categories)
            egui::SidePanel::left("settings_left_tree")
                .resizable(false)
                .default_width(180.0)
                .show_inside(ui, |ui| {
                    ui.add_space(4.0);
                    egui::ScrollArea::vertical()
                        .id_salt("settings_cat_scroll")
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());

                            if ui
                                .selectable_label(
                                    selected_cat == "general",
                                    format!("⚙ {}", i18n.get("settings-category-general")),
                                )
                                .clicked()
                            {
                                selected_cat = "general".to_string();
                            }
                            if ui
                                .selectable_label(
                                    selected_cat == "editor",
                                    format!("📝 {}", i18n.get("settings-category-editor")),
                                )
                                .clicked()
                            {
                                selected_cat = "editor".to_string();
                            }
                            if ui
                                .selectable_label(
                                    selected_cat == "ai",
                                    format!("🤖 {}", i18n.get("settings-category-ai")),
                                )
                                .clicked()
                            {
                                selected_cat = "ai".to_string();
                            }
                        });
                });

            // 2. RIGHT PANEL (Content)
            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("settings_content_scroll")
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        if selected_cat == "general" {
                            ui.strong(
                                egui::RichText::new(i18n.get("settings-category-general"))
                                    .size(18.0),
                            );
                            ui.add_space(12.0);

                            ui.strong(i18n.get("settings-language"));
                            ui.add_space(4.0);
                            egui::ComboBox::from_id_salt("settings_lang_combo")
                                .selected_text(crate::i18n::lang_display_name(&draft.lang))
                                .width(160.0)
                                .show_ui(ui, |ui| {
                                    for &lang in crate::i18n::SUPPORTED_LANGS {
                                        if ui
                                            .selectable_label(
                                                draft.lang == lang,
                                                crate::i18n::lang_display_name(lang),
                                            )
                                            .clicked()
                                        {
                                            draft.lang = lang.to_string();
                                        }
                                    }
                                });
                            ui.add_space(16.0);

                            ui.strong(i18n.get("settings-default-path"));
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut draft.default_project_path)
                                        .desired_width(ui.available_width() - 40.0),
                                );
                                if ui.button("…").clicked() {
                                    browse_requested = true;
                                }
                            });
                            ui.add_space(20.0);

                            ui.separator();
                            ui.add_space(10.0);
                            let sandbox_mode_row = ui.allocate_ui_with_layout(
                                egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    ui.checkbox(
                                        &mut draft.sandbox_mode,
                                        i18n.get("settings-safe-mode"),
                                    );
                                    ui.add_space(4.0);
                                    ui.label(egui::RichText::new("ℹ").strong());
                                },
                            );
                            sandbox_mode_row
                                .response
                                .on_hover_text(i18n.get("settings-safe-mode-tooltip"));
                            ui.label(
                                egui::RichText::new(i18n.get("settings-safe-mode-hint")).strong(),
                            );
                            ui.label(i18n.get("settings-safe-mode-terminal-note"));
                        } else if selected_cat == "editor" {
                            ui.strong(
                                egui::RichText::new(i18n.get("settings-category-editor"))
                                    .size(18.0),
                            );
                            ui.add_space(12.0);

                            ui.strong(i18n.get("settings-theme"));
                            let theme_before = theme_fingerprint(draft);
                            let mut theme_controls_changed = false;
                            ui.horizontal(|ui| {
                                theme_controls_changed |= ui
                                    .radio_value(
                                        &mut draft.dark_theme,
                                        true,
                                        i18n.get("settings-theme-dark"),
                                    )
                                    .changed();
                                theme_controls_changed |= ui
                                    .radio_value(
                                        &mut draft.dark_theme,
                                        false,
                                        i18n.get("settings-theme-light"),
                                    )
                                    .changed();
                            });
                            ui.add_space(16.0);

                            if !draft.dark_theme {
                                ui.strong(i18n.get("settings-light-variant"));
                                ui.add_space(6.0);
                                ui.horizontal_wrapped(|ui| {
                                    for variant in [
                                        LightVariant::WarmIvory,
                                        LightVariant::CoolGray,
                                        LightVariant::Sepia,
                                    ] {
                                        theme_controls_changed |=
                                            show_light_variant_card(ui, draft, i18n, variant);
                                        ui.add_space(8.0);
                                    }
                                });
                                ui.add_space(16.0);
                            }

                            if theme_controls_changed && theme_before != theme_fingerprint(draft) {
                                apply_theme_preview(shared, draft);
                            }

                            ui.strong(i18n.get("settings-editor-font"));
                            ui.add_space(4.0);
                            ui.add(
                                egui::Slider::new(&mut draft.editor_font_size, 10.0..=24.0)
                                    .step_by(1.0)
                                    .suffix(" px"),
                            );
                        } else if selected_cat == "ai" {
                            ui.strong(
                                egui::RichText::new(i18n.get("settings-category-ai")).size(18.0),
                            );
                            ui.add_space(12.0);

                            ui.label(i18n.get("settings-ai-hint"));
                            ui.add_space(12.0);

                            let mut to_remove = None;
                            for (idx, agent) in draft.custom_agents.iter_mut().enumerate() {
                                ui.group(|ui| {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.vertical(|ui| {
                                                ui.label(i18n.get("settings-ai-name"));
                                                ui.add(
                                                    egui::TextEdit::singleline(&mut agent.name)
                                                        .hint_text("Gemini"),
                                                );
                                            });
                                            ui.vertical(|ui| {
                                                ui.label(i18n.get("settings-ai-command"));
                                                ui.add(
                                                    egui::TextEdit::singleline(&mut agent.command)
                                                        .hint_text("gemini"),
                                                );
                                            });
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui.button("🗑").clicked() {
                                                        to_remove = Some(idx);
                                                    }
                                                },
                                            );
                                        });

                                        ui.add_space(4.0);

                                        ui.vertical(|ui| {
                                            ui.label(i18n.get("settings-ai-args"));
                                            ui.add(
                                                egui::TextEdit::singleline(&mut agent.args)
                                                    .hint_text("--chat")
                                                    .desired_width(ui.available_width()),
                                            );
                                        });
                                    });
                                });
                                ui.add_space(8.0);
                            }

                            if let Some(idx) = to_remove {
                                draft.custom_agents.remove(idx);
                            }

                            if ui
                                .button(format!("+ {}", i18n.get("settings-ai-add")))
                                .clicked()
                            {
                                draft.custom_agents.push(crate::settings::CustomAgent {
                                    name: "".to_string(),
                                    command: "".to_string(),
                                    args: "".to_string(),
                                });
                            }
                        }
                    });
            });
        });
    });

    ws.selected_settings_category = Some(selected_cat);
    ws.show_settings = show_flag;

    let sandbox_change = ws
        .settings_draft
        .as_ref()
        .map(|draft| sandbox_mode_change(ws.settings_original.as_ref(), draft));
    let requires_off_confirm = sandbox_change
        .map(requires_sandbox_off_confirm)
        .unwrap_or(false);

    if save_requested {
        let already_confirmed = ws
            .pending_settings_save
            .as_ref()
            .map(|pending| pending.sandbox_off_confirmed)
            .unwrap_or(false);
        if requires_off_confirm && !already_confirmed {
            ws.pending_settings_save =
                Some(crate::app::ui::workspace::state::PendingSettingsSave {
                    sandbox_off_confirmed: false,
                });
            save_requested = false;
        }
    }

    if let Some(pending) = ws.pending_settings_save.as_mut()
        && requires_off_confirm
        && !pending.sandbox_off_confirmed
    {
        if let Some(confirmed) = show_sandbox_off_confirm(ctx, i18n, id_salt) {
            if confirmed {
                pending.sandbox_off_confirmed = true;
                save_requested = true;
            } else {
                ws.pending_settings_save = None;
            }
        }
    }

    if browse_requested && ws.settings_folder_pick_rx.is_none() {
        let start_dir = ws
            .settings_draft
            .as_ref()
            .map(|d| d.default_project_path.clone())
            .unwrap_or_default();
        ws.settings_folder_pick_rx = Some(spawn_task(move || {
            let d = rfd::FileDialog::new();
            if start_dir.trim().is_empty() {
                d.pick_folder()
            } else {
                d.set_directory(start_dir).pick_folder()
            }
        }));
    }

    if cancel_requested {
        ws.pending_settings_save = None;
        ws.pending_sandbox_apply = None;
        ws.sandbox_persist_failure = None;
        discard_settings_draft(ws, shared);
        ws.show_settings = false;
    } else if save_requested {
        if let Some(draft) = ws.settings_draft.take() {
            let original_settings = ws.settings_original.clone();
            let sandbox_change = sandbox_mode_change(original_settings.as_ref(), &draft);
            let sandbox_dirty = !matches!(sandbox_change, SandboxModeChange::None);
            let mut persist_error: Option<String> = None;
            if should_persist_settings_change(original_settings.as_ref(), &draft) {
                if let Err(err) = draft.try_save() {
                    persist_error = Some(err);
                }
            }
            let draft_sandbox_mode = draft.sandbox_mode;
            ws.wizard.path = draft.default_project_path.clone();
            let lang = draft.lang.clone();
            let mut toast_message: Option<String> = None;
            let mut should_prompt_apply = false;
            let mut s = shared.lock().expect("lock");

            // Immediate agent registry update
            s.registry.agents.clear();
            for ca in &draft.custom_agents {
                let cmd = if ca.args.is_empty() {
                    ca.command.clone()
                } else {
                    format!("{} {}", ca.command, ca.args)
                };
                s.registry.agents.register(crate::app::registry::Agent {
                    id: ca.name.to_lowercase().replace(' ', "_"),
                    label: ca.name.clone(),
                    command: cmd,
                    context_aware: true,
                });
            }

            if sandbox_dirty {
                match sandbox_change {
                    SandboxModeChange::Disabled => {
                        toast_message = Some(i18n.get("settings-sandbox-toast-off"));
                        should_prompt_apply = should_block_sandbox_apply(ws);
                    }
                    SandboxModeChange::Enabled => {
                        toast_message = Some(i18n.get("settings-sandbox-toast-on"));
                        should_prompt_apply = should_block_sandbox_apply(ws);
                    }
                    SandboxModeChange::None => {}
                }
            }

            if let Some(err) = persist_error {
                let original_snapshot = original_settings
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| draft.clone());
                ws.sandbox_persist_failure =
                    Some(crate::app::ui::workspace::state::SandboxPersistFailure {
                        draft,
                        original: original_snapshot,
                    });
                ws.toasts.push(crate::app::types::Toast::error(err));
                ws.toasts.push(crate::app::types::Toast::info_with_actions(
                    i18n.get("settings-sandbox-persist-actions"),
                    vec![
                        ToastAction::new(
                            "settings-sandbox-persist-revert",
                            ToastActionKind::SandboxPersistRevert,
                        ),
                        ToastAction::new(
                            "settings-sandbox-persist-keep",
                            ToastActionKind::SandboxPersistKeep,
                        ),
                    ],
                ));
                ws.pending_sandbox_apply = None;
            } else {
                s.settings = Arc::new(draft);
                s.i18n = Arc::new(crate::i18n::I18n::new(&lang));
                let new_version = s
                    .settings_version
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                    + 1;
                drop(s);

                if sandbox_dirty {
                    ws.pending_sandbox_apply =
                        Some(crate::app::ui::workspace::state::SandboxApplyRequest {
                            target_mode: draft_sandbox_mode,
                            version: new_version,
                            defer_until_clear: should_prompt_apply,
                            force_apply: false,
                            prompted: false,
                            notify_on_apply: false,
                        });
                }

                if let Some(message) = toast_message {
                    ws.toasts.push(crate::app::types::Toast::info(message));
                }
                if should_prompt_apply {
                    ws.toasts.push(crate::app::types::Toast::info_with_actions(
                        i18n.get("settings-sandbox-apply-prompt"),
                        vec![
                            ToastAction::new(
                                "settings-sandbox-apply-now",
                                ToastActionKind::SandboxApplyNow,
                            ),
                            ToastAction::new(
                                "settings-sandbox-apply-defer",
                                ToastActionKind::SandboxApplyLater,
                            ),
                        ],
                    ));
                }
            }
        }
        ws.pending_settings_save = None;
        ws.settings_original = None;
        ws.show_settings = false;
    }
}

#[cfg(test)]
mod tests {
    use super::{SandboxModeChange, requires_sandbox_off_confirm, sandbox_mode_change};
    use crate::settings::Settings;

    #[test]
    fn test_sandbox_mode_change_off_requires_confirm() {
        let mut original = Settings::default();
        original.sandbox_mode = true;
        let mut draft = original.clone();
        draft.sandbox_mode = false;

        let change = sandbox_mode_change(Some(&original), &draft);
        assert_eq!(change, SandboxModeChange::Disabled);
        assert!(requires_sandbox_off_confirm(change));
    }

    #[test]
    fn test_sandbox_mode_change_on_no_confirm() {
        let mut original = Settings::default();
        original.sandbox_mode = false;
        let mut draft = original.clone();
        draft.sandbox_mode = true;

        let change = sandbox_mode_change(Some(&original), &draft);
        assert_eq!(change, SandboxModeChange::Enabled);
        assert!(!requires_sandbox_off_confirm(change));
    }

    #[test]
    fn test_sandbox_mode_change_none() {
        let mut original = Settings::default();
        original.sandbox_mode = true;
        let mut draft = original.clone();
        draft.sandbox_mode = true;

        let change = sandbox_mode_change(Some(&original), &draft);
        assert_eq!(change, SandboxModeChange::None);
        assert!(!requires_sandbox_off_confirm(change));
    }
}
