use eframe::egui;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Cesta ke custom sublime-syntax definicím (vedle binárky nebo ve zdrojovém stromu).
const CUSTOM_SYNTAXES_DIR: &str = "assets/syntaxes";

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    /// Current theme name (for cache invalidation).
    current_theme: std::sync::Mutex<String>,
    /// Fast full-file cache (for scrolling/rendering).
    /// Using Arc to avoid cloning massive LayoutJobs for large files.
    cache: std::sync::Mutex<HashMap<u64, Arc<egui::text::LayoutJob>>>,
}

/// Načte výchozí syntaxe + custom `.sublime-syntax` soubory z `assets/syntaxes/`.
/// Hledá adresář vedle spustitelného souboru, v CWD, nebo ve zdrojovém stromu.
fn build_syntax_set() -> SyntaxSet {
    let defaults = SyntaxSet::load_defaults_newlines();
    let mut builder = defaults.into_builder();

    // Hledáme adresář s custom syntaxemi na několika místech
    let candidates: Vec<std::path::PathBuf> = vec![
        // Vedle binárky (typicky pro release build)
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join(CUSTOM_SYNTAXES_DIR)))
            .unwrap_or_default(),
        // CWD (typicky pro cargo run z kořene projektu)
        std::path::PathBuf::from(CUSTOM_SYNTAXES_DIR),
        // CARGO_MANIFEST_DIR (pro testy)
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(CUSTOM_SYNTAXES_DIR),
    ];

    for dir in &candidates {
        if dir.is_dir() {
            if let Err(e) = builder.add_from_folder(dir, true) {
                eprintln!(
                    "[highlighter] Chyba při načítání syntaxí z {}: {}",
                    dir.display(),
                    e
                );
            } else {
                eprintln!("[highlighter] Načteny custom syntaxe z {}", dir.display());
            }
            break;
        }
    }

    builder.build()
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: build_syntax_set(),
            theme_set: ThemeSet::load_defaults(),
            current_theme: std::sync::Mutex::new("base16-ocean.dark".to_string()),
            cache: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Sets the theme and invalidates cache if theme changed.
    pub fn set_theme(&self, theme_name: &str) {
        let mut current = self
            .current_theme
            .lock()
            .expect("Highlighter current_theme lock");
        if current.as_str() != theme_name {
            *current = theme_name.to_string();
            self.cache.lock().expect("Highlighter cache lock").clear();
        }
    }

    pub fn highlight(
        &self,
        text: &str,
        extension: &str,
        filename: &str,
        font_size: f32,
        theme_name: &str,
    ) -> Arc<egui::text::LayoutJob> {
        // Compute cache key (includes theme_name for cache invalidation)
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        text.hash(&mut hasher);
        extension.hash(&mut hasher);
        filename.hash(&mut hasher);
        ((font_size * 100.0) as u32).hash(&mut hasher);
        theme_name.hash(&mut hasher);
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

        let theme = self
            .theme_set
            .themes
            .get(theme_name)
            .unwrap_or_else(|| &self.theme_set.themes["base16-ocean.dark"]);
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

    pub fn background_color(&self, theme_name: &str) -> egui::Color32 {
        let theme = self
            .theme_set
            .themes
            .get(theme_name)
            .unwrap_or_else(|| &self.theme_set.themes["base16-ocean.dark"]);
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
    fn test_toml_syntax_available() {
        let ss = build_syntax_set();
        let syntax = ss.find_syntax_by_extension("toml");
        assert!(
            syntax.is_some(),
            "TOML syntaxe musí být dostupná po načtení custom syntaxí"
        );
        assert_eq!(syntax.unwrap().name, "TOML");
    }

    #[test]
    fn test_toml_highlighting_produces_sections() {
        let h = Highlighter::new();
        let toml_text = "[package]\nname = \"test\"\nversion = \"1.0.0\"\n# komentář\nenabled = true\ncount = 42\n";
        let job = h.highlight(toml_text, "toml", "Cargo.toml", 14.0, "base16-ocean.dark");
        // TOML s highlighting musí vytvořit víc sekcí než plain text (kde je 1 sekce na řádek)
        assert!(
            job.sections.len() > 6,
            "TOML highlighting by měl vytvořit více sekcí, dostal {}",
            job.sections.len()
        );
    }

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
        let job1 = h.highlight(
            &text,
            "rs",
            "performance_test.rs",
            14.0,
            "base16-ocean.dark",
        );
        let duration1 = start.elapsed();
        println!("First run (no cache): {:?}", duration1);

        let start = Instant::now();
        let job2 = h.highlight(
            &text,
            "rs",
            "performance_test.rs",
            14.0,
            "base16-ocean.dark",
        );
        let duration2 = start.elapsed();
        println!("Second run (with cache): {:?}", duration2);

        assert_eq!(job1.sections.len(), job2.sections.len());
        assert!(duration2 < duration1);
        println!(
            "Performance gain: {:.2}x",
            duration1.as_secs_f64() / duration2.as_secs_f64()
        );
    }
}
