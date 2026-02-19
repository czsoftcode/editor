use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Pomocné typy
// ---------------------------------------------------------------------------

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum FocusedPanel {
    Build,
    Claude,
    Editor,
}

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum ProjectType {
    Rust,
    Symfony,
}

impl ProjectType {
    pub(crate) fn subdir(&self) -> &'static str {
        match self {
            ProjectType::Rust => "Rust",
            ProjectType::Symfony => "Symfony",
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) enum AiTool {
    ClaudeCode,
    Codex,
    CopilotCli,
    GeminiCli,
    Aider,
    KiroCli,
}

impl AiTool {
    pub(crate) const ALL: [AiTool; 6] = [
        AiTool::ClaudeCode,
        AiTool::Codex,
        AiTool::CopilotCli,
        AiTool::GeminiCli,
        AiTool::Aider,
        AiTool::KiroCli,
    ];

    pub(crate) fn label(&self) -> &'static str {
        match self {
            AiTool::ClaudeCode => "Claude Code",
            AiTool::Codex => "Codex",
            AiTool::CopilotCli => "Copilot CLI",
            AiTool::GeminiCli => "Gemini CLI",
            AiTool::Aider => "Aider",
            AiTool::KiroCli => "Kiro CLI",
        }
    }

    pub(crate) fn command(&self) -> &'static str {
        match self {
            AiTool::ClaudeCode => "claude",
            AiTool::Codex => "codex",
            AiTool::CopilotCli => "copilot",
            AiTool::GeminiCli => "gemini",
            AiTool::Aider => "aider",
            AiTool::KiroCli => "kiro-cli",
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
}

impl Default for PersistentState {
    fn default() -> Self {
        Self {
            show_left_panel: true,
            show_right_panel: true,
            show_build_terminal: true,
            claude_float: false,
            ai_font_scale: 100,
        }
    }
}

// ---------------------------------------------------------------------------
// AppShared — sdílený stav mezi viewporty (chráněný Mutexem)
// ---------------------------------------------------------------------------

pub(crate) enum AppAction {
    /// Otevřít projekt v novém okně
    OpenInNewWindow(PathBuf),
    /// Zavřít sekundární viewport (po zavření × okna)
    CloseWorkspace(eframe::egui::ViewportId),
    /// Přidat cestu do nedávných projektů
    AddRecent(PathBuf),
    /// Ukončit celou aplikaci
    QuitAll,
}

pub(crate) struct AppShared {
    pub recent_projects: Vec<PathBuf>,
    pub actions: Vec<AppAction>,
    pub settings: crate::settings::Settings,
}

// ---------------------------------------------------------------------------
// Toast — krátkodobá notifikace v UI
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

    #[allow(dead_code)]
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

// ---------------------------------------------------------------------------
// Pomocné free funkce
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
