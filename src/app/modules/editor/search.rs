use std::time::Instant;

use eframe::egui;

use super::{Editor, SaveStatus};

impl Editor {
    // --- Search logic ---

    pub(super) fn update_search(&mut self) {
        self.search_matches.clear();
        self.current_match = None;

        if !self.show_search || self.search_query.is_empty() {
            return;
        }

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

    pub(super) fn next_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.current_match = Some(match self.current_match {
            Some(i) => (i + 1) % self.search_matches.len(),
            None => 0,
        });
    }

    pub(super) fn prev_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.current_match = Some(match self.current_match {
            Some(0) | None => self.search_matches.len() - 1,
            Some(i) => i - 1,
        });
    }

    pub(super) fn replace_current(&mut self) {
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

    pub(super) fn replace_all(&mut self) {
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

    // --- Search bar UI ---

    pub(super) fn search_bar(&mut self, ui: &mut egui::Ui) {
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
            if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if ui.input(|i| i.modifiers.shift) {
                    do_prev = true;
                } else {
                    do_next = true;
                }
            }

            if ui.small_button("\u{25B2}").clicked() { do_prev = true; }
            if ui.small_button("\u{25BC}").clicked() { do_next = true; }

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
            if ui.small_button("\u{00D7}").clicked() { do_close = true; }
        });

        if self.show_replace {
            ui.horizontal(|ui| {
                ui.label("Nahradit:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.replace_query)
                        .desired_width(200.0),
                );
                if ui.small_button("Nahradit").clicked() { do_replace = true; }
                if ui.small_button("Nahradit v\u{0161}e").clicked() { do_replace_all = true; }
            });
        }

        ui.separator();

        self.search_focus_requested = false;
        if query_changed   { self.update_search(); }
        if do_next         { self.next_match(); }
        if do_prev         { self.prev_match(); }
        if do_replace      { self.replace_current(); }
        if do_replace_all  { self.replace_all(); }
        if do_close {
            self.show_search = false;
            self.show_replace = false;
            self.search_matches.clear();
            self.current_match = None;
        }
    }
}

// ---------------------------------------------------------------------------
// Zvýraznění výsledků hledání v LayoutJobu
// ---------------------------------------------------------------------------

pub(super) fn apply_search_highlights(
    job: &mut egui::text::LayoutJob,
    matches: &[(usize, usize)],
    current: Option<usize>,
) {
    if matches.is_empty() {
        return;
    }
    let highlight_bg = egui::Color32::from_rgba_unmultiplied(255, 255, 0, 60);
    let current_bg  = egui::Color32::from_rgba_unmultiplied(255, 165, 0, 100);

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
