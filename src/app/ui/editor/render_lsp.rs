use super::*;
use eframe::egui;

impl Editor {
    // -----------------------------------------------------------------------
    // LSP interaction logic
    // -----------------------------------------------------------------------

    /// Handles all LSP interactions: hover, go-to-definition (F12), completion (Ctrl+Space).
    /// Called after the TextEdit frame is rendered so self can be freely borrowed.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn process_lsp_interactions(
        &mut self,
        ui: &egui::Ui,
        _idx: usize,
        tab_path: &std::path::Path,
        lsp_client: Option<&crate::app::lsp::LspClient>,
        saved_response: &Option<egui::text_edit::TextEditOutput>,
        i18n: &crate::i18n::I18n,
    ) {
        use super::LSP_HOVER_DEBOUNCE_MS;
        use super::LspCompletionState;
        use super::LspHoverPopup;

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
                    let line = loc.range.start.line as usize + 1; // → 1-based
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
                    // Single result -> jump directly
                    let loc = &locations[0];
                    if let Ok(path) = loc.uri.to_file_path() {
                        let line = loc.range.start.line as usize + 1;
                        let col = loc.range.start.character as usize + 1;
                        self.pending_lsp_navigate = Some((path, line, col));
                        self.lsp_status =
                            Some((i18n.get("lsp-references-found"), std::time::Instant::now()));
                    }
                } else {
                    // Multiple results -> show picker
                    let mut items = Vec::new();
                    for loc in locations {
                        if let Ok(path) = loc.uri.to_file_path() {
                            let line_idx = loc.range.start.line as usize;
                            let mut line_text = String::new();

                            // Try to get line text from open tabs first
                            if let Some(tab) = self.tabs.iter().find(|t| t.path == path) {
                                if let Some(line) = tab.content.lines().nth(line_idx) {
                                    line_text = line.trim().to_string();
                                }
                            } else {
                                // Fallback to reading from disk
                                if let Ok(content) = std::fs::read_to_string(&path)
                                    && let Some(line) = content.lines().nth(line_idx)
                                {
                                    line_text = line.trim().to_string();
                                }
                            }

                            items.push(super::LspReferenceItem {
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

                    self.lsp_references = Some(super::LspReferencesState {
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

        // --- Skip LSP interactions for binary files or when LSP not ready ---
        if tab_is_binary || tab_lsp_version == 0 {
            return;
        }

        // --- Pre-compute cursor/galley data from saved response ---
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

        // --- Keyboard: F12 — go to definition ---
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

        // --- Keyboard: Ctrl+Space — trigger completion ---
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

        // --- Completion keyboard navigation (when popup is open) ---
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

        // --- Accept completion item ---
        // --- Render completion popup (before acceptance so click can set accept_idx) ---
        if let Some(ref comp) = self.lsp_completion {
            let comp_screen_pos = comp.screen_pos;
            let comp_selected = comp.selected;
            let comp_items = comp.items.clone();
            let mock_comp = LspCompletionState {
                items: comp_items,
                selected: comp_selected,
                screen_pos: comp_screen_pos,
            };
            if let Some(clicked_idx) = Self::render_completion_popup(ui, &mock_comp) {
                // Click overrides keyboard selection
                completion_accepted_idx = Some(clicked_idx);
            }
        }

        // --- Accept completion item (keyboard Enter/Tab or mouse click) ---
        if let Some(accept_idx) = completion_accepted_idx {
            // Clone item data to avoid borrow conflict
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
                // Find word start: walk back while alphanumeric or '_'
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
                    tab.save_status = super::SaveStatus::Modified;
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

        // --- Mouse hover for documentation ---
        if let (Some((galley_pos, galley)), Some(rect)) = (galley_info, editor_rect) {
            let ptr_pos = ui.ctx().pointer_hover_pos();
            if let Some(pos) = ptr_pos {
                if rect.contains(pos) {
                    // Convert mouse pos to LSP position via galley
                    let rel = pos - galley_pos;
                    let cursor = galley.cursor_from_pos(rel);
                    let lsp_pos = async_lsp::lsp_types::Position {
                        line: cursor.rcursor.row as u32,
                        character: cursor.rcursor.column as u32,
                    };
                    let pos_changed = self.lsp_hover_last_pos != Some(lsp_pos);
                    if pos_changed {
                        self.lsp_hover_last_pos = Some(lsp_pos);
                        self.lsp_hover_timer = Some(std::time::Instant::now());
                        self.lsp_hover_popup = None;
                        self.lsp_hover_rx = None;
                        self.lsp_hover_screen_pos = Some(pos);
                    }
                    // Debounce: trigger after LSP_HOVER_DEBOUNCE_MS of no movement
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
                    // Mouse left the editor area — dismiss popup
                    self.lsp_hover_popup = None;
                    self.lsp_hover_last_pos = None;
                    self.lsp_hover_timer = None;
                    self.lsp_hover_rx = None;
                }
            }
        }

        // --- Render hover popup ---
        if let Some(ref popup) = self.lsp_hover_popup {
            let content = popup.content.clone();
            let screen_pos = popup.screen_pos;
            Self::render_hover_popup(ui, &content, screen_pos);
        }

        // --- Render searching indicator ---
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
            });
        }

        // --- Render references picker ---
        if self.lsp_references.is_some()
            && let Some((path, line, col)) = self.render_references_picker(ui.ctx(), i18n)
        {
            self.pending_lsp_navigate = Some((path, line, col));
            self.lsp_references = None;
        }
    }

    // -----------------------------------------------------------------------
    // LSP popup rendering
    // -----------------------------------------------------------------------

    /// Renders a modal-like picker for multiple LSP references.
    fn render_references_picker(
        &mut self,
        ctx: &egui::Context,
        i18n: &crate::i18n::I18n,
    ) -> Option<(std::path::PathBuf, usize, usize)> {
        let picker = self.lsp_references.as_mut()?;

        let key_up = ctx.input(|i| i.key_pressed(egui::Key::ArrowUp));
        let key_down = ctx.input(|i| i.key_pressed(egui::Key::ArrowDown));
        let key_enter = ctx.input(|i| i.key_pressed(egui::Key::Enter));
        let key_esc = ctx.input(|i| i.key_pressed(egui::Key::Escape));

        if key_up && picker.selected > 0 {
            picker.selected -= 1;
            picker.scroll_to_selected = true;
        }
        if key_down && picker.selected + 1 < picker.items.len() {
            picker.selected += 1;
            picker.scroll_to_selected = true;
        }

        let mut result = None;
        let mut show_flag = true;
        let mut close_requested = false;

        if key_enter && !picker.items.is_empty() {
            let item = &picker.items[picker.selected];
            result = Some((item.path.clone(), item.line, item.character));
            close_requested = true;
        }

        use crate::app::ui::widgets::modal::StandardModal;
        let modal = StandardModal::new(i18n.get("lsp-references-heading"), "lsp_references_modal")
            .with_size(700.0, 500.0);

        modal.show(ctx, &mut show_flag, |ui| {
            // FOOTER
            if let Some(close) = modal.ui_footer(ui, |ui| {
                if ui.button(i18n.get("btn-close")).clicked() {
                    return Some(true);
                }
                None
            }) && close
            {
                close_requested = true;
            }

            // BODY
            modal.ui_body(ui, |ui| {
                if picker.focus_requested {
                    ui.memory_mut(|m| m.request_focus(ui.id()));
                    picker.focus_requested = false;
                }
                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .id_salt("lsp_ref_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        for (i, item) in picker.items.iter().enumerate() {
                            let is_sel = i == picker.selected;
                            let filename = item
                                .path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "???".to_string());

                            let display_text = if item.text.is_empty() {
                                format!("{}:{}:{}", filename, item.line, item.character)
                            } else {
                                let mut truncated = item.text.clone();
                                if truncated.len() > 100 {
                                    truncated.truncate(97);
                                    truncated.push_str("...");
                                }
                                format!(
                                    "{}:{}:{}  {}",
                                    filename, item.line, item.character, truncated
                                )
                            };

                            let text = egui::RichText::new(display_text).monospace().size(12.0);

                            let r = ui.selectable_label(is_sel, text);
                            if is_sel && picker.scroll_to_selected {
                                r.scroll_to_me(None);
                            }
                            if r.clicked() {
                                result = Some((item.path.clone(), item.line, item.character));
                                close_requested = true;
                            }
                            if ui.is_rect_visible(r.rect) {
                                ui.label(
                                    egui::RichText::new(item.path.to_string_lossy())
                                        .weak()
                                        .size(10.0),
                                );
                            }
                            ui.separator();
                        }
                    });
            });
        });

        // Reset scroll flag after rendering the selected item
        if let Some(p) = self.lsp_references.as_mut()
            && p.scroll_to_selected
        {
            p.scroll_to_selected = false;
        }

        if close_requested || key_esc || !show_flag {
            self.lsp_references = None;
        }
        result
    }

    /// Renders the hover documentation popup as a floating Area.
    /// Properly parses markdown code fences — code blocks are rendered monospace,
    /// prose sections are rendered as regular text.
    pub(super) fn render_hover_popup(ui: &egui::Ui, content: &str, screen_pos: egui::Pos2) {
        let popup_pos = screen_pos + egui::vec2(8.0, 16.0);
        egui::Area::new(egui::Id::new("lsp_hover_popup"))
            .fixed_pos(popup_pos)
            .order(egui::Order::Tooltip)
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(35, 38, 46))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 80, 100)))
                    .inner_margin(egui::Margin::same(10))
                    .corner_radius(4.0)
                    .show(ui, |ui| {
                        ui.set_max_width(520.0);
                        egui::ScrollArea::vertical()
                            .id_salt("lsp_hover_scroll")
                            .max_height(260.0)
                            .show(ui, |ui| {
                                let segments = parse_hover_segments(content);
                                let mut first = true;
                                for seg in &segments {
                                    if !first {
                                        ui.add_space(6.0);
                                        ui.separator();
                                        ui.add_space(4.0);
                                    }
                                    first = false;
                                    match seg {
                                        HoverSegment::Code(code) => {
                                            let trimmed = code.trim();
                                            if !trimmed.is_empty() {
                                                ui.label(
                                                    egui::RichText::new(trimmed).monospace().color(
                                                        egui::Color32::from_rgb(180, 210, 255),
                                                    ),
                                                );
                                            }
                                        }
                                        HoverSegment::Prose(text) => {
                                            let trimmed = text.trim();
                                            if !trimmed.is_empty() {
                                                ui.label(
                                                    egui::RichText::new(trimmed).color(
                                                        egui::Color32::from_rgb(200, 200, 200),
                                                    ),
                                                );
                                            }
                                        }
                                    }
                                }
                            });
                    });
            });
    }

    /// Renders the completion dropdown and returns true if an item was accepted.
    /// Also handles keyboard navigation (arrow keys consumed pre-frame, but selected
    /// index is already updated by the time this runs).
    pub(super) fn render_completion_popup(
        ui: &egui::Ui,
        state: &super::LspCompletionState,
    ) -> Option<usize> {
        let mut accepted = None;

        let font_size = Self::current_editor_font_size(ui);
        let popup_pos = state.screen_pos + egui::vec2(0.0, font_size + 4.0);

        egui::Area::new(egui::Id::new("lsp_completion_popup"))
            .fixed_pos(popup_pos)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(38, 41, 50))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgb(80, 100, 130),
                    ))
                    .inner_margin(egui::Margin::same(4))
                    .corner_radius(4.0)
                    .show(ui, |ui| {
                        ui.set_max_width(380.0);
                        egui::ScrollArea::vertical()
                            .id_salt("lsp_completion_scroll")
                            .max_height(220.0)
                            .show(ui, |ui| {
                                for (i, item) in state.items.iter().enumerate() {
                                    let is_selected = i == state.selected;
                                    let kind_label = completion_kind_label(item.kind);
                                    let label = format!(
                                        "{} {}",
                                        kind_label,
                                        item.detail
                                            .as_deref()
                                            .map(|d| format!("{} — {}", item.label, d))
                                            .unwrap_or_else(|| item.label.clone()),
                                    );
                                    let text = egui::RichText::new(&label)
                                        .monospace()
                                        .size(font_size - 1.0);
                                    let response = ui.selectable_label(is_selected, text);
                                    if is_selected {
                                        response.scroll_to_me(None);
                                    }
                                    if response.clicked() {
                                        accepted = Some(i);
                                    }
                                }
                            });
                    });
            });

        accepted
    }

    /// Handles smart typing: auto-indent on Enter and smart un-indent on '}'.
    pub(super) fn handle_smart_typing(&mut self, ui: &mut egui::Ui, edit_id: egui::Id, idx: usize) {
        if !ui.memory(|m| m.has_focus(edit_id))
            || self.lsp_completion.is_some()
            || self.lsp_references.is_some()
        {
            return;
        }

        let mut typed_brace = false;
        let enter = ui.input_mut(|i| {
            // Check for '}' text event
            i.events.retain(|e| {
                if let egui::Event::Text(t) = e
                    && t == "}"
                {
                    typed_brace = true;
                    return false; // consume
                }
                true
            });
            i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
        });

        if !enter && !typed_brace {
            return;
        }

        let tab = &mut self.tabs[idx];
        let mut state = egui::text_edit::TextEditState::load(ui.ctx(), edit_id).unwrap_or_default();

        if let Some(range) = state.cursor.char_range() {
            let cursor_char_idx = range.primary.index;
            let byte_idx = tab
                .content
                .char_indices()
                .nth(cursor_char_idx)
                .map(|(i, _)| i)
                .unwrap_or(tab.content.len());

            if enter {
                let prefix = &tab.content[..byte_idx];
                let line_start_byte_idx = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
                let current_line = &tab.content[line_start_byte_idx..byte_idx];

                let whitespace: String = current_line
                    .chars()
                    .take_while(|c| c.is_whitespace() && *c != '\n')
                    .collect();

                let mut to_insert = format!("\n{}", whitespace);
                if current_line.trim_end().ends_with('{') {
                    to_insert.push_str("    ");
                }

                tab.content.insert_str(byte_idx, &to_insert);
                let new_cursor_char_idx = cursor_char_idx + to_insert.chars().count();
                state
                    .cursor
                    .set_char_range(Some(egui::text::CCursorRange::one(
                        egui::text::CCursor::new(new_cursor_char_idx),
                    )));
            } else if typed_brace {
                let prefix = &tab.content[..byte_idx];
                let line_start_byte_idx = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_to_cursor = &tab.content[line_start_byte_idx..byte_idx];

                // If line is only whitespace and has at least 4 spaces, un-indent
                if line_to_cursor.chars().all(|c| c.is_whitespace())
                    && line_to_cursor.ends_with("    ")
                {
                    tab.content.replace_range(byte_idx - 4..byte_idx, "}");
                    let new_cursor_char_idx = cursor_char_idx - 3; // -4 spaces + 1 brace
                    state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::one(
                            egui::text::CCursor::new(new_cursor_char_idx),
                        )));
                } else {
                    tab.content.insert(byte_idx, '}');
                    let new_cursor_char_idx = cursor_char_idx + 1;
                    state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::one(
                            egui::text::CCursor::new(new_cursor_char_idx),
                        )));
                }
            }

            tab.modified = true;
            tab.last_edit = Some(std::time::Instant::now());
            tab.save_status = super::SaveStatus::Modified;
            state.store(ui.ctx(), edit_id);
        }
    }
}

// ---------------------------------------------------------------------------
// Markdown hover parsing helpers
// ---------------------------------------------------------------------------

/// A segment of parsed markdown hover content.
enum HoverSegment {
    /// Content of a fenced code block (``` ... ```).
    Code(String),
    /// Plain prose text outside code blocks.
    Prose(String),
}

/// Parses markdown content into alternating Code/Prose segments.
/// Properly handles fenced code blocks (```lang ... ```).
fn parse_hover_segments(content: &str) -> Vec<HoverSegment> {
    let mut segments: Vec<HoverSegment> = Vec::new();
    let mut in_fence = false;
    let mut current: Vec<&str> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if in_fence {
                // Close fence — emit code block
                let code = current.join("\n");
                if !code.trim().is_empty() {
                    segments.push(HoverSegment::Code(code));
                }
                current.clear();
                in_fence = false;
            } else {
                // Open fence — flush accumulated prose first
                let prose = current.join("\n");
                if !prose.trim().is_empty() {
                    segments.push(HoverSegment::Prose(prose));
                }
                current.clear();
                in_fence = true;
                // Skip the fence-opening line (it contains the language tag, not code)
            }
        } else {
            current.push(line);
        }
    }

    // Flush remaining content
    if !current.is_empty() {
        let text = current.join("\n");
        if !text.trim().is_empty() {
            if in_fence {
                segments.push(HoverSegment::Code(text));
            } else {
                segments.push(HoverSegment::Prose(text));
            }
        }
    }

    segments
}

/// Converts LSP Hover content to a plain string for display.
pub(super) fn hover_content_to_string(hover: &async_lsp::lsp_types::Hover) -> String {
    match &hover.contents {
        async_lsp::lsp_types::HoverContents::Markup(mc) => mc.value.clone(),
        async_lsp::lsp_types::HoverContents::Scalar(ms) => marked_string_value(ms),
        async_lsp::lsp_types::HoverContents::Array(arr) => arr
            .iter()
            .map(marked_string_value)
            .collect::<Vec<_>>()
            .join("\n\n"),
    }
}

fn marked_string_value(ms: &async_lsp::lsp_types::MarkedString) -> String {
    match ms {
        async_lsp::lsp_types::MarkedString::String(s) => s.clone(),
        async_lsp::lsp_types::MarkedString::LanguageString(ls) => ls.value.clone(),
    }
}

/// Short kind label for a completion item (shown before the label).
fn completion_kind_label(kind: Option<async_lsp::lsp_types::CompletionItemKind>) -> &'static str {
    use async_lsp::lsp_types::CompletionItemKind;
    match kind {
        Some(CompletionItemKind::FUNCTION) | Some(CompletionItemKind::METHOD) => "fn",
        Some(CompletionItemKind::CONSTRUCTOR) => "fn",
        Some(CompletionItemKind::STRUCT) | Some(CompletionItemKind::CLASS) => "st",
        Some(CompletionItemKind::INTERFACE) => "if",
        Some(CompletionItemKind::ENUM) => "en",
        Some(CompletionItemKind::ENUM_MEMBER) => "ev",
        Some(CompletionItemKind::CONSTANT) => "co",
        Some(CompletionItemKind::VARIABLE)
        | Some(CompletionItemKind::FIELD)
        | Some(CompletionItemKind::PROPERTY) => "va",
        Some(CompletionItemKind::KEYWORD) => "kw",
        Some(CompletionItemKind::MODULE) => "mo",
        Some(CompletionItemKind::TYPE_PARAMETER) => "ty",
        Some(CompletionItemKind::SNIPPET) => "sn",
        _ => "  ",
    }
}

#[cfg(test)]
mod tests {
    use super::goto_centered_scroll_offset;

    #[test]
    fn goto_scroll_centers_when_possible() {
        let offset = goto_centered_scroll_offset(50, 200, 20.0, 200.0);
        assert!((offset - 890.0).abs() < f32::EPSILON);
    }

    #[test]
    fn goto_scroll_clamps_near_top() {
        let offset = goto_centered_scroll_offset(1, 200, 20.0, 200.0);
        assert!((offset - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn goto_scroll_clamps_near_bottom() {
        let offset = goto_centered_scroll_offset(200, 200, 20.0, 200.0);
        assert!((offset - 3800.0).abs() < f32::EPSILON);
    }
}
