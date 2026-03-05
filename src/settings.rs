use crate::app::ai::{AiExpertiseRole, AiReasoningDepth};
use std::collections::HashMap;
use std::path::PathBuf;

const SETTINGS_FILE: &str = "settings.toml";
const OLD_SETTINGS_FILE: &str = "settings.json";
const CONFIG_DIR_NAME: &str = "polycredo-editor";

// ---------------------------------------------------------------------------
// PluginSettings — configuration for individual WASM plugins
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
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

#[derive(serde::Serialize, serde::Deserialize, Clone, Default, PartialEq)]
pub struct CustomAgent {
    pub name: String,
    pub command: String,
    pub args: String,
}

// ---------------------------------------------------------------------------
// LightVariant — color variant for light mode (Phase 3 will use per-variant colors)
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum LightVariant {
    #[default]
    WarmIvory,
    CoolGray,
    Sepia,
}

// ---------------------------------------------------------------------------
// Settings — persistent application configuration
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct Settings {
    /// Editor font size in px (10–24).
    #[serde(default = "default_editor_font_size")]
    pub editor_font_size: f32,

    /// true = dark theme, false = light theme.
    #[serde(default = "default_dark_theme")]
    pub dark_theme: bool,

    /// Light mode color variant (only relevant when dark_theme = false).
    /// Phase 3 will use this for per-variant panel colors.
    #[serde(default)]
    pub light_variant: LightVariant,

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

    /// Whether the project runs in sandbox mode.
    /// Legacy key `project_read_only` is accepted during migration.
    #[serde(default = "default_true", alias = "project_read_only")]
    pub sandbox_mode: bool,

    /// Configuration for individual plugins. Key = plugin ID (file stem).
    #[serde(default)]
    pub plugins: HashMap<String, PluginSettings>,

    /// Global blacklist for plugins (glob patterns, e.g. ["*.env", "secret/*"]).
    #[serde(default)]
    pub blacklist: Vec<String>,

    /// User-defined CLI AI agents.
    #[serde(default)]
    pub custom_agents: Vec<CustomAgent>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            editor_font_size: default_editor_font_size(),
            dark_theme: default_dark_theme(),
            light_variant: LightVariant::default(),
            default_project_path: default_project_path(),
            lang: default_lang(),
            diff_side_by_side: false,
            privacy_accepted: false,
            auto_show_ai_diff: true,
            sandbox_mode: true,
            plugins: HashMap::new(),
            blacklist: vec![
                ".env*".to_string(),
                "*.key".to_string(),
                "id_rsa*".to_string(),
                "Cargo.lock".to_string(),
            ],
            custom_agents: vec![
                CustomAgent {
                    name: "Gemini CLI".to_string(),
                    command: "gemini".to_string(),
                    args: "".to_string(),
                },
                CustomAgent {
                    name: "Claude Code".to_string(),
                    command: "claude".to_string(),
                    args: "".to_string(),
                },
                CustomAgent {
                    name: "Aider".to_string(),
                    command: "aider".to_string(),
                    args: "".to_string(),
                },
            ],
        }
    }
}

fn settings_path() -> PathBuf {
    settings_path_in(&config_dir())
}

fn old_settings_path() -> PathBuf {
    old_settings_path_in(&config_dir())
}

fn settings_path_in(config_root: &std::path::Path) -> PathBuf {
    config_root.join(CONFIG_DIR_NAME).join(SETTINGS_FILE)
}

fn old_settings_path_in(config_root: &std::path::Path) -> PathBuf {
    config_root.join(CONFIG_DIR_NAME).join(OLD_SETTINGS_FILE)
}

fn config_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| PathBuf::from("."))
}

impl Settings {
    fn load_from_config_dir(config_root: &std::path::Path) -> Self {
        let path = settings_path_in(config_root);
        if let Ok(content) = std::fs::read_to_string(&path) {
            return toml::from_str(&content).unwrap_or_default();
        }

        // Migration from JSON
        let old_path = old_settings_path_in(config_root);
        if let Ok(content) = std::fs::read_to_string(&old_path) {
            let settings: Self = serde_json::from_str(&content).unwrap_or_default();
            let _ = settings.try_save_to_config_dir(config_root); // Save as TOML immediately
            let _ = std::fs::remove_file(old_path); // Cleanup
            return settings;
        }

        Self::default()
    }

    fn try_save_to_config_dir(&self, config_root: &std::path::Path) -> Result<(), String> {
        let path = settings_path_in(config_root);
        if let Some(parent) = path.parent()
            && let Err(e) = std::fs::create_dir_all(parent)
        {
            return Err(format!(
                "settings: cannot create directory {}: {e}",
                parent.display()
            ));
        }
        let toml_str = toml::to_string_pretty(self)
            .map_err(|_| "settings: TOML serialization failed".to_string())?;
        std::fs::write(&path, toml_str)
            .map_err(|e| format!("settings: cannot write {}: {e}", path.display()))?;
        Ok(())
    }

    /// Returns the syntect theme name for the current mode.
    /// Dark → "base16-ocean.dark", Light → "Solarized (light)".
    pub fn syntect_theme_name(&self) -> &'static str {
        if self.dark_theme {
            "base16-ocean.dark"
        } else {
            "Solarized (light)"
        }
    }

    /// Returns egui Visuals for the current theme.
    /// Phase 1: basic Visuals::dark()/light(). Phase 3 will add per-variant colors.
    pub fn to_egui_visuals(&self) -> eframe::egui::Visuals {
        if self.dark_theme {
            eframe::egui::Visuals::dark()
        } else {
            let mut visuals = eframe::egui::Visuals::light();

            match self.light_variant {
                LightVariant::WarmIvory => {
                    visuals.panel_fill = eframe::egui::Color32::from_rgb(255, 252, 240);
                    visuals.window_fill = eframe::egui::Color32::from_rgb(250, 246, 235);
                    visuals.faint_bg_color = eframe::egui::Color32::from_rgb(247, 241, 226);
                }
                LightVariant::CoolGray => {
                    visuals.panel_fill = eframe::egui::Color32::from_rgb(242, 242, 242);
                    visuals.window_fill = eframe::egui::Color32::from_rgb(236, 236, 236);
                    visuals.faint_bg_color = eframe::egui::Color32::from_rgb(227, 231, 236);
                }
                LightVariant::Sepia => {
                    visuals.panel_fill = eframe::egui::Color32::from_rgb(240, 230, 210);
                    visuals.window_fill = eframe::egui::Color32::from_rgb(234, 223, 202);
                    visuals.faint_bg_color = eframe::egui::Color32::from_rgb(223, 210, 186);
                }
            }

            visuals
        }
    }

    /// Loads settings from disk. Tries settings.toml first, then migrates from settings.json.
    pub fn load() -> Self {
        Self::load_from_config_dir(&config_dir())
    }

    /// Saves settings to disk (~/.config/polycredo-editor/settings.toml).
    pub fn save(&self) {
        if let Err(err) = self.try_save() {
            eprintln!("{err}");
        }
    }

    pub fn try_save(&self) -> Result<(), String> {
        self.try_save_to_config_dir(&config_dir())
    }

    /// Applies settings to the egui Context (theme + editor font size).
    pub fn apply(&self, ctx: &eframe::egui::Context) {
        // Phase 1: deleguje na to_egui_visuals(). Phase 3: přidá per-variant colors.
        ctx.set_visuals(self.to_egui_visuals());
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

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Color32;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempConfigDir {
        path: PathBuf,
    }

    impl TempConfigDir {
        fn new(name: &str) -> Self {
            let mut path = std::env::temp_dir();
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time")
                .as_nanos();
            path.push(format!(
                "polycredo-settings-tests-{name}-{}-{nanos}",
                std::process::id()
            ));
            std::fs::create_dir_all(&path).expect("create temp config dir");
            Self { path }
        }

        fn app_config_dir(&self) -> PathBuf {
            self.path.join(CONFIG_DIR_NAME)
        }
    }

    impl Drop for TempConfigDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    fn rgb(color: Color32) -> (u8, u8, u8) {
        (color.r(), color.g(), color.b())
    }

    // THEME-01: Serde default pro light_variant
    #[test]
    fn test_theme01_light_variant_serde_default() {
        // TOML bez light_variant klíče - nesmí spadnout
        let toml_str = r#"
editor_font_size = 14.0
dark_theme = true
"#;
        let result: Settings = toml::from_str(toml_str).expect("Should parse without panic");
        assert_eq!(result.light_variant, LightVariant::WarmIvory);
    }

    #[test]
    fn test_theme01_light_variant_roundtrip() {
        // Round-trip serializace a deserializace LightVariant::CoolGray
        let settings = Settings {
            light_variant: LightVariant::CoolGray,
            ..Default::default()
        };
        let toml_str = toml::to_string(&settings).expect("Should serialize");
        let parsed: Settings = toml::from_str(&toml_str).expect("Should deserialize");
        assert_eq!(parsed.light_variant, LightVariant::CoolGray);
    }

    // THEME-02: syntect_theme_name()
    #[test]
    fn test_theme02_syntect_theme_name_dark() {
        let settings = Settings {
            dark_theme: true,
            ..Default::default()
        };
        assert_eq!(settings.syntect_theme_name(), "base16-ocean.dark");
    }

    #[test]
    fn test_theme02_syntect_theme_name_light() {
        let settings = Settings {
            dark_theme: false,
            ..Default::default()
        };
        assert_eq!(settings.syntect_theme_name(), "Solarized (light)");
    }

    // THEME-02: to_egui_visuals()
    #[test]
    fn test_theme02_to_egui_visuals_dark() {
        let settings = Settings {
            dark_theme: true,
            ..Default::default()
        };
        let visuals = settings.to_egui_visuals();
        assert!(visuals.dark_mode);
    }

    #[test]
    fn test_theme02_to_egui_visuals_light() {
        let settings = Settings {
            dark_theme: false,
            ..Default::default()
        };
        let visuals = settings.to_egui_visuals();
        assert!(!visuals.dark_mode);
    }

    #[test]
    fn test_lite_light_variants_have_distinct_panel_fill() {
        let warm = Settings {
            dark_theme: false,
            light_variant: LightVariant::WarmIvory,
            ..Default::default()
        }
        .to_egui_visuals()
        .panel_fill;
        let cool = Settings {
            dark_theme: false,
            light_variant: LightVariant::CoolGray,
            ..Default::default()
        }
        .to_egui_visuals()
        .panel_fill;
        let sepia = Settings {
            dark_theme: false,
            light_variant: LightVariant::Sepia,
            ..Default::default()
        }
        .to_egui_visuals()
        .panel_fill;

        assert_ne!(warm, cool);
        assert_ne!(warm, sepia);
        assert_ne!(cool, sepia);
    }

    #[test]
    fn test_lite01_warm_ivory_panel_fill_rgb() {
        let visuals = Settings {
            dark_theme: false,
            light_variant: LightVariant::WarmIvory,
            ..Default::default()
        }
        .to_egui_visuals();
        assert_eq!(rgb(visuals.panel_fill), (255, 252, 240));
    }

    #[test]
    fn test_lite02_cool_gray_panel_fill_rgb() {
        let visuals = Settings {
            dark_theme: false,
            light_variant: LightVariant::CoolGray,
            ..Default::default()
        }
        .to_egui_visuals();
        assert_eq!(rgb(visuals.panel_fill), (242, 242, 242));
    }

    #[test]
    fn test_lite03_sepia_panel_fill_rgb() {
        let visuals = Settings {
            dark_theme: false,
            light_variant: LightVariant::Sepia,
            ..Default::default()
        }
        .to_egui_visuals();
        assert_eq!(rgb(visuals.panel_fill), (240, 230, 210));
    }

    #[test]
    fn test_lite04_faint_bg_differs_from_panel_and_between_variants() {
        let warm = Settings {
            dark_theme: false,
            light_variant: LightVariant::WarmIvory,
            ..Default::default()
        }
        .to_egui_visuals();
        let cool = Settings {
            dark_theme: false,
            light_variant: LightVariant::CoolGray,
            ..Default::default()
        }
        .to_egui_visuals();
        let sepia = Settings {
            dark_theme: false,
            light_variant: LightVariant::Sepia,
            ..Default::default()
        }
        .to_egui_visuals();

        assert_ne!(warm.faint_bg_color, warm.panel_fill);
        assert_ne!(cool.faint_bg_color, cool.panel_fill);
        assert_ne!(sepia.faint_bg_color, sepia.panel_fill);

        assert_ne!(warm.faint_bg_color, cool.faint_bg_color);
        assert_ne!(warm.faint_bg_color, sepia.faint_bg_color);
        assert_ne!(cool.faint_bg_color, sepia.faint_bg_color);
    }

    #[test]
    fn test_lite_dark_mode_visuals_regression() {
        let visuals = Settings {
            dark_theme: true,
            ..Default::default()
        }
        .to_egui_visuals();
        assert!(visuals.dark_mode);
    }

    // SETT-04: Zpětná kompatibilita
    #[test]
    fn test_sett04_backward_compat() {
        // TOML bez light_variant klíče se deserializuje na WarmIvory default bez paniku
        let toml_str = r#"
editor_font_size = 16.0
default_project_path = "/home/test"
"#;
        let result: Settings = toml::from_str(toml_str).expect("Should parse without panic");
        assert_eq!(result.light_variant, LightVariant::WarmIvory);
    }

    #[test]
    fn test_sett03_canonical_toml_roundtrip_preserves_theme_fingerprint() {
        let temp = TempConfigDir::new("canonical-roundtrip");
        let settings = Settings {
            dark_theme: false,
            light_variant: LightVariant::Sepia,
            ..Default::default()
        };

        settings
            .try_save_to_config_dir(&temp.path)
            .expect("save settings");
        let saved_path = temp.app_config_dir().join(SETTINGS_FILE);
        assert!(saved_path.is_file());

        let loaded = Settings::load_from_config_dir(&temp.path);
        assert!(!loaded.dark_theme);
        assert_eq!(loaded.light_variant, LightVariant::Sepia);
    }

    #[test]
    fn test_sett03_legacy_json_migrates_to_canonical_toml() {
        let temp = TempConfigDir::new("legacy-migration");
        let app_config = temp.app_config_dir();
        std::fs::create_dir_all(&app_config).expect("create app config dir");

        let legacy_path = app_config.join(OLD_SETTINGS_FILE);
        let legacy_json = r#"
{
  "dark_theme": false,
  "light_variant": "cool_gray",
  "editor_font_size": 16.0
}
"#;
        std::fs::write(&legacy_path, legacy_json).expect("write legacy settings.json");

        let loaded = Settings::load_from_config_dir(&temp.path);
        assert!(!loaded.dark_theme);
        assert_eq!(loaded.light_variant, LightVariant::CoolGray);

        let canonical_path = app_config.join(SETTINGS_FILE);
        assert!(canonical_path.is_file());
        assert!(!legacy_path.exists());

        let canonical_content =
            std::fs::read_to_string(&canonical_path).expect("read canonical settings.toml");
        let canonical: Settings = toml::from_str(&canonical_content).expect("parse canonical TOML");
        assert!(!canonical.dark_theme);
        assert_eq!(canonical.light_variant, LightVariant::CoolGray);
    }

    #[test]
    fn test_sett02_canonical_toml_persists_sandbox_mode() {
        let temp = TempConfigDir::new("sandbox-mode-roundtrip");
        let settings = Settings {
            sandbox_mode: false,
            ..Default::default()
        };

        settings
            .try_save_to_config_dir(&temp.path)
            .expect("save settings");

        let canonical_path = temp.app_config_dir().join(SETTINGS_FILE);
        let canonical_content =
            std::fs::read_to_string(&canonical_path).expect("read canonical settings.toml");
        assert!(canonical_content.contains("sandbox_mode = false"));
        assert!(!canonical_content.contains("project_read_only"));

        let loaded = Settings::load_from_config_dir(&temp.path);
        assert!(!loaded.sandbox_mode);
    }

    #[test]
    fn test_sett05_legacy_project_read_only_maps_to_sandbox_mode() {
        let temp = TempConfigDir::new("legacy-safe-mode-migration");
        let app_config = temp.app_config_dir();
        std::fs::create_dir_all(&app_config).expect("create app config dir");

        let legacy_path = app_config.join(OLD_SETTINGS_FILE);
        let legacy_json = r#"
{
  "project_read_only": false,
  "editor_font_size": 16.0
}
"#;
        std::fs::write(&legacy_path, legacy_json).expect("write legacy settings.json");

        let loaded = Settings::load_from_config_dir(&temp.path);
        assert!(!loaded.sandbox_mode);

        let canonical_path = app_config.join(SETTINGS_FILE);
        let canonical_content =
            std::fs::read_to_string(&canonical_path).expect("read canonical settings.toml");
        assert!(canonical_content.contains("sandbox_mode = false"));
        assert!(!canonical_content.contains("project_read_only"));
    }
}
