mod app;
mod editor;
mod file_tree;
mod highlighter;
mod terminal;
mod watcher;

use std::path::PathBuf;

fn main() -> eframe::Result<()> {
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
        ..Default::default()
    };

    eframe::run_native(
        "Rust Editor",
        options,
        Box::new(move |_cc| Ok(Box::new(app::EditorApp::new(root_path)))),
    )
}
