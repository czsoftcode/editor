use crate::app::types::AppShared;
use crate::app::ui::widgets::modal::StandardModal;
use crate::app::ui::workspace::{MenuActions, WorkspaceState};
use eframe::egui;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommandId {
    OpenFile,
    ProjectSearch,
    Build,
    Run,
    Save,
    CloseTab,
    NewProject,
    OpenProject,
    OpenFolder,
    ToggleLeft,
    ToggleRight,
    ToggleBuild,
    ToggleFloat,
    About,
    Settings,
    Plugins,
    Quit,
}

pub(crate) struct CommandPaletteState {
    pub query: String,
    pub selected: usize,
    pub focus_requested: bool,
    pub commands: Vec<crate::app::registry::Command>,
    pub filtered: Vec<usize>,
}

impl CommandPaletteState {
    pub fn new(commands: Vec<crate::app::registry::Command>) -> Self {
        let filtered = (0..commands.len()).collect();
        Self {
            query: String::new(),
            selected: 0,
            focus_requested: true,
            commands,
            filtered,
        }
    }

    pub fn update_filter(&mut self, i18n: &crate::i18n::I18n) {
        let q = self.query.to_lowercase();
        self.filtered = self
            .commands
            .iter()
            .enumerate()
            .filter(|(_, cmd)| {
                let name = i18n.get(cmd.i18n_key).to_lowercase();
                crate::app::ui::search_picker::fuzzy_match(&q, &name)
            })
            .map(|(i, _)| i)
            .collect();
        self.selected = 0;
    }
}

pub(crate) fn render_command_palette(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    _shared: &Arc<Mutex<AppShared>>,
    i18n: &crate::i18n::I18n,
) -> Option<crate::app::registry::CommandAction> {
    let palette = ws.command_palette.as_mut()?;

    // Global navigation keys
    let key_up = ctx.input(|i| i.key_pressed(egui::Key::ArrowUp));
    let key_down = ctx.input(|i| i.key_pressed(egui::Key::ArrowDown));
    let key_enter = ctx.input(|i| i.key_pressed(egui::Key::Enter));
    let key_esc = ctx.input(|i| i.key_pressed(egui::Key::Escape));

    let filtered_len = palette.filtered.len();
    let mut scroll_to_selected = false;

    if key_up && palette.selected > 0 {
        palette.selected -= 1;
        scroll_to_selected = true;
    }
    if key_down && palette.selected + 1 < filtered_len {
        palette.selected += 1;
        scroll_to_selected = true;
    }

    let mut executed_action: Option<crate::app::registry::CommandAction> = None;
    let mut close = key_esc;
    let mut show_flag = true;

    if key_enter && !palette.filtered.is_empty() {
        executed_action = Some(
            palette.commands[palette.filtered[palette.selected]]
                .action
                .clone(),
        );
        close = true;
    }

    let modal = StandardModal::new(i18n.get("command-palette-heading"), "command_palette_modal")
        .with_size(600.0, 450.0);

    modal.show(ctx, &mut show_flag, |ui| {
        // FOOTER
        if let Some(c) = modal.ui_footer_actions(ui, i18n, |f| {
            if f.close() || f.cancel() {
                return Some(true);
            }
            None
        }) {
            close = c;
        }

        // BODY
        modal.ui_body(ui, |ui| {
            ui.add_space(4.0);
            let resp = ui.add(
                egui::TextEdit::singleline(&mut palette.query)
                    .hint_text(i18n.get("command-palette-placeholder"))
                    .desired_width(ui.available_width())
                    .id(egui::Id::new("command_palette_input")),
            );

            if palette.focus_requested {
                resp.request_focus();
                palette.focus_requested = false;
            }

            if resp.changed() {
                palette.update_filter(i18n);
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .max_height(300.0)
                .id_salt("cp_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    if palette.filtered.is_empty() {
                        ui.label(
                            egui::RichText::new(i18n.get("command-palette-no-results")).weak(),
                        );
                    } else {
                        for (disp_idx, &cmd_idx) in palette.filtered.iter().enumerate() {
                            let cmd = &palette.commands[cmd_idx];
                            let is_sel = disp_idx == palette.selected;

                            ui.horizontal(|ui| {
                                let text = egui::RichText::new(i18n.get(cmd.i18n_key)).size(13.0);
                                let r = ui.selectable_label(is_sel, text);

                                if is_sel && scroll_to_selected {
                                    r.scroll_to_me(None);
                                }
                                if r.clicked() {
                                    executed_action = Some(cmd.action.clone());
                                    close = true;
                                }

                                if let Some(shortcut) = cmd.shortcut {
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                egui::RichText::new(shortcut).weak().size(11.0),
                                            );
                                        },
                                    );
                                }
                            });
                        }
                    }
                });
        });
    });

    if close || !show_flag {
        ws.command_palette = None;
    }

    executed_action
}

pub(crate) fn execute_command(
    action: crate::app::registry::CommandAction,
    actions: &mut MenuActions,
    shared: &Arc<Mutex<AppShared>>,
) -> Option<String> {
    match action {
        crate::app::registry::CommandAction::Internal(id) => {
            match id {
                CommandId::OpenFile => actions.open_file_picker = true,
                CommandId::ProjectSearch => actions.project_search = true,
                CommandId::Build => actions.build = true,
                CommandId::Run => actions.run = true,
                CommandId::Save => actions.save = true,
                CommandId::CloseTab => actions.close_file = true,
                CommandId::NewProject => actions.new_project = true,
                CommandId::OpenProject => actions.open_project = true,
                CommandId::OpenFolder => actions.open_folder = true,
                CommandId::ToggleLeft => actions.toggle_left = true,
                CommandId::ToggleRight => actions.toggle_right = true,
                CommandId::ToggleBuild => actions.toggle_build = true,
                CommandId::ToggleFloat => actions.toggle_float = true,
                CommandId::About => actions.about = true,
                CommandId::Settings => actions.settings = true,
                CommandId::Plugins => actions.plugins = true,
                CommandId::Quit => actions.quit = true,
            }
            None
        }
        crate::app::registry::CommandAction::Plugin {
            plugin_id,
            func_name,
        } => {
            if plugin_id == "gemini" || plugin_id == "ollama" {
                // Special case for our interactive AI plugins
                // We'll signal the UI to open the unified AI chat modal
                return Some("OPEN_AI_CHAT_MODAL".to_string());
            }
            let shared = shared
                .lock()
                .expect("Failed to lock AppShared in execute_command");
            let config = shared
                .settings
                .plugins
                .get(&plugin_id)
                .map(|s| s.config.clone())
                .unwrap_or_default();

            match shared
                .registry
                .plugins
                .call(&plugin_id, &func_name, "command-palette", &config)
            {
                Ok(res) => Some(res),
                Err(e) => {
                    eprintln!("Plugin command failed: {}", e);
                    Some(format!("Plugin error: {}", e))
                }
            }
        }
    }
}
