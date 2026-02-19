mod app;
mod config;
mod highlighter;
pub mod i18n;
mod ipc;
mod settings;
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

    let args: Vec<String> = std::env::args().collect();
    // --new-instance přeskočí IPC singleton check — umožňuje spustit nezávislou instanci
    // pro vývoj a testování (ekvivalent --no-remote ve Firefoxu).
    let new_instance = args.iter().any(|a| a == "--new-instance");
    let root_path = args.into_iter().skip(1)
        .find(|a| !a.starts_with("--"))
        .map(|arg| {
            let p = PathBuf::from(arg);
            p.canonicalize().unwrap_or(p)
        });

    // Single-process multi-window architektura:
    // Pokud primární instance již běží, delegujeme na ni a skončíme.
    // S --new-instance tento krok přeskočíme a spustíme novou nezávislou instanci.
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
