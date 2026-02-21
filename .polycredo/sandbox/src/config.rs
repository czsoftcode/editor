/// Maximum number of recent projects in history.
pub const MAX_RECENT_PROJECTS: usize = 10;

/// UI repaint interval for autosave and watcher (in ms).
/// The terminal and watcher manage their own repaints via ctx.request_repaint() —
/// this interval serves only as a fallback for background channel polling.
pub const REPAINT_INTERVAL_MS: u64 = 500;

/// Base font size for the editor (monospace).
pub const EDITOR_FONT_SIZE: f32 = 14.0;

/// Font size for tabs, code in markdown preview, and toasts.
pub const UI_FONT_SIZE: f32 = 13.0;

/// Font size for the directory tree.
pub const FILE_TREE_FONT_SIZE: f32 = 16.0;

/// Height of the status bar at the bottom of the window.
pub const STATUS_BAR_HEIGHT: f32 = 22.0;

/// Default width of the left panel (files + build).
pub const LEFT_PANEL_DEFAULT_WIDTH: f32 = 300.0;
pub const LEFT_PANEL_MIN_WIDTH: f32 = 200.0;
pub const LEFT_PANEL_MAX_WIDTH: f32 = 500.0;

/// Default width of the right panel (AI terminal).
pub const AI_PANEL_DEFAULT_WIDTH: f32 = 400.0;
pub const AI_PANEL_MIN_WIDTH: f32 = 200.0;
pub const AI_PANEL_MAX_WIDTH: f32 = 600.0;

/// Default dimensions of the application window.
pub const WINDOW_DEFAULT_WIDTH: f32 = 1200.0;
pub const WINDOW_DEFAULT_HEIGHT: f32 = 800.0;

/// Tab scroll step when clicking arrows.
pub const TAB_SCROLL_STEP: f32 = 150.0;

/// Width of ◀/▶ buttons in the tab bar.
pub const TAB_BTN_WIDTH: f32 = 22.0;

/// Scrollbar width in the terminal widget.
pub const TERMINAL_SCROLLBAR_WIDTH: f32 = 10.0;

/// Maximum height of the build error list in the left panel.
pub const BUILD_ERROR_LIST_MAX_HEIGHT: f32 = 150.0;

/// Maximum number of PTY events processed per UI frame.
/// Prevents UI blocking during output bursts from the terminal process.
pub const TERMINAL_MAX_EVENTS_PER_FRAME: usize = 256;

/// Interval for automatic re-detection of AI CLI tools (claude, aider, …) in seconds.
/// Detection runs in the background and is unobtrusive; manual re-check is provided by the ↻ button.
pub const AI_TOOL_CHECK_INTERVAL_SECS: u64 = 60;
