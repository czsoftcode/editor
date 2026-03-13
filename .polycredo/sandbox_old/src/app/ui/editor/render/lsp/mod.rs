pub mod completion;
pub mod hover;
pub mod navigation;
pub mod typing;

use self::completion::render_completion_popup;
use self::hover::hover_content_to_string;
use crate::app::ui::editor::Editor;
use eframe::egui;

impl Editor {
    /// Handles all LSP interactions: hover, go-to-definition (F12), completion (Ctrl+Space).
    #[allow(clippy::too_many_arguments)]
    pub fn process_lsp_interactions(
        &mut self,
        ui: &egui::Ui,
        _idx: usize,
        tab_path: &std::path::Path,
        lsp_client: Option<&crate::app::lsp::LspClient>,
        saved_response: &Option<egui::text_edit::TextEditOutput>,
        i18n: &crate::i18n::I18n,
    ) {
        use crate::app::ui::editor::{LSP_HOVER_DEBOUNCE_MS, LspCompletionState, LspHoverPopup};

        let tab_idx = _idx;
        let tab_lsp_version = self.tabs[tab_idx].lsp_version;
        let tab_is_binary = self.tabs[tab_idx].is_binary;

        // --- Process pending async results ---

        // Hover result
        if let Some(rx) = &self.lsp_hover_rx
            && let Ok(result) = rx.try_recv()
        {
            self.lsp_hover_rx = None;
            if let Some(hover) = result {
                let content = hover_content_to_string(&hover);
                if !content.trim().is_empty() {
                    self.lsp_hover_popup = Some(LspHoverPopup {
                        content,
                        screen_pos: self.lsp_hover_screen_pos.unwrap_or_default(),
                    });
                }
            }
        }

        // Go-to-definition result
        if let Some(rx) = &self.lsp_definition_rx
            && let Ok(result) = rx.try_recv()
        {
            self.lsp_definition_rx = None;
            if let Some(def_resp) = result {
                let location = match def_resp {
                    async_lsp::lsp_types::GotoDefinitionResponse::Scalar(loc) => Some(loc),
                    async_lsp::lsp_types::GotoDefinitionResponse::Array(locs) => {
                        locs.into_iter().next()
                    }
                    async_lsp::lsp_types::GotoDefinitionResponse::Link(links) => links
                        .into_iter()
                        .next()
                        .map(|l| async_lsp::lsp_types::Location {
                            uri: l.target_uri,
                            range: l.target_range,
                        }),
                };
                if let Some(loc) = location
                    && let Ok(path) = loc.uri.to_file_path()
                {
                    let line = loc.range.start.line as usize + 1;
                    let col = loc.range.start.character as usize + 1;
                    self.pending_lsp_navigate = Some((path, line, col));
                }
            }
        }

        // Completion result
        if let Some(rx) = &self.lsp_completion_rx
            && let Ok(result) = rx.try_recv()
        {
            self.lsp_completion_rx = None;
            let items: Vec<_> = match result {
                Some(async_lsp::lsp_types::CompletionResponse::Array(items)) => items,
                Some(async_lsp::lsp_types::CompletionResponse::List(list)) => list.items,
                None => Vec::new(),
            };
            let items: Vec<_> = items.into_iter().take(25).collect();
            if !items.is_empty() {
                self.lsp_completion = Some(LspCompletionState {
                    items,
                    selected: 0,
                    screen_pos: self.lsp_completion_cursor_pos.unwrap_or_default(),
                });
            }
        }

        // References result
        if let Some(rx) = &self.lsp_references_rx
            && let Ok(result) = rx.try_recv()
        {
            self.lsp_references_rx = None;
            if let Some(locations) = result {
                if locations.is_empty() {
                    self.lsp_status =
                        Some((i18n.get("lsp-references-none"), std::time::Instant::now()));
                } else if locations.len() == 1 {
                    let loc = &locations[0];
                    if let Ok(path) = loc.uri.to_file_path() {
                        let line = loc.range.start.line as usize + 1;
                        let col = loc.range.start.character as usize + 1;
                        self.pending_lsp_navigate = Some((path, line, col));
                        self.lsp_status =
                            Some((i18n.get("lsp-references-found"), std::time::Instant::now()));
                    }
                } else {
                    let mut items = Vec::new();
                    for loc in locations {
                        if let Ok(path) = loc.uri.to_file_path() {
                            let line_idx = loc.range.start.line as usize;
                            let mut line_text = String::new();
                            if let Some(tab) = self.tabs.iter().find(|t| t.path == path) {
                                if let Some(line) = tab.content.lines().nth(line_idx) {
                                    line_text = line.trim().to_string();
                                }
                            } else if let Ok(content) = std::fs::read_to_string(&path)
                                && let Some(line) = content.lines().nth(line_idx)
                            {
                                line_text = line.trim().to_string();
                            }
                            items.push(crate::app::ui::editor::LspReferenceItem {
                                path,
                                line: line_idx + 1,
                                character: loc.range.start.character as usize + 1,
                                text: line_text,
                            });
                        }
                    }
                    let mut args = fluent_bundle::FluentArgs::new();
                    args.set("count", items.len() as i64);
                    let msg = i18n.get_args("lsp-references-found", &args);
                    self.lsp_status = Some((msg, std::time::Instant::now()));

                    self.lsp_references = Some(crate::app::ui::editor::LspReferencesState {
                        items,
                        selected: 0,
                        focus_requested: true,
                        scroll_to_selected: true,
                    });
                }
            } else {
                self.lsp_status =
                    Some((i18n.get("lsp-references-error"), std::time::Instant::now()));
            }
        }

        if tab_is_binary || tab_lsp_version == 0 {
            return;
        }

        let cursor_lsp_pos = saved_response.as_ref().and_then(|r| {
            r.cursor_range.map(|cr| async_lsp::lsp_types::Position {
                line: cr.primary.rcursor.row as u32,
                character: cr.primary.rcursor.column as u32,
            })
        });
        let cursor_screen_pos = saved_response.as_ref().and_then(|r| {
            r.cursor_range
                .map(|cr| r.galley_pos + r.galley.pos_from_cursor(&cr.primary).min.to_vec2())
        });
        let editor_rect = saved_response.as_ref().map(|r| r.response.rect);
        let galley_info = saved_response
            .as_ref()
            .map(|r| (r.galley_pos, r.galley.clone()));

        // F12 — go to definition
        let f12 = ui.ctx().input(|i| i.key_pressed(egui::Key::F12));
        if f12
            && let (Some(lsp), Some(pos)) = (lsp_client, cursor_lsp_pos)
            && let Ok(uri) = async_lsp::lsp_types::Url::from_file_path(tab_path)
        {
            if ui.ctx().input(|i| i.modifiers.shift) {
                self.lsp_references_rx = Some(lsp.request_references(uri, pos));
                self.lsp_status = Some((
                    i18n.get("lsp-references-searching"),
                    std::time::Instant::now(),
                ));
            } else {
                self.lsp_definition_rx = Some(lsp.request_goto_definition(uri, pos));
            }
        }

        // Ctrl+Space — trigger completion
        let ctrl_space = ui
            .ctx()
            .input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Space));
        if ctrl_space
            && let (Some(lsp), Some(pos), Some(screen)) =
                (lsp_client, cursor_lsp_pos, cursor_screen_pos)
            && let Ok(uri) = async_lsp::lsp_types::Url::from_file_path(tab_path)
        {
            self.lsp_completion_cursor_pos = Some(screen);
            self.lsp_completion = None;
            self.lsp_completion_rx = Some(lsp.request_completion(uri, pos));
        }

        // Completion keyboard navigation
        let mut completion_accepted_idx: Option<usize> = None;
        let mut completion_close = false;
        if self.lsp_completion.is_some() {
            let (up, down, enter, tab, esc) = ui.ctx().input_mut(|i| {
                (
                    i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp),
                    i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown),
                    i.consume_key(egui::Modifiers::NONE, egui::Key::Enter),
                    i.consume_key(egui::Modifiers::NONE, egui::Key::Tab),
                    i.consume_key(egui::Modifiers::NONE, egui::Key::Escape),
                )
            });
            if let Some(ref mut comp) = self.lsp_completion {
                if up && comp.selected > 0 {
                    comp.selected -= 1;
                }
                if down && comp.selected + 1 < comp.items.len() {
                    comp.selected += 1;
                }
                if enter || tab {
                    completion_accepted_idx = Some(comp.selected);
                }
                if esc {
                    completion_close = true;
                }
            }
        }

        if let Some(ref comp) = self.lsp_completion
            && let Some(clicked_idx) = render_completion_popup(ui, comp)
        {
            completion_accepted_idx = Some(clicked_idx);
        }

        if let Some(accept_idx) = completion_accepted_idx {
            let insert_text = self
                .lsp_completion
                .as_ref()
                .and_then(|c| c.items.get(accept_idx))
                .map(|item| {
                    item.insert_text
                        .clone()
                        .unwrap_or_else(|| item.label.clone())
                });

            if let Some(insert_text) = insert_text {
                let cursor_char_idx = saved_response
                    .as_ref()
                    .and_then(|r| r.cursor_range)
                    .map(|cr| cr.primary.ccursor.index)
                    .unwrap_or(0);
                let content = &self.tabs[tab_idx].content;
                let word_start = content
                    .char_indices()
                    .take(cursor_char_idx)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .take_while(|(_, c)| c.is_alphanumeric() || *c == '_')
                    .last()
                    .map(|(i, _)| content[..i].chars().count())
                    .unwrap_or(cursor_char_idx);
                let byte_start = content
                    .char_indices()
                    .nth(word_start)
                    .map(|(i, _)| i)
                    .unwrap_or(content.len());
                let byte_end = content
                    .char_indices()
                    .nth(cursor_char_idx)
                    .map(|(i, _)| i)
                    .unwrap_or(content.len());
                {
                    let tab = &mut self.tabs[tab_idx];
                    tab.content
                        .replace_range(byte_start..byte_end, &insert_text);
                    tab.modified = true;
                    tab.last_edit = Some(std::time::Instant::now());
                    tab.save_status = crate::app::ui::editor::SaveStatus::Modified;
                }
                let new_cursor_char = word_start + insert_text.chars().count();
                let edit_id = egui::Id::new("editor_text").with(tab_path);
                let mut state =
                    egui::text_edit::TextEditState::load(ui.ctx(), edit_id).unwrap_or_default();
                state
                    .cursor
                    .set_char_range(Some(egui::text::CCursorRange::one(
                        egui::text::CCursor::new(new_cursor_char),
                    )));
                state.store(ui.ctx(), edit_id);
            }
            self.lsp_completion = None;
        }
        if completion_close {
            self.lsp_completion = None;
        }

        // Hover
        if let (Some((galley_pos, galley)), Some(rect)) = (galley_info, editor_rect) {
            let ptr_pos = ui.ctx().pointer_hover_pos();
            if let Some(pos) = ptr_pos {
                if rect.contains(pos) {
                    let rel = pos - galley_pos;
                    let cursor = galley.cursor_from_pos(rel);
                    let lsp_pos = async_lsp::lsp_types::Position {
                        line: cursor.rcursor.row as u32,
                        character: cursor.rcursor.column as u32,
                    };
                    if self.lsp_hover_last_pos != Some(lsp_pos) {
                        self.lsp_hover_last_pos = Some(lsp_pos);
                        self.lsp_hover_timer = Some(std::time::Instant::now());
                        self.lsp_hover_popup = None;
                        self.lsp_hover_rx = None;
                        self.lsp_hover_screen_pos = Some(pos);
                    }
                    if let Some(timer) = self.lsp_hover_timer
                        && timer.elapsed().as_millis() >= LSP_HOVER_DEBOUNCE_MS
                        && self.lsp_hover_rx.is_none()
                        && self.lsp_hover_popup.is_none()
                        && let (Some(lsp), Ok(uri)) = (
                            lsp_client,
                            async_lsp::lsp_types::Url::from_file_path(tab_path),
                        )
                    {
                        self.lsp_hover_rx = Some(lsp.request_hover(uri, lsp_pos));
                    }
                } else {
                    self.lsp_hover_popup = None;
                    self.lsp_hover_last_pos = None;
                    self.lsp_hover_timer = None;
                    self.lsp_hover_rx = None;
                }
            }
        }

        if let Some(ref popup) = self.lsp_hover_popup {
            hover::render_hover_popup(ui, &popup.content, popup.screen_pos);
        }

        if self.lsp_references_rx.is_some() {
            let mut show_flag = true;
            use crate::app::ui::widgets::modal::StandardModal;
            let modal =
                StandardModal::new(i18n.get("lsp-references-searching"), "lsp_searching_modal")
                    .with_size(350.0, 150.0);
            modal.show(ui.ctx(), &mut show_flag, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.add_space(8.0);
                        ui.label(i18n.get("lsp-references-searching"));
                    });
                });
                modal.ui_footer_actions(ui, i18n, |f| {
                    if f.close() || f.cancel() {
                        self.lsp_references_rx = None;
                    }
                    None::<()>
                });
            });
        }

        if self.lsp_references.is_some()
            && let Some((path, line, col)) = self.render_references_picker(ui.ctx(), i18n)
        {
            self.pending_lsp_navigate = Some((path, line, col));
            self.lsp_references = None;
        }
    }
}
