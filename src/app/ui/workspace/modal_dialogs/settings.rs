use crate::app::types::AppShared;
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

    let card_id = ui.id().with(("settings-light-variant-card", light_variant_label_key(&variant)));
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

fn should_persist_theme_change(
    original: Option<&crate::settings::Settings>,
    draft: &crate::settings::Settings,
) -> bool {
    original
        .map(|snapshot| theme_fingerprint(snapshot) != theme_fingerprint(draft))
        .unwrap_or(true)
}

fn apply_theme_preview(shared: &Arc<Mutex<AppShared>>, draft: &crate::settings::Settings) {
    let mut shared_state = shared.lock().expect("lock");
    shared_state.settings = Arc::new(draft.clone());
    shared_state.settings_version.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

fn restore_runtime_settings_from_snapshot(
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

pub(super) fn discard_settings_draft(
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
) {
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
    _id_salt: &std::ffi::OsStr,
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

    let mut action = None;
    let mut browse_requested = false;
    let mut selected_cat = ws
        .selected_settings_category
        .clone()
        .unwrap_or_else(|| "general".to_string());
    let mut show_flag = ws.show_settings;

    let modal = StandardModal::new(i18n.get("settings-title"), "main_settings");

    modal.show(ctx, &mut show_flag, |ui| {
        // FOOTER
        action = modal.ui_footer_actions(ui, i18n, |f| {
            if f.confirm_cancel(ws) {
                return Some(SettingsModalAction::Cancel);
            }
            if f.save() {
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
                            ui.checkbox(
                                &mut draft.project_read_only,
                                i18n.get("settings-safe-mode"),
                            );
                            ui.label(
                                egui::RichText::new(i18n.get("settings-safe-mode-hint"))
                                    .small()
                                    .weak(),
                            );
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

    if let Some(act) = action {
        match act {
            SettingsModalAction::Save => {
                if let Some(draft) = ws.settings_draft.take() {
                    if should_persist_theme_change(ws.settings_original.as_ref(), &draft) {
                        draft.save();
                    }
                    ws.wizard.path = draft.default_project_path.clone();
                    let lang = draft.lang.clone();
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

                    s.settings = Arc::new(draft);
                    s.i18n = Arc::new(crate::i18n::I18n::new(&lang));
                    s.settings_version.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }
                ws.settings_original = None;
                ws.show_settings = false;
            }
            SettingsModalAction::Cancel => {
                discard_settings_draft(ws, shared);
                ws.show_settings = false;
            }
        }
    }
}
