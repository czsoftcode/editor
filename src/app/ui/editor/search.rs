use std::time::Instant;

use eframe::egui;

use super::super::search_picker::build_regex;
use super::super::workspace::state::SearchOptions;
use super::{Editor, SaveStatus};

impl Editor {
    // --- Search logic ---

    pub(super) fn update_search(&mut self) {
        self.search_matches.clear();
        self.current_match = None;
        self.search_regex_error = None;

        if !self.show_search || self.search_query.is_empty() {
            return;
        }

        let active_idx = match self.active_tab {
            Some(i) => i,
            None => return,
        };
        let content = match self.tabs.get(active_idx) {
            Some(t) => &t.content,
            None => return,
        };

        let opts = SearchOptions {
            use_regex: self.search_use_regex,
            case_sensitive: self.search_case_sensitive,
            whole_word: self.search_whole_word,
            file_filter: String::new(),
        };

        let regex = match build_regex(&self.search_query, &opts) {
            Ok(r) => r,
            Err(msg) => {
                self.search_regex_error = Some(msg);
                return;
            }
        };

        self.search_matches = regex
            .find_iter(content)
            .map(|m| (m.start(), m.end()))
            .collect();

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

    pub(super) fn search_bar(&mut self, ui: &mut egui::Ui, i18n: &crate::i18n::I18n) {
        let match_count = self.search_matches.len();
        let current_idx = self.current_match;
        let focus_requested = self.search_focus_requested;

        let mut do_next = false;
        let mut do_prev = false;
        let mut do_replace = false;
        let mut do_replace_all = false;
        let mut do_close = false;
        let mut query_changed = false;
        let mut toggles_changed = false;

        ui.horizontal(|ui| {
            ui.label(i18n.get("search-label"));
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

            // Toggle buttons: .* (regex), Aa (case), W (whole-word)
            if ui
                .selectable_label(self.search_use_regex, ".*")
                .on_hover_text(i18n.get("search-regex-toggle"))
                .clicked()
            {
                self.search_use_regex = !self.search_use_regex;
                toggles_changed = true;
            }
            if ui
                .selectable_label(self.search_case_sensitive, "Aa")
                .on_hover_text(i18n.get("search-case-toggle"))
                .clicked()
            {
                self.search_case_sensitive = !self.search_case_sensitive;
                toggles_changed = true;
            }
            if ui
                .selectable_label(self.search_whole_word, "W")
                .on_hover_text(i18n.get("search-word-toggle"))
                .clicked()
            {
                self.search_whole_word = !self.search_whole_word;
                toggles_changed = true;
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

            // Regex error — krátká hláška červeně
            if let Some(ref err) = self.search_regex_error {
                let truncated: String = err.chars().take(40).collect();
                ui.colored_label(egui::Color32::RED, truncated);
            }

            if !self.show_replace && ui.small_button(i18n.get("search-replace-expand")).clicked() {
                self.show_replace = true;
            }
            if ui.small_button("\u{00D7}").clicked() {
                do_close = true;
            }
        });

        if self.show_replace {
            ui.horizontal(|ui| {
                ui.label(i18n.get("replace-label"));
                ui.add(egui::TextEdit::singleline(&mut self.replace_query).desired_width(200.0));
                if ui.small_button(i18n.get("search-replace-one")).clicked() {
                    do_replace = true;
                }
                if ui.small_button(i18n.get("search-replace-all")).clicked() {
                    do_replace_all = true;
                }
            });
        }

        ui.separator();

        self.search_focus_requested = false;
        if query_changed || toggles_changed {
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
}

// ---------------------------------------------------------------------------
// Search result highlighting in LayoutJob
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
    let current_bg = egui::Color32::from_rgba_unmultiplied(255, 165, 0, 100);

    let mut match_idx = 0;
    for section in &mut job.sections {
        // Skip matches that end before this section starts
        while match_idx < matches.len() && matches[match_idx].1 <= section.byte_range.start {
            match_idx += 1;
        }

        // Check if any subsequent matches overlap this section
        let mut i = match_idx;
        while i < matches.len() && matches[i].0 < section.byte_range.end {
            // Overlap detected
            let is_current = Some(i) == current;
            section.format.background = if is_current { current_bg } else { highlight_bg };
            // Priority to current match
            if is_current {
                break;
            }
            i += 1;
        }
    }
}
