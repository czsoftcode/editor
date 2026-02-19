/// Maximální počet nedávných projektů v historii.
pub const MAX_RECENT_PROJECTS: usize = 10;

/// Interval překreslování UI pro autosave a watcher (v ms).
/// Terminál a watcher si řídí vlastní repaints přes ctx.request_repaint() —
/// tento interval slouží jen jako záloha pro polling kanalů na pozadí.
pub const REPAINT_INTERVAL_MS: u64 = 500;

/// Základní velikost fontu editoru (monospace).
pub const EDITOR_FONT_SIZE: f32 = 14.0;

/// Velikost fontu záložek, kódu v markdown náhledu a toastů.
pub const UI_FONT_SIZE: f32 = 13.0;

/// Velikost fontu adresářového stromu.
pub const FILE_TREE_FONT_SIZE: f32 = 16.0;

/// Výška status baru ve spodní části okna.
pub const STATUS_BAR_HEIGHT: f32 = 22.0;

/// Výchozí šířka levého panelu (soubory + build).
pub const LEFT_PANEL_DEFAULT_WIDTH: f32 = 300.0;
pub const LEFT_PANEL_MIN_WIDTH: f32 = 200.0;
pub const LEFT_PANEL_MAX_WIDTH: f32 = 500.0;

/// Výchozí šířka pravého panelu (AI terminál).
pub const AI_PANEL_DEFAULT_WIDTH: f32 = 400.0;
pub const AI_PANEL_MIN_WIDTH: f32 = 200.0;
pub const AI_PANEL_MAX_WIDTH: f32 = 600.0;

/// Výchozí rozměry okna aplikace.
pub const WINDOW_DEFAULT_WIDTH: f32 = 1200.0;
pub const WINDOW_DEFAULT_HEIGHT: f32 = 800.0;

/// Krok posunu záložek při kliknutí na šipku.
pub const TAB_SCROLL_STEP: f32 = 150.0;

/// Šířka tlačítek ◀/▶ v tab baru.
pub const TAB_BTN_WIDTH: f32 = 22.0;

/// Šířka scrollbaru v terminálovém widgetu.
pub const TERMINAL_SCROLLBAR_WIDTH: f32 = 10.0;

/// Maximální výška build error listu v levém panelu.
pub const BUILD_ERROR_LIST_MAX_HEIGHT: f32 = 150.0;

/// Maximální počet PTY eventů zpracovaných za jeden snímek UI.
/// Brání blokování UI při burstu výstupu terminálového procesu.
pub const TERMINAL_MAX_EVENTS_PER_FRAME: usize = 256;

/// Interval automatické re-detekce AI CLI nástrojů (claude, aider, …) v sekundách.
/// Detekce probíhá na pozadí a je nenápadná; manuální re-check zajišťuje tlačítko ↻.
pub const AI_TOOL_CHECK_INTERVAL_SECS: u64 = 60;
