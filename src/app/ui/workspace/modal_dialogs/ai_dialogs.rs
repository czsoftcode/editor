use crate::app::types::AppShared;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub fn show(
    _ctx: &egui::Context,
    _ws: &mut WorkspaceState,
    _shared: &Arc<Mutex<AppShared>>,
    _i18n: &I18n,
) {
    // Plugin error dialog removed — WASM plugin system no longer exists.
}
