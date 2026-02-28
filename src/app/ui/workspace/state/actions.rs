use super::WorkspaceState;
use crate::app::types::{FocusedPanel, PersistentState};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;

pub fn spawn_ai_tool_check(
    check_list: Vec<(String, String)>,
) -> mpsc::Receiver<HashMap<String, bool>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut results = HashMap::new();
        for (id, cmd) in check_list {
            let found = std::process::Command::new("which")
                .arg(&cmd)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            results.insert(id, found);
        }
        let _ = tx.send(results);
    });
    rx
}

pub fn spawn_win_tool_check() -> mpsc::Receiver<HashMap<String, bool>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut results = HashMap::new();

        // Check binaries in PATH
        for (id, cmd) in [
            ("xwin", "cargo-xwin"),
            ("clang", "clang"),
            ("lld", "lld"),
            ("nsis", "makensis"),
            ("generate-rpm", "cargo-generate-rpm"),
            ("rpm", "rpmbuild"),
            ("appimage", "cargo-appimage"),
            ("appimagetool", "appimagetool"),
            ("tar", "tar"),
            ("deb", "dpkg-deb"),
            ("aur", "cargo-aur"),
            ("flatpak", "flatpak-builder"),
            ("snap", "snapcraft"),
        ] {
            let found = std::process::Command::new("which")
                .arg(cmd)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            results.insert(id.to_string(), found);
        }

        // Check rustup target
        let target_found = std::process::Command::new("rustup")
            .args(["target", "list", "--installed"])
            .output()
            .map(|o| {
                let stdout = String::from_utf8_lossy(&o.stdout);
                stdout.lines().any(|l| l.trim() == "x86_64-pc-windows-msvc")
            })
            .unwrap_or(false);
        results.insert("windows-target".to_string(), target_found);

        let _ = tx.send(results);
    });
    rx
}

pub fn open_and_jump(ws: &mut WorkspaceState, path: PathBuf, line: usize) {
    open_file_in_ws(ws, path);
    ws.editor.jump_to_location(line, 1);
    ws.focused_panel = FocusedPanel::Editor;
}

pub fn open_file_in_ws(ws: &mut WorkspaceState, path: PathBuf) {
    if !path.exists() {
        return;
    }
    if let Some(existing_idx) = ws.editor.tabs.iter().position(|t| t.path == path) {
        ws.editor.active_tab = Some(existing_idx);
    } else {
        ws.editor.open_file(&path);
    }
    ws.focused_panel = FocusedPanel::Editor;
}

pub fn ws_to_panel_state(ws: &WorkspaceState) -> PersistentState {
    PersistentState {
        show_left_panel: ws.show_left_panel,
        show_right_panel: ws.show_right_panel,
        show_build_terminal: ws.show_build_terminal,
        claude_float: ws.claude_float,
        ai_font_scale: ws.ai_font_scale,
        ai_selected_provider: Some(ws.ai_selected_provider.clone()),
        ai_system_prompt: Some(ws.ai_system_prompt.clone()),
        ai_language: Some(ws.ai_language.clone()),
        ai_expertise: Some(ws.ai_expertise),
        ai_reasoning_depth: Some(ws.ai_reasoning_depth),
    }
}
