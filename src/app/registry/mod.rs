use crate::app::ui::widgets::command_palette::CommandId;
use eframe::egui::{Key, KeyboardShortcut, Modifiers};
use std::collections::HashMap;

/// Metadata for an AI agent or CLI tool.
#[derive(Clone)]
pub struct Agent {
    pub id: String,
    pub label: String,
    pub command: String,
    /// If true, the editor will send file context and build errors upon start/sync.
    pub context_aware: bool,
}

/// Registry for managing available AI agents.
pub struct AgentRegistry {
    agents: Vec<Agent>,
    by_id: HashMap<String, usize>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
            by_id: HashMap::new(),
        }
    }

    pub fn register(&mut self, agent: Agent) {
        let id = agent.id.clone();
        let idx = self.agents.len();
        self.agents.push(agent);
        self.by_id.insert(id, idx);
    }

    pub fn clear(&mut self) {
        self.agents.clear();
        self.by_id.clear();
    }

    pub fn get_all(&self) -> &[Agent] {
        &self.agents
    }

    #[allow(dead_code)]
    pub fn find(&self, id: &str) -> Option<&Agent> {
        self.by_id.get(id).map(|&idx| &self.agents[idx])
    }
}

/// Represents an area where a panel can be placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum PanelArea {
    Left,
    Right,
    Bottom,
}

/// Příkaz vyvolávaný přes Command Palette nebo klávesovou zkratku.
#[derive(Clone)]
pub struct Command {
    pub id: String,
    pub i18n_key: &'static str,
    pub shortcut: Option<KeyboardShortcut>,
    pub action: CommandAction,
}

/// The type of action a command performs.
#[derive(Clone)]
pub enum CommandAction {
    /// An internal command handled by the hardcoded `execute_command`.
    Internal(CommandId),
}

/// Registry for managing all commands in the application.
pub struct CommandRegistry {
    commands: Vec<Command>,
    by_id: HashMap<String, usize>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            by_id: HashMap::new(),
        }
    }

    pub fn register(&mut self, cmd: Command) {
        let id = cmd.id.clone();
        let idx = self.commands.len();
        self.commands.push(cmd);
        self.by_id.insert(id, idx);
    }

    pub fn get_all(&self) -> &[Command] {
        &self.commands
    }

    #[allow(dead_code)]
    pub fn find(&self, id: &str) -> Option<&Command> {
        self.by_id.get(id).map(|&idx| &self.commands[idx])
    }
}

/// A panel that can be displayed in the UI.
#[allow(dead_code)]
pub struct Panel {
    pub id: String,
    pub title_i18n_key: &'static str,
    pub area: PanelArea,
}

/// Registry for managing UI panels.
pub struct PanelRegistry {
    #[allow(dead_code)]
    panels: HashMap<PanelArea, Vec<Panel>>,
}

impl PanelRegistry {
    pub fn new() -> Self {
        Self {
            panels: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn register(&mut self, panel: Panel) {
        self.panels.entry(panel.area).or_default().push(panel);
    }

    #[allow(dead_code)]
    pub fn get_for_area(&self, area: PanelArea) -> &[Panel] {
        self.panels.get(&area).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

/// The root registry holding all extension points.
pub struct Registry {
    pub commands: CommandRegistry,
    pub agents: AgentRegistry,
    #[allow(dead_code)]
    pub panels: PanelRegistry,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            commands: CommandRegistry::new(),
            agents: AgentRegistry::new(),
            panels: PanelRegistry::new(),
        }
    }

    /// Inicializuje registry výchozími interními příkazy.
    pub fn init_defaults(&mut self) {
        use CommandId::*;

        // Helper: shortcut s jedním modifikátorem (COMMAND = Ctrl na Linuxu, Cmd na macOS)
        let cmd = |key: Key| -> Option<KeyboardShortcut> {
            Some(KeyboardShortcut::new(Modifiers::COMMAND, key))
        };
        // Helper: shortcut se dvěma modifikátory (COMMAND + SHIFT)
        let cmd_shift = |key: Key| -> Option<KeyboardShortcut> {
            Some(KeyboardShortcut::new(
                Modifiers::COMMAND | Modifiers::SHIFT,
                key,
            ))
        };
        // Helper: shortcut se dvěma modifikátory (COMMAND + ALT)
        let cmd_alt = |key: Key| -> Option<KeyboardShortcut> {
            Some(KeyboardShortcut::new(
                Modifiers::COMMAND | Modifiers::ALT,
                key,
            ))
        };

        let defaults: Vec<(&str, &str, Option<KeyboardShortcut>, CommandId)> = vec![
            (
                "editor.open_file",
                "command-name-open-file",
                cmd(Key::P),
                OpenFile,
            ),
            (
                "editor.project_search",
                "command-name-project-search",
                cmd_shift(Key::F),
                ProjectSearch,
            ),
            ("build.run_build", "command-name-build", cmd(Key::B), Build),
            ("build.run_profile", "command-name-run", cmd(Key::R), Run),
            ("editor.save", "command-name-save", cmd(Key::S), Save),
            (
                "editor.close_tab",
                "command-name-close-tab",
                cmd(Key::W),
                CloseTab,
            ),
            ("project.new", "command-name-new-project", None, NewProject),
            (
                "project.open",
                "command-name-open-project",
                None,
                OpenProject,
            ),
            (
                "project.open_folder",
                "command-name-open-folder",
                None,
                OpenFolder,
            ),
            (
                "project.trash_preview",
                "command-name-trash-preview",
                None,
                TrashPreview,
            ),
            (
                "ui.toggle_left",
                "command-name-toggle-left",
                None,
                ToggleLeft,
            ),
            (
                "ui.toggle_right",
                "command-name-toggle-right",
                None,
                ToggleRight,
            ),
            (
                "ui.toggle_build",
                "command-name-toggle-build",
                None,
                ToggleBuild,
            ),
            (
                "ui.toggle_float",
                "command-name-toggle-float",
                None,
                ToggleFloat,
            ),
            (
                "ui.show_settings",
                "command-name-show-settings",
                cmd(Key::Comma),
                Settings,
            ),
            ("ui.show_about", "command-name-show-about", None, About),
            ("app.quit", "command-name-quit", None, Quit),
            // Trojkombinace pro přepínání fokus panelů
            (
                "ui.focus_editor",
                "command-name-focus-editor",
                cmd_alt(Key::E),
                FocusEditor,
            ),
            (
                "ui.focus_build",
                "command-name-focus-build",
                cmd_alt(Key::B),
                FocusBuild,
            ),
            (
                "ui.focus_claude",
                "command-name-focus-claude",
                cmd_alt(Key::A),
                FocusClaude,
            ),
            // Find / Replace / Go to Line
            ("editor.find", "command-name-find", cmd(Key::F), Find),
            (
                "editor.replace",
                "command-name-replace",
                cmd(Key::H),
                Replace,
            ),
            (
                "editor.goto_line",
                "command-name-goto-line",
                cmd(Key::G),
                GotoLine,
            ),
            // Command Palette — Ctrl+Shift+P (primární)
            (
                "ui.command_palette",
                "command-name-command-palette",
                cmd_shift(Key::P),
                CommandPalette,
            ),
            // Command Palette — F1 (alternativní, bez modifikátoru)
            (
                "ui.command_palette_f1",
                "command-name-command-palette",
                Some(KeyboardShortcut::new(Modifiers::NONE, Key::F1)),
                CommandPalette,
            ),
        ];

        for (id, i18n, shortcut, internal_id) in defaults {
            self.commands.register(Command {
                id: id.to_string(),
                i18n_key: i18n,
                shortcut,
                action: CommandAction::Internal(internal_id),
            });
        }
    }
}
