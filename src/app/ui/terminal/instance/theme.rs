use eframe::egui;
use egui_term::{ColorPalette, TerminalTheme};

fn mix_channel(a: u8, b: u8, t: f32) -> u8 {
    let t = t.clamp(0.0, 1.0);
    ((a as f32) * (1.0 - t) + (b as f32) * t).round() as u8
}

fn mix_color(base: egui::Color32, target: egui::Color32, t: f32) -> egui::Color32 {
    egui::Color32::from_rgb(
        mix_channel(base.r(), target.r(), t),
        mix_channel(base.g(), target.g(), t),
        mix_channel(base.b(), target.b(), t),
    )
}

fn parse_hex_rgb(hex: &str) -> egui::Color32 {
    let value = hex.strip_prefix('#').unwrap_or(hex);
    let parse = |range: std::ops::Range<usize>| {
        value
            .get(range)
            .and_then(|chunk| u8::from_str_radix(chunk, 16).ok())
    };
    match (parse(0..2), parse(2..4), parse(4..6)) {
        (Some(r), Some(g), Some(b)) => egui::Color32::from_rgb(r, g, b),
        _ => egui::Color32::WHITE,
    }
}

fn to_hex_rgb(color: egui::Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b())
}

fn blend_hex(base: &str, tone: egui::Color32, amount: f32) -> String {
    to_hex_rgb(mix_color(parse_hex_rgb(base), tone, amount))
}

fn warm_ivory_bg(panel_fill: egui::Color32) -> &'static str {
    // WarmIvory panel_fill je (255,252,240) — r > b+10 signalizuje teplý tón
    if panel_fill.r() as i32 - panel_fill.b() as i32 > 10 {
        "#f5f2e8" // teplejší základ pro ivory blending
    } else {
        "#f3f5f7" // původní studená base pro CoolGray/Sepia
    }
}

fn light_terminal_base_palette() -> ColorPalette {
    ColorPalette {
        foreground: String::from("#1f2328"),
        background: String::from("#f3f5f7"),
        black: String::from("#2b3137"),
        red: String::from("#b4232c"),
        green: String::from("#2f7d32"),
        yellow: String::from("#8f5d00"),
        blue: String::from("#1558b0"),
        magenta: String::from("#8b3fa4"),
        cyan: String::from("#006f7a"),
        white: String::from("#d9dde1"),
        bright_black: String::from("#6b7280"),
        bright_red: String::from("#cf3f4b"),
        bright_green: String::from("#3f9f46"),
        bright_yellow: String::from("#a86b00"),
        bright_blue: String::from("#2f79d8"),
        bright_magenta: String::from("#a457c0"),
        bright_cyan: String::from("#0c8f9a"),
        bright_white: String::from("#ffffff"),
        bright_foreground: Some(String::from("#0f1419")),
        dim_foreground: String::from("#5f6771"),
        dim_black: String::from("#4d545c"),
        dim_red: String::from("#8f2b31"),
        dim_green: String::from("#3f6a42"),
        dim_yellow: String::from("#70511a"),
        dim_blue: String::from("#355d90"),
        dim_magenta: String::from("#714a83"),
        dim_cyan: String::from("#3b6870"),
        dim_white: String::from("#9ca5ae"),
    }
}

fn tone_light_palette(palette: ColorPalette, visuals: &egui::Visuals) -> ColorPalette {
    let tone = visuals.panel_fill;
    let bg_base = warm_ivory_bg(tone);
    ColorPalette {
        foreground: blend_hex(&palette.foreground, tone, 0.06),
        background: blend_hex(bg_base, tone, 0.55),
        black: blend_hex(&palette.black, tone, 0.18),
        red: blend_hex(&palette.red, tone, 0.18),
        green: blend_hex(&palette.green, tone, 0.18),
        yellow: blend_hex(&palette.yellow, tone, 0.18),
        blue: blend_hex(&palette.blue, tone, 0.18),
        magenta: blend_hex(&palette.magenta, tone, 0.18),
        cyan: blend_hex(&palette.cyan, tone, 0.18),
        white: blend_hex(&palette.white, tone, 0.28),
        bright_black: blend_hex(&palette.bright_black, tone, 0.14),
        bright_red: blend_hex(&palette.bright_red, tone, 0.16),
        bright_green: blend_hex(&palette.bright_green, tone, 0.16),
        bright_yellow: blend_hex(&palette.bright_yellow, tone, 0.16),
        bright_blue: blend_hex(&palette.bright_blue, tone, 0.16),
        bright_magenta: blend_hex(&palette.bright_magenta, tone, 0.16),
        bright_cyan: blend_hex(&palette.bright_cyan, tone, 0.16),
        bright_white: blend_hex(&palette.bright_white, tone, 0.12),
        bright_foreground: palette
            .bright_foreground
            .as_ref()
            .map(|color| blend_hex(color, tone, 0.06)),
        dim_foreground: blend_hex(&palette.dim_foreground, tone, 0.12),
        dim_black: blend_hex(&palette.dim_black, tone, 0.16),
        dim_red: blend_hex(&palette.dim_red, tone, 0.16),
        dim_green: blend_hex(&palette.dim_green, tone, 0.16),
        dim_yellow: blend_hex(&palette.dim_yellow, tone, 0.16),
        dim_blue: blend_hex(&palette.dim_blue, tone, 0.16),
        dim_magenta: blend_hex(&palette.dim_magenta, tone, 0.16),
        dim_cyan: blend_hex(&palette.dim_cyan, tone, 0.16),
        dim_white: blend_hex(&palette.dim_white, tone, 0.2),
    }
}

pub(crate) fn terminal_palette(visuals: &egui::Visuals) -> ColorPalette {
    if visuals.dark_mode {
        return ColorPalette::default();
    }

    tone_light_palette(light_terminal_base_palette(), visuals)
}

pub(crate) fn terminal_theme_for_visuals(visuals: &egui::Visuals) -> TerminalTheme {
    TerminalTheme::new(Box::new(terminal_palette(visuals)))
}

#[cfg(test)]
mod tests {
    use super::terminal_theme_for_visuals;
    use alacritty_terminal::vte::ansi::{Color, NamedColor};
    use crate::settings::{LightVariant, Settings};
    use eframe::egui::{Color32, Visuals};
    use egui_term::{ColorPalette, TerminalTheme};
    use std::collections::HashSet;

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

    fn contrast_ratio(a: Color32, b: Color32) -> f32 {
        let l1 = relative_luminance(a);
        let l2 = relative_luminance(b);
        if l1 > l2 {
            (l1 + 0.05) / (l2 + 0.05)
        } else {
            (l2 + 0.05) / (l1 + 0.05)
        }
    }

    fn light_visuals(variant: LightVariant) -> Visuals {
        Settings {
            dark_theme: false,
            light_variant: variant,
            ..Default::default()
        }
        .to_egui_visuals()
    }

    #[test]
    fn terminal_theme_light_background_is_light() {
        let visuals = Visuals::light();
        let theme = terminal_theme_for_visuals(&visuals);
        let bg = theme.get_color(Color::Named(NamedColor::Background));

        assert!(relative_luminance(bg) > 0.7, "expected light background, got {bg:?}");
    }

    #[test]
    fn terminal_theme_light_foreground_is_readable_on_light_background() {
        let visuals = Visuals::light();
        let theme = terminal_theme_for_visuals(&visuals);
        let bg = theme.get_color(Color::Named(NamedColor::Background));
        let fg = theme.get_color(Color::Named(NamedColor::Foreground));

        assert!(
            relative_luminance(fg) < relative_luminance(bg),
            "light mode foreground should be darker than background: fg={fg:?} bg={bg:?}"
        );
        assert!(
            contrast_ratio(fg, bg) >= 4.5,
            "foreground contrast too low: fg={fg:?} bg={bg:?}"
        );
    }

    #[test]
    fn terminal_theme_light_ansi_yellow_and_cyan_are_readable() {
        let visuals = Visuals::light();
        let theme = terminal_theme_for_visuals(&visuals);
        let bg = theme.get_color(Color::Named(NamedColor::Background));
        let yellow = theme.get_color(Color::Named(NamedColor::Yellow));
        let cyan = theme.get_color(Color::Named(NamedColor::Cyan));

        assert!(contrast_ratio(yellow, bg) >= 2.2, "yellow contrast too low");
        assert!(contrast_ratio(cyan, bg) >= 2.2, "cyan contrast too low");
    }

    #[test]
    fn terminal_theme_dark_background_stays_dark() {
        let visuals = Visuals::dark();
        let theme = terminal_theme_for_visuals(&visuals);
        let bg = theme.get_color(Color::Named(NamedColor::Background));

        assert!(relative_luminance(bg) < 0.2, "expected dark background, got {bg:?}");
    }

    #[test]
    fn terminal_theme_light_variant_background_differs_between_warm_and_cool() {
        let warm = terminal_theme_for_visuals(&light_visuals(LightVariant::WarmIvory))
            .get_color(Color::Named(NamedColor::Background));
        let cool = terminal_theme_for_visuals(&light_visuals(LightVariant::CoolGray))
            .get_color(Color::Named(NamedColor::Background));

        assert_ne!(
            warm, cool,
            "light variants must produce different terminal background tones"
        );
    }

    #[test]
    fn terminal_theme_light_variant_ansi_cyan_differs_between_cool_and_sepia() {
        let cool = terminal_theme_for_visuals(&light_visuals(LightVariant::CoolGray))
            .get_color(Color::Named(NamedColor::Cyan));
        let sepia = terminal_theme_for_visuals(&light_visuals(LightVariant::Sepia))
            .get_color(Color::Named(NamedColor::Cyan));

        assert_ne!(
            cool, sepia,
            "light variants must produce different ANSI tones"
        );
    }

    #[test]
    fn terminal_theme_light_variant_backgrounds_are_distinct_across_all_three() {
        let backgrounds: HashSet<Color32> = [
            LightVariant::WarmIvory,
            LightVariant::CoolGray,
            LightVariant::Sepia,
        ]
        .into_iter()
        .map(|variant| {
            terminal_theme_for_visuals(&light_visuals(variant))
                .get_color(Color::Named(NamedColor::Background))
        })
        .collect();

        assert_eq!(backgrounds.len(), 3);
    }

    #[test]
    fn terminal_theme_dark_palette_matches_default_theme_for_key_colors() {
        let dark_theme = terminal_theme_for_visuals(&Visuals::dark());
        let default_theme = TerminalTheme::new(Box::new(ColorPalette::default()));
        let sample = [
            NamedColor::Background,
            NamedColor::Foreground,
            NamedColor::Red,
            NamedColor::Green,
            NamedColor::Yellow,
            NamedColor::Blue,
            NamedColor::Cyan,
        ];

        for named in sample {
            assert_eq!(
                dark_theme.get_color(Color::Named(named)),
                default_theme.get_color(Color::Named(named))
            );
        }
    }
}
