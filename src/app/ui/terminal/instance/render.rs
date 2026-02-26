use super::Terminal;
use crate::config;
use alacritty_terminal::grid::Dimensions;
use eframe::egui;

impl Terminal {
    pub(crate) fn draw_scrollbar(&mut self, ui: &mut egui::Ui, term_rect: egui::Rect, height: f32) {
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
