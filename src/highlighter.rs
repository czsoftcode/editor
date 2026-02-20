use eframe::egui;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    pub fn highlight(
        &self,
        text: &str,
        extension: &str,
        filename: &str,
        font_size: f32,
    ) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();

        let is_env_file = filename.starts_with(".env");
        let mapped_ext = if is_env_file {
            "sh"
        } else {
            match extension {
                "dockerignore" => "gitignore",
                "lock" => "toml",
                _ => extension,
            }
        };

        // Try extension, then full filename, then fallback to plain text
        let syntax = self
            .syntax_set
            .find_syntax_by_extension(mapped_ext)
            .or_else(|| self.syntax_set.find_syntax_by_extension(filename))
            .or_else(|| {
                let name_lower = filename.to_lowercase();
                match name_lower.as_str() {
                    "dockerfile" => self.syntax_set.find_syntax_by_extension("Dockerfile"),
                    "makefile" => self.syntax_set.find_syntax_by_name("Makefile"),
                    _ => None,
                }
            })
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = &self.theme_set.themes["base16-ocean.dark"];
        let mut h = HighlightLines::new(syntax, theme);

        for line in LinesWithEndings::from(text) {
            let ranges = h.highlight_line(line, &self.syntax_set).unwrap_or_default();
            for (style, segment) in ranges {
                let color = egui::Color32::from_rgb(
                    style.foreground.r,
                    style.foreground.g,
                    style.foreground.b,
                );
                let mut text_format = egui::TextFormat {
                    font_id: egui::FontId::monospace(font_size),
                    color,
                    ..Default::default()
                };
                if style.font_style.contains(FontStyle::BOLD) {
                    text_format.font_id = egui::FontId::new(font_size, egui::FontFamily::Monospace);
                }
                if style.font_style.contains(FontStyle::ITALIC) {
                    text_format.italics = true;
                }
                job.append(segment, 0.0, text_format);
            }
        }

        job
    }

    pub fn background_color(&self) -> egui::Color32 {
        let theme = &self.theme_set.themes["base16-ocean.dark"];
        if let Some(bg) = theme.settings.background {
            egui::Color32::from_rgb(bg.r, bg.g, bg.b)
        } else {
            egui::Color32::from_rgb(43, 48, 59)
        }
    }
}
