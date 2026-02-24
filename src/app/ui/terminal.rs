use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use alacritty_terminal::grid::Dimensions;
use eframe::egui;
use egui_term::{BackendSettings, PtyEvent, TerminalBackend, TerminalView};
use regex::Regex;

use crate::config;

#[derive(Clone, Debug)]
pub enum TerminalAction {
    /// User is hovering or interacting with the terminal area.
    Hovered,
    /// User clicked in the terminal area, but not on a specific path.
    Clicked,
    /// User clicked on a file path: (path, line, col).
    Navigate(PathBuf, usize, usize),
}

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
    /// Regex for Rust error paths: "file.rs:line:col" or "file.rs:line"
    path_regex: Regex,
    /// Cache for path detection to save CPU: (Point, Option<Action>)
    path_cache: Option<((i32, usize), Option<TerminalAction>)>,
    /// Whether the user is currently selecting text via mouse drag
    is_selecting: bool,
}

impl Terminal {
    pub fn new(
        id: u64,
        ctx: &egui::Context,
        working_dir: &std::path::Path,
        init_command: Option<&str>,
    ) -> Self {
        // Path regex: looks for something like src/main.rs:10:5 or src/main.rs:10
        let path_regex = Regex::new(r"([a-zA-Z0-9._/\\-]+\.rs):(\d+)(?::(\d+))?").unwrap();

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

    pub fn is_exited(&self) -> bool {
        self.exited
    }

    /// Renders the terminal. Returns Some(TerminalAction) if the user clicked or requested navigation.
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        focused: bool,
        font_size: f32,
        i18n: &crate::i18n::I18n,
    ) -> Option<TerminalAction> {
        let mut action = None;

        let mut pty_writes: Vec<Vec<u8>> = Vec::new();
        if let Some(pty_receiver) = &self.pty_receiver {
            for _ in 0..config::TERMINAL_MAX_EVENTS_PER_FRAME {
                match pty_receiver.try_recv() {
                    Ok((_, PtyEvent::Exit)) => {
                        self.exited = true;
                    }
                    Ok((_, PtyEvent::PtyWrite(text))) => {
                        pty_writes.push(text.into_bytes());
                    }
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
        }
        for bytes in pty_writes {
            if let Some(backend) = &mut self.backend {
                backend.process_command(egui_term::BackendCommand::Write(bytes));
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
                .set_focus(focused && !self.exited)
                .set_font(term_font)
                .set_size(egui::Vec2::new(term_width, term_height))
        };

        let response = ui.add(terminal);

        // --- SAFE AUTO-SCROLL AND SELECTION UPDATE ---
        if focused && !self.exited {
            let pointer = ui.input(|i| i.pointer.clone());
            if pointer.primary_down() {
                if response.contains_pointer() || self.is_selecting {
                    self.is_selecting = true;

                    if let Some(pos) = pointer.interact_pos() {
                        let rel_y = pos.y - response.rect.min.y;

                        let mut scroll_amount = 0;
                        if rel_y < 0.0 {
                            scroll_amount = 1; // Scroll up
                        } else if rel_y > response.rect.height() {
                            scroll_amount = -1; // Scroll down
                        }

                        if let Some(backend) = &mut self.backend {
                            if scroll_amount != 0 {
                                backend.process_command(egui_term::BackendCommand::Scroll(
                                    scroll_amount,
                                ));
                            }

                            // Only update selection if we actually moved or scrolled
                            // We MUST clamp the Y coordinate to be within the visible area (0 to height-1)
                            // to avoid the "assertion failed: requested.0 < self.visible_lines" panic in Alacritty.
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

        if response.clicked() || response.has_focus() {
            action = Some(TerminalAction::Clicked);
        } else if response.hovered() {
            action = Some(TerminalAction::Hovered);
        }

        // --- PATH DETECTION AND NAVIGATION ---
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

                    let current_point = (grid_line_idx, col_idx);
                    let cached_result = if let Some((point, res)) = &self.path_cache {
                        if *point == current_point {
                            Some(res.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let nav_action = if let Some(res) = cached_result {
                        res
                    } else {
                        let mut detected = None;
                        // Use a safer line access method to avoid panics
                        let num_lines = content.grid.total_lines() as i32;
                        let history_size = content.grid.history_size() as i32;

                        // Line index must be between -history_size and screen_lines
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
                                if col_idx >= mat.start() && col_idx < mat.end() {
                                    let path_str = &cap[1];
                                    let line = cap[2].parse().unwrap_or(1);
                                    let col = cap
                                        .get(3)
                                        .map(|m| m.as_str().parse().unwrap_or(1))
                                        .unwrap_or(1);
                                    detected = Some(TerminalAction::Navigate(
                                        PathBuf::from(path_str),
                                        line,
                                        col,
                                    ));
                                    break;
                                }
                            }
                        }
                        self.path_cache = Some((current_point, detected.clone()));
                        detected
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

        let menu_size = 15.0;
        response.context_menu(|ui| {
            let selected = if let Some(backend) = self.backend.as_ref() {
                let content = backend.last_content();
                let mut result = String::new();
                if let Some(range) = content.selectable_range {
                    let mut last_line = None;
                    let mut current_line_buffer = String::new();
                    let mut was_wrapped = false;

                    let num_cols = content.grid.columns();
                    // BEWARE: Alacritty line indexing is tricky.
                    // Let's use a VERY safe iteration method to avoid panics.
                    let total_lines = content.grid.total_lines() as i32;
                    let history_size = content.grid.history_size() as i32;

                    for line_idx in -history_size..(total_lines - history_size) {
                        let line = alacritty_terminal::index::Line(line_idx);
                        let row = &content.grid[line];
                        for col_idx in 0..num_cols {
                            let col = alacritty_terminal::index::Column(col_idx);
                            let point = alacritty_terminal::index::Point::new(line, col);

                            if range.contains(point) {
                                let cell = &row[col];
                                if let Some(last) = last_line
                                    && line != last
                                {
                                    if was_wrapped {
                                        let trimmed = current_line_buffer.trim_end();
                                        result.push_str(trimmed);
                                        if current_line_buffer.len() > trimmed.len() {
                                            result.push(' ');
                                        }
                                    } else {
                                        result.push_str(current_line_buffer.trim_end());
                                        result.push('\n');
                                    }
                                    current_line_buffer.clear();
                                }
                                current_line_buffer.push(cell.c);
                                last_line = Some(line);
                                was_wrapped = cell
                                    .flags
                                    .contains(alacritty_terminal::term::cell::Flags::WRAPLINE);
                            }
                        }
                    }
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
        let scroll_frac = display_offset as f32 / history_size as f32;
        let thumb_top = sb_rect.min.y + (1.0 - scroll_frac) * track_range;
        let thumb_rect = egui::Rect::from_min_size(
            egui::Pos2::new(sb_rect.min.x + 2.0, thumb_top),
            egui::Vec2::new(config::TERMINAL_SCROLLBAR_WIDTH - 4.0, thumb_h),
        );

        let sb_id = ui.id().with("scrollbar");
        let sb_response = ui.interact(sb_rect, sb_id, egui::Sense::drag());
        let thumb_color = if sb_response.hovered() || sb_response.dragged() {
            egui::Color32::from_gray(140)
        } else {
            egui::Color32::from_gray(90)
        };
        painter.rect_filled(thumb_rect, egui::CornerRadius::same(3), thumb_color);

        if sb_response.dragged() {
            let dy = sb_response.drag_delta().y;
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

fn terminal_key_bytes(key: egui::Key, modifiers: egui::Modifiers) -> Option<Vec<u8>> {
    use egui::Key::*;
    if modifiers.ctrl && !modifiers.shift && !modifiers.alt {
        let b: u8 = match key {
            A => 0x01,
            B => 0x02,
            C => 0x03,
            D => 0x04,
            E => 0x05,
            F => 0x06,
            G => 0x07,
            H => 0x08,
            I => 0x09,
            J => 0x0a,
            K => 0x0b,
            L => 0x0c,
            M => 0x0d,
            N => 0x0e,
            O => 0x0f,
            P => 0x10,
            Q => 0x11,
            R => 0x12,
            S => 0x13,
            T => 0x14,
            U => 0x15,
            V => 0x16,
            W => 0x17,
            X => 0x18,
            Y => 0x19,
            Z => 0x1a,
            _ => return None,
        };
        return Some(vec![b]);
    }
    if modifiers.is_none() {
        return match key {
            Enter => Some(vec![0x0d]),
            Backspace => Some(vec![0x7f]),
            Escape => Some(vec![0x1b]),
            Tab => Some(vec![0x09]),
            Delete => Some(b"\x1b[3~".to_vec()),
            Insert => Some(b"\x1b[2~".to_vec()),
            Home => Some(b"\x1b[H".to_vec()),
            End => Some(b"\x1b[F".to_vec()),
            PageUp => Some(b"\x1b[5~".to_vec()),
            PageDown => Some(b"\x1b[6~".to_vec()),
            ArrowUp => Some(b"\x1b[A".to_vec()),
            ArrowDown => Some(b"\x1b[B".to_vec()),
            ArrowLeft => Some(b"\x1b[D".to_vec()),
            ArrowRight => Some(b"\x1b[C".to_vec()),
            _ => None,
        };
    }
    if modifiers == egui::Modifiers::SHIFT && key == Tab {
        return Some(b"\x1b[Z".to_vec());
    }
    None
}

impl Drop for Terminal {
    fn drop(&mut self) {
        if !self.exited
            && let Some(backend) = &mut self.backend
        {
            backend.process_command(egui_term::BackendCommand::Write(b"exit\n".to_vec()));
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        #[cfg(unix)]
        self.kill_process_group();
    }
}
