use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use alacritty_terminal::grid::Dimensions;
use eframe::egui;
use egui_term::{BackendSettings, PtyEvent, TerminalBackend, TerminalView};

use crate::config;

pub struct Terminal {
    id: u64,
    working_dir: PathBuf,
    init_command: Option<String>,
    backend: Option<TerminalBackend>,
    pty_receiver: Option<Receiver<(u64, PtyEvent)>>,
    error: Option<String>,
    exited: bool,
    /// Akumulátor pro sub-pixelový drag scrollbaru
    scroll_drag_acc: f32,
}

impl Terminal {
    pub fn new(
        id: u64,
        ctx: &egui::Context,
        working_dir: &PathBuf,
        init_command: Option<&str>,
    ) -> Self {
        match Self::create_backend(id, ctx, working_dir, init_command) {
            Ok((backend, pty_receiver)) => Self {
                id,
                working_dir: working_dir.clone(),
                init_command: init_command.map(|s| s.to_string()),
                backend: Some(backend),
                pty_receiver: Some(pty_receiver),
                error: None,
                exited: false,
                scroll_drag_acc: 0.0,
            },
            Err(err) => Self {
                id,
                working_dir: working_dir.clone(),
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
        working_dir: &PathBuf,
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
                working_directory: Some(working_dir.clone()),
                ..Default::default()
            },
        )
        .map_err(|e| format!("Nelze vytvořit terminálový backend: {e}"))?;

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

    /// Vykreslí terminál. Vrací `true` pokud uživatel klikl do oblasti terminálu.
    pub fn ui(&mut self, ui: &mut egui::Ui, focused: bool, font_size: f32) -> bool {
        // Zpracovat události z PTY
        if let Some(pty_receiver) = &self.pty_receiver {
            while let Ok((_, event)) = pty_receiver.try_recv() {
                if let PtyEvent::Exit = event {
                    self.exited = true;
                }
            }
        }

        if let Some(err) = &self.error {
            let mut restart = false;
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.colored_label(
                    egui::Color32::from_rgb(230, 120, 120),
                    "Terminál není dostupný.",
                );
                ui.add_space(4.0);
                ui.label(err);
                ui.add_space(8.0);
                if ui.button("Zkusit znovu").clicked() {
                    restart = true;
                }
            });
            if restart {
                self.restart(ui.ctx());
            }
            return false;
        }

        if self.exited {
            let mut restart = false;
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label("Terminál ukončen.");
                ui.add_space(8.0);
                if ui.button("Restartovat").clicked() {
                    restart = true;
                }
            });
            if restart {
                self.restart(ui.ctx());
            }
            return false;
        }

        let term_height = ui.available_height();
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
                .set_focus(focused)
                .set_font(term_font)
                .set_size(egui::Vec2::new(term_width, term_height))
        };

        let response = ui.add(terminal);

        // Neaktivní terminál: ztmavení + dutý kurzor místo plného bloku
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

        // Kontext menu
        let menu_size = 15.0;
        response.context_menu(|ui| {
            let selected = self
                .backend
                .as_ref()
                .map(|b| b.selectable_content())
                .unwrap_or_default();
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
                        if let Some(backend) = &mut self.backend {
                            backend.process_command(egui_term::BackendCommand::Write(
                                text.as_bytes().to_vec(),
                            ));
                        }
                    }
                }
                ui.close_menu();
            }
        });

        // Scrollbar
        self.draw_scrollbar(ui, response.rect, term_height);

        response.hovered()
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

        // Pozadí scrollbaru
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
            // Žádná historie — scrollbar jako dekorace (plný track = vše viditelné)
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

        // display_offset = 0 → jsme dole → palec dole
        // display_offset = history_size → jsme nahoře → palec nahoře
        let scroll_frac = display_offset as f32 / history_size as f32;
        let thumb_top = sb_rect.min.y + (1.0 - scroll_frac) * track_range;

        let thumb_rect = egui::Rect::from_min_size(
            egui::Pos2::new(sb_rect.min.x + 2.0, thumb_top),
            egui::Vec2::new(config::TERMINAL_SCROLLBAR_WIDTH - 4.0, thumb_h),
        );

        // Interakce
        let sb_id = ui.id().with("scrollbar");
        let sb_response = ui.interact(sb_rect, sb_id, egui::Sense::drag());

        let thumb_color = if sb_response.hovered() || sb_response.dragged() {
            egui::Color32::from_gray(140)
        } else {
            egui::Color32::from_gray(90)
        };
        painter.rect_filled(thumb_rect, egui::CornerRadius::same(3), thumb_color);

        // Drag: přeložit pixely na řádky
        if sb_response.dragged() {
            let dy = sb_response.drag_delta().y;
            // Negativní dy (tažení nahoru) = scroll do historie = kladný delta
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

        // Klik na track (mimo palec) = skok o stránku
        if sb_response.clicked() {
            if let Some(pos) = sb_response.interact_pointer_pos() {
                let page = screen_lines as i32;
                if pos.y < thumb_rect.min.y {
                    backend.process_command(egui_term::BackendCommand::Scroll(page));
                } else if pos.y > thumb_rect.max.y {
                    backend.process_command(egui_term::BackendCommand::Scroll(-page));
                }
            }
        }
    }
}
