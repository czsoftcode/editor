use crate::app::keymap::{is_reserved_shortcut, parse_shortcut};
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

    /// Vrátí mutable referenci na command dle id. Používá se pro keybinding overrides.
    pub fn find_mut(&mut self, id: &str) -> Option<&mut Command> {
        self.by_id.get(id).map(|&idx| &mut self.commands[idx])
    }

    /// Aplikuje uživatelské keybinding overrides na commands v registry.
    ///
    /// Pro každý override:
    /// - Neexistující command id → warning, skip
    /// - Prázdný shortcut string → vymaže shortcut (Command.shortcut = None)
    /// - Nevalidní shortcut string → warning, skip
    /// - Reserved klávesa (Ctrl+A/C/V/X/Z/Y) → warning, skip
    /// - Konflikt (stejná zkratka na jiném commandu) → warning, ale aplikuje se
    pub fn apply_keybinding_overrides(&mut self, overrides: &HashMap<String, String>) {
        // Sbíráme přiřazené zkratky pro detekci konfliktů
        let mut assigned: HashMap<KeyboardShortcut, String> = HashMap::new();

        // Předplníme existující přiřazení z commands
        for cmd in &self.commands {
            if let Some(shortcut) = cmd.shortcut {
                assigned.insert(shortcut, cmd.id.clone());
            }
        }

        for (id, shortcut_str) in overrides {
            // Lookup command
            let idx = match self.by_id.get(id.as_str()) {
                Some(&idx) => idx,
                None => {
                    eprintln!("[keybinding] unknown command: {id}");
                    continue;
                }
            };

            // Prázdný string = vymazání zkratky
            if shortcut_str.is_empty() {
                // Odstranit z assigned mapy pokud tam byl
                if let Some(old_shortcut) = self.commands[idx].shortcut {
                    assigned.remove(&old_shortcut);
                }
                self.commands[idx].shortcut = None;
                continue;
            }

            // Parsovat shortcut
            let parsed = match parse_shortcut(shortcut_str) {
                Some(s) => s,
                None => {
                    eprintln!("[keybinding] invalid shortcut for {id}: {shortcut_str}");
                    continue;
                }
            };

            // Kontrola reserved kláves
            if is_reserved_shortcut(&parsed) {
                eprintln!("[keybinding] reserved key for {id}: {shortcut_str}");
                continue;
            }

            // Kontrola konfliktu
            if let Some(existing_id) = assigned.get(&parsed)
                && existing_id != id
            {
                eprintln!(
                    "[keybinding] conflict: {shortcut_str} already assigned to {existing_id}, overriding for {id}"
                );
            }

            // Odstranit starou zkratku z assigned
            if let Some(old_shortcut) = self.commands[idx].shortcut {
                assigned.remove(&old_shortcut);
            }

            // Přepsat shortcut
            self.commands[idx].shortcut = Some(parsed);
            assigned.insert(parsed, id.clone());
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Pomocný setup: vytvoří registry s init_defaults a vrátí Registry.
    fn setup_registry() -> Registry {
        let mut registry = Registry::new();
        registry.init_defaults();
        registry
    }

    #[test]
    fn test_find_mut_returns_command() {
        let mut registry = setup_registry();
        let cmd = registry.commands.find_mut("editor.save");
        assert!(cmd.is_some(), "find_mut musí najít existující command");
        assert_eq!(cmd.unwrap().id, "editor.save");
    }

    #[test]
    fn test_find_mut_returns_none_for_unknown() {
        let mut registry = setup_registry();
        let cmd = registry.commands.find_mut("nonexistent.command");
        assert!(
            cmd.is_none(),
            "find_mut musí vrátit None pro neexistující command"
        );
    }

    #[test]
    fn test_apply_override_basic() {
        let mut registry = setup_registry();
        let mut overrides = HashMap::new();
        overrides.insert("editor.save".to_string(), "Ctrl+Shift+S".to_string());

        registry.commands.apply_keybinding_overrides(&overrides);

        let cmd = registry
            .commands
            .find("editor.save")
            .expect("command must exist");
        let shortcut = cmd.shortcut.expect("shortcut must be set");
        assert_eq!(shortcut.logical_key, Key::S);
        assert_eq!(
            shortcut.modifiers,
            Modifiers::COMMAND | Modifiers::SHIFT,
            "Override musí nastavit Ctrl+Shift"
        );
    }

    #[test]
    fn test_apply_override_empty_string() {
        let mut registry = setup_registry();
        assert!(
            registry
                .commands
                .find("editor.save")
                .unwrap()
                .shortcut
                .is_some(),
            "editor.save musí mít default shortcut"
        );

        let mut overrides = HashMap::new();
        overrides.insert("editor.save".to_string(), "".to_string());

        registry.commands.apply_keybinding_overrides(&overrides);

        let cmd = registry.commands.find("editor.save").unwrap();
        assert!(
            cmd.shortcut.is_none(),
            "Prázdný override string musí vymazat shortcut"
        );
    }

    #[test]
    fn test_apply_override_invalid_shortcut() {
        let mut registry = setup_registry();
        let original_shortcut = registry.commands.find("editor.save").unwrap().shortcut;

        let mut overrides = HashMap::new();
        overrides.insert("editor.save".to_string(), "Foo+Bar".to_string());

        registry.commands.apply_keybinding_overrides(&overrides);

        let cmd = registry.commands.find("editor.save").unwrap();
        assert_eq!(
            cmd.shortcut, original_shortcut,
            "Nevalidní override se musí ignorovat — shortcut zůstane default"
        );
    }

    #[test]
    fn test_apply_override_unknown_id() {
        let mut registry = setup_registry();
        let commands_before: Vec<_> = registry
            .commands
            .get_all()
            .iter()
            .map(|c| (c.id.clone(), c.shortcut))
            .collect();

        let mut overrides = HashMap::new();
        overrides.insert(
            "nonexistent.command".to_string(),
            "Ctrl+Shift+X".to_string(),
        );

        registry.commands.apply_keybinding_overrides(&overrides);

        let commands_after: Vec<_> = registry
            .commands
            .get_all()
            .iter()
            .map(|c| (c.id.clone(), c.shortcut))
            .collect();
        assert_eq!(
            commands_before, commands_after,
            "Neexistující id se musí ignorovat"
        );
    }

    #[test]
    fn test_apply_override_reserved_key() {
        let mut registry = setup_registry();
        let original_shortcut = registry.commands.find("editor.save").unwrap().shortcut;

        let mut overrides = HashMap::new();
        overrides.insert("editor.save".to_string(), "Ctrl+C".to_string());

        registry.commands.apply_keybinding_overrides(&overrides);

        let cmd = registry.commands.find("editor.save").unwrap();
        assert_eq!(
            cmd.shortcut, original_shortcut,
            "Reserved key (Ctrl+C) override se musí odmítnout"
        );
    }

    #[test]
    fn test_apply_override_conflict_detection() {
        let mut registry = setup_registry();

        // Override dvou commandů na stejnou zkratku — obě by měly projít
        let mut overrides = HashMap::new();
        overrides.insert("editor.save".to_string(), "Ctrl+Shift+Q".to_string());
        overrides.insert("editor.find".to_string(), "Ctrl+Shift+Q".to_string());

        registry.commands.apply_keybinding_overrides(&overrides);

        let save_cmd = registry.commands.find("editor.save").unwrap();
        let find_cmd = registry.commands.find("editor.find").unwrap();

        // HashMap iterace je nedeterministická — alespoň jeden musí mít Ctrl+Shift+Q
        let expected = crate::app::keymap::parse_shortcut("Ctrl+Shift+Q").unwrap();
        let save_has_it = save_cmd.shortcut == Some(expected);
        let find_has_it = find_cmd.shortcut == Some(expected);
        assert!(
            save_has_it || find_has_it,
            "Alespoň jeden command musí mít overridden shortcut Ctrl+Shift+Q"
        );
    }

    #[test]
    fn test_apply_override_reserved_keys_comprehensive() {
        let reserved = ["Ctrl+A", "Ctrl+C", "Ctrl+V", "Ctrl+X", "Ctrl+Z", "Ctrl+Y"];

        for key_str in reserved {
            let mut registry = setup_registry();
            let original = registry.commands.find("editor.find").unwrap().shortcut;

            let mut overrides = HashMap::new();
            overrides.insert("editor.find".to_string(), key_str.to_string());

            registry.commands.apply_keybinding_overrides(&overrides);

            let cmd = registry.commands.find("editor.find").unwrap();
            assert_eq!(
                cmd.shortcut, original,
                "Reserved key {key_str} musí být odmítnuta"
            );
        }
    }

    #[test]
    fn test_apply_override_ctrl_shift_not_reserved() {
        // Ctrl+Shift+C NENÍ reserved (reserved je jen Ctrl+C bez Shift)
        let mut registry = setup_registry();

        let mut overrides = HashMap::new();
        overrides.insert("editor.find".to_string(), "Ctrl+Shift+C".to_string());

        registry.commands.apply_keybinding_overrides(&overrides);

        let cmd = registry.commands.find("editor.find").unwrap();
        let shortcut = cmd.shortcut.expect("Ctrl+Shift+C by měl projít");
        assert_eq!(shortcut.logical_key, Key::C);
        assert_eq!(shortcut.modifiers, Modifiers::COMMAND | Modifiers::SHIFT);
    }
}
