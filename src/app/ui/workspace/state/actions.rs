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
        let found_deb = std::process::Command::new("which")
            .arg("dpkg-deb")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        results.insert("deb".to_string(), found_deb);

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
        ai_font_scale: ws.ai_panel.font_scale,
        ai_selected_provider: None,
        ai_system_prompt: None,
        ai_language: None,
        ai_expertise: None,
        ai_reasoning_depth: None,
        ollama_selected_model: None,
    }
}
