use super::AiChatAction;
use super::approval::{render_approval_ui, render_ask_user_ui};
use super::inspector::render_inspector;
use crate::app::types::AppShared;
use crate::app::ui::widgets::ai::AiChatWidget;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use eframe::egui;
use std::sync::{Arc, Mutex};

// ── HEAD ─────────────────────────────────────────────────────────────────────

/// Checks whether the selected AI plugin still needs authorization and triggers
/// it automatically. Shows nothing visible when no authorization is pending.
pub fn render_head(ui: &mut egui::Ui, ws: &mut WorkspaceState, shared: &Arc<Mutex<AppShared>>) {
    let pending_auth = {
        let sh = shared.lock().expect("lock");
        sh.registry
            .plugins
            .get_pending_authorizations()
            .into_iter()
            .find(|(id, _)| id == &ws.ai.settings.selected_provider)
    };

    if let Some((id, _meta)) = pending_auth {
        let (plugin_manager, config) = {
            let sh = shared.lock().expect("lock");
            (
                Arc::clone(&sh.registry.plugins),
                sh.settings
                    .plugins
                    .get(&id)
                    .map(|s| s.config.clone())
                    .unwrap_or_default(),
            )
        };
        if ws.plugin_error.is_none() {
            if let Err(e) = plugin_manager.authorize(&id, &config) {
                ws.plugin_error = Some(format!("Auto-authorization failed: {}", e));
            }
        }
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(100));
    }
}

// ── BODY ─────────────────────────────────────────────────────────────────────

/// Renders the main chat area. When the inspector is open the body splits
/// horizontally: chat on the left, inspector on the right.
pub fn render_body(
    ui: &mut egui::Ui,
    ws: &mut WorkspaceState,
    i18n: &I18n,
    prompt: &mut String,
    font_size: f32,
    loading: bool,
    body_h: f32,
) -> Option<AiChatAction> {
    if ws.ai.inspector_open {
        let mut action = None;
        ui.horizontal_top(|ui| {
            let left_w = (ui.available_width() * 0.55).max(400.0);
            ui.allocate_ui_with_layout(
                egui::vec2(left_w, body_h),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    action = render_chat_content(ui, ws, i18n, prompt, font_size, loading, body_h);
                },
            );
            ui.separator();
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), body_h),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    render_inspector(ui, ws, font_size);
                },
            );
        });
        action
    } else {
        render_chat_content(ui, ws, i18n, prompt, font_size, loading, body_h)
    }
}

/// Renders the chat column.
///
/// Terminal-style layout:
///   [history — grows with content, max = body_h - prompt_reserved]
///   [separator + info bar + prompt — pinned just below history]
///   [gap — fills remaining body_h; shrinks as history grows]
///
/// The gap is computed from the history content height stored in ui.memory
/// from the previous frame (one-frame lag, imperceptible).  When content
/// exceeds history_h_max the scrollbar appears and the gap is zero.
fn render_chat_content(
    ui: &mut egui::Ui,
    ws: &mut WorkspaceState,
    i18n: &I18n,
    prompt: &mut String,
    font_size: f32,
    loading: bool,
    body_h: f32,
) -> Option<AiChatAction> {
    let viewer_bg = ui.visuals().extreme_bg_color;
    let prompt_bg = ui.visuals().faint_bg_color;
    let mut action = None;

    ui.spacing_mut().item_spacing.y = 0.0;

    // Heights from previous frame stored in ui.memory
    let prompt_mem_id = egui::Id::new("ai_prompt_frame_h");
    let history_mem_id = egui::Id::new("ai_history_content_h");

    let prompt_h_prev = ui
        .memory(|m| m.data.get_temp::<f32>(prompt_mem_id))
        .unwrap_or(font_size * 2.0 + 16.0);
    let history_content_h_prev = ui
        .memory(|m| m.data.get_temp::<f32>(history_mem_id))
        .unwrap_or(0.0);

    let info_bar_h = 30.0;
    let sep_h = 14.0;
    let settings_h = if ws.ai.settings.show_settings { 280.0 } else { 0.0 };
    let reserved_h = prompt_h_prev + info_bar_h + sep_h + settings_h;

    // Maximum height before a scrollbar is needed
    let history_h_max = (body_h - reserved_h).max(60.0);
    // Actual rendered height of the history area (capped at max)
    let history_display_h = history_content_h_prev.min(history_h_max).max(0.0);
    // Gap between prompt bottom and footer
    let gap_h = (body_h - history_display_h - reserved_h).max(0.0);

    // ── CONVERSATION HISTORY ──────────────────────────────────────────────────
    let scroll_out = egui::ScrollArea::both()
        .id_salt("ai_chat_terminal_history")
        .auto_shrink([false, false])
        .max_height(history_display_h.max(1.0))
        .stick_to_bottom(ws.ai.chat.auto_scroll)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 8.0;

            if !ws.ai.chat.conversation.is_empty() {
                AiChatWidget::ui_conversation(
                    ui,
                    &ws.ai.chat.conversation,
                    font_size,
                    &mut ws.ai.markdown_cache,
                    &ws.ai.ollama.selected_model,
                    ws.ai.chat.out_tokens,
                    ws.ai.chat.loading,
                );
                if !ws.ai.chat.monologue.is_empty() {
                    ui.add_space(8.0);
                    AiChatWidget::ui_monologue(
                        ui,
                        &ws.ai.chat.monologue,
                        font_size,
                        &mut ws.ai.markdown_cache,
                    );
                }
            } else if loading {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(egui::RichText::new(i18n.get("ai-chat-loading")).strong());
                });
                ui.add_space(4.0);
                AiChatWidget::ui_monologue(ui, &ws.ai.chat.monologue, font_size, &mut ws.ai.markdown_cache);
            }
        });

    // Store actual content height for the next frame
    ui.memory_mut(|m| {
        m.data
            .insert_temp(history_mem_id, scroll_out.content_size.y)
    });

    // Auto-scroll detection: stop auto-scroll when user scrolls up during streaming
    if ws.ai.chat.loading {
        let at_bottom = scroll_out.state.offset.y
            >= scroll_out.content_size.y - scroll_out.inner_rect.height() - 30.0;
        if !at_bottom {
            ws.ai.chat.auto_scroll = false;
        }
    }

    // Scroll-to-bottom button when auto-scroll is disabled during streaming
    if !ws.ai.chat.auto_scroll && ws.ai.chat.loading {
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("\u{2193} Scroll to bottom").clicked() {
                    ws.ai.chat.auto_scroll = true;
                }
            });
        });
    }

    // ── APPROVAL UI  /  ASK USER  /  PROMPT ──────────────────────────────────
    if let Some((id, action_name, details, sender)) = ws.pending_plugin_approval.take() {
        render_approval_ui(ui, id, action_name, details, sender, ws);
    } else if let Some((id, question, options, mut input_buf, sender)) = ws.pending_ask_user.take()
    {
        render_ask_user_ui(ui, id, question, options, &mut input_buf, sender, ws);
    } else {
        ui.add_space(4.0);
        ui.scope(|ui| {
            let sep_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
            ui.visuals_mut().widgets.noninteractive.bg_stroke =
                egui::Stroke::new(1.0, sep_color);
            ui.separator();
        });

        // Info bar
        ui.add_space(4.0);
        egui::Frame::new()
            .fill(viewer_bg)
            .inner_margin(egui::Margin::symmetric(8, 2))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                render_info_bar(ui, ws, loading);
            });

        ui.add_space(4.0);

        // Prompt — measure height, store for next frame
        let text_color = ui.visuals().text_color();
        let prompt_resp = egui::Frame::new()
            .fill(prompt_bg)
            .inner_margin(egui::Margin::symmetric(8, 2))
            .corner_radius(4.0)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                let visuals = ui.visuals_mut();
                visuals.override_text_color = Some(text_color);
                visuals.extreme_bg_color = prompt_bg;
                visuals.selection.stroke = egui::Stroke::NONE;
                visuals.widgets.hovered.expansion = 0.0;
                visuals.widgets.active.expansion = 0.0;

                // Stop/Escape handler during streaming
                if loading && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    stop_streaming(ws);
                }

                let max_prompt_h = font_size * 1.6 * 5.0 + font_size;
                ui.horizontal(|ui| {
                    let input_result = egui::ScrollArea::vertical()
                        .id_salt("ai_prompt_scroll")
                        .max_height(max_prompt_h)
                        .auto_shrink([false, true])
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            AiChatWidget::ui_input(
                                ui,
                                prompt,
                                font_size,
                                &i18n.get("ai-chat-placeholder-prompt"),
                                &ws.ai.chat.history,
                                &mut ws.ai.chat.history_index,
                            )
                        })
                        .inner;

                    // Stop/Send toggle button
                    if loading {
                        let stop_color = ui.visuals().error_fg_color;
                        if ui.button(egui::RichText::new("Stop").color(stop_color)).clicked() {
                            stop_streaming(ws);
                        }
                    }

                    input_result
                })
                .inner
            });

        ui.memory_mut(|m| {
            m.data
                .insert_temp(prompt_mem_id, prompt_resp.response.rect.height())
        });

        let (send_via_kb, resp) = prompt_resp.inner;
        if ws.ai.chat.focus_requested {
            resp.request_focus();
            ws.ai.chat.focus_requested = false;
        }
        if send_via_kb {
            action = Some(AiChatAction::Send);
        }
    }

    // ── SETTINGS PANEL ────────────────────────────────────────────────────────
    if ws.ai.settings.show_settings {
        ui.add_space(8.0);
        let settings_fill = ui.visuals().faint_bg_color;
        let settings_stroke_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
        egui::Frame::new()
            .fill(settings_fill)
            .inner_margin(12.0)
            .corner_radius(4.0)
            .stroke(egui::Stroke::new(1.0, settings_stroke_color))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                AiChatWidget::ui_settings(
                    ui,
                    &mut ws.ai.settings.expertise,
                    &mut ws.ai.settings.reasoning_depth,
                    &mut ws.ai.settings.language,
                    &mut ws.ai.chat.system_prompt,
                    i18n,
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(format!("✔ {}", i18n.get("btn-save"))).clicked() {
                        action = Some(AiChatAction::SaveSettings);
                    }
                    if ui.button(i18n.get("btn-close")).clicked() {
                        ws.ai.settings.show_settings = false;
                    }
                });
            });
    }

    // ── GAP — fills remaining space so the window stays full-height ───────────
    if gap_h > 0.0 {
        ui.add_space(gap_h);
    }

    action
}

// ── FOOTER ────────────────────────────────────────────────────────────────────

/// Renders the window footer bar: settings toggle, inspector toggle,
/// new-query and close buttons.
pub fn render_footer(
    ui: &mut egui::Ui,
    ws: &mut WorkspaceState,
    i18n: &I18n,
) -> Option<AiChatAction> {
    let mut action = None;

    if ui
        .selectable_label(ws.ai.settings.show_settings, i18n.get("ai-chat-settings-title"))
        .clicked()
    {
        ws.ai.settings.show_settings = !ws.ai.settings.show_settings;
    }

    let inspector_label = if ws.ai.inspector_open {
        "\u{1F50D} Hide Inspector"
    } else {
        "\u{1F50D} Show Inspector"
    };
    if ui
        .selectable_label(ws.ai.inspector_open, inspector_label)
        .clicked()
    {
        action = Some(AiChatAction::ToggleInspector);
    }

    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if ui.button(i18n.get("btn-close")).clicked() {
            action = Some(AiChatAction::Close);
            return;
        }
        if (!ws.ai.chat.conversation.is_empty() || ws.ai.chat.response.is_some())
            && ui.button(i18n.get("ai-chat-btn-new")).clicked()
        {
            action = Some(AiChatAction::NewQuery);
        }
    });

    action
}

// ── HELPERS ───────────────────────────────────────────────────────────────────

fn render_info_bar(ui: &mut egui::Ui, ws: &WorkspaceState, loading: bool) {
    let weak_color = ui.visuals().weak_text_color();
    ui.horizontal(|ui| {
        if loading {
            ui.spinner();
            ui.label(
                egui::RichText::new("Generating...")
                    .color(weak_color)
                    .small(),
            );
        } else {
            ui.label("\u{1F4C1}");
            ui.label(
                egui::RichText::new(ws.root_path.to_string_lossy())
                    .color(weak_color),
            );
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!(
                    "In: {} | Out: {}",
                    ws.ai.chat.in_tokens, ws.ai.chat.out_tokens
                ))
                .color(weak_color),
            );
        });
    });
}

/// Stops an in-progress streaming response.
fn stop_streaming(ws: &mut WorkspaceState) {
    ws.ai.cancellation_token
        .store(true, std::sync::atomic::Ordering::Relaxed);
    // Preserve partial response with interruption marker
    if !ws.ai.chat.streaming_buffer.is_empty() {
        if let Some(last) = ws.ai.chat.conversation.last_mut() {
            last.1 = format!("{}\n\n*[preruseno]*", ws.ai.chat.streaming_buffer);
        }
    }
    ws.ai.chat.streaming_buffer.clear();
    ws.ai.chat.loading = false;
    ws.ai.chat.stream_rx = None;
    ws.ai.chat.auto_scroll = true;
}
