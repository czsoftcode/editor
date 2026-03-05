use std::path::PathBuf;

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

use crate::app::ai::{AiExpertiseRole, AiReasoningDepth};

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
    /// Result from a background plugin call
    PluginResponse(String, Result<String, String>),
    /// Incremental "thought" or log from a plugin
    PluginMonologue(String, String),
    /// Token usage info from a plugin (id, in_tokens, out_tokens)
    PluginUsage(String, u32, u32),
    /// RAW JSON payload from a plugin for inspection
    PluginPayload(String, String),
    /// Request for user approval for a dangerous AI action (plugin_id, action_name, action_details, sender)
    PluginApprovalRequest(
        String,
        String,
        String,
        std::sync::mpsc::Sender<PluginApprovalResponse>,
    ),
    /// Agent asks the user a clarifying question and blocks for the answer.
    /// (plugin_id, question, options, response_sender)
    PluginAskUser(String, String, Vec<String>, std::sync::mpsc::Sender<String>),
    /// Agent signals successful task completion with a summary.
    /// (plugin_id, summary)
    PluginCompleted(String, String),
}

pub(crate) enum PluginApprovalResponse {
    Approve,
    ApproveAlways,
    Deny,
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
    /// Session-only guard for one-time sandbox OFF toast.
    pub sandbox_off_toast_shown: bool,
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
    pub actions: Vec<ToastAction>,
}

#[derive(Clone, Debug)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum ToastActionKind {
    SandboxApplyNow,
    SandboxApplyLater,
    SandboxPersistRevert,
    SandboxPersistKeep,
    SandboxRemapTabs,
    SandboxSkipRemap,
}

#[derive(Clone, Debug)]
pub(crate) struct ToastAction {
    pub label_key: &'static str,
    pub kind: ToastActionKind,
}

impl ToastAction {
    pub(crate) fn new(label_key: &'static str, kind: ToastActionKind) -> Self {
        Self { label_key, kind }
    }
}

impl Toast {
    pub(crate) fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            created: std::time::Instant::now(),
            is_error: true,
            actions: Vec::new(),
        }
    }

    pub(crate) fn info(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            created: std::time::Instant::now(),
            is_error: false,
            actions: Vec::new(),
        }
    }

    pub(crate) fn info_with_actions(message: impl Into<String>, actions: Vec<ToastAction>) -> Self {
        Self {
            message: message.into(),
            created: std::time::Instant::now(),
            is_error: false,
            actions,
        }
    }

    pub(crate) fn is_expired(&self) -> bool {
        let lifetime = if self.actions.is_empty() { 4 } else { 10 };
        self.created.elapsed().as_secs() >= lifetime
    }
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
