pub mod backend;
pub mod input;
pub mod render;
pub mod theme;

use self::input::terminal_key_bytes;
use self::theme::terminal_theme_for_visuals_with_focus;
use crate::config;
use alacritty_terminal::grid::Dimensions;
use eframe::egui;
use egui_term::{PtyEvent, TerminalBackend, TerminalView};
use regex::Regex;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

#[derive(Clone, Debug)]
pub enum TerminalAction {
    /// User is hovering or interacting with the terminal area.
    Hovered,
    /// User clicked in the terminal area, but not on a specific path.
    Clicked,
    /// User clicked on a file path: (path, line, col).
    Navigate(PathBuf, usize, usize),
}

pub struct Terminal {
    pub(crate) id: u64,
    pub(crate) working_dir: PathBuf,
    pub(crate) init_command: Option<String>,
    pub(crate) backend: Option<TerminalBackend>,
    pub(crate) pty_receiver: Option<Receiver<(u64, PtyEvent)>>,
    pub(crate) error: Option<String>,
    pub(crate) exited: bool,
    /// Accumulator for sub-pixel drag scrolling
    pub(crate) scroll_drag_acc: f32,
    /// Regex for Rust error paths: "file.rs:line:col" or "file.rs:line"
    pub(crate) path_regex: Regex,
    /// Cache for path detection to save CPU: (GridLineIdx, Vec<TerminalAction>)
    #[allow(clippy::type_complexity)]
    pub(crate) path_cache: Option<(i32, Vec<(std::ops::Range<usize>, TerminalAction)>)>,
    /// Whether the user is currently selecting text via mouse drag
    pub(crate) is_selecting: bool,
    /// Whether new output has arrived while the terminal was not focused.
    pub(crate) has_unread_output: bool,
    /// Whether a graceful exit has already been requested.
    pub(crate) exit_requested: bool,
}

impl Terminal {
    pub fn new(
        id: u64,
        ctx: &egui::Context,
        working_dir: &std::path::Path,
        init_command: Option<&str>,
    ) -> Self {
        let path_regex = Regex::new(r"([a-zA-Z0-9._/\-]+\.rs):(\d+)(?::(\d+))?").unwrap();

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
                path_regex,
                path_cache: None,
                is_selecting: false,
                has_unread_output: false,
                exit_requested: false,
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
                path_regex,
                path_cache: None,
                is_selecting: false,
                has_unread_output: false,
                exit_requested: false,
            },
        }
    }

    pub fn send_command(&mut self, command: &str) {
        let cmd = format!(
            "{}
",
            command
        );
        if let Some(backend) = &mut self.backend {
            backend.process_command(egui_term::BackendCommand::Write(cmd.as_bytes().to_vec()));
        }
    }

    pub fn is_exited(&self) -> bool {
        self.exited
    }

    pub fn request_graceful_exit(&mut self) {
        if self.exit_requested || self.exited {
            return;
        }
        if let Some(backend) = &mut self.backend {
            backend.process_command(egui_term::BackendCommand::Write(b"exit\n".to_vec()));
            self.exit_requested = true;
        }
    }

    pub fn tick_background(&mut self) {
        if let Some(pty_receiver) = &self.pty_receiver {
            for _ in 0..config::TERMINAL_MAX_EVENTS_PER_FRAME {
                match pty_receiver.try_recv() {
                    Ok((_, PtyEvent::Exit)) => {
                        self.exited = true;
                        break;
                    }
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        focused: bool,
        font_size: f32,
        i18n: &crate::i18n::I18n,
    ) -> Option<TerminalAction> {
        let mut action = None;

        if focused {
            self.has_unread_output = false;
        }

        let mut pty_batch: Vec<u8> = Vec::new();
        if let Some(pty_receiver) = &self.pty_receiver {
            let start_time = std::time::Instant::now();
            for _ in 0..config::TERMINAL_MAX_EVENTS_PER_FRAME {
                match pty_receiver.try_recv() {
                    Ok((_, PtyEvent::Exit)) => {
                        self.exited = true;
                    }
                    Ok((_, PtyEvent::PtyWrite(text))) => {
                        pty_batch.extend_from_slice(text.as_bytes());
                        if !focused {
                            self.has_unread_output = true;
                        }
                    }
                    Ok(_) => {}
                    Err(_) => break,
                }
                // Time-based throttle (Plan 01): max 2ms per frame
                if start_time.elapsed().as_millis() >= 2 {
                    break;
                }
            }
        }

        // Batch write (Plan 02): call process_command only once
        if !pty_batch.is_empty()
            && let Some(backend) = &mut self.backend
        {
            backend.process_command(egui_term::BackendCommand::Write(pty_batch));
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
                return Some(TerminalAction::Clicked);
            }
            return None;
        }

        if self.exited && focused && ui.input(|i| i.key_pressed(egui::Key::R)) {
            self.restart(ui.ctx());
            return Some(TerminalAction::Clicked);
        }

        let exit_banner_height = if self.exited { 24.0 } else { 0.0 };
        let term_height = (ui.available_height() - exit_banner_height).max(1.0);
        let term_width = (ui.available_width() - config::TERMINAL_SCROLLBAR_WIDTH).max(10.0);
        self.backend.as_ref()?;

        let term_font = egui_term::TerminalFont::new(egui_term::FontSettings {
            font_type: egui::FontId::monospace(font_size),
        });
        let terminal = {
            let backend = self.backend.as_mut()?;
            TerminalView::new(ui, backend)
                .set_theme(terminal_theme_for_visuals_with_focus(ui.visuals(), focused))
                .set_focus(focused && !self.exited)
                .set_font(term_font)
                .set_size(egui::Vec2::new(term_width, term_height))
        };

        let response = ui.add(terminal);

        if focused && !self.exited {
            let pointer = ui.input(|i| i.pointer.clone());
            if pointer.primary_down() {
                if response.contains_pointer() || self.is_selecting {
                    self.is_selecting = true;
                    if let Some(pos) = pointer.interact_pos() {
                        let rel_y = pos.y - response.rect.min.y;
                        let mut scroll_amount = 0;
                        if rel_y < 0.0 {
                            scroll_amount = 1;
                        } else if rel_y > response.rect.height() {
                            scroll_amount = -1;
                        }

                        if let Some(backend) = &mut self.backend {
                            if scroll_amount != 0 {
                                backend.process_command(egui_term::BackendCommand::Scroll(
                                    scroll_amount,
                                ));
                            }
                            let clamped_x = (pos.x - response.rect.min.x)
                                .clamp(0.0, response.rect.width() - 1.0);
                            let clamped_y = rel_y.clamp(0.0, response.rect.height() - 1.0);
                            backend.process_command(egui_term::BackendCommand::SelectUpdate(
                                clamped_x, clamped_y,
                            ));
                        }
                    }
                }
            } else {
                self.is_selecting = false;
            }
        }

        if response.clicked() {
            action = Some(TerminalAction::Clicked);
        } else if response.hovered() {
            action = Some(TerminalAction::Hovered);
        }

        if let Some(pos) = response.interact_pointer_pos() {
            if let Some(backend) = &self.backend {
                let content = backend.last_content();
                let cell_w = content.terminal_size.cell_width as f32;
                let cell_h = content.terminal_size.cell_height as f32;
                if cell_w > 0.0 && cell_h > 0.0 {
                    let rel_x = pos.x - response.rect.min.x;
                    let rel_y = pos.y - response.rect.min.y;
                    let col_idx = (rel_x / cell_w) as usize;
                    let display_offset = content.grid.display_offset();
                    let line_idx = (rel_y / cell_h) as i32;
                    let grid_line_idx = line_idx - display_offset as i32;

                    let nav_action = if let Some((cached_line, actions)) = &self.path_cache
                        && *cached_line == grid_line_idx
                    {
                        // Use cached actions for this line
                        actions
                            .iter()
                            .find(|(range, _)| range.contains(&col_idx))
                            .map(|(_, act)| act.clone())
                    } else {
                        // Re-parse the whole line and update cache
                        let mut line_actions = Vec::new();
                        let num_lines = content.grid.total_lines() as i32;
                        let history_size = content.grid.history_size() as i32;

                        if grid_line_idx >= -history_size
                            && grid_line_idx < (num_lines - history_size)
                        {
                            let grid_line = alacritty_terminal::index::Line(grid_line_idx);
                            let row = &content.grid[grid_line];
                            let mut line_text = String::new();
                            let num_cols = content.grid.columns();
                            for col in 0..num_cols {
                                let cell = &row[alacritty_terminal::index::Column(col)];
                                line_text.push(cell.c);
                            }

                            for cap in self.path_regex.captures_iter(&line_text) {
                                let mat = cap.get(0).unwrap();
                                let path_str = &cap[1];
                                let line = cap[2].parse().unwrap_or(1);
                                let col = cap
                                    .get(3)
                                    .map(|m| m.as_str().parse().unwrap_or(1))
                                    .unwrap_or(1);

                                line_actions.push((
                                    mat.range(),
                                    TerminalAction::Navigate(PathBuf::from(path_str), line, col),
                                ));
                            }
                        }

                        let result = line_actions
                            .iter()
                            .find(|(range, _)| range.contains(&col_idx))
                            .map(|(_, act)| act.clone());

                        self.path_cache = Some((grid_line_idx, line_actions));
                        result
                    };

                    if let Some(TerminalAction::Navigate(p, l, c)) = nav_action {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        if response.clicked() {
                            action = Some(TerminalAction::Navigate(p, l, c));
                        }
                    }
                }
            }
        } else {
            self.path_cache = None;
        }

        if focused
            && !self.exited
            && ui.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Cut)))
            && let Some(backend) = self.backend.as_mut()
        {
            backend.process_command(egui_term::BackendCommand::Write(vec![0x18]));
        }

        if focused && !self.exited && !response.contains_pointer() {
            let mut writes: Vec<Vec<u8>> = Vec::new();
            ui.input(|i| {
                for event in &i.events {
                    match event {
                        egui::Event::Text(text) if !text.is_empty() => {
                            writes.push(text.as_bytes().to_vec());
                        }
                        egui::Event::Key {
                            key,
                            pressed: true,
                            modifiers,
                            ..
                        } => {
                            if let Some(bytes) = terminal_key_bytes(*key, *modifiers) {
                                writes.push(bytes);
                            }
                        }
                        _ => {}
                    }
                }
            });
            for bytes in writes {
                if let Some(backend) = self.backend.as_mut() {
                    backend.process_command(egui_term::BackendCommand::Write(bytes));
                }
            }
        }

        if !focused {
            let painter = ui.painter_at(response.rect);
            let visuals = ui.visuals();
            let Some(content) = self.backend.as_ref().map(|b| b.last_content()) else {
                return action;
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

            // Neaktivni terminal v light modu netreba "svitit do bila".
            // Tlumi ho tmavou slonovou kosti, aby text pusobil sede a jemneji.
            let cursor_fill = if visuals.dark_mode {
                visuals.panel_fill.gamma_multiply(0.85)
            } else {
                egui::Color32::from_rgb(190, 182, 166)
            };
            let overlay_fill = if visuals.dark_mode {
                egui::Color32::from_black_alpha(60)
            } else {
                egui::Color32::from_rgba_premultiplied(0, 0, 0, 0)
            };
            let cursor_stroke = if visuals.dark_mode {
                visuals.widgets.active.fg_stroke.color.gamma_multiply(1.1)
            } else {
                visuals
                    .widgets
                    .noninteractive
                    .fg_stroke
                    .color
                    .gamma_multiply(0.9)
            };

            if let Some(rect) = cursor_rect {
                painter.rect_filled(rect, egui::CornerRadius::ZERO, cursor_fill);
            }
            painter.rect_filled(response.rect, egui::CornerRadius::ZERO, overlay_fill);
            if let Some(rect) = cursor_rect {
                painter.rect_stroke(
                    rect,
                    egui::CornerRadius::ZERO,
                    egui::Stroke::new(1.5, cursor_stroke),
                    egui::StrokeKind::Middle,
                );
            }
        }

        response.context_menu(|ui| {
            let selected_text = if let Some(backend) = self.backend.as_ref() {
                backend.selectable_content()
            } else {
                String::new()
            };

            let has_selection = !selected_text.trim().is_empty();
            let menu_size = 15.0;
            if ui
                .add_enabled(
                    has_selection,
                    egui::Button::new(egui::RichText::new(i18n.get("btn-copy")).size(menu_size)),
                )
                .clicked()
            {
                ui.ctx().copy_text(selected_text);
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

        self.draw_scrollbar(ui, response.rect, term_height);
        if self.exited {
            ui.horizontal(|ui| {
                ui.add_space(6.0);
                ui.colored_label(
                    egui::Color32::from_rgb(200, 180, 80),
                    i18n.get("terminal-exited"),
                );
            });
        }
        action
    }

    #[cfg(unix)]
    pub(crate) fn kill_process_group(&self) {
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
}

impl Drop for Terminal {
    fn drop(&mut self) {
        if !self.exited
            && !self.exit_requested
            && let Some(backend) = &mut self.backend
        {
            backend.process_command(egui_term::BackendCommand::Write(
                b"exit
"
                .to_vec(),
            ));
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        #[cfg(unix)]
        self.kill_process_group();
    }
}
