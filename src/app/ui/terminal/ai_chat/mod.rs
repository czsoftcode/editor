pub mod approval;
pub mod gsd;
pub mod inspector;
pub mod logic;
pub mod render;
pub mod slash;

use crate::app::ai_core::AiManager;
use crate::app::types::{AppShared, FocusedPanel};
use crate::app::ui::terminal::StandardTerminalWindow;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::config;
use crate::i18n::I18n;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub enum AiChatAction {
    Send,
    Retry,
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

    let mut prompt = ws.ai.chat.prompt.clone();
    let loading = ws.ai.chat.loading;
    let mut show_flag = ws.show_ai_chat;
    let font_size = shared.lock().expect("lock").settings.editor_font_size;

    let win = StandardTerminalWindow::new(
        i18n.get("cli-chat-title"),
        "ai_chat_terminal_win",
        FocusedPanel::AiChat,
    );

    let (interacted, action) = win.show(
        ctx,
        ws,
        &mut show_flag,
        |ui, ws_arg| {
            render::render_head(ui, ws_arg, shared, i18n);
        },
        |ui, ws_arg, body_h| {
            render::render_body(ui, ws_arg, i18n, &mut prompt, font_size, loading, body_h)
        },
        |ui, ws_arg| render::render_footer(ui, ws_arg, i18n),
    );

    ws.ai.chat.prompt = prompt;
    ws.show_ai_chat = show_flag;

    if let Some(act) = action {
        handle_action(act, ws, shared, i18n);
    }

    interacted
}

pub fn handle_action(
    act: AiChatAction,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &I18n,
) {
    match act {
        AiChatAction::Send => {
            logic::send_query_to_agent(ws, shared, i18n);
        }
        AiChatAction::Retry => {
            logic::retry_last_prompt(ws, shared, i18n);
        }
        AiChatAction::NewQuery => {
            let model = {
                let sh = shared.lock().expect("lock");
                if !sh.settings.ai_default_model.is_empty() {
                    sh.settings.ai_default_model.clone()
                } else {
                    "llama3.1".to_string()
                }
            };
            ws.ai.chat.response = None;
            ws.ai.chat.prompt.clear();
            ws.ai.chat.conversation.clear();
            ws.ai.chat.monologue.clear();
            ws.ai.chat.in_tokens = 0;
            ws.ai.chat.out_tokens = 0;
            ws.ai.chat.retry_available = false;
            ws.ai.chat.conversation.push((
                String::new(),
                AiManager::get_logo(
                    config::CLI_VERSION,
                    &model,
                    ws.ai.settings.expertise,
                    ws.ai.settings.reasoning_depth,
                ),
            ));
        }
        AiChatAction::SaveSettings => {
            let mut sh = shared.lock().expect("lock");
            let mut settings = (*sh.settings).clone();
            settings.ai_expertise = ws.ai.settings.expertise;
            settings.ai_reasoning_depth = ws.ai.settings.reasoning_depth;
            settings.save();
            sh.settings = Arc::new(settings);
            ws.toasts
                .push(crate::app::types::Toast::info("AI settings saved."));
        }
        AiChatAction::ToggleInspector => {
            ws.ai.inspector_open = !ws.ai.inspector_open;
        }
        AiChatAction::Close => {
            ws.show_ai_chat = false;
        }
    }
}
