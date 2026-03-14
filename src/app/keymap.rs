//! Centrální keymap modul — dispatch klávesových zkratek, parser a formátovač.
//!
//! Tento modul nemá UI závislosti a je plně testovatelný unit testy.
//! `Keymap` drží seřazené bindings (od nejvíce modifikátorů po nejméně),
//! takže trojkombinace (Ctrl+Alt+B) matchne před dvoukombinací (Ctrl+B).

use crate::app::registry::Command;
use crate::app::ui::widgets::command_palette::CommandId;
use eframe::egui;
use egui::{Key, KeyboardShortcut, Modifiers};

/// Centrální keymap — drží seřazené klávesové zkratky a dispatchuje je.
pub struct Keymap {
    /// Bindings seřazené od nejvíce modifikátorů po nejméně.
    /// Díky tomuto řazení `dispatch` matchne trojkombinace (Ctrl+Alt+B)
    /// před dvoukombinací (Ctrl+B) a zabrání falešným matchům.
    pub bindings: Vec<(KeyboardShortcut, CommandId)>,
}

impl Keymap {
    /// Vytvoří keymapu z command registry — extrahuje shortcuty a seřadí
    /// od nejvíce modifikátorů po nejméně.
    pub fn from_commands(commands: &[Command]) -> Self {
        let mut bindings: Vec<(KeyboardShortcut, CommandId)> = commands
            .iter()
            .filter_map(|cmd| {
                let shortcut = cmd.shortcut?;
                let cmd_id = match &cmd.action {
                    crate::app::registry::CommandAction::Internal(id) => *id,
                };
                Some((shortcut, cmd_id))
            })
            .collect();

        // Seřazení: víc modifikátorů = vyšší priorita (dřív v poli)
        bindings.sort_by(|a, b| modifier_count(b.0.modifiers).cmp(&modifier_count(a.0.modifiers)));

        Self { bindings }
    }

    /// Dispatchuje klávesovou zkratku — iteruje seřazené bindings,
    /// volá `input.consume_shortcut()`, vrátí první match.
    ///
    /// Vrací `None` pokud žádný binding nematchuje (diagnostický signál).
    /// Vrací `Some(CommandId)` pokud binding matchl (event je konzumován).
    pub fn dispatch(&self, input: &mut egui::InputState) -> Option<CommandId> {
        for (shortcut, cmd_id) in &self.bindings {
            if input.consume_shortcut(shortcut) {
                return Some(*cmd_id);
            }
        }
        None
    }
}

/// Spočítá počet aktivních modifikátorů pro řazení.
fn modifier_count(m: Modifiers) -> u8 {
    let mut count = 0u8;
    if m.ctrl || m.command {
        count += 1;
    }
    if m.alt {
        count += 1;
    }
    if m.shift {
        count += 1;
    }
    if m.mac_cmd {
        count += 1;
    }
    count
}

/// Parsuje stringovou zkratku na `KeyboardShortcut`.
///
/// Rozpoznává formáty jako "Ctrl+S", "Ctrl+Alt+B", "Ctrl+Shift+F".
/// "Ctrl" a "Cmd" se mapují na `Modifiers::COMMAND` (cross-platform).
///
/// Vrací `None` pro nevalidní vstup (ne panic) — diagnostický signál.
pub fn parse_shortcut(s: &str) -> Option<KeyboardShortcut> {
    let parts: Vec<&str> = s.split('+').collect();
    if parts.is_empty() {
        return None;
    }

    let mut modifiers = Modifiers::NONE;
    let key_part = parts.last()?;

    // Zpracování modifikátorů (vše kromě posledního dílu)
    for &part in &parts[..parts.len() - 1] {
        match part {
            "Ctrl" | "Cmd" => modifiers |= Modifiers::COMMAND,
            "Alt" | "Option" => modifiers |= Modifiers::ALT,
            "Shift" => modifiers |= Modifiers::SHIFT,
            _ => return None, // Neznámý modifikátor
        }
    }

    // Parsování klávesy — egui Key::from_name akceptuje názvy jako "S", "B", "F1" atd.
    // Pro speciální znaky potřebujeme mapování
    let key = match *key_part {
        "," => Some(Key::Comma),
        "." => Some(Key::Period),
        "-" => Some(Key::Minus),
        "+" => Some(Key::Plus),
        "=" => Some(Key::Equals),
        ";" => Some(Key::Semicolon),
        "/" => Some(Key::Slash),
        "\\" => Some(Key::Backslash),
        "[" => Some(Key::OpenBracket),
        "]" => Some(Key::CloseBracket),
        "`" => Some(Key::Backtick),
        "'" => Some(Key::Quote),
        name => Key::from_name(name),
    }?;

    // Musí mít aspoň jeden modifikátor, POKUD klávesa není na whitelistu
    // standalone kláves (F1–F12, Escape, Delete, Insert).
    if modifiers == Modifiers::NONE && !is_standalone_key_allowed(&key) {
        return None;
    }

    Some(KeyboardShortcut::new(modifiers, key))
}

/// Whitelist kláves, které mohou fungovat jako zkratky bez modifikátoru.
/// Zahrnuje F1–F12, Escape, Delete a Insert.
fn is_standalone_key_allowed(key: &Key) -> bool {
    matches!(
        key,
        Key::F1
            | Key::F2
            | Key::F3
            | Key::F4
            | Key::F5
            | Key::F6
            | Key::F7
            | Key::F8
            | Key::F9
            | Key::F10
            | Key::F11
            | Key::F12
            | Key::Escape
            | Key::Delete
            | Key::Insert
    )
}

/// Formátuje `KeyboardShortcut` na čitelný string, platform-aware.
///
/// Na Linuxu: "Ctrl+Shift+F", na macOS: "Shift+Cmd+F".
pub fn format_shortcut(shortcut: &KeyboardShortcut) -> String {
    let is_mac = cfg!(target_os = "macos");
    shortcut.format(&egui::ModifierNames::NAMES, is_mac)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shortcut_basic() {
        let s = parse_shortcut("Ctrl+S").expect("mělo se parsovat 'Ctrl+S'");
        assert_eq!(s.modifiers, Modifiers::COMMAND, "Ctrl se mapuje na COMMAND");
        assert_eq!(s.logical_key, Key::S);
    }

    #[test]
    fn test_parse_shortcut_triple() {
        let s = parse_shortcut("Ctrl+Alt+B").expect("mělo se parsovat 'Ctrl+Alt+B'");
        assert_eq!(
            s.modifiers,
            Modifiers::COMMAND | Modifiers::ALT,
            "Ctrl+Alt se mapuje na COMMAND | ALT"
        );
        assert_eq!(s.logical_key, Key::B);
    }

    #[test]
    fn test_parse_shortcut_shift() {
        let s = parse_shortcut("Ctrl+Shift+F").expect("mělo se parsovat 'Ctrl+Shift+F'");
        assert_eq!(
            s.modifiers,
            Modifiers::COMMAND | Modifiers::SHIFT,
            "Ctrl+Shift se mapuje na COMMAND | SHIFT"
        );
        assert_eq!(s.logical_key, Key::F);
    }

    #[test]
    fn test_parse_shortcut_invalid() {
        assert!(
            parse_shortcut("Foo+Bar").is_none(),
            "neznámý modifikátor vrátí None"
        );
        assert!(
            parse_shortcut("S").is_none(),
            "samotná klávesa bez modifikátoru vrátí None"
        );
        assert!(parse_shortcut("").is_none(), "prázdný string vrátí None");
        assert!(
            parse_shortcut("Ctrl+Xyz123").is_none(),
            "neznámá klávesa vrátí None"
        );
    }

    #[test]
    fn test_parse_shortcut_comma() {
        let s = parse_shortcut("Ctrl+,").expect("mělo se parsovat 'Ctrl+,'");
        assert_eq!(s.modifiers, Modifiers::COMMAND);
        assert_eq!(s.logical_key, Key::Comma);
    }

    #[test]
    fn test_format_shortcut() {
        // Roundtrip parse→format na Linuxu
        let shortcut = parse_shortcut("Ctrl+Shift+F").unwrap();
        let formatted = shortcut.format(&egui::ModifierNames::NAMES, false);
        assert_eq!(formatted, "Ctrl+Shift+F");

        let shortcut2 = parse_shortcut("Ctrl+Alt+B").unwrap();
        let formatted2 = shortcut2.format(&egui::ModifierNames::NAMES, false);
        assert_eq!(formatted2, "Ctrl+Alt+B");

        let shortcut3 = parse_shortcut("Ctrl+S").unwrap();
        let formatted3 = shortcut3.format(&egui::ModifierNames::NAMES, false);
        assert_eq!(formatted3, "Ctrl+S");
    }

    #[test]
    fn test_dispatch_ordering() {
        // Keymap s Ctrl+B (Build) a Ctrl+Alt+B (FocusBuild):
        // dispatch pro Ctrl+Alt+B musí vrátit FocusBuild, ne Build.

        use crate::app::registry::{Command, CommandAction};

        let commands = vec![
            Command {
                id: "build.run_build".to_string(),
                i18n_key: "build",
                shortcut: Some(KeyboardShortcut::new(Modifiers::COMMAND, Key::B)),
                action: CommandAction::Internal(CommandId::Build),
            },
            Command {
                id: "ui.focus_build".to_string(),
                i18n_key: "focus-build",
                shortcut: Some(KeyboardShortcut::new(
                    Modifiers::COMMAND | Modifiers::ALT,
                    Key::B,
                )),
                action: CommandAction::Internal(CommandId::FocusBuild),
            },
        ];

        let keymap = Keymap::from_commands(&commands);

        // Ověříme řazení: trojkombinace (2 modifikátory) je před dvoukombinací (1 modifikátor)
        assert_eq!(
            keymap.bindings[0].1,
            CommandId::FocusBuild,
            "Ctrl+Alt+B (FocusBuild) musí být v keymapu před Ctrl+B (Build)"
        );
        assert_eq!(keymap.bindings[1].1, CommandId::Build);

        // Simulujeme dispatch přes egui RawInput
        let ctx = egui::Context::default();
        // Vložíme Ctrl+Alt+B event
        let mut raw = egui::RawInput::default();
        raw.events.push(egui::Event::Key {
            key: Key::B,
            physical_key: Some(Key::B),
            pressed: true,
            repeat: false,
            modifiers: Modifiers {
                alt: true,
                ctrl: true,
                shift: false,
                mac_cmd: false,
                command: true,
            },
        });
        ctx.begin_pass(raw);

        let result = ctx.input_mut(|input| keymap.dispatch(input));

        assert_eq!(
            result,
            Some(CommandId::FocusBuild),
            "Ctrl+Alt+B musí matchnout FocusBuild, ne Build"
        );

        let _ = ctx.end_pass();
    }

    #[test]
    fn test_dispatch_consumes_event() {
        // Po dispatch musí vrátit None pro stejný event (konzumace)

        use crate::app::registry::{Command, CommandAction};

        let commands = vec![Command {
            id: "editor.save".to_string(),
            i18n_key: "save",
            shortcut: Some(KeyboardShortcut::new(Modifiers::COMMAND, Key::S)),
            action: CommandAction::Internal(CommandId::Save),
        }];

        let keymap = Keymap::from_commands(&commands);

        let ctx = egui::Context::default();
        let mut raw = egui::RawInput::default();
        raw.events.push(egui::Event::Key {
            key: Key::S,
            physical_key: Some(Key::S),
            pressed: true,
            repeat: false,
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
                mac_cmd: false,
                command: true,
            },
        });
        ctx.begin_pass(raw);

        // První dispatch — matchne
        let first = ctx.input_mut(|input| keymap.dispatch(input));
        assert_eq!(
            first,
            Some(CommandId::Save),
            "první dispatch musí matchnout Save"
        );

        // Druhý dispatch — event je konzumován, vrátí None
        let second = ctx.input_mut(|input| keymap.dispatch(input));
        assert_eq!(
            second, None,
            "druhý dispatch musí vrátit None (event konzumován)"
        );

        let _ = ctx.end_pass();
    }

    #[test]
    fn test_parse_shortcut_f1() {
        // F1 bez modifikátoru — whitelist standalone kláves
        let s = parse_shortcut("F1").expect("mělo se parsovat 'F1'");
        assert_eq!(s.modifiers, Modifiers::NONE, "F1 nemá modifikátor");
        assert_eq!(s.logical_key, Key::F1);
    }

    #[test]
    fn test_parse_shortcut_escape() {
        // Escape bez modifikátoru — whitelist standalone kláves
        let s = parse_shortcut("Escape").expect("mělo se parsovat 'Escape'");
        assert_eq!(s.modifiers, Modifiers::NONE, "Escape nemá modifikátor");
        assert_eq!(s.logical_key, Key::Escape);
    }

    #[test]
    fn test_dispatch_new_commands() {
        // Keymap s Find (Ctrl+F) → dispatch Ctrl+F vrátí Find
        use crate::app::registry::{Command, CommandAction};

        let commands = vec![
            Command {
                id: "editor.find".to_string(),
                i18n_key: "command-name-find",
                shortcut: Some(KeyboardShortcut::new(Modifiers::COMMAND, Key::F)),
                action: CommandAction::Internal(CommandId::Find),
            },
            Command {
                id: "editor.replace".to_string(),
                i18n_key: "command-name-replace",
                shortcut: Some(KeyboardShortcut::new(Modifiers::COMMAND, Key::H)),
                action: CommandAction::Internal(CommandId::Replace),
            },
            Command {
                id: "editor.goto_line".to_string(),
                i18n_key: "command-name-goto-line",
                shortcut: Some(KeyboardShortcut::new(Modifiers::COMMAND, Key::G)),
                action: CommandAction::Internal(CommandId::GotoLine),
            },
        ];

        let keymap = Keymap::from_commands(&commands);

        let ctx = egui::Context::default();
        let mut raw = egui::RawInput::default();
        raw.events.push(egui::Event::Key {
            key: Key::F,
            physical_key: Some(Key::F),
            pressed: true,
            repeat: false,
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
                mac_cmd: false,
                command: true,
            },
        });
        ctx.begin_pass(raw);

        let result = ctx.input_mut(|input| keymap.dispatch(input));
        assert_eq!(result, Some(CommandId::Find), "Ctrl+F musí matchnout Find");

        let _ = ctx.end_pass();
    }

    #[test]
    fn test_dispatch_command_palette_ordering() {
        // Ctrl+Shift+P vrátí CommandPalette, ne OpenFile (Ctrl+P)
        use crate::app::registry::{Command, CommandAction};

        let commands = vec![
            Command {
                id: "editor.open_file".to_string(),
                i18n_key: "command-name-open-file",
                shortcut: Some(KeyboardShortcut::new(Modifiers::COMMAND, Key::P)),
                action: CommandAction::Internal(CommandId::OpenFile),
            },
            Command {
                id: "ui.command_palette".to_string(),
                i18n_key: "command-name-command-palette",
                shortcut: Some(KeyboardShortcut::new(
                    Modifiers::COMMAND | Modifiers::SHIFT,
                    Key::P,
                )),
                action: CommandAction::Internal(CommandId::CommandPalette),
            },
        ];

        let keymap = Keymap::from_commands(&commands);

        // Ověříme řazení: Ctrl+Shift+P (2 modifikátory) je před Ctrl+P (1 modifikátor)
        assert_eq!(
            keymap.bindings[0].1,
            CommandId::CommandPalette,
            "Ctrl+Shift+P (CommandPalette) musí být v keymapu před Ctrl+P (OpenFile)"
        );
        assert_eq!(keymap.bindings[1].1, CommandId::OpenFile);

        // Dispatch Ctrl+Shift+P
        let ctx = egui::Context::default();
        let mut raw = egui::RawInput::default();
        raw.events.push(egui::Event::Key {
            key: Key::P,
            physical_key: Some(Key::P),
            pressed: true,
            repeat: false,
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
                shift: true,
                mac_cmd: false,
                command: true,
            },
        });
        ctx.begin_pass(raw);

        let result = ctx.input_mut(|input| keymap.dispatch(input));
        assert_eq!(
            result,
            Some(CommandId::CommandPalette),
            "Ctrl+Shift+P musí matchnout CommandPalette, ne OpenFile"
        );

        let _ = ctx.end_pass();
    }

    #[test]
    fn test_empty_keymap_dispatch() {
        // Prázdná keymapa vrátí None — diagnostický signál
        let keymap = Keymap {
            bindings: Vec::new(),
        };

        let ctx = egui::Context::default();
        let mut raw = egui::RawInput::default();
        raw.events.push(egui::Event::Key {
            key: Key::S,
            physical_key: Some(Key::S),
            pressed: true,
            repeat: false,
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
                mac_cmd: false,
                command: true,
            },
        });
        ctx.begin_pass(raw);

        let result = ctx.input_mut(|input| keymap.dispatch(input));
        assert_eq!(result, None, "prázdná keymapa musí vrátit None");

        let _ = ctx.end_pass();
    }
}
