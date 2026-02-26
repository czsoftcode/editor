use super::Terminal;
use egui_term::{BackendSettings, PtyEvent, TerminalBackend};
use std::sync::mpsc::Receiver;

impl Terminal {
    pub(crate) fn create_backend(
        id: u64,
        ctx: &eframe::egui::Context,
        working_dir: &std::path::Path,
        init_command: Option<&str>,
    ) -> Result<(TerminalBackend, Receiver<(u64, PtyEvent)>), String> {
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
                working_directory: Some(working_dir.to_path_buf()),
                ..Default::default()
            },
        )
        .map_err(|e| format!("Cannot create terminal backend: {e}"))?;

        if let Some(cmd) = init_command {
            let cmd_with_newline = format!(
                "{}
",
                cmd
            );
            backend.process_command(egui_term::BackendCommand::Write(
                cmd_with_newline.as_bytes().to_vec(),
            ));
        }

        Ok((backend, pty_receiver))
    }

    pub fn restart(&mut self, ctx: &eframe::egui::Context) {
        match Self::create_backend(
            self.id,
            ctx,
            &self.working_dir,
            self.init_command.as_deref(),
        ) {
            Ok((backend, pty_receiver)) => {
                self.backend = Some(backend);
                self.pty_receiver = Some(pty_receiver);
                self.error = None;
                self.exited = false;
                self.scroll_drag_acc = 0.0;
            }
            Err(err) => {
                self.backend = None;
                self.pty_receiver = None;
                self.error = Some(err);
                self.exited = false;
                self.scroll_drag_acc = 0.0;
            }
        }
    }
}
