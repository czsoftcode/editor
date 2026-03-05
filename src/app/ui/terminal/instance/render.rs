use super::Terminal;
use crate::config;
use alacritty_terminal::grid::Dimensions;
use eframe::egui;

fn mix_channel(a: u8, b: u8, t: f32) -> u8 {
    let t = t.clamp(0.0, 1.0);
    ((a as f32) * (1.0 - t) + (b as f32) * t).round() as u8
}

fn mix_color(base: egui::Color32, target: egui::Color32, t: f32) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        mix_channel(base.r(), target.r(), t),
        mix_channel(base.g(), target.g(), t),
        mix_channel(base.b(), target.b(), t),
        255,
    )
}

pub(crate) fn scrollbar_track_color(visuals: &egui::Visuals) -> egui::Color32 {
    if visuals.dark_mode {
        mix_color(visuals.panel_fill, egui::Color32::BLACK, 0.18)
    } else {
        mix_color(visuals.panel_fill, egui::Color32::WHITE, 0.12)
    }
}

pub(crate) fn scrollbar_thumb_color(visuals: &egui::Visuals, active: bool) -> egui::Color32 {
    let track = scrollbar_track_color(visuals);
    let text = visuals.widgets.active.fg_stroke.color;
    let mix = match (visuals.dark_mode, active) {
        (true, false) => 0.52,
        (true, true) => 0.72,
        (false, false) => 0.62,
        (false, true) => 0.82,
    };
    mix_color(track, text, mix)
}

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
            scrollbar_track_color(ui.visuals()),
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
                scrollbar_thumb_color(ui.visuals(), false),
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
            scrollbar_thumb_color(ui.visuals(), true)
        } else {
            scrollbar_thumb_color(ui.visuals(), false)
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

#[cfg(test)]
mod tests {
    use super::{scrollbar_thumb_color, scrollbar_track_color};
    use eframe::egui::{Color32, Visuals};

    fn srgb_to_linear(v: u8) -> f32 {
        let value = v as f32 / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }

    fn relative_luminance(color: Color32) -> f32 {
        0.2126 * srgb_to_linear(color.r())
            + 0.7152 * srgb_to_linear(color.g())
            + 0.0722 * srgb_to_linear(color.b())
    }

    fn contrast_distance(a: Color32, b: Color32) -> f32 {
        (relative_luminance(a) - relative_luminance(b)).abs()
    }

    #[test]
    fn terminal_scrollbar_track_differs_between_light_and_dark() {
        let dark = scrollbar_track_color(&Visuals::dark());
        let light = scrollbar_track_color(&Visuals::light());
        assert_ne!(dark, light, "track color must react to theme mode");
    }

    #[test]
    fn terminal_scrollbar_thumb_active_has_higher_contrast_than_idle() {
        for visuals in [Visuals::dark(), Visuals::light()] {
            let track = scrollbar_track_color(&visuals);
            let idle = scrollbar_thumb_color(&visuals, false);
            let active = scrollbar_thumb_color(&visuals, true);
            assert!(
                contrast_distance(active, track) > contrast_distance(idle, track),
                "active thumb should stand out more than idle"
            );
        }
    }

    #[test]
    fn terminal_scrollbar_light_track_is_not_darker_than_panel() {
        let visuals = Visuals::light();
        let track = scrollbar_track_color(&visuals);
        assert!(
            relative_luminance(track) >= relative_luminance(visuals.panel_fill),
            "light track should not be darker than panel background"
        );
    }
}
