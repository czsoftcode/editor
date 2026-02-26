pub mod build_bar;
pub mod compile_bar;
pub mod git_bar;

use crate::app::types::FocusedPanel;
use crate::app::ui::terminal::instance::TerminalAction;
use crate::app::ui::workspace::state::{WorkspaceState, open_and_jump, open_file_in_ws};
use crate::config;
use eframe::egui;
use std::path::PathBuf;

/// Renders the bottom panel (or its floating window version).
pub fn render_bottom_panel(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    dialog_open: bool,
    i18n: &crate::i18n::I18n,
) {
    if !ws.show_build_terminal {
        return;
    }

    if ws.build_terminal_float {
        let mut is_open = true;
        egui::Window::new(i18n.get("build-terminal-title"))
            .id(egui::Id::new("build_terminal_float_win"))
            .default_size([600.0, 400.0])
            .min_size([300.0, 200.0])
            .resizable(true)
            .collapsible(false)
            .open(&mut is_open)
            .show(ctx, |ui| {
                render_bottom_content(ui, ws, dialog_open, i18n);
            });
        if !is_open {
            ws.show_build_terminal = false;
        }
    }
}

/// Renders ONLY the content of the bottom panel (to be used inside another panel).
/// Returns true if the terminal was clicked or hovered.
pub fn render_bottom_content(
    ui: &mut egui::Ui,
    ws: &mut WorkspaceState,
    dialog_open: bool,
    i18n: &crate::i18n::I18n,
) -> bool {
    if !ws.show_build_terminal {
        return false;
    }
    let mut interacted = false;

    ui.vertical(|ui| {
        // 1. Control Bars (Stacked vertically to save width in narrow side panels)
        build_bar::render_build_bar(ui, ws, i18n);
        compile_bar::render_compile_bar(ui, ws, i18n);
        if !ws.build_in_sandbox {
            git_bar::render_git_bar(ui, ws, i18n);
        }

        ui.separator();

        // 2. The Terminal
        let font_size = config::EDITOR_FONT_SIZE * ws.ai_font_scale as f32 / 100.0;
        if let Some(terminal) = &mut ws.build_terminal {
            let is_focused = ws.focused_panel == FocusedPanel::Build && !dialog_open;
            let action = terminal.ui(ui, is_focused, font_size, i18n);

            match action {
                Some(TerminalAction::Clicked) | Some(TerminalAction::Hovered) => {
                    ws.focused_panel = FocusedPanel::Build;
                    interacted = true;
                }
                Some(TerminalAction::Navigate(path, line, col)) => {
                    let abs_path = if path.is_absolute() {
                        path
                    } else {
                        ws.root_path.join(path)
                    };
                    open_file_in_ws(ws, abs_path);
                    ws.editor.jump_to_location(line, col);
                    ws.focused_panel = FocusedPanel::Editor;
                    interacted = true;
                }
                None => {}
            }
        }

        // 3. Error List
        if !ws.build_errors.is_empty() {
            ui.separator();
            render_error_list(ui, ws, i18n);
        }
    });
    interacted
}

fn render_error_list(ui: &mut egui::Ui, ws: &mut WorkspaceState, i18n: &crate::i18n::I18n) {
    let mut err_args = fluent_bundle::FluentArgs::new();
    err_args.set("count", ws.build_errors.len() as i64);
    ui.label(
        egui::RichText::new(i18n.get_args("panel-build-errors", &err_args))
            .strong()
            .size(12.0),
    );

    let mut open_error_file: Option<(PathBuf, usize)> = None;
    egui::ScrollArea::vertical()
        .id_salt("bottom_errors_scroll")
        .max_height(100.0)
        .show(ui, |ui| {
            for error in &ws.build_errors {
                let color = if error.is_warning {
                    egui::Color32::from_rgb(230, 180, 60)
                } else {
                    egui::Color32::from_rgb(230, 80, 80)
                };
                let text = format!("{}:{}  {}", error.file.display(), error.line, error.message);
                if ui
                    .add(
                        egui::Label::new(egui::RichText::new(&text).size(11.0).color(color))
                            .sense(egui::Sense::click()),
                    )
                    .clicked()
                {
                    open_error_file = Some((ws.root_path.join(&error.file), error.line));
                }
            }
        });
    if let Some((path, line)) = open_error_file {
        open_and_jump(ws, path, line);
    }
}
