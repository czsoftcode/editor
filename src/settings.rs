use std::path::PathBuf;

const SETTINGS_FILE: &str = "settings.json";

// ---------------------------------------------------------------------------
// Výchozí hodnoty (potřebné pro serde default attrs)
// ---------------------------------------------------------------------------

fn default_editor_font_size() -> f32 {
    14.0
}
fn default_dark_theme() -> bool {
    true
}

pub fn default_project_path() -> String {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/"))
        .join("MyProject")
        .to_string_lossy()
        .to_string()
}

// ---------------------------------------------------------------------------
// Settings — perzistentní konfigurace aplikace
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Settings {
    /// Velikost fontu editoru v px (10–24).
    #[serde(default = "default_editor_font_size")]
    pub editor_font_size: f32,

    /// true = tmavé téma, false = světlé.
    #[serde(default = "default_dark_theme")]
    pub dark_theme: bool,

    /// Výchozí adresář pro nové projekty.
    #[serde(default = "default_project_path")]
    pub default_project_path: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            editor_font_size: default_editor_font_size(),
            dark_theme: default_dark_theme(),
            default_project_path: default_project_path(),
        }
    }
}

fn settings_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rust_editor")
        .join(SETTINGS_FILE)
}

impl Settings {
    /// Načte nastavení z disku. Chybějící soubor → Default.
    pub fn load() -> Self {
        let path = settings_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Uloží nastavení na disk (~/.config/rust_editor/settings.json).
    pub fn save(&self) {
        let path = settings_path();
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("settings: nelze vytvořit adresář {}: {e}", parent.display());
                return;
            }
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            if let Err(e) = std::fs::write(&path, json) {
                eprintln!("settings: nelze zapsat {}: {e}", path.display());
            }
        } else {
            eprintln!("settings: serializace JSON selhala");
        }
    }

    /// Aplikuje nastavení na egui Context (téma + velikost fontu editoru).
    pub fn apply(&self, ctx: &eframe::egui::Context) {
        if self.dark_theme {
            ctx.set_visuals(eframe::egui::Visuals::dark());
        } else {
            ctx.set_visuals(eframe::egui::Visuals::light());
        }
        ctx.style_mut(|style| {
            style.text_styles.insert(
                eframe::egui::TextStyle::Monospace,
                eframe::egui::FontId::new(
                    self.editor_font_size,
                    eframe::egui::FontFamily::Monospace,
                ),
            );
        });
    }
}
