use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use alacritty_terminal::grid::Dimensions;
use eframe::egui;
use egui_term::{BackendSettings, PtyEvent, TerminalBackend, TerminalView};

use crate::config;

#[cfg(unix)]
pub struct Terminal {
    id: u64,
    working_dir: PathBuf,
    init_command: Option<String>,
    backend: Option<TerminalBackend>,
    pty_receiver: Option<Receiver<(u64, PtyEvent)>>,
    error: Option<String>,
    exited: bool,
    /// Accumulator for sub-pixel drag scrolling
    scroll_drag_acc: f32,
}

impl Terminal {
    pub fn new(
        id: u64,
        ctx: &egui::Context,
        working_dir: &std::path::Path,
        init_command: Option<&str>,
    ) -> Self {
        match Self::create_backend(id, ctx, working_dir, init_command) {
            Ok((backend, pty_receiver)) => Self {
                id,
                working_dir: working_dir.to_path_buf(),
                init_command: init_command.map(|s| s.to_string()),
                backend: Some(backend),
                pty_receiver: Some(pty_receiver),
                error: None,
                exited: false,
                scroll_drag_acc: 0.0,
            },
            Err(err) => Self {
                id,
                working_dir: working_dir.to_path_buf(),
                init_command: init_command.map(|s| s.to_string()),
                backend: None,
                pty_receiver: None,
                error: Some(err),
                exited: false,
                scroll_drag_acc: 0.0,
            },
        }
    }

    fn create_backend(
        id: u64,
        ctx: &egui::Context,
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
            let cmd_with_newline = format!("{}\n", cmd);
            backend.process_command(egui_term::BackendCommand::Write(
                cmd_with_newline.as_bytes().to_vec(),
            ));
        }

        Ok((backend, pty_receiver))
    }

    fn restart(&mut self, ctx: &egui::Context) {
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

    pub fn send_command(&mut self, command: &str) {
        let cmd = format!("{}\n", command);
        if let Some(backend) = &mut self.backend {
            backend.process_command(egui_term::BackendCommand::Write(cmd.as_bytes().to_vec()));
        }
    }

    /// Renders the terminal. Returns `true` if the user clicked into the terminal area.
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        focused: bool,
        font_size: f32,
        i18n: &crate::i18n::I18n,
    ) -> bool {
        // Process events from PTY — limit per frame, the rest will be consumed in the next frame.
        // Without a limit, an output burst (cargo build, grep, etc.) would block the UI for tens of ms.
        if let Some(pty_receiver) = &self.pty_receiver {
            for _ in 0..config::TERMINAL_MAX_EVENTS_PER_FRAME {
                match pty_receiver.try_recv() {
                    Ok((_, PtyEvent::Exit)) => {
                        self.exited = true;
                    }
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
        }

        if let Some(err) = &self.error {
            let mut restart = false;
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.colored_label(
                    egui::Color32::from_rgb(230, 120, 120),
                    i18n.get("terminal-unavailable"),
                );
                ui.add_space(4.0);
                ui.label(err);
                ui.add_space(8.0);
                if ui.button(i18n.get("terminal-retry")).clicked() {
                    restart = true;
                }
            });
            if restart {
                self.restart(ui.ctx());
            }
            return false;
        }

        // 'R' key restarts the terminal after exit (must be focused)
        if self.exited && focused && ui.input(|i| i.key_pressed(egui::Key::R)) {
            self.restart(ui.ctx());
            return true;
        }

        // On exit, reserve the bottom strip for the exit banner; the terminal is still displayed
        // so the user can see the output history.
        let exit_banner_height = if self.exited { 24.0 } else { 0.0 };
        let term_height = (ui.available_height() - exit_banner_height).max(1.0);
        let term_width = (ui.available_width() - config::TERMINAL_SCROLLBAR_WIDTH).max(10.0);
        if self.backend.is_none() {
            return false;
        }

        let term_font = egui_term::TerminalFont::new(egui_term::FontSettings {
            font_type: egui::FontId::monospace(font_size),
        });
        let terminal = {
            let Some(backend) = self.backend.as_mut() else {
                return false;
            };
            TerminalView::new(ui, backend)
                // Defocus terminal after exit — history is shown without cursor
                .set_focus(focused && !self.exited)
                .set_font(term_font)
                .set_size(egui::Vec2::new(term_width, term_height))
        };

        let response = ui.add(terminal);

        // Inactive terminal: dimming + hollow cursor instead of solid block
        if !focused {
            let painter = ui.painter_at(response.rect);
            let Some(content) = self.backend.as_ref().map(|b| b.last_content()) else {
                return false;
            };
            let cell_w = content.terminal_size.cell_width as f32;
            let cell_h = content.terminal_size.cell_height as f32;

            let cursor_rect = if cell_w > 0.0 && cell_h > 0.0 {
                let col = content.grid.cursor.point.column.0 as f32;
                let line = (content.grid.cursor.point.line.0 + content.grid.display_offset() as i32)
                    as f32;
                let cx = response.rect.min.x + cell_w * col;
                let cy = response.rect.min.y + cell_h * line;
                Some(egui::Rect::from_min_size(
                    egui::Pos2::new(cx, cy),
                    egui::Vec2::new(cell_w, cell_h),
                ))
            } else {
                None
            };

            if let Some(rect) = cursor_rect {
                painter.rect_filled(
                    rect,
                    egui::CornerRadius::ZERO,
                    egui::Color32::from_rgb(0x18, 0x18, 0x18),
                );
            }
            painter.rect_filled(
                response.rect,
                egui::CornerRadius::ZERO,
                egui::Color32::from_black_alpha(60),
            );
            if let Some(rect) = cursor_rect {
                painter.rect_stroke(
                    rect,
                    egui::CornerRadius::ZERO,
                    egui::Stroke::new(1.5, egui::Color32::from_gray(200)),
                    egui::StrokeKind::Middle,
                );
            }
        }

        // Context menu
        let menu_size = 15.0;
        response.context_menu(|ui| {
            let selected = if let Some(backend) = self.backend.as_ref() {
                let content = backend.last_content();
                let mut result = String::new();
                if let Some(range) = content.selectable_range {
                    let mut last_line = None;
                    let mut current_line_buffer = String::new();
                    for indexed in content.grid.display_iter() {
                        if range.contains(indexed.point) {
                            if let Some(last) = last_line
                                && indexed.point.line != last
                            {
                                // New line started, trim previous line buffer and add newline
                                result.push_str(current_line_buffer.trim_end());
                                result.push('\n');
                                current_line_buffer.clear();
                            }
                            current_line_buffer.push(indexed.c);
                            last_line = Some(indexed.point.line);
                        }
                    }
                    // Append the last line buffer
                    result.push_str(current_line_buffer.trim_end());
                }
                result
            } else {
                String::new()
            };
            let has_selection = !selected.trim().is_empty();

            if ui
                .add_enabled(
                    has_selection,
                    egui::Button::new(egui::RichText::new(i18n.get("btn-copy")).size(menu_size)),
                )
                .clicked()
            {
                ui.ctx().copy_text(selected);
                ui.close_menu();
            }

            if ui
                .button(egui::RichText::new(i18n.get("btn-paste")).size(menu_size))
                .clicked()
            {
                if let Ok(mut clipboard) = arboard::Clipboard::new()
                    && let Ok(text) = clipboard.get_text()
                    && let Some(backend) = &mut self.backend
                {
                    backend.process_command(egui_term::BackendCommand::Write(
                        text.as_bytes().to_vec(),
                    ));
                }
                ui.close_menu();
            }
        });

        // Scrollbar
        self.draw_scrollbar(ui, response.rect, term_height);

        // Exit banner — displayed below the terminal history
        if self.exited {
            ui.horizontal(|ui| {
                ui.add_space(6.0);
                ui.colored_label(
                    egui::Color32::from_rgb(200, 180, 80),
                    i18n.get("terminal-exited"),
                );
            });
        }

        response.hovered()
    }

    /// Explicitly terminates the shell process group (Unix). Called implicitly from Drop.
    /// Uses SIGTERM on -pid (entire process group), so cargo run and similar
    /// children terminate along with the shell — they don't survive as orphans.
    #[cfg(unix)]
    fn kill_process_group(&self) {
        if self.exited {
            return;
        }
        if let Some(backend) = &self.backend {
            let pid = backend.child_pid as libc::pid_t;
            if pid > 1 {
                unsafe {
                    libc::kill(-pid, libc::SIGTERM);
                }
            }
        }
    }

    fn draw_scrollbar(&mut self, ui: &mut egui::Ui, term_rect: egui::Rect, height: f32) {
        let Some(backend) = self.backend.as_mut() else {
            return;
        };
        let sb_rect = egui::Rect::from_min_size(
            egui::Pos2::new(term_rect.max.x, term_rect.min.y),
            egui::Vec2::new(config::TERMINAL_SCROLLBAR_WIDTH, height),
        );

        let painter = ui.painter_at(sb_rect);

        // Scrollbar background
        painter.rect_filled(
            sb_rect,
            egui::CornerRadius::ZERO,
            egui::Color32::from_rgb(0x18, 0x18, 0x18),
        );

        let content = backend.last_content();
        let history_size = content.grid.history_size();
        let screen_lines = content.grid.screen_lines();
        let display_offset = content.grid.display_offset();

        if history_size == 0 {
            // No history — scrollbar as decoration (full track = everything visible)
            let thumb_rect = sb_rect.shrink2(egui::Vec2::new(2.0, 0.0));
            painter.rect_filled(
                thumb_rect,
                egui::CornerRadius::same(3),
                egui::Color32::from_gray(60),
            );
            ui.allocate_rect(sb_rect, egui::Sense::hover());
            return;
        }

        let total_lines = screen_lines + history_size;
        let track_h = sb_rect.height();
        let thumb_ratio = screen_lines as f32 / total_lines as f32;
        let thumb_h = (thumb_ratio * track_h).max(20.0);
        let track_range = (track_h - thumb_h).max(1.0);

        // display_offset = 0 → bottom → thumb at bottom
        // display_offset = history_size → top → thumb at top
        let scroll_frac = display_offset as f32 / history_size as f32;
        let thumb_top = sb_rect.min.y + (1.0 - scroll_frac) * track_range;

        let thumb_rect = egui::Rect::from_min_size(
            egui::Pos2::new(sb_rect.min.x + 2.0, thumb_top),
            egui::Vec2::new(config::TERMINAL_SCROLLBAR_WIDTH - 4.0, thumb_h),
        );

        // Interaction
        let sb_id = ui.id().with("scrollbar");
        let sb_response = ui.interact(sb_rect, sb_id, egui::Sense::drag());

        let thumb_color = if sb_response.hovered() || sb_response.dragged() {
            egui::Color32::from_gray(140)
        } else {
            egui::Color32::from_gray(90)
        };
        painter.rect_filled(thumb_rect, egui::CornerRadius::same(3), thumb_color);

        // Drag: translate pixels to lines
        if sb_response.dragged() {
            let dy = sb_response.drag_delta().y;
            // Negative dy (dragging up) = scroll to history = positive delta
            self.scroll_drag_acc -= dy;
            let lines_per_pixel = history_size as f32 / track_range;
            let line_delta = (self.scroll_drag_acc * lines_per_pixel) as i32;
            if line_delta != 0 {
                self.scroll_drag_acc -= line_delta as f32 / lines_per_pixel;
                backend.process_command(egui_term::BackendCommand::Scroll(line_delta));
            }
        } else {
            self.scroll_drag_acc = 0.0;
        }

        // Click on track (outside thumb) = page scroll
        if sb_response.clicked()
            && let Some(pos) = sb_response.interact_pointer_pos()
        {
            let page = screen_lines as i32;
            if pos.y < thumb_rect.min.y {
                backend.process_command(egui_term::BackendCommand::Scroll(page));
            } else if pos.y > thumb_rect.max.y {
                backend.process_command(egui_term::BackendCommand::Scroll(-page));
            }
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        #[cfg(unix)]
        self.kill_process_group();
    }
}
