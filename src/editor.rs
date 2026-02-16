use std::path::PathBuf;
use std::time::Instant;

use eframe::egui;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::highlighter::Highlighter;

const AUTOSAVE_DELAY_MS: u128 = 500;

#[derive(PartialEq)]
pub enum SaveStatus {
    None,
    Modified,
    Saving,
    Saved,
}

pub struct Editor {
    pub content: String,
    pub path: Option<PathBuf>,
    pub modified: bool,
    pub last_edit: Option<Instant>,
    pub save_status: SaveStatus,
    pub last_saved_content: String,
    highlighter: Highlighter,
    scroll_offset: f32,
    last_cursor_range: Option<egui::text::CursorRange>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            path: None,
            modified: false,
            last_edit: None,
            save_status: SaveStatus::None,
            last_saved_content: String::new(),
            highlighter: Highlighter::new(),
            scroll_offset: 0.0,
            last_cursor_range: None,
        }
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.path = None;
        self.modified = false;
        self.last_edit = None;
        self.save_status = SaveStatus::None;
        self.last_saved_content.clear();
        self.scroll_offset = 0.0;
    }

    pub fn open_file(&mut self, path: &PathBuf) {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                self.content = content.clone();
                self.last_saved_content = content;
                self.path = Some(path.clone());
                self.modified = false;
                self.last_edit = None;
                self.save_status = SaveStatus::None;
                self.scroll_offset = 0.0;
            }
            Err(e) => {
                self.content = format!("Chyba při čtení souboru: {}", e);
                self.path = Some(path.clone());
                self.modified = false;
                self.save_status = SaveStatus::None;
            }
        }
    }

    pub fn reload_from_disk(&mut self) {
        if let Some(path) = &self.path {
            if let Ok(content) = std::fs::read_to_string(path) {
                self.content = content.clone();
                self.last_saved_content = content;
                self.modified = false;
                self.last_edit = None;
                self.save_status = SaveStatus::Saved;
            }
        }
    }

    pub fn try_autosave(&mut self) {
        if !self.modified {
            return;
        }

        if let Some(last_edit) = self.last_edit {
            if last_edit.elapsed().as_millis() >= AUTOSAVE_DELAY_MS {
                self.save();
            }
        }
    }

    pub fn save(&mut self) {
        if let Some(path) = &self.path {
            self.save_status = SaveStatus::Saving;
            match std::fs::write(path, &self.content) {
                Ok(()) => {
                    self.last_saved_content = self.content.clone();
                    self.modified = false;
                    self.last_edit = None;
                    self.save_status = SaveStatus::Saved;
                }
                Err(_) => {
                    self.save_status = SaveStatus::Modified;
                }
            }
        }
    }

    fn extension(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.extension())
            .map(|e| e.to_string_lossy().to_string())
            .or_else(|| {
                // Pro soubory jako ".env" — nemají příponu, ale název začíná tečkou
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
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    fn is_markdown(&self) -> bool {
        let ext = self.extension();
        ext == "md" || ext == "markdown"
    }

    fn status_bar(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if let Some(path) = &self.path {
                ui.label(path.to_string_lossy().to_string());
            }
            ui.separator();
            match self.save_status {
                SaveStatus::None => {}
                SaveStatus::Modified => {
                    ui.label("Neuloženo");
                }
                SaveStatus::Saving => {
                    ui.label("Ukládání...");
                }
                SaveStatus::Saved => {
                    ui.label("Uloženo");
                }
            }
        });
        ui.separator();
    }

    /// Vrací `true` pokud uživatel klikl do editoru (pro přepnutí fokusu).
    pub fn ui(&mut self, ui: &mut egui::Ui, dialog_open: bool) -> bool {
        if self.path.is_none() {
            ui.centered_and_justified(|ui| {
                ui.label("Otevřete soubor z adresářového stromu vlevo");
            });
            return false;
        }

        self.status_bar(ui);

        if self.is_markdown() {
            self.ui_markdown_split(ui, dialog_open)
        } else {
            self.ui_normal(ui, dialog_open)
        }
    }

    fn ui_normal(&mut self, ui: &mut egui::Ui, dialog_open: bool) -> bool {
        let bg = self.highlighter.background_color();
        let ext = self.extension();
        let fname = self.filename();
        let mut clicked = false;
        let mut saved_response: Option<egui::text_edit::TextEditOutput> = None;

        let frame = egui::Frame::new()
            .fill(bg)
            .inner_margin(egui::Margin::same(8));

        frame.show(ui, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let previous_content = self.content.clone();

                    let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                        let mut job = self.highlighter.highlight(text, &ext, &fname);
                        job.wrap.max_width = wrap_width;
                        ui.fonts(|f| f.layout_job(job))
                    };

                    let line_count = self.content.lines().count().max(1)
                        + if self.content.ends_with('\n') { 1 } else { 0 };
                    let gutter_width = Self::gutter_width(ui, line_count);

                    ui.horizontal_top(|ui| {
                        let (gutter_rect, _) = ui.allocate_exact_size(
                            egui::vec2(gutter_width, ui.available_height()),
                            egui::Sense::hover(),
                        );

                        let response = egui::TextEdit::multiline(&mut self.content)
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

                    if self.content != previous_content {
                        self.modified = true;
                        self.last_edit = Some(Instant::now());
                        self.save_status = SaveStatus::Modified;
                    }
                });
        });

        if let Some(response) = &saved_response {
            self.show_editor_context_menu(response);
        }

        clicked
    }

    fn ui_markdown_split(&mut self, ui: &mut egui::Ui, dialog_open: bool) -> bool {
        let bg = self.highlighter.background_color();
        let ext = self.extension();
        let fname = self.filename();
        let mut clicked = false;
        let mut saved_response: Option<egui::text_edit::TextEditOutput> = None;

        ui.columns(2, |columns| {
            // Levý sloupec — editor
            columns[0].label(egui::RichText::new("Editor").strong());
            columns[0].separator();

            let frame = egui::Frame::new()
                .fill(bg)
                .inner_margin(egui::Margin::same(8));

            frame.show(&mut columns[0], |ui| {
                let scroll_output = egui::ScrollArea::both()
                    .id_salt("md_editor_scroll")
                    .auto_shrink([false, false])
                    .vertical_scroll_offset(self.scroll_offset)
                    .show(ui, |ui| {
                        let previous_content = self.content.clone();

                        let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                            let mut job = self.highlighter.highlight(text, &ext, &fname);
                            job.wrap.max_width = wrap_width;
                            ui.fonts(|f| f.layout_job(job))
                        };

                        let line_count = self.content.lines().count().max(1)
                            + if self.content.ends_with('\n') { 1 } else { 0 };
                        let gutter_width = Self::gutter_width(ui, line_count);

                        ui.horizontal_top(|ui| {
                            let (gutter_rect, _) = ui.allocate_exact_size(
                                egui::vec2(gutter_width, ui.available_height()),
                                egui::Sense::hover(),
                            );

                            let response = egui::TextEdit::multiline(&mut self.content)
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

                        if self.content != previous_content {
                            self.modified = true;
                            self.last_edit = Some(Instant::now());
                            self.save_status = SaveStatus::Modified;
                        }
                    });

                self.scroll_offset = scroll_output.state.offset.y;
            });

            // Pravý sloupec — náhled
            columns[1].label(egui::RichText::new("Náhled").strong());
            columns[1].separator();

            let preview_frame = egui::Frame::new()
                .fill(egui::Color32::from_rgb(40, 44, 52))
                .inner_margin(egui::Margin::same(12));

            preview_frame.show(&mut columns[1], |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("md_preview_scroll")
                    .auto_shrink([false, false])
                    .vertical_scroll_offset(self.scroll_offset)
                    .show(ui, |ui| {
                        self.render_markdown_preview(ui);
                    });
            });
        });

        if let Some(response) = &saved_response {
            self.show_editor_context_menu(response);
        }

        clicked
    }

    fn show_editor_context_menu(
        &mut self,
        response: &egui::text_edit::TextEditOutput,
    ) {
        // Uložit cursor range pro použití v kontextovém menu
        if response.cursor_range.is_some() {
            self.last_cursor_range = response.cursor_range;
        }

        let menu_size = 15.0;
        response.response.context_menu(|ui| {
            let selected_text = self.last_cursor_range.and_then(|cr| {
                let start = cr.primary.ccursor.index.min(cr.secondary.ccursor.index);
                let end = cr.primary.ccursor.index.max(cr.secondary.ccursor.index);
                if start != end {
                    Some(
                        self.content
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
                        let insert_pos = self
                            .last_cursor_range
                            .map(|cr| {
                                cr.primary.ccursor.index.max(cr.secondary.ccursor.index)
                            })
                            .unwrap_or(self.content.chars().count());
                        // Pokud je vybraný text, nahradíme ho
                        let (start, end) = if let Some(cr) = self.last_cursor_range {
                            let s = cr.primary.ccursor.index.min(cr.secondary.ccursor.index);
                            let e = cr.primary.ccursor.index.max(cr.secondary.ccursor.index);
                            if s != e { (s, e) } else { (insert_pos, insert_pos) }
                        } else {
                            (insert_pos, insert_pos)
                        };
                        let byte_start = self
                            .content
                            .char_indices()
                            .nth(start)
                            .map(|(i, _)| i)
                            .unwrap_or(self.content.len());
                        let byte_end = self
                            .content
                            .char_indices()
                            .nth(end)
                            .map(|(i, _)| i)
                            .unwrap_or(self.content.len());
                        self.content.replace_range(byte_start..byte_end, &text);
                        self.modified = true;
                        self.last_edit = Some(Instant::now());
                        self.save_status = SaveStatus::Modified;
                    }
                }
                ui.close_menu();
            }
        });
    }

    fn gutter_width(ui: &egui::Ui, line_count: usize) -> f32 {
        let font_id = egui::FontId::monospace(14.0);
        let digits = ((line_count.max(1) as f64).log10().floor() as usize) + 1;
        let char_width = ui.fonts(|f| f.glyph_width(&font_id, '0'));
        (digits as f32) * char_width + 12.0
    }

    fn paint_line_numbers(
        ui: &egui::Ui,
        output: &egui::text_edit::TextEditOutput,
        gutter_rect: egui::Rect,
    ) {
        let font_id = egui::FontId::monospace(14.0);
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

    fn render_markdown_preview(&self, ui: &mut egui::Ui) {
        let options = Options::all();
        let parser = Parser::new_ext(&self.content, options);

        let text_color = egui::Color32::from_rgb(220, 220, 220);

        // Sbíráme eventy do bloků, pak renderujeme celý blok najednou
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
                                // Sbíráme text odkazu
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
