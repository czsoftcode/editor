use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use eframe::egui;
use egui_term::{BackendSettings, PtyEvent, TerminalBackend, TerminalView};

pub struct Terminal {
    backend: TerminalBackend,
    pty_receiver: Receiver<(u64, PtyEvent)>,
    exited: bool,
}

impl Terminal {
    pub fn new(id: u64, ctx: &egui::Context, working_dir: &PathBuf) -> Self {
        let (sender, pty_receiver) = std::sync::mpsc::channel();

        #[cfg(unix)]
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
        #[cfg(windows)]
        let shell = "cmd.exe".to_string();

        let backend = TerminalBackend::new(
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
        response.clicked() || response.hovered() && ui.input(|i| i.pointer.any_pressed())
    }
}
