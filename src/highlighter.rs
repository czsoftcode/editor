use eframe::egui;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    /// Fast full-file cache (for scrolling/rendering).
    /// Using Arc to avoid cloning massive LayoutJobs for large files.
    cache: std::sync::Mutex<HashMap<u64, Arc<egui::text::LayoutJob>>>,
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            cache: std::sync::Mutex::new(HashMap::new()),
        }
    }

    pub fn highlight(
        &self,
        text: &str,
        extension: &str,
        filename: &str,
        font_size: f32,
    ) -> Arc<egui::text::LayoutJob> {
        // Compute cache key
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        text.hash(&mut hasher);
        extension.hash(&mut hasher);
        filename.hash(&mut hasher);
        ((font_size * 100.0) as u32).hash(&mut hasher);
        let key = hasher.finish();

        {
            let cache = self.cache.lock().expect("Failed to lock Highlighter cache");
            if let Some(job) = cache.get(&key) {
                return Arc::clone(job);
            }
        }

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

        let job_arc = Arc::new(job);
        {
            let mut cache = self
                .cache
                .lock()
                .expect("Failed to lock Highlighter cache for storage");
            if cache.len() >= 20 {
                cache.clear();
            }
            cache.insert(key, Arc::clone(&job_arc));
        }

        job_arc
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_highlight_performance_10k() {
        let h = Highlighter::new();
        let mut text = String::from("fn main() {\n");
        for i in 0..10000 {
            text.push_str(&format!("    let x_{} = {};\n", i, i));
        }
        text.push_str("}\n");

        println!("Starting benchmark for 10k lines...");

        let start = Instant::now();
        let job1 = h.highlight(&text, "rs", "performance_test.rs", 14.0);
        let duration1 = start.elapsed();
        println!("First run (no cache): {:?}", duration1);

        let start = Instant::now();
        let job2 = h.highlight(&text, "rs", "performance_test.rs", 14.0);
        let duration2 = start.elapsed();
        println!("Second run (with cache): {:?}", duration2);

        assert_eq!(job1.sections.len(), job2.sections.len());
        assert!(duration2 < duration1);
        println!("Performance gain: {:.2}x", duration1.as_secs_f64() / duration2.as_secs_f64());
    }
}
