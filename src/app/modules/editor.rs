use std::path::PathBuf;
use std::time::Instant;

use eframe::egui;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::config;
use crate::highlighter::Highlighter;

const AUTOSAVE_DELAY_MS: u128 = 500;

#[derive(PartialEq)]
pub enum SaveStatus {
    None,
    Modified,
    Saving,
    Saved,
}

enum TabAction {
    Switch(usize),
    Close(usize),
}

struct Tab {
    content: String,
    path: PathBuf,
    modified: bool,
    deleted: bool,
    last_edit: Option<Instant>,
    save_status: SaveStatus,
    last_saved_content: String,
    scroll_offset: f32,
    last_cursor_range: Option<egui::text::CursorRange>,
}

pub struct Editor {
    tabs: Vec<Tab>,
    active_tab: Option<usize>,
    highlighter: Highlighter,
    show_search: bool,
    search_query: String,
    replace_query: String,
    show_replace: bool,
    search_matches: Vec<(usize, usize)>,
    current_match: Option<usize>,
    search_focus_requested: bool,
    md_split_ratio: f32,
    tab_scroll_x: f32,
    scroll_to_active: bool,
    /// Přejít na řádek — čekající skok (1-based číslo řádku)
    pending_jump: Option<usize>,
    /// Dialog "Přejít na řádek" (Ctrl+G)
    show_goto_line: bool,
    goto_line_input: String,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: None,
            highlighter: Highlighter::new(),
            show_search: false,
            search_query: String::new(),
            replace_query: String::new(),
            show_replace: false,
            search_matches: Vec::new(),
            current_match: None,
            search_focus_requested: false,
            md_split_ratio: 0.5,
            tab_scroll_x: 0.0,
            scroll_to_active: false,
            pending_jump: None,
            show_goto_line: false,
            goto_line_input: String::new(),
        }
    }

    // --- Helpers ---

    fn active(&self) -> Option<&Tab> {
        self.active_tab.and_then(|i| self.tabs.get(i))
    }

    fn active_mut(&mut self) -> Option<&mut Tab> {
        self.active_tab.and_then(|i| self.tabs.get_mut(i))
    }

    pub fn active_path(&self) -> Option<&PathBuf> {
        self.active().map(|t| &t.path)
    }

    pub fn is_modified(&self) -> bool {
        self.active().is_some_and(|t| t.modified)
    }

    fn extension(&self) -> String {
        self.active()
            .and_then(|t| t.path.extension())
            .map(|e| e.to_string_lossy().to_string())
            .or_else(|| {
                let name = self.filename();
                if name.starts_with('.') && name.len() > 1 {
                    Some(name[1..].to_string())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    fn filename(&self) -> String {
        self.active()
            .and_then(|t| t.path.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    fn is_markdown(&self) -> bool {
        let ext = self.extension();
        ext == "md" || ext == "markdown"
    }

    // --- Tab management ---

    pub fn open_file(&mut self, path: &PathBuf) {
        // Pokud je soubor už otevřený, přepnout na záložku
        if let Some(idx) = self.tabs.iter().position(|t| t.path == *path) {
            self.active_tab = Some(idx);
            self.update_search();
            return;
        }

        match std::fs::read_to_string(path) {
            Ok(content) => {
                let tab = Tab {
                    last_saved_content: content.clone(),
                    content,
                    path: path.clone(),
                    modified: false,
                    deleted: false,
                    last_edit: None,
                    save_status: SaveStatus::None,
                    scroll_offset: 0.0,
                    last_cursor_range: None,
                };
                self.tabs.push(tab);
                self.active_tab = Some(self.tabs.len() - 1);
            }
            Err(e) => {
                let tab = Tab {
                    content: format!("Chyba při čtení souboru: {}", e),
                    last_saved_content: String::new(),
                    path: path.clone(),
                    modified: false,
                    deleted: false,
                    last_edit: None,
                    save_status: SaveStatus::None,
                    scroll_offset: 0.0,
                    last_cursor_range: None,
                };
                self.tabs.push(tab);
                self.active_tab = Some(self.tabs.len() - 1);
            }
        }
        self.update_search();
        self.scroll_to_active = true;
    }

    pub fn close_tab(&mut self, index: usize) {
        if index >= self.tabs.len() {
            return;
        }
        self.tabs.remove(index);
        if self.tabs.is_empty() {
            self.active_tab = None;
        } else if let Some(active) = self.active_tab {
            if active == index {
                self.active_tab = Some(active.min(self.tabs.len() - 1));
            } else if active > index {
                self.active_tab = Some(active - 1);
            }
        }
        self.update_search();
    }

    pub fn clear(&mut self) {
        if let Some(idx) = self.active_tab {
            self.close_tab(idx);
        }
    }

    pub fn close_tabs_for_path(&mut self, path: &PathBuf) {
        let indices: Vec<usize> = self
            .tabs
            .iter()
            .enumerate()
            .filter(|(_, t)| t.path == *path || t.path.starts_with(path))
            .map(|(i, _)| i)
            .collect();
        for idx in indices.into_iter().rev() {
            self.close_tab(idx);
        }
    }

    /// Označí záložky pro daný soubor jako smazané — zakáže autosave a zobrazí indikátor.
    pub fn notify_file_deleted(&mut self, path: &PathBuf) {
        for tab in &mut self.tabs {
            if tab.path == *path {
                tab.deleted = true;
                tab.modified = false;
                tab.last_edit = None;
                tab.save_status = SaveStatus::None;
            }
        }
    }

    /// Naplánuje přechod kurzoru na daný řádek (1-based). Provede se při příštím renderu.
    pub fn jump_to_line(&mut self, line: usize) {
        self.pending_jump = Some(line.max(1));
    }

    // --- File operations ---

    pub fn reload_from_disk(&mut self) {
        if let Some(tab) = self.active_mut() {
            if let Ok(content) = std::fs::read_to_string(&tab.path) {
                tab.content = content.clone();
                tab.last_saved_content = content;
                tab.modified = false;
                tab.last_edit = None;
                tab.save_status = SaveStatus::Saved;
            }
        }
        self.update_search();
    }

    pub fn try_autosave(&mut self) {
        let should_save = self
            .active()
            .is_some_and(|t| {
                !t.deleted
                    && t.modified
                    && t.last_edit
                        .is_some_and(|e| e.elapsed().as_millis() >= AUTOSAVE_DELAY_MS)
            });
        if should_save {
            self.save();
        }
    }

    pub fn save(&mut self) {
        if let Some(tab) = self.active_mut() {
            tab.save_status = SaveStatus::Saving;
            match std::fs::write(&tab.path, &tab.content) {
                Ok(()) => {
                    tab.last_saved_content = tab.content.clone();
                    tab.modified = false;
                    tab.last_edit = None;
                    tab.save_status = SaveStatus::Saved;
                }
                Err(_) => {
                    tab.save_status = SaveStatus::Modified;
                }
            }
        }
    }

    // --- Search ---

    fn update_search(&mut self) {
        self.search_matches.clear();
        self.current_match = None;

        if !self.show_search || self.search_query.is_empty() {
            return;
        }

        // Zkopírujeme pouze dotaz (malý string) a hledáme přes referenci na obsah souboru,
        // abychom se vyhnuli klonování celého obsahu (potenciálně megabajty dat).
        let query = self.search_query.clone();
        let query_len = query.len();

        let active_idx = match self.active_tab {
            Some(i) => i,
            None => return,
        };
        let content = match self.tabs.get(active_idx) {
            Some(t) => &t.content,
            None => return,
        };

        if query_len > content.len() {
            return;
        }

        let mut matches = Vec::new();
        let mut start = 0;
        while start + query_len <= content.len() {
            if content[start..start + query_len].eq_ignore_ascii_case(&query) {
                matches.push((start, start + query_len));
            }
            start += 1;
            while start < content.len() && !content.is_char_boundary(start) {
                start += 1;
            }
        }

        self.search_matches = matches;
        if !self.search_matches.is_empty() {
            self.current_match = Some(0);
        }
    }

    fn next_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.current_match = Some(match self.current_match {
            Some(i) => (i + 1) % self.search_matches.len(),
            None => 0,
        });
    }

    fn prev_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.current_match = Some(match self.current_match {
            Some(0) | None => self.search_matches.len() - 1,
            Some(i) => i - 1,
        });
    }

    fn replace_current(&mut self) {
        let match_idx = match self.current_match {
            Some(i) => i,
            None => return,
        };
        let (start, end) = match self.search_matches.get(match_idx) {
            Some(&m) => m,
            None => return,
        };
        let replace = self.replace_query.clone();
        if let Some(tab) = self.active_mut() {
            tab.content.replace_range(start..end, &replace);
            tab.modified = true;
            tab.last_edit = Some(Instant::now());
            tab.save_status = SaveStatus::Modified;
        }
        self.update_search();
    }

    fn replace_all(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        let replace = self.replace_query.clone();
        let matches: Vec<(usize, usize)> = self.search_matches.iter().rev().copied().collect();
        if let Some(tab) = self.active_mut() {
            for (start, end) in matches {
                tab.content.replace_range(start..end, &replace);
            }
            tab.modified = true;
            tab.last_edit = Some(Instant::now());
            tab.save_status = SaveStatus::Modified;
        }
        self.update_search();
    }

    // --- UI ---

    /// Vrací `true` pokud uživatel klikl do editoru (pro přepnutí fokusu).
    pub fn ui(&mut self, ui: &mut egui::Ui, dialog_open: bool) -> bool {
        if self.tabs.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("Otevřete soubor z adresářového stromu vlevo");
            });
            return false;
        }

        // Tab bar
        let mut tab_action = None;
        self.tab_bar(ui, &mut tab_action);
        match tab_action {
            Some(TabAction::Switch(idx)) => {
                self.active_tab = Some(idx);
                self.update_search();
            }
            Some(TabAction::Close(idx)) => {
                self.close_tab(idx);
            }
            None => {}
        }

        if self.tabs.is_empty() {
            return false;
        }

        // Klávesové zkratky pro search + goto
        let ctrl_f = ui.ctx().input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::F));
        let ctrl_h = ui.ctx().input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::H));
        let ctrl_g = ui.ctx().input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::G));
        let escape = ui.ctx().input(|i| i.key_pressed(egui::Key::Escape));

        if ctrl_f {
            self.show_search = true;
            self.show_replace = false;
            self.search_focus_requested = true;
            self.show_goto_line = false;
            self.update_search();
        }
        if ctrl_h {
            self.show_search = true;
            self.show_replace = true;
            self.search_focus_requested = true;
            self.show_goto_line = false;
            self.update_search();
        }
        if ctrl_g {
            self.show_goto_line = !self.show_goto_line;
            if self.show_goto_line {
                self.goto_line_input.clear();
                self.show_search = false;
            }
        }
        if escape {
            if self.show_search {
                self.show_search = false;
                self.show_replace = false;
                self.search_matches.clear();
                self.current_match = None;
            } else if self.show_goto_line {
                self.show_goto_line = false;
            }
        }

        if self.show_search {
            self.search_bar(ui);
        }
        if self.show_goto_line {
            self.goto_line_bar(ui);
        }

        if self.is_markdown() {
            self.ui_markdown_split(ui, dialog_open)
        } else {
            self.ui_normal(ui, dialog_open)
        }
    }

    fn tab_bar(&mut self, ui: &mut egui::Ui, action: &mut Option<TabAction>) {
        let btn_w = config::TAB_BTN_WIDTH;
        let initial_scroll = self.tab_scroll_x;
        let active_tab = self.active_tab;
        let tab_count = self.tabs.len();
        let need_scroll = self.scroll_to_active;

        // Záložková data vytáhneme před closure, abychom se vyhnuli vnořeným &mut borrow konfliktům
        let tab_data: Vec<(String, bool, bool)> = self.tabs.iter()
            .map(|t| {
                let name = t.path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "???".to_string());
                let label = if t.deleted {
                    format!("{} \u{26A0}", name)  // ⚠ smazán
                } else if t.modified {
                    format!("{} \u{25CF}", name)  // ● neuloženo
                } else {
                    name
                };
                (label, t.modified, t.deleted)
            })
            .collect();

        let mut scroll_left = false;
        let mut scroll_right = false;
        let mut new_scroll_x = initial_scroll;
        let mut active_rect: Option<egui::Rect> = None;

        ui.horizontal(|ui| {
            // ◀ tlačítko
            if ui.add_enabled(
                initial_scroll > 0.0,
                egui::Button::new("◀").min_size(egui::vec2(btn_w, 0.0)),
            ).clicked() {
                scroll_left = true;
            }

            // Vyhradíme místo pro ▶ vpravo
            let avail_w = (ui.available_width() - btn_w - ui.spacing().item_spacing.x).max(50.0);

            let mut tab_action: Option<TabAction> = None;

            let out = egui::ScrollArea::horizontal()
                .id_salt("tab_scroll")
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .scroll_offset(egui::vec2(initial_scroll, 0.0))
                .max_width(avail_w)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for (idx, (label, _, deleted)) in tab_data.iter().enumerate() {
                            let is_active = active_tab == Some(idx);
                            let mut text = egui::RichText::new(label).size(config::UI_FONT_SIZE);
                            if is_active { text = text.strong(); }
                            if *deleted {
                                text = text.color(egui::Color32::from_rgb(200, 100, 80));
                            }

                            let r = ui.selectable_label(is_active, text);
                            if is_active { active_rect = Some(r.rect); }
                            if r.clicked() { tab_action = Some(TabAction::Switch(idx)); }
                            if r.clicked_by(egui::PointerButton::Middle) { tab_action = Some(TabAction::Close(idx)); }
                            if ui.small_button("\u{00D7}").clicked() { tab_action = Some(TabAction::Close(idx)); }
                            if idx + 1 < tab_count { ui.separator(); }
                        }
                    });
                });

            if let Some(a) = tab_action { *action = Some(a); }

            new_scroll_x = out.state.offset.x;

            // Pokud byl právě otevřen nový soubor, doscrolujeme aktivní záložku do pohledu
            if need_scroll {
                if let Some(tab_rect) = active_rect {
                    let inner = out.inner_rect;
                    if tab_rect.max.x > inner.max.x {
                        new_scroll_x += tab_rect.max.x - inner.max.x + 8.0;
                    } else if tab_rect.min.x < inner.min.x {
                        new_scroll_x = (new_scroll_x - (inner.min.x - tab_rect.min.x) - 8.0).max(0.0);
                    }
                }
            }

            let content_w = out.content_size.x;
            let visible_w = out.inner_rect.width();
            let can_right = new_scroll_x + visible_w < content_w - 1.0;

            // ▶ tlačítko
            if ui.add_enabled(
                can_right,
                egui::Button::new("▶").min_size(egui::vec2(btn_w, 0.0)),
            ).clicked() {
                scroll_right = true;
            }
        });

        if scroll_left {
            new_scroll_x = (new_scroll_x - config::TAB_SCROLL_STEP).max(0.0);
        }
        if scroll_right {
            new_scroll_x += config::TAB_SCROLL_STEP;
        }

        self.tab_scroll_x = new_scroll_x;
        self.scroll_to_active = false;
        ui.separator();
    }

    fn search_bar(&mut self, ui: &mut egui::Ui) {
        let match_count = self.search_matches.len();
        let current_idx = self.current_match;
        let focus_requested = self.search_focus_requested;

        let mut do_next = false;
        let mut do_prev = false;
        let mut do_replace = false;
        let mut do_replace_all = false;
        let mut do_close = false;
        let mut query_changed = false;

        ui.horizontal(|ui| {
            ui.label("Hledat:");
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.search_query)
                    .desired_width(200.0)
                    .id(egui::Id::new("search_input")),
            );
            if response.changed() {
                query_changed = true;
            }
            if focus_requested {
                response.request_focus();
            }
            // Enter → next, Shift+Enter → prev
            if response.has_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter))
            {
                if ui.input(|i| i.modifiers.shift) {
                    do_prev = true;
                } else {
                    do_next = true;
                }
            }

            if ui.small_button("\u{25B2}").clicked() {
                do_prev = true;
            }
            if ui.small_button("\u{25BC}").clicked() {
                do_next = true;
            }

            if match_count > 0 {
                let current = current_idx.map(|i| i + 1).unwrap_or(0);
                ui.label(format!("{}/{}", current, match_count));
            } else if !self.search_query.is_empty() {
                ui.label("0/0");
            }

            if !self.show_replace {
                if ui.small_button("Nahradit\u{2026}").clicked() {
                    self.show_replace = true;
                }
            }

            if ui.small_button("\u{00D7}").clicked() {
                do_close = true;
            }
        });

        if self.show_replace {
            ui.horizontal(|ui| {
                ui.label("Nahradit:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.replace_query)
                        .desired_width(200.0),
                );
                if ui.small_button("Nahradit").clicked() {
                    do_replace = true;
                }
                if ui.small_button("Nahradit v\u{0161}e").clicked() {
                    do_replace_all = true;
                }
            });
        }

        ui.separator();

        self.search_focus_requested = false;
        if query_changed {
            self.update_search();
        }
        if do_next {
            self.next_match();
        }
        if do_prev {
            self.prev_match();
        }
        if do_replace {
            self.replace_current();
        }
        if do_replace_all {
            self.replace_all();
        }
        if do_close {
            self.show_search = false;
            self.show_replace = false;
            self.search_matches.clear();
            self.current_match = None;
        }
    }

    fn goto_line_bar(&mut self, ui: &mut egui::Ui) {
        let mut do_jump = false;
        let mut do_close = false;

        ui.horizontal(|ui| {
            ui.label("Přejít na řádek:");
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.goto_line_input)
                    .desired_width(80.0)
                    .id(egui::Id::new("goto_line_input")),
            );
            response.request_focus();
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                do_jump = true;
            }
            if ui.button("OK").clicked() {
                do_jump = true;
            }
            if ui.small_button("\u{00D7}").clicked() {
                do_close = true;
            }
        });
        ui.separator();

        if do_jump {
            if let Ok(n) = self.goto_line_input.trim().parse::<usize>() {
                if n >= 1 {
                    self.pending_jump = Some(n);
                }
            }
            self.show_goto_line = false;
        }
        if do_close {
            self.show_goto_line = false;
        }
    }

    pub fn status_bar(&self, ui: &mut egui::Ui, git_branch: Option<&str>) {
        let tab = match self.active() {
            Some(t) => t,
            None => return,
        };

        let cursor_text = tab.last_cursor_range.map(|cr| {
            let rc = cr.primary.rcursor;
            format!("{}:{}", rc.row + 1, rc.column + 1)
        });

        let file_type = ext_to_file_type(&self.extension());

        ui.horizontal(|ui| {
            ui.label(tab.path.to_string_lossy().to_string());
            ui.separator();
            match tab.save_status {
                SaveStatus::None => {}
                SaveStatus::Modified => { ui.label("Neuloženo"); }
                SaveStatus::Saving  => { ui.label("Ukládání…"); }
                SaveStatus::Saved   => { ui.label("Uloženo"); }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(pos) = cursor_text {
                    ui.label(egui::RichText::new(pos).monospace());
                    ui.separator();
                }
                if let Some(branch) = git_branch {
                    ui.label(
                        egui::RichText::new(format!("\u{2387} {}", branch))
                            .color(egui::Color32::from_rgb(100, 210, 130)),
                    );
                    ui.separator();
                }
                ui.label(egui::RichText::new("UTF-8").weak());
                ui.separator();
                ui.label(egui::RichText::new(file_type).weak());
            });
        });
        ui.separator();
    }

    fn ui_normal(&mut self, ui: &mut egui::Ui, dialog_open: bool) -> bool {
        let idx = match self.active_tab {
            Some(i) => i,
            None => return false,
        };

        let bg = self.highlighter.background_color();
        let ext = self.extension();
        let fname = self.filename();
        let search_matches = self.search_matches.clone();
        let current_match = self.current_match;

        // Zpracování pending_jump před renderem: vypočítáme char index a odhadovaný scroll offset
        let jump_line = self.pending_jump.take();
        let jump_char_idx: Option<usize> = jump_line.and_then(|line| {
            self.tabs.get(idx).map(|tab| {
                tab.content
                    .lines()
                    .take(line.saturating_sub(1))
                    .map(|l| l.chars().count() + 1)
                    .sum::<usize>()
            })
        });
        let desired_scroll_y: Option<f32> = jump_line.map(|line| {
            (line.saturating_sub(1) as f32) * config::EDITOR_FONT_SIZE * 1.5
        });

        let mut clicked = false;
        let mut saved_response: Option<egui::text_edit::TextEditOutput> = None;
        let mut content_changed = false;

        let frame = egui::Frame::new()
            .fill(bg)
            .inner_margin(egui::Margin::same(8));

        frame.show(ui, |ui| {
            let mut scroll = egui::ScrollArea::both().auto_shrink([false, false]);
            if let Some(y) = desired_scroll_y {
                scroll = scroll.vertical_scroll_offset(y);
            }
            scroll.show(ui, |ui| {
                    let highlighter = &self.highlighter;
                    let tab = &mut self.tabs[idx];

                    let previous_content = tab.content.clone();

                    let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                        let mut job = highlighter.highlight(text, &ext, &fname);
                        job.wrap.max_width = wrap_width;
                        apply_search_highlights(&mut job, &search_matches, current_match);
                        ui.fonts(|f| f.layout_job(job))
                    };

                    let line_count = tab.content.lines().count().max(1)
                        + if tab.content.ends_with('\n') { 1 } else { 0 };
                    let gutter_width = Self::gutter_width(ui, line_count);

                    ui.horizontal_top(|ui| {
                        let (gutter_rect, _) = ui.allocate_exact_size(
                            egui::vec2(gutter_width, ui.available_height()),
                            egui::Sense::hover(),
                        );

                        let response = egui::TextEdit::multiline(&mut tab.content)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .interactive(!dialog_open)
                            .desired_width(f32::INFINITY)
                            .layouter(&mut layouter)
                            .show(ui);

                        Self::paint_line_numbers(ui, &response, gutter_rect);

                        if response.response.clicked() || response.response.has_focus() {
                            clicked = true;
                        }

                        saved_response = Some(response);
                    });

                    if tab.content != previous_content {
                        tab.modified = true;
                        tab.last_edit = Some(Instant::now());
                        tab.save_status = SaveStatus::Modified;
                        content_changed = true;
                    }
                });
        });

        if let Some(response) = &saved_response {
            self.show_editor_context_menu(response);
        }

        // Nastavení pozice kurzoru po skoku na řádek
        if let (Some(char_idx), Some(response)) = (jump_char_idx, &saved_response) {
            let id = response.response.id;
            let mut state = egui::text_edit::TextEditState::load(ui.ctx(), id).unwrap_or_default();
            state.cursor.set_char_range(Some(egui::text::CCursorRange::one(
                egui::text::CCursor::new(char_idx),
            )));
            state.store(ui.ctx(), id);
        }

        if content_changed && self.show_search {
            self.update_search();
        }

        clicked
    }

    fn ui_markdown_split(&mut self, ui: &mut egui::Ui, dialog_open: bool) -> bool {
        let idx = match self.active_tab {
            Some(i) => i,
            None => return false,
        };

        let bg = self.highlighter.background_color();
        let ext = self.extension();
        let fname = self.filename();
        let search_matches = self.search_matches.clone();
        let current_match = self.current_match;

        let mut clicked = false;
        let mut saved_response: Option<egui::text_edit::TextEditOutput> = None;
        let mut content_changed = false;

        let available = ui.available_size();
        let handle_h = 6.0_f32;
        let top_h = (available.y * self.md_split_ratio)
            .max(50.0)
            .min(available.y - handle_h - 50.0);
        let bottom_h = (available.y - top_h - handle_h).max(50.0);

        // --- Horní polovina: Editor ---
        ui.allocate_ui_with_layout(
            egui::vec2(available.x, top_h),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.label(egui::RichText::new("Editor").strong());
                ui.separator();

                let frame = egui::Frame::new()
                    .fill(bg)
                    .inner_margin(egui::Margin::same(8));

                frame.show(ui, |ui| {
                    let highlighter = &self.highlighter;
                    let tab = &mut self.tabs[idx];

                    let scroll_output = egui::ScrollArea::both()
                        .id_salt("md_editor_scroll")
                        .auto_shrink([false, false])
                        .vertical_scroll_offset(tab.scroll_offset)
                        .show(ui, |ui| {
                            let previous_content = tab.content.clone();

                            let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                                let mut job = highlighter.highlight(text, &ext, &fname);
                                job.wrap.max_width = wrap_width;
                                apply_search_highlights(&mut job, &search_matches, current_match);
                                ui.fonts(|f| f.layout_job(job))
                            };

                            let line_count = tab.content.lines().count().max(1)
                                + if tab.content.ends_with('\n') { 1 } else { 0 };
                            let gutter_width = Self::gutter_width(ui, line_count);

                            ui.horizontal_top(|ui| {
                                let (gutter_rect, _) = ui.allocate_exact_size(
                                    egui::vec2(gutter_width, ui.available_height()),
                                    egui::Sense::hover(),
                                );

                                let response = egui::TextEdit::multiline(&mut tab.content)
                                    .font(egui::TextStyle::Monospace)
                                    .code_editor()
                                    .interactive(!dialog_open)
                                    .desired_width(f32::INFINITY)
                                    .layouter(&mut layouter)
                                    .show(ui);

                                Self::paint_line_numbers(ui, &response, gutter_rect);

                                if response.response.clicked() || response.response.has_focus() {
                                    clicked = true;
                                }

                                saved_response = Some(response);
                            });

                            if tab.content != previous_content {
                                tab.modified = true;
                                tab.last_edit = Some(Instant::now());
                                tab.save_status = SaveStatus::Modified;
                                content_changed = true;
                            }
                        });

                    tab.scroll_offset = scroll_output.state.offset.y;
                });
            },
        );

        // --- Táhlo pro změnu velikosti ---
        let (handle_rect, handle_response) = ui.allocate_exact_size(
            egui::vec2(available.x, handle_h),
            egui::Sense::drag(),
        );

        let handle_color = if handle_response.hovered() || handle_response.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
            egui::Color32::from_rgb(100, 140, 200)
        } else {
            egui::Color32::from_rgb(55, 60, 70)
        };
        ui.painter().rect_filled(handle_rect, 0.0, handle_color);

        // Tři tečky jako vizuální indikátor táhla
        let dot_y = handle_rect.center().y;
        let dot_r = 1.5_f32;
        for dx in [-6.0_f32, 0.0, 6.0] {
            ui.painter().circle_filled(
                egui::pos2(handle_rect.center().x + dx, dot_y),
                dot_r,
                egui::Color32::from_rgb(160, 170, 190),
            );
        }

        if handle_response.dragged() {
            let delta = handle_response.drag_delta().y;
            self.md_split_ratio =
                ((self.md_split_ratio * available.y + delta) / available.y).clamp(0.1, 0.9);
        }

        // --- Dolní polovina: Náhled ---
        let preview_content = self.tabs[idx].content.clone();
        let scroll_offset = self.tabs[idx].scroll_offset;

        ui.allocate_ui_with_layout(
            egui::vec2(available.x, bottom_h),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.label(egui::RichText::new("Náhled").strong());
                ui.separator();

                let preview_frame = egui::Frame::new()
                    .fill(egui::Color32::from_rgb(40, 44, 52))
                    .inner_margin(egui::Margin::same(12));

                preview_frame.show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("md_preview_scroll")
                        .auto_shrink([false, false])
                        .vertical_scroll_offset(scroll_offset)
                        .show(ui, |ui| {
                            Self::render_markdown_preview(ui, &preview_content);
                        });
                });
            },
        );

        if let Some(response) = &saved_response {
            self.show_editor_context_menu(response);
        }

        if content_changed && self.show_search {
            self.update_search();
        }

        clicked
    }

    fn show_editor_context_menu(
        &mut self,
        response: &egui::text_edit::TextEditOutput,
    ) {
        let idx = match self.active_tab {
            Some(i) => i,
            None => return,
        };

        if response.cursor_range.is_some() {
            self.tabs[idx].last_cursor_range = response.cursor_range;
        }

        let tab = &mut self.tabs[idx];
        let menu_size = 15.0;

        response.response.context_menu(|ui| {
            let selected_text = tab.last_cursor_range.and_then(|cr| {
                let start = cr.primary.ccursor.index.min(cr.secondary.ccursor.index);
                let end = cr.primary.ccursor.index.max(cr.secondary.ccursor.index);
                if start != end {
                    Some(
                        tab.content
                            .chars()
                            .skip(start)
                            .take(end - start)
                            .collect::<String>(),
                    )
                } else {
                    None
                }
            });
            let has_selection = selected_text.is_some();

            if ui
                .add_enabled(
                    has_selection,
                    egui::Button::new(egui::RichText::new("Kopírovat").size(menu_size)),
                )
                .clicked()
            {
                if let Some(text) = &selected_text {
                    ui.ctx().copy_text(text.to_string());
                }
                ui.close_menu();
            }

            if ui
                .button(egui::RichText::new("Vložit").size(menu_size))
                .clicked()
            {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    if let Ok(text) = clipboard.get_text() {
                        let insert_pos = tab
                            .last_cursor_range
                            .map(|cr| {
                                cr.primary.ccursor.index.max(cr.secondary.ccursor.index)
                            })
                            .unwrap_or(tab.content.chars().count());
                        let (start, end) = if let Some(cr) = tab.last_cursor_range {
                            let s = cr.primary.ccursor.index.min(cr.secondary.ccursor.index);
                            let e = cr.primary.ccursor.index.max(cr.secondary.ccursor.index);
                            if s != e { (s, e) } else { (insert_pos, insert_pos) }
                        } else {
                            (insert_pos, insert_pos)
                        };
                        let byte_start = tab
                            .content
                            .char_indices()
                            .nth(start)
                            .map(|(i, _)| i)
                            .unwrap_or(tab.content.len());
                        let byte_end = tab
                            .content
                            .char_indices()
                            .nth(end)
                            .map(|(i, _)| i)
                            .unwrap_or(tab.content.len());
                        tab.content.replace_range(byte_start..byte_end, &text);
                        tab.modified = true;
                        tab.last_edit = Some(Instant::now());
                        tab.save_status = SaveStatus::Modified;
                    }
                }
                ui.close_menu();
            }
        });
    }

    fn gutter_width(ui: &egui::Ui, line_count: usize) -> f32 {
        let font_id = egui::FontId::monospace(config::EDITOR_FONT_SIZE);
        let digits = ((line_count.max(1) as f64).log10().floor() as usize) + 1;
        let char_width = ui.fonts(|f| f.glyph_width(&font_id, '0'));
        (digits as f32) * char_width + 12.0
    }

    fn paint_line_numbers(
        ui: &egui::Ui,
        output: &egui::text_edit::TextEditOutput,
        gutter_rect: egui::Rect,
    ) {
        let font_id = egui::FontId::monospace(config::EDITOR_FONT_SIZE);
        let gutter_color = egui::Color32::from_rgb(130, 130, 130);
        let highlight_color = egui::Color32::from_rgba_unmultiplied(80, 65, 15, 50);
        let painter = ui.painter();

        let galley_pos = output.galley_pos;
        let galley = &output.galley;

        let cursor_row = output
            .cursor_range
            .map(|cr| cr.primary.rcursor.row);

        let mut line_num: usize = 1;
        let mut is_new_line = true;

        for (row_idx, row) in galley.rows.iter().enumerate() {
            let y = galley_pos.y + row.rect.min.y;
            let row_height = row.rect.height();

            if cursor_row == Some(row_idx) {
                let highlight_rect = egui::Rect::from_min_size(
                    egui::pos2(gutter_rect.left(), y),
                    egui::vec2(
                        output.response.rect.right() - gutter_rect.left(),
                        row_height,
                    ),
                );
                painter.rect_filled(highlight_rect, 0.0, highlight_color);
            }

            if is_new_line {
                let text = format!("{}", line_num);
                painter.text(
                    egui::pos2(gutter_rect.right() - 4.0, y),
                    egui::Align2::RIGHT_TOP,
                    text,
                    font_id.clone(),
                    gutter_color,
                );
                line_num += 1;
            }
            is_new_line = row.ends_with_newline;
        }
    }

    fn render_markdown_preview(ui: &mut egui::Ui, content: &str) {
        let options = Options::all();
        let parser = Parser::new_ext(content, options);

        let text_color = egui::Color32::from_rgb(220, 220, 220);

        let events: Vec<Event> = parser.collect();
        let mut i = 0;

        while i < events.len() {
            match &events[i] {
                Event::Start(Tag::Heading { level, .. }) => {
                    let level = *level;
                    i += 1;
                    let mut text = String::new();
                    while i < events.len() {
                        match &events[i] {
                            Event::End(TagEnd::Heading(_)) => {
                                i += 1;
                                break;
                            }
                            Event::Text(t) => text.push_str(t),
                            Event::Code(c) => text.push_str(c),
                            Event::SoftBreak => text.push(' '),
                            _ => {}
                        }
                        i += 1;
                    }
                    let size = match level {
                        HeadingLevel::H1 => 28.0,
                        HeadingLevel::H2 => 24.0,
                        HeadingLevel::H3 => 20.0,
                        HeadingLevel::H4 => 18.0,
                        HeadingLevel::H5 => 16.0,
                        HeadingLevel::H6 => 14.0,
                    };
                    let rt = egui::RichText::new(&text)
                        .size(size)
                        .strong()
                        .color(egui::Color32::WHITE);
                    ui.add(egui::Label::new(rt).wrap_mode(egui::TextWrapMode::Wrap));
                    ui.add_space(4.0);
                }
                Event::Start(Tag::Paragraph) => {
                    i += 1;
                    let mut job = egui::text::LayoutJob::default();
                    while i < events.len() {
                        match &events[i] {
                            Event::End(TagEnd::Paragraph) => {
                                i += 1;
                                break;
                            }
                            Event::Text(t) => {
                                job.append(
                                    t,
                                    0.0,
                                    egui::TextFormat {
                                        font_id: egui::FontId::proportional(14.0),
                                        color: text_color,
                                        ..Default::default()
                                    },
                                );
                            }
                            Event::Start(Tag::Strong) => {}
                            Event::End(TagEnd::Strong) => {}
                            Event::Start(Tag::Emphasis) => {}
                            Event::End(TagEnd::Emphasis) => {}
                            Event::Start(Tag::Strikethrough) => {}
                            Event::End(TagEnd::Strikethrough) => {}
                            Event::Start(Tag::Link { dest_url, .. }) => {
                                let _url = dest_url.to_string();
                                i += 1;
                                while i < events.len() {
                                    match &events[i] {
                                        Event::Text(t) => {
                                            job.append(
                                                t,
                                                0.0,
                                                egui::TextFormat {
                                                    font_id: egui::FontId::proportional(14.0),
                                                    color: egui::Color32::from_rgb(100, 160, 255),
                                                    underline: egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 160, 255)),
                                                    ..Default::default()
                                                },
                                            );
                                        }
                                        Event::End(TagEnd::Link) => break,
                                        _ => {}
                                    }
                                    i += 1;
                                }
                            }
                            Event::Code(c) => {
                                job.append(
                                    c,
                                    0.0,
                                    egui::TextFormat {
                                        font_id: egui::FontId::monospace(13.0),
                                        color: egui::Color32::from_rgb(230, 180, 100),
                                        background: egui::Color32::from_rgb(50, 55, 65),
                                        ..Default::default()
                                    },
                                );
                            }
                            Event::SoftBreak => {
                                job.append(
                                    " ",
                                    0.0,
                                    egui::TextFormat {
                                        font_id: egui::FontId::proportional(14.0),
                                        color: text_color,
                                        ..Default::default()
                                    },
                                );
                            }
                            Event::HardBreak => {
                                job.append(
                                    "\n",
                                    0.0,
                                    egui::TextFormat {
                                        font_id: egui::FontId::proportional(14.0),
                                        color: text_color,
                                        ..Default::default()
                                    },
                                );
                            }
                            _ => {}
                        }
                        i += 1;
                    }
                    job.wrap.max_width = ui.available_width();
                    ui.add(egui::Label::new(job).wrap_mode(egui::TextWrapMode::Wrap));
                    ui.add_space(8.0);
                }
                Event::Start(Tag::CodeBlock(_)) => {
                    i += 1;
                    let mut code_text = String::new();
                    while i < events.len() {
                        match &events[i] {
                            Event::End(TagEnd::CodeBlock) => {
                                i += 1;
                                break;
                            }
                            Event::Text(t) => code_text.push_str(t),
                            _ => {}
                        }
                        i += 1;
                    }
                    ui.add_space(4.0);
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(30, 33, 40))
                        .corner_radius(4.0)
                        .inner_margin(egui::Margin::same(8))
                        .show(ui, |ui| {
                            let rt = egui::RichText::new(code_text.trim_end())
                                .family(egui::FontFamily::Monospace)
                                .size(13.0)
                                .color(egui::Color32::from_rgb(180, 210, 170));
                            ui.add(egui::Label::new(rt).wrap_mode(egui::TextWrapMode::Wrap));
                        });
                    ui.add_space(4.0);
                }
                Event::Start(Tag::List(start)) => {
                    let mut list_idx = *start;
                    i += 1;
                    while i < events.len() {
                        match &events[i] {
                            Event::End(TagEnd::List(_)) => {
                                i += 1;
                                break;
                            }
                            Event::Start(Tag::Item) => {
                                i += 1;
                                let mut item_text = String::new();
                                let mut depth = 0;
                                while i < events.len() {
                                    match &events[i] {
                                        Event::Start(Tag::Paragraph) if depth == 0 => {
                                            depth += 1;
                                        }
                                        Event::End(TagEnd::Paragraph) if depth > 0 => {
                                            depth -= 1;
                                        }
                                        Event::End(TagEnd::Item) => {
                                            i += 1;
                                            break;
                                        }
                                        Event::Text(t) => item_text.push_str(t),
                                        Event::Code(c) => {
                                            item_text.push('`');
                                            item_text.push_str(c);
                                            item_text.push('`');
                                        }
                                        Event::SoftBreak => item_text.push(' '),
                                        _ => {}
                                    }
                                    i += 1;
                                }
                                let prefix = if let Some(ref mut idx) = list_idx {
                                    let p = format!("  {}. ", idx);
                                    *idx += 1;
                                    p
                                } else {
                                    "  \u{2022} ".to_string()
                                };
                                let rt = egui::RichText::new(format!("{}{}", prefix, item_text))
                                    .size(14.0)
                                    .color(text_color);
                                ui.add(egui::Label::new(rt).wrap_mode(egui::TextWrapMode::Wrap));
                                continue;
                            }
                            _ => {}
                        }
                        i += 1;
                    }
                    ui.add_space(4.0);
                }
                Event::Start(Tag::BlockQuote(_)) => {
                    i += 1;
                    let mut quote_text = String::new();
                    let mut depth = 0;
                    while i < events.len() {
                        match &events[i] {
                            Event::Start(Tag::BlockQuote(_)) => depth += 1,
                            Event::End(TagEnd::BlockQuote(_)) if depth > 0 => depth -= 1,
                            Event::End(TagEnd::BlockQuote(_)) => {
                                i += 1;
                                break;
                            }
                            Event::Text(t) => quote_text.push_str(t),
                            Event::SoftBreak => quote_text.push(' '),
                            Event::Start(Tag::Paragraph) | Event::End(TagEnd::Paragraph) => {}
                            _ => {}
                        }
                        i += 1;
                    }
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(50, 55, 65))
                        .inner_margin(egui::Margin {
                            left: 12,
                            right: 8,
                            top: 6,
                            bottom: 6,
                        })
                        .show(ui, |ui| {
                            let rt = egui::RichText::new(&quote_text)
                                .size(14.0)
                                .italics()
                                .color(egui::Color32::from_rgb(180, 180, 190));
                            ui.add(egui::Label::new(rt).wrap_mode(egui::TextWrapMode::Wrap));
                        });
                    ui.add_space(4.0);
                }
                Event::Rule => {
                    ui.separator();
                    ui.add_space(4.0);
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }
    }
}

fn ext_to_file_type(ext: &str) -> &'static str {
    match ext {
        "rs"                => "Rust",
        "php"               => "PHP",
        "js" | "mjs"        => "JavaScript",
        "ts"                => "TypeScript",
        "tsx"               => "TSX",
        "jsx"               => "JSX",
        "md" | "markdown"   => "Markdown",
        "html" | "htm"      => "HTML",
        "css"               => "CSS",
        "scss"              => "SCSS",
        "json"              => "JSON",
        "toml"              => "TOML",
        "yaml" | "yml"      => "YAML",
        "sh" | "bash"       => "Shell",
        "py"                => "Python",
        "c"                 => "C",
        "cpp" | "cc" | "cxx" => "C++",
        "h" | "hpp"         => "C/C++ Header",
        "go"                => "Go",
        "java"              => "Java",
        "xml"               => "XML",
        "sql"               => "SQL",
        "txt"               => "Text",
        _                   => "Plain text",
    }
}

fn apply_search_highlights(
    job: &mut egui::text::LayoutJob,
    matches: &[(usize, usize)],
    current: Option<usize>,
) {
    if matches.is_empty() {
        return;
    }
    let highlight_bg = egui::Color32::from_rgba_unmultiplied(255, 255, 0, 60);
    let current_bg = egui::Color32::from_rgba_unmultiplied(255, 165, 0, 100);

    for section in &mut job.sections {
        for (idx, &(m_start, m_end)) in matches.iter().enumerate() {
            if m_start < section.byte_range.end && m_end > section.byte_range.start {
                section.format.background = if Some(idx) == current {
                    current_bg
                } else {
                    highlight_bg
                };
                break;
            }
        }
    }
}
