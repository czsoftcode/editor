use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Build / Runner Profiles
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum ErrorParserType {
    #[default]
    None,
    Rust,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(crate) struct BuildProfile {
    /// Display name of the profile (e.g., "Run Server", "Cargo Test")
    pub name: String,
    /// Main command to execute
    pub command: String,
    /// List of arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// Working directory (relative to project root)
    #[serde(default)]
    pub working_dir: Option<String>,
    /// Environment variables
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    /// Should save all files before running?
    #[serde(default = "default_true")]
    pub auto_save: bool,
    /// Type of error parsing for output analysis
    #[serde(default)]
    pub error_parser: ErrorParserType,
}

fn default_true() -> bool {
    true
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub(crate) struct ProjectProfiles {
    /// List of configured runners
    #[serde(default)]
    pub runners: Vec<BuildProfile>,
}

// ---------------------------------------------------------------------------
// Helper types
// ---------------------------------------------------------------------------

use crate::app::cli::{AiExpertiseRole, AiReasoningDepth};

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum FocusedPanel {
    Build,
    Claude,
    Editor,
    AiChat,
    Files,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub(crate) enum ProjectType {
    Rust,
    Symfony74,
    Symfony80,
    Laravel12,
    Nette32,
    Nette30,
    FastApi,
    NextJs,
    ExpressJs,
}

impl ProjectType {
    pub(crate) fn subdir(&self) -> &'static str {
        match self {
            ProjectType::Rust => "Rust",
            ProjectType::Symfony74 | ProjectType::Symfony80 => "Symfony",
            ProjectType::Laravel12 => "Laravel",
            ProjectType::Nette32 | ProjectType::Nette30 => "Nette",
            ProjectType::FastApi => "Python",
            ProjectType::NextJs | ProjectType::ExpressJs => "NodeJS",
        }
    }
}

pub(crate) const STORAGE_KEY: &str = "panel_state";

pub(crate) fn default_font_scale() -> u32 {
    100
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct PersistentState {
    pub show_left_panel: bool,
    pub show_right_panel: bool,
    pub show_build_terminal: bool,
    pub claude_float: bool,
    #[serde(default = "default_font_scale")]
    pub ai_font_scale: u32,
    pub ai_selected_provider: Option<String>,
    pub ai_system_prompt: Option<String>,
    pub ai_language: Option<String>,
    pub ai_expertise: Option<AiExpertiseRole>,
    pub ai_reasoning_depth: Option<AiReasoningDepth>,
    #[serde(default)]
    pub ollama_selected_model: Option<String>,
}

impl Default for PersistentState {
    fn default() -> Self {
        Self {
            show_left_panel: true,
            show_right_panel: false,
            show_build_terminal: false,
            claude_float: false,
            ai_font_scale: 100,
            ai_selected_provider: Some("gemini".to_string()),
            ai_system_prompt: None,
            ai_language: None,
            ai_expertise: None,
            ai_reasoning_depth: None,
            ollama_selected_model: None,
        }
    }
}

// ---------------------------------------------------------------------------
// AppShared — Shared state between viewports (protected by Mutex)
// ---------------------------------------------------------------------------

pub(crate) enum AppAction {
    /// Open project in a new window
    OpenInNewWindow(PathBuf),
    /// Close secondary viewport (after clicking X window button)
    CloseWorkspace(eframe::egui::ViewportId),
    /// Add path to recent projects
    AddRecent(PathBuf),
    /// Terminate the whole application
    QuitAll,
}

pub(crate) struct AppShared {
    pub recent_projects: Vec<PathBuf>,
    pub actions: Vec<AppAction>,
    pub settings: std::sync::Arc<crate::settings::Settings>,
    /// Active UI translations. `Arc` allows sharing without repeated mutex locking.
    pub i18n: std::sync::Arc<crate::i18n::I18n>,
    /// Flag to distinguish between editor's own saves and external (AI) modifications.
    pub is_internal_save: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// Extension registry for commands and panels.
    pub registry: crate::app::registry::Registry,
    /// Version of settings, incremented on change to ensure all viewports re-apply them (Audit S-4).
    pub settings_version: std::sync::atomic::AtomicU64,
    /// Shared BERT model for semantic search (all-MiniLM-L6-v2)
    pub bert_model: Option<std::sync::Arc<candle_transformers::models::bert::BertModel>>,
    /// Shared BERT tokenizer
    pub bert_tokenizer: Option<std::sync::Arc<tokenizers::Tokenizer>>,
}

// ---------------------------------------------------------------------------
// Toast — Short-term UI notification
// ---------------------------------------------------------------------------

pub(crate) struct Toast {
    pub message: String,
    pub created: std::time::Instant,
    pub is_error: bool,
}

impl Toast {
    pub(crate) fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            created: std::time::Instant::now(),
            is_error: true,
        }
    }

    pub(crate) fn info(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            created: std::time::Instant::now(),
            is_error: false,
        }
    }

    pub(crate) fn is_expired(&self) -> bool {
        self.created.elapsed().as_secs() >= 4
    }
}

pub(crate) const SAVE_ERROR_DEDUPE_WINDOW: Duration = Duration::from_millis(1500);
static SAVE_ERROR_LAST_SEEN: OnceLock<Mutex<std::collections::HashMap<String, Instant>>> =
    OnceLock::new();

pub(crate) fn save_error_dedupe_decision(
    last_seen: Option<Instant>,
    now: Instant,
    window: Duration,
) -> bool {
    match last_seen {
        None => true,
        Some(prev) => now.duration_since(prev) > window,
    }
}

pub(crate) fn should_emit_save_error_toast(error_key: &str) -> bool {
    let now = Instant::now();
    let mut seen = SAVE_ERROR_LAST_SEEN
        .get_or_init(|| Mutex::new(std::collections::HashMap::new()))
        .lock()
        .expect("Failed to lock save error dedupe map");
    let should_emit =
        save_error_dedupe_decision(seen.get(error_key).copied(), now, SAVE_ERROR_DEDUPE_WINDOW);
    if should_emit {
        seen.insert(error_key.to_string(), now);
    }

    let retention = SAVE_ERROR_DEDUPE_WINDOW + SAVE_ERROR_DEDUPE_WINDOW;
    seen.retain(|_, ts| now.duration_since(*ts) <= retention);
    should_emit
}

// ---------------------------------------------------------------------------
// Helper free functions
// ---------------------------------------------------------------------------

pub(crate) fn path_env() -> String {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    let current = std::env::var("PATH").unwrap_or_default();
    let cargo_bin = home.join(".cargo/bin");
    let local_bin = home.join(".local/bin");
    format!(
        "{}:{}:{}",
        cargo_bin.display(),
        local_bin.display(),
        current
    )
}

pub(crate) fn default_wizard_path() -> String {
    crate::settings::default_project_path()
}

#[cfg(test)]
mod tests {
    use super::SAVE_ERROR_DEDUPE_WINDOW;
    use super::save_error_dedupe_decision;

    #[test]
    fn save_error_dedupe_suppresses_repeated_error_within_window() {
        let now = std::time::Instant::now();
        let old = now - (SAVE_ERROR_DEDUPE_WINDOW + std::time::Duration::from_millis(1));
        let recent = now - std::time::Duration::from_millis(5);

        assert!(save_error_dedupe_decision(None, now, SAVE_ERROR_DEDUPE_WINDOW));
        assert!(save_error_dedupe_decision(
            Some(old),
            now,
            SAVE_ERROR_DEDUPE_WINDOW
        ));
        assert!(!save_error_dedupe_decision(
            Some(recent),
            now,
            SAVE_ERROR_DEDUPE_WINDOW
        ));
    }
}
