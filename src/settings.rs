use crate::app::ui::widgets::ai_cli::{AiExpertiseRole, AiReasoningDepth};
use std::collections::HashMap;
use std::path::PathBuf;

const SETTINGS_FILE: &str = "settings.toml";
const OLD_SETTINGS_FILE: &str = "settings.json";
const CONFIG_DIR_NAME: &str = "polycredo-editor";

// ---------------------------------------------------------------------------
// PluginSettings — configuration for individual WASM plugins
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct PluginSettings {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub expertise: AiExpertiseRole,
    #[serde(default)]
    pub reasoning_depth: AiReasoningDepth,
    #[serde(default)]
    pub config: HashMap<String, String>,
}

impl Default for PluginSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            expertise: AiExpertiseRole::default(),
            reasoning_depth: AiReasoningDepth::default(),
            config: HashMap::new(),
        }
    }
}

fn default_true() -> bool {
    true
}

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
fn default_auto_show_ai_diff() -> bool {
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

    /// Whether to show the AI diff viewer in side-by-side mode.
    /// false = inline mode, true = side-by-side mode.
    #[serde(default)]
    pub diff_side_by_side: bool,

    /// Whether the user has accepted the privacy policy.
    #[serde(default)]
    pub privacy_accepted: bool,

    /// Whether to automatically show the AI diff modal when changes are detected.
    #[serde(default = "default_auto_show_ai_diff")]
    pub auto_show_ai_diff: bool,

    /// Whether the main project is read-only (Safe Mode).
    #[serde(default = "default_true")]
    pub project_read_only: bool,

    /// Configuration for individual plugins. Key = plugin ID (file stem).
    #[serde(default)]
    pub plugins: HashMap<String, PluginSettings>,

    /// Global blacklist for plugins (glob patterns, e.g. ["*.env", "secret/*"]).
    #[serde(default)]
    pub blacklist: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            editor_font_size: default_editor_font_size(),
            dark_theme: default_dark_theme(),
            default_project_path: default_project_path(),
            lang: default_lang(),
            diff_side_by_side: false,
            privacy_accepted: false,
            auto_show_ai_diff: true,
            project_read_only: true,
            plugins: HashMap::new(),
            blacklist: vec![
                ".env*".to_string(),
                "*.key".to_string(),
                "id_rsa*".to_string(),
                "Cargo.lock".to_string(),
            ],
        }
    }
}

fn settings_path() -> PathBuf {
    config_dir().join(CONFIG_DIR_NAME).join(SETTINGS_FILE)
}

fn old_settings_path() -> PathBuf {
    config_dir().join(CONFIG_DIR_NAME).join(OLD_SETTINGS_FILE)
}

fn config_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| PathBuf::from("."))
}

impl Settings {
    /// Loads settings from disk. Tries settings.toml first, then migrates from settings.json.
    pub fn load() -> Self {
        let path = settings_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            return toml::from_str(&content).unwrap_or_default();
        }

        // Migration from JSON
        let old_path = old_settings_path();
        if let Ok(content) = std::fs::read_to_string(&old_path) {
            let settings: Self = serde_json::from_str(&content).unwrap_or_default();
            settings.save(); // Save as TOML immediately
            let _ = std::fs::remove_file(old_path); // Cleanup
            return settings;
        }

        Self::default()
    }

    /// Saves settings to disk (~/.config/polycredo-editor/settings.toml).
    pub fn save(&self) {
        let path = settings_path();
        if let Some(parent) = path.parent()
            && let Err(e) = std::fs::create_dir_all(parent)
        {
            eprintln!(
                "settings: cannot create directory {}: {e}",
                parent.display()
            );
            return;
        }
        if let Ok(toml_str) = toml::to_string_pretty(self) {
            if let Err(e) = std::fs::write(&path, toml_str) {
                eprintln!("settings: cannot write {}: {e}", path.display());
            }
        } else {
            eprintln!("settings: TOML serialization failed");
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
