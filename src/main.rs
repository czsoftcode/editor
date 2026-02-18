mod app;
mod editor;
mod file_tree;
mod highlighter;
mod ipc;
mod terminal;
mod watcher;

use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    // Nastavit terminálové proměnné prostředí před vznikem jakýchkoliv vláken.
    // set_var je v Edition 2024 unsafe — volání zde je bezpečné, protože vlákna
    // zatím neexistují.
    unsafe {
        std::env::set_var("TERM", "xterm-256color");
        std::env::set_var("COLORTERM", "truecolor");
        std::env::set_var("PROMPT_COMMAND", "PS1='\\$ '");
    }

    let root_path = std::env::args().nth(1).map(|arg| {
        let p = PathBuf::from(arg);
        p.canonicalize().unwrap_or(p)
    });

    let title = match &root_path {
        Some(p) => format!("Rust Editor — {}", p.display()),
        None => "Rust Editor".to_string(),
    };

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title(title),
        persist_window: true,
        ..Default::default()
    };

    eframe::run_native(
        "Rust Editor",
        options,
        Box::new(move |cc| Ok(Box::new(app::EditorApp::new(cc, root_path)))),
    )
}
