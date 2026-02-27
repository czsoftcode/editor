pub mod approval;
pub mod inspector;
pub mod logic;
pub mod render;

use crate::app::ai::AiManager;
use crate::app::types::{AppShared, FocusedPanel};
use crate::app::ui::terminal::StandardTerminalWindow;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::config;
use crate::i18n::I18n;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub enum AiChatAction {
    Send,
    NewQuery,
    SaveSettings,
    ToggleInspector,
    Close,
}

/// Renders the AI chat as a floating `StandardTerminalWindow`.
/// Returns `true` if the window was interacted with this frame.
pub fn show(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &I18n,
) -> bool {
    if !ws.show_ai_chat {
        return false;
    }

    let mut prompt = ws.ai_prompt.clone();
    let loading = ws.ai_loading;
    let mut show_flag = ws.show_ai_chat;
    let font_size = shared.lock().expect("lock").settings.editor_font_size;

    let win = StandardTerminalWindow::new(
        i18n.get("ai-chat-title"),
        "ai_chat_terminal_win",
        FocusedPanel::AiChat,
    );

    let (interacted, action) = win.show(
        ctx,
        ws,
        &mut show_flag,
        |ui, ws_arg| {
            render::render_head(ui, ws_arg, shared);
        },
        |ui, ws_arg, body_h| {
            render::render_body(ui, ws_arg, i18n, &mut prompt, font_size, loading, body_h)
        },
        |ui, ws_arg| render::render_footer(ui, ws_arg, i18n),
    );

    ws.ai_prompt = prompt;
    ws.show_ai_chat = show_flag;

    if let Some(act) = action {
        handle_action(act, ws, shared);
    }

    interacted
}

pub fn handle_action(
    act: AiChatAction,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
) {
    match act {
        AiChatAction::Send => {
            logic::send_query_to_agent(ws, shared);
        }
        AiChatAction::NewQuery => {
            let model = {
                let sh = shared.lock().expect("lock");
                sh.settings
                    .plugins
                    .get(&ws.ai_selected_provider)
                    .and_then(|s| s.config.get("MODEL").cloned())
                    .unwrap_or_else(|| "gemini-1.5-flash".to_string())
            };
            ws.ai_response = None;
            ws.ai_prompt.clear();
            ws.ai_conversation.clear();
            ws.ai_monologue.clear();
            ws.ai_in_tokens = 0;
            ws.ai_out_tokens = 0;
            ws.ai_conversation.push((
                String::new(),
                AiManager::get_logo(
                    config::CLI_VERSION,
                    &model,
                    ws.ai_expertise,
                    ws.ai_reasoning_depth,
                ),
            ));
        }
        AiChatAction::SaveSettings => {
            let mut sh = shared.lock().expect("lock");
            let mut settings = (*sh.settings).clone();
            let ps = settings
                .plugins
                .entry(ws.ai_selected_provider.clone())
                .or_default();
            ps.expertise = ws.ai_expertise;
            ps.reasoning_depth = ws.ai_reasoning_depth;
            ps.config
                .insert("SYSTEM_PROMPT".to_string(), ws.ai_system_prompt.clone());
            ps.config
                .insert("LANGUAGE".to_string(), ws.ai_language.clone());
            settings.save();
            sh.settings = Arc::new(settings);
            ws.toasts
                .push(crate::app::types::Toast::info("AI settings saved."));
        }
        AiChatAction::ToggleInspector => {
            ws.ai_inspector_open = !ws.ai_inspector_open;
        }
        AiChatAction::Close => {
            ws.show_ai_chat = false;
        }
    }
}
