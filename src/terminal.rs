use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;

use eframe::egui;
use egui_term::{BackendSettings, PtyEvent, TerminalBackend, TerminalView};

static ENV_INITIALIZED: AtomicBool = AtomicBool::new(false);

fn ensure_terminal_env() {
    if !ENV_INITIALIZED.swap(true, Ordering::SeqCst) {
        unsafe {
            std::env::set_var("TERM", "xterm-256color");
            std::env::set_var("COLORTERM", "truecolor");
            std::env::set_var("PROMPT_COMMAND", "PS1='\\$ '");
        }
    }
}

pub struct Terminal {
    backend: TerminalBackend,
    pty_receiver: Receiver<(u64, PtyEvent)>,
    exited: bool,
}

impl Terminal {
    pub fn new(id: u64, ctx: &egui::Context, working_dir: &PathBuf, init_command: Option<&str>) -> Self {
        ensure_terminal_env();
        let (sender, pty_receiver) = std::sync::mpsc::channel();

        #[cfg(unix)]
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
        #[cfg(windows)]
        let shell = "cmd.exe".to_string();

        let mut backend = TerminalBackend::new(
            id,
            ctx.clone(),
            sender,
            BackendSettings {
                shell,
                working_directory: Some(working_dir.clone()),
                ..Default::default()
            },
        )
        .expect("Nelze vytvořit terminálový backend");

        if let Some(cmd) = init_command {
            let cmd_with_newline = format!("{}\n", cmd);
            backend.process_command(egui_term::BackendCommand::Write(
                cmd_with_newline.as_bytes().to_vec(),
            ));
        }

        Self {
            backend,
            pty_receiver,
            exited: false,
        }
    }

    /// Vykreslí terminál. Vrací `true` pokud uživatel klikl do oblasti terminálu.
    pub fn ui(&mut self, ui: &mut egui::Ui, focused: bool) -> bool {
        // Zpracovat události z PTY
        while let Ok((_, event)) = self.pty_receiver.try_recv() {
            if let PtyEvent::Exit = event {
                self.exited = true;
            }
        }

        if self.exited {
            ui.centered_and_justified(|ui| {
                ui.label("Terminál ukončen.");
            });
            return false;
        }

        let terminal = TerminalView::new(ui, &mut self.backend)
            .set_focus(focused)
            .set_size(egui::Vec2::new(
                ui.available_width(),
                ui.available_height(),
            ));

        let response = ui.add(terminal);

        let menu_size = 15.0;
        response.context_menu(|ui| {
            let selected = self.backend.selectable_content();
            let has_selection = !selected.trim().is_empty();

            if ui
                .add_enabled(
                    has_selection,
                    egui::Button::new(egui::RichText::new("Kopírovat").size(menu_size)),
                )
                .clicked()
            {
                ui.ctx().copy_text(selected);
                ui.close_menu();
            }

            if ui
                .button(egui::RichText::new("Vložit").size(menu_size))
                .clicked()
            {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    if let Ok(text) = clipboard.get_text() {
                        self.backend.process_command(
                            egui_term::BackendCommand::Write(text.as_bytes().to_vec()),
                        );
                    }
                }
                ui.close_menu();
            }
        });

        response.clicked() || response.hovered() && ui.input(|i| i.pointer.any_pressed())
    }
}
