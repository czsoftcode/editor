pub mod conversation;
pub mod input;
pub mod render;
pub mod settings;

use crate::app::ai::{AiExpertiseRole, AiReasoningDepth};
use eframe::egui;

/// A unified UI widget for AI chat interfaces.
pub struct AiChatWidget;

impl AiChatWidget {
    /// Renders a multiline text edit with CLI-like behavior.
    pub fn ui_input(
        ui: &mut egui::Ui,
        text: &mut String,
        font_size: f32,
        hint: &str,
        history: &[String],
        history_index: &mut Option<usize>,
    ) -> (bool, egui::Response) {
        input::ui_input(ui, text, font_size, hint, history, history_index)
    }

    /// Renders AI conversation history in a terminal-like style.
    pub fn ui_conversation(
        ui: &mut egui::Ui,
        conversation: &[(String, String)],
        font_size: f32,
        cache: &mut egui_commonmark::CommonMarkCache,
    ) {
        conversation::ui_conversation(ui, conversation, font_size, cache)
    }

    /// Renders the real-time "thinking" monologue.
    pub fn ui_monologue(
        ui: &mut egui::Ui,
        monologue: &[String],
        font_size: f32,
        cache: &mut egui_commonmark::CommonMarkCache,
    ) {
        render::ui_monologue(ui, monologue, font_size, cache)
    }

    /// Renders settings for an AI agent (Rank, Depth, Language).
    pub fn ui_settings(
        ui: &mut egui::Ui,
        expertise: &mut AiExpertiseRole,
        depth: &mut AiReasoningDepth,
        language: &mut String,
        system_prompt: &mut String,
        i18n: &crate::i18n::I18n,
    ) -> bool {
        settings::ui_settings(ui, expertise, depth, language, system_prompt, i18n)
    }
}
