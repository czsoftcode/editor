use std::path::PathBuf;

const SETTINGS_FILE: &str = "settings.json";
const CONFIG_DIR_NAME: &str = "polycredo-editor";

// ---------------------------------------------------------------------------
// Default values (needed for serde default attrs)
// ---------------------------------------------------------------------------

fn default_editor_font_size() -> f32 {
    14.0
}
fn default_dark_theme() -> bool {
    true
}
fn default_lang() -> String {
    crate::i18n::detect_system_lang()
}

pub fn default_project_path() -> String {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/"))
        .join("MyProject")
        .to_string_lossy()
        .to_string()
}

// ---------------------------------------------------------------------------
// Settings — persistent application configuration
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Settings {
    /// Editor font size in px (10–24).
    #[serde(default = "default_editor_font_size")]
    pub editor_font_size: f32,

    /// true = dark theme, false = light theme.
    #[serde(default = "default_dark_theme")]
    pub dark_theme: bool,

    /// Default directory for new projects.
    #[serde(default = "default_project_path")]
    pub default_project_path: String,

    /// UI language code (BCP 47, e.g., "cs", "en").
    /// Empty string or unsupported language → autodetect from system.
    #[serde(default = "default_lang")]
    pub lang: String,

    /// Whether the user has accepted the privacy policy.
    #[serde(default)]
    pub privacy_accepted: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            editor_font_size: default_editor_font_size(),
            dark_theme: default_dark_theme(),
            default_project_path: default_project_path(),
            lang: default_lang(),
            privacy_accepted: false,
        }
    }
}

fn settings_path() -> PathBuf {
    config_dir()
        .join(CONFIG_DIR_NAME)
        .join(SETTINGS_FILE)
}

fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
}

impl Settings {
    /// Loads settings from disk. Missing file → Default.
    pub fn load() -> Self {
        let path = settings_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Saves settings to disk (~/.config/polycredo-editor/settings.json).
    pub fn save(&self) {
        let path = settings_path();
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("settings: cannot create directory {}: {e}", parent.display());
                return;
            }
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            if let Err(e) = std::fs::write(&path, json) {
                eprintln!("settings: cannot write {}: {e}", path.display());
            }
        } else {
            eprintln!("settings: JSON serialization failed");
        }
    }

    /// Applies settings to the egui Context (theme + editor font size).
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
