mod app;
mod editor;
mod file_tree;
mod highlighter;
mod terminal;
mod watcher;

use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let root_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let root_path = root_path.canonicalize().unwrap_or(root_path);

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title(format!("Rust Editor — {}", root_path.display())),
        ..Default::default()
    };

    eframe::run_native(
        "Rust Editor",
        options,
        Box::new(move |_cc| Ok(Box::new(app::EditorApp::new(root_path)))),
    )
}
