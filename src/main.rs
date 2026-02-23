#![allow(dead_code)]
mod app;
mod config;
mod highlighter;
pub mod i18n;
mod ipc;
mod settings;
mod watcher;

use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    // Set terminal environment variables before any threads are spawned.
    // set_var is unsafe in Edition 2024 — calling it here is safe because threads
    // do not exist yet.
    unsafe {
        std::env::set_var("TERM", "xterm-256color");
        std::env::set_var("COLORTERM", "truecolor");
        std::env::set_var("PROMPT_COMMAND", "PS1='\\$ '");
    }

    let args: Vec<String> = std::env::args().collect();
    // --new-instance skips the IPC singleton check — allows running an independent instance
    // for development and testing (equivalent to --no-remote in Firefox).
    let new_instance = args.iter().any(|a| a == "--new-instance");
    let root_path = args
        .into_iter()
        .skip(1)
        .find(|a| !a.starts_with("--"))
        .map(|arg| {
            let p = PathBuf::from(arg);
            p.canonicalize().unwrap_or(p)
        });

    // Single-process multi-window architecture:
    // If a primary instance is already running, delegate to it and exit.
    // With --new-instance, skip this step and start a new independent instance.
    if !new_instance && ipc::Ipc::ping() {
        if let Some(ref path) = root_path {
            ipc::Ipc::open_in_new_window(path);
        } else {
            ipc::Ipc::focus_main();
        }
        return Ok(());
    }

    let title = match &root_path {
        Some(p) => format!("PolyCredo Editor — {}", p.display()),
        None => "PolyCredo Editor".to_string(),
    };

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([config::WINDOW_DEFAULT_WIDTH, config::WINDOW_DEFAULT_HEIGHT])
            .with_title(title),
        persist_window: true,
        ..Default::default()
    };

    eframe::run_native(
        "polycredo-editor",
        options,
        Box::new(move |cc| Ok(Box::new(app::EditorApp::new(cc, root_path)))),
    )
}
