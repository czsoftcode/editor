use crate::app::ui::widgets::modal::StandardModal;
use crate::app::ui::workspace::state::{SearchBatch, SearchOptions, SearchResult, WorkspaceState};
use crate::i18n::I18n;
use eframe::egui;
use regex::RegexBuilder;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{Arc, mpsc};

pub fn fuzzy_match(query: &str, text: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    // Simple contains for now, could be improved to real fuzzy
    text_lower.contains(&query_lower)
}

pub fn render_file_picker(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    i18n: &I18n,
) -> Option<PathBuf> {
    let picker = ws.file_picker.as_mut()?;

    let key_up = ctx.input(|i| i.key_pressed(egui::Key::ArrowUp));
    let key_down = ctx.input(|i| i.key_pressed(egui::Key::ArrowDown));
    let key_enter = ctx.input(|i| i.key_pressed(egui::Key::Enter));
    let key_esc = ctx.input(|i| i.key_pressed(egui::Key::Escape));

    let filtered_len = picker.filtered.len();
    if key_up && picker.selected > 0 {
        picker.selected -= 1;
    }
    if key_down && picker.selected + 1 < filtered_len {
        picker.selected += 1;
    }

    let mut selected_file: Option<PathBuf> = None;
    let mut show_flag = true;
    let mut close = key_esc;

    if key_enter && !picker.filtered.is_empty() {
        let idx = picker.filtered[picker.selected];
        selected_file = Some(ws.root_path.join(&picker.files[idx]));
        close = true;
    }

    let modal = StandardModal::new(i18n.get("file-picker-heading"), "file_picker_modal")
        .with_size(600.0, 450.0);

    modal.show(ctx, &mut show_flag, |ui| {
        // FOOTER
        if let Some(c) = modal.ui_footer_actions(ui, i18n, |f| {
            if f.close() || f.cancel() {
                return Some(true);
            }
            None
        }) {
            close = c;
        }

        // BODY
        modal.ui_body(ui, |ui| {
            ui.add_space(4.0);
            let resp = ui.add(
                egui::TextEdit::singleline(&mut picker.query)
                    .hint_text(i18n.get("file-picker-placeholder"))
                    .desired_width(ui.available_width())
                    .id(egui::Id::new("file_picker_input")),
            );
            if picker.focus_requested {
                resp.request_focus();
                picker.focus_requested = false;
            }
            if resp.changed() {
                picker.update_filter();
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let mut args = fluent_bundle::FluentArgs::new();
                args.set("total", picker.files.len() as i64);
                args.set("filtered", picker.filtered.len() as i64);
                ui.label(i18n.get_args("file-picker-count-filtered", &args));
            });
            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .id_salt("file_picker_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    for (disp_idx, &file_idx) in picker.filtered.iter().enumerate() {
                        let path = &picker.files[file_idx];
                        let is_sel = disp_idx == picker.selected;
                        let text = path.to_string_lossy();
                        let r = ui.selectable_label(is_sel, text);
                        if is_sel
                            && ctx.input(|i| {
                                i.key_pressed(egui::Key::ArrowUp)
                                    || i.key_pressed(egui::Key::ArrowDown)
                            })
                        {
                            // r.scroll_to_me(None); // Handled by scroll_area implicitly often, but can be explicit
                        }
                        if r.clicked() {
                            selected_file = Some(ws.root_path.join(path));
                            close = true;
                        }
                    }
                });
        });
    });

    if close || !show_flag {
        ws.file_picker = None;
    }

    selected_file
}

/// Spustí project search s aktuálním query a options.
fn start_project_search(ws: &mut WorkspaceState) {
    ws.project_search.results.clear();
    ws.project_search.searching = true;
    let epoch = ws
        .project_search
        .cancel_epoch
        .fetch_add(1, Ordering::Relaxed)
        + 1;
    let cancel_epoch = Arc::clone(&ws.project_search.cancel_epoch);
    let files = ws.project_index.get_files();
    ws.project_search.rx = Some(run_project_search(
        ws.root_path.clone(),
        files,
        ws.project_search.query.trim().to_string(),
        ws.project_search.options.clone(),
        epoch,
        cancel_epoch,
    ));
}

/// Inline search panel jako `TopBottomPanel::bottom` — náhrada za modální dialog.
///
/// Panel zobrazuje query input s togglery (regex, case, word, replace),
/// file filter, a spouští search přes `run_project_search()`.
/// Výsledky se streamují do `ws.project_search.results` a renderují se
/// v pozdější fázi panelu.
pub fn render_search_panel(ctx: &egui::Context, ws: &mut WorkspaceState, i18n: &I18n) {
    if !ws.project_search.show_panel {
        return;
    }

    // --- Poll loop: akumulace výsledků z background threadu ---
    if let Some(rx) = &ws.project_search.rx {
        loop {
            match rx.try_recv() {
                Ok(SearchBatch::Results(batch)) => {
                    ws.project_search.results.extend(batch);
                }
                Ok(SearchBatch::Done) => {
                    ws.project_search.rx = None;
                    ws.project_search.searching = false;
                    break;
                }
                Ok(SearchBatch::Error(e)) => {
                    ws.toasts
                        .push(crate::app::types::Toast::error(format!("Search: {}", e)));
                    ws.project_search.rx = None;
                    ws.project_search.searching = false;
                    break;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    ws.project_search.rx = None;
                    ws.project_search.searching = false;
                    break;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    break;
                }
            }
        }
    }
    // Pokud stále probíhá search, vyžádáme repaint pro další poll
    if ws.project_search.searching {
        ctx.request_repaint();
    }

    let mut start_search = false;
    let mut start_replace_preview = false;
    let mut close_panel = false;
    let focus_req = ws.project_search.focus_requested;

    // Escape zavře panel (pokud není otevřený modální dialog — replace preview)
    if !ws.project_search.show_replace_preview && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        close_panel = true;
    }

    egui::TopBottomPanel::bottom("search_panel")
        .resizable(true)
        .default_height(250.0)
        .min_height(100.0)
        .max_height(ctx.screen_rect().height() * 0.6)
        .show(ctx, |ui| {
            ui.add_space(4.0);

            // Panel titulek
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(i18n.get("project-search-panel-title").as_str())
                        .strong()
                        .size(13.0),
                );
            });
            ui.add_space(2.0);

            // --- Horní řádek: query input + togglery + close ---
            let mut toggles_changed = false;
            ui.horizontal(|ui| {
                // Query input
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut ws.project_search.query)
                        .hint_text(i18n.get("project-search-hint"))
                        .desired_width(ui.available_width() - 200.0)
                        .id(egui::Id::new("search_panel_query_input")),
                );
                if focus_req {
                    resp.request_focus();
                }
                if resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    start_search = true;
                }

                // Toggle buttons: .* (regex), Aa (case), W (whole-word), ↔ (replace)
                if ui
                    .selectable_label(ws.project_search.options.use_regex, ".*")
                    .on_hover_text(i18n.get("project-search-regex-toggle"))
                    .clicked()
                {
                    ws.project_search.options.use_regex = !ws.project_search.options.use_regex;
                    toggles_changed = true;
                }
                if ui
                    .selectable_label(ws.project_search.options.case_sensitive, "Aa")
                    .on_hover_text(i18n.get("project-search-case-toggle"))
                    .clicked()
                {
                    ws.project_search.options.case_sensitive =
                        !ws.project_search.options.case_sensitive;
                    toggles_changed = true;
                }
                if ui
                    .selectable_label(ws.project_search.options.whole_word, "W")
                    .on_hover_text(i18n.get("project-search-word-toggle"))
                    .clicked()
                {
                    ws.project_search.options.whole_word = !ws.project_search.options.whole_word;
                    toggles_changed = true;
                }
                // Replace toggle
                if ui
                    .selectable_label(ws.project_search.show_replace, "↔")
                    .on_hover_text(i18n.get("project-search-replace-heading"))
                    .clicked()
                {
                    ws.project_search.show_replace = !ws.project_search.show_replace;
                }

                // Close button
                if ui.button("✕").on_hover_text("Close").clicked() {
                    close_panel = true;
                }
            });

            // Replace input s Replace All buttonem (podmíněný na show_replace)
            if ws.project_search.show_replace {
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut ws.project_search.replace_text)
                            .hint_text(i18n.get("project-search-replace-with"))
                            .desired_width(ui.available_width() - 100.0)
                            .id(egui::Id::new("search_panel_replace_input")),
                    );
                    // Replace All button — aktivní jen když jsou výsledky
                    let has_results = !ws.project_search.results.is_empty();
                    if ui
                        .add_enabled(
                            has_results,
                            egui::Button::new(i18n.get("project-search-replace-btn")),
                        )
                        .clicked()
                    {
                        start_replace_preview = true;
                    }
                });
            }

            ui.add_space(2.0);

            // File filter input
            ui.add(
                egui::TextEdit::singleline(&mut ws.project_search.options.file_filter)
                    .hint_text(i18n.get("project-search-file-filter-hint"))
                    .desired_width(ui.available_width())
                    .id(egui::Id::new("search_panel_file_filter")),
            );

            // Inline regex error
            if let Some(ref err) = ws.project_search.regex_error {
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(err)
                        .color(egui::Color32::from_rgb(255, 80, 80))
                        .small(),
                );
            }

            // Automatický search při změně togglerů
            if toggles_changed && !ws.project_search.query.trim().is_empty() {
                start_search = true;
            }

            ui.add_space(4.0);
            ui.separator();

            // --- Výsledky ---
            let result_count = ws.project_search.results.len();

            // Loading indikátor
            if ws.project_search.searching {
                ui.horizontal(|ui| {
                    ui.spinner();
                    let mut args = fluent_bundle::FluentArgs::new();
                    args.set("count", result_count as i64);
                    ui.label(format!(
                        "{} ({} ...)",
                        i18n.get("project-search-searching"),
                        result_count,
                    ));
                });
                ui.add_space(2.0);
            } else if result_count > 0 {
                let mut args = fluent_bundle::FluentArgs::new();
                args.set("count", result_count as i64);
                ui.label(i18n.get_args("project-search-results-count", &args));
                ui.add_space(2.0);
            }

            // Scrollovatelný seznam výsledků, seskupených per-soubor
            let font_id = egui::FontId::monospace(12.0);
            let text_color = ui.visuals().text_color();
            let separator_text = i18n.get("project-search-context-separator");

            egui::ScrollArea::vertical()
                .id_salt("search_panel_results_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    let results = &ws.project_search.results;
                    let mut i = 0;
                    while i < results.len() {
                        let current_file = &results[i].file;

                        // Soubor heading
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(current_file.to_string_lossy().as_ref())
                                .strong()
                                .size(12.0),
                        );
                        ui.separator();

                        let mut prev_end_line: Option<usize> = None;

                        // Všechny výsledky pro tento soubor
                        while i < results.len() && results[i].file == *current_file {
                            let result = &results[i];

                            // Separator mezi nesouvisejícími bloky
                            let ctx_start_line =
                                result.line.saturating_sub(result.context_before.len());
                            if let Some(prev_end) = prev_end_line
                                && ctx_start_line > prev_end + 1
                            {
                                ui.label(
                                    egui::RichText::new(separator_text.as_str())
                                        .color(egui::Color32::GRAY)
                                        .small(),
                                );
                            }

                            // Context before
                            for (ci, ctx_line) in result.context_before.iter().enumerate() {
                                let line_num = result.line - result.context_before.len() + ci;
                                let job = build_context_layout_job(line_num, ctx_line, &font_id);
                                ui.add(egui::Label::new(job).wrap_mode(egui::TextWrapMode::Extend));
                            }

                            // Match řádek — zvýrazněný, klikací
                            let is_last_selected = ws.project_search.last_selected_index == Some(i);
                            let job = build_match_layout_job(
                                result.line,
                                &result.text,
                                &result.match_ranges,
                                &font_id,
                                text_color,
                            );
                            // Vizuální indikace naposledy navštíveného výsledku
                            if is_last_selected {
                                let rect = ui.available_rect_before_wrap();
                                let row_rect = egui::Rect::from_min_size(
                                    rect.min,
                                    egui::vec2(
                                        ui.available_width(),
                                        ui.text_style_height(&egui::TextStyle::Monospace) + 2.0,
                                    ),
                                );
                                ui.painter().rect_filled(
                                    row_rect,
                                    0.0,
                                    egui::Color32::from_rgba_unmultiplied(80, 120, 200, 40),
                                );
                            }
                            let resp = ui.add(
                                egui::Label::new(job)
                                    .wrap_mode(egui::TextWrapMode::Extend)
                                    .sense(egui::Sense::click()),
                            );
                            if resp.clicked() {
                                // Uložit kliknutý výsledek: highlight zůstává, navigace se spotřebuje
                                ws.project_search.last_selected_index = Some(i);
                                ws.project_search.pending_jump_index = Some(i);
                            }
                            if resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }

                            // Context after
                            for (ci, ctx_line) in result.context_after.iter().enumerate() {
                                let line_num = result.line + 1 + ci;
                                let job = build_context_layout_job(line_num, ctx_line, &font_id);
                                ui.add(egui::Label::new(job).wrap_mode(egui::TextWrapMode::Extend));
                            }

                            prev_end_line = Some(result.line + result.context_after.len());
                            i += 1;
                        }
                    }
                });
        });

    ws.project_search.focus_requested = false;

    // Spustit search
    if start_search && !ws.project_search.query.trim().is_empty() {
        match build_regex(ws.project_search.query.trim(), &ws.project_search.options) {
            Ok(_) => {
                ws.project_search.regex_error = None;
                start_project_search(ws);
            }
            Err(e) => {
                ws.project_search.regex_error = Some(e);
            }
        }
    }

    // Spustit replace preview z panelu
    if start_replace_preview && !ws.project_search.results.is_empty() {
        match build_regex(ws.project_search.query.trim(), &ws.project_search.options) {
            Ok(regex) => {
                match compute_replace_previews(
                    &ws.root_path,
                    &ws.project_search.results,
                    &regex,
                    &ws.project_search.replace_text,
                ) {
                    Ok(previews) => {
                        if previews.is_empty() {
                            ws.toasts.push(crate::app::types::Toast::info(
                                i18n.get("project-search-no-results"),
                            ));
                        } else {
                            ws.project_search.replace_previews = previews;
                            ws.project_search.show_replace_preview = true;
                        }
                    }
                    Err(e) => {
                        ws.toasts
                            .push(crate::app::types::Toast::error(format!("{}", e)));
                    }
                }
            }
            Err(e) => {
                ws.project_search.regex_error = Some(e);
            }
        }
    }

    // Zavřít panel
    if close_panel {
        ws.project_search.show_panel = false;
    }
}

/// Renderuje replace preview dialog — modal s per-file diff a checkboxy.
pub fn render_replace_preview_dialog(ctx: &egui::Context, ws: &mut WorkspaceState, i18n: &I18n) {
    if !ws.project_search.show_replace_preview {
        return;
    }

    let mut confirm = false;
    let mut cancel = false;
    let mut show_flag = true;

    let preview_count = ws.project_search.replace_previews.len();
    let selected_count = ws
        .project_search
        .replace_previews
        .iter()
        .filter(|p| p.selected)
        .count();

    let mut args = fluent_bundle::FluentArgs::new();
    args.set("count", preview_count as i64);
    args.set("selected", selected_count as i64);

    let title = i18n.get("project-search-replace-preview-title");

    let modal = StandardModal::new(&title, "replace_preview_modal").with_size(850.0, 600.0);

    modal.show(ctx, &mut show_flag, |ui| {
        // FOOTER
        if let Some((conf, canc)) = modal.ui_footer_actions(ui, i18n, |f| {
            if f.close() || f.cancel() {
                return Some((false, true));
            }
            if f.button("project-search-replace-confirm").clicked() {
                return Some((true, false));
            }
            None
        }) {
            confirm = conf;
            cancel = canc;
        }

        // BODY
        modal.ui_body(ui, |ui| {
            // Select all / Deselect all
            ui.horizontal(|ui| {
                if ui
                    .button(i18n.get("project-search-replace-select-all"))
                    .clicked()
                {
                    for p in &mut ws.project_search.replace_previews {
                        p.selected = true;
                    }
                }
                if ui
                    .button(i18n.get("project-search-replace-deselect-all"))
                    .clicked()
                {
                    for p in &mut ws.project_search.replace_previews {
                        p.selected = false;
                    }
                }
                ui.add_space(8.0);
                let mut sel_args = fluent_bundle::FluentArgs::new();
                sel_args.set("selected", selected_count as i64);
                sel_args.set("total", preview_count as i64);
                ui.label(i18n.get_args("project-search-replace-selection-info", &sel_args));
            });

            ui.add_space(4.0);
            ui.separator();

            let font_id = egui::FontId::monospace(12.0);
            let bg_added = egui::Color32::from_rgba_unmultiplied(40, 100, 40, 100);
            let bg_removed = egui::Color32::from_rgba_unmultiplied(120, 30, 30, 100);
            let fg_added = egui::Color32::from_rgb(150, 255, 150);
            let fg_removed = egui::Color32::from_rgb(255, 150, 150);
            let fg_normal = ui.visuals().text_color();

            egui::ScrollArea::vertical()
                .id_salt("replace_preview_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    for idx in 0..ws.project_search.replace_previews.len() {
                        let preview = &ws.project_search.replace_previews[idx];
                        let file_display = preview
                            .file
                            .strip_prefix(&ws.root_path)
                            .unwrap_or(&preview.file)
                            .to_string_lossy()
                            .to_string();
                        let match_count = preview.match_count;

                        // Per-file header: checkbox + filename + match count
                        ui.add_space(4.0);
                        let header_id = egui::Id::new(("replace_preview_file", idx));
                        let mut selected = preview.selected;
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut selected, "");
                            let mut match_args = fluent_bundle::FluentArgs::new();
                            match_args.set("file", file_display.clone());
                            match_args.set("count", match_count as i64);
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} ({} {})",
                                    file_display,
                                    match_count,
                                    if match_count == 1 { "match" } else { "matches" }
                                ))
                                .strong()
                                .size(12.0),
                            );
                        });
                        ws.project_search.replace_previews[idx].selected = selected;

                        // Collapsible diff sekce
                        let collapsing_id = header_id.with("diff");
                        egui::CollapsingHeader::new(egui::RichText::new("Diff").small())
                            .id_salt(collapsing_id)
                            .default_open(preview_count <= 5)
                            .show(ui, |ui| {
                                let preview = &ws.project_search.replace_previews[idx];
                                let diff = similar::TextDiff::from_lines(
                                    &preview.original_content,
                                    &preview.new_content,
                                );

                                for change in diff.iter_all_changes() {
                                    let (bg_color, fg_color, prefix) = match change.tag() {
                                        similar::ChangeTag::Delete => {
                                            (Some(bg_removed), fg_removed, "- ")
                                        }
                                        similar::ChangeTag::Insert => {
                                            (Some(bg_added), fg_added, "+ ")
                                        }
                                        similar::ChangeTag::Equal => (None, fg_normal, "  "),
                                    };

                                    let line_text = change.value();
                                    let mut layout_job = egui::text::LayoutJob::default();
                                    layout_job.append(
                                        &format!("{}{}", prefix, line_text),
                                        0.0,
                                        egui::text::TextFormat {
                                            font_id: font_id.clone(),
                                            color: fg_color,
                                            background: bg_color
                                                .unwrap_or(egui::Color32::TRANSPARENT),
                                            ..Default::default()
                                        },
                                    );

                                    ui.add(
                                        egui::Label::new(layout_job)
                                            .wrap_mode(egui::TextWrapMode::Extend),
                                    );
                                }
                            });

                        ui.separator();
                    }
                });
        });
    });

    if confirm {
        ws.project_search.show_replace_preview = false;
        ws.project_search.pending_replace = true;
    }
    if cancel || !show_flag {
        ws.project_search.show_replace_preview = false;
        ws.project_search.replace_previews.clear();
    }
}

/// Sestaví LayoutJob pro jeden řádek s match výsledkem.
/// Match ranges jsou zvýrazněny oranžovým pozadím.
fn build_match_layout_job(
    line_num: usize,
    text: &str,
    match_ranges: &[(usize, usize)],
    font_id: &egui::FontId,
    text_color: egui::Color32,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let prefix = format!("{:>5}: ", line_num);
    let highlight_bg = egui::Color32::from_rgba_unmultiplied(200, 130, 0, 120);

    // Prefix s číslem řádku — tlumená barva
    job.append(
        &prefix,
        0.0,
        egui::text::TextFormat {
            font_id: font_id.clone(),
            color: egui::Color32::GRAY,
            ..Default::default()
        },
    );

    // Text řádku se zvýrazněnými match ranges
    let mut last_end = 0;
    for &(start, end) in match_ranges {
        let start = start.min(text.len());
        let end = end.min(text.len());
        if start > last_end {
            // Nezdůrazněný text před matchem
            job.append(
                &text[last_end..start],
                0.0,
                egui::text::TextFormat {
                    font_id: font_id.clone(),
                    color: text_color,
                    ..Default::default()
                },
            );
        }
        if end > start {
            // Zvýrazněný match
            job.append(
                &text[start..end],
                0.0,
                egui::text::TextFormat {
                    font_id: font_id.clone(),
                    color: text_color,
                    background: highlight_bg,
                    ..Default::default()
                },
            );
        }
        last_end = end;
    }
    // Zbytek textu za posledním matchem
    if last_end < text.len() {
        job.append(
            &text[last_end..],
            0.0,
            egui::text::TextFormat {
                font_id: font_id.clone(),
                color: text_color,
                ..Default::default()
            },
        );
    }

    job
}

/// Sestaví LayoutJob pro kontextový řádek (bez zvýraznění).
fn build_context_layout_job(
    line_num: usize,
    text: &str,
    font_id: &egui::FontId,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let prefix = format!("{:>5}: ", line_num);
    let dim_color = egui::Color32::from_rgb(140, 140, 140);

    job.append(
        &prefix,
        0.0,
        egui::text::TextFormat {
            font_id: font_id.clone(),
            color: egui::Color32::GRAY,
            ..Default::default()
        },
    );
    job.append(
        text,
        0.0,
        egui::text::TextFormat {
            font_id: font_id.clone(),
            color: dim_color,
            ..Default::default()
        },
    );

    job
}

/// Sestaví regex z query a SearchOptions.
///
/// Režimy:
/// - regex mode: query se použije přímo
/// - plain mode: query se escapuje přes `regex::escape()`
/// - whole-word: obalí patternem `\b...\b`
/// - case: nastaví case_insensitive přes RegexBuilder
///
/// Vrací `Err(String)` s popisnou chybou při nevalidním patternu.
pub fn build_regex(query: &str, opts: &SearchOptions) -> Result<regex::Regex, String> {
    if query.is_empty() {
        return Err("Prázdný vyhledávací dotaz".to_string());
    }

    let base_pattern = if opts.use_regex {
        query.to_string()
    } else {
        regex::escape(query)
    };

    let pattern = if opts.whole_word {
        format!(r"\b{}\b", base_pattern)
    } else {
        base_pattern
    };

    RegexBuilder::new(&pattern)
        .case_insensitive(!opts.case_sensitive)
        .build()
        .map_err(|e| format!("Neplatný regex: {}", e))
}

/// Prohledá soubor a vrátí výsledky s kontextovými řádky.
///
/// `context_lines` určuje počet řádků kontextu před/za matchem.
/// Blízké matche (vzdálenost ≤ 2*context_lines) se sloučí do jednoho bloku.
pub fn search_file_with_context(
    path: &Path,
    regex: &regex::Regex,
    context_lines: usize,
) -> std::io::Result<Vec<SearchResult>> {
    let content = std::fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();
    let total = lines.len();

    // Najdi všechny matchující řádky s match_ranges
    let mut match_entries: Vec<(usize, Vec<(usize, usize)>)> = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        let ranges: Vec<(usize, usize)> = regex
            .find_iter(line)
            .map(|m| (m.start(), m.end()))
            .collect();
        if !ranges.is_empty() {
            match_entries.push((idx, ranges));
        }
    }

    if match_entries.is_empty() {
        return Ok(Vec::new());
    }

    // Sloučení blízkých matchů do bloků
    // Blok = skupina matchů kde vzdálenost mezi sousedními je ≤ 2*context_lines
    let mut blocks: Vec<Vec<usize>> = Vec::new();
    let mut current_block: Vec<usize> = vec![0]; // indexy do match_entries

    for i in 1..match_entries.len() {
        let prev_line = match_entries[i - 1].0;
        let curr_line = match_entries[i].0;
        if curr_line - prev_line <= 2 * context_lines {
            current_block.push(i);
        } else {
            blocks.push(current_block);
            current_block = vec![i];
        }
    }
    blocks.push(current_block);

    // Pro každý blok vytvoříme SearchResult s kontextem
    let mut results = Vec::new();
    for block in &blocks {
        // Pro každý match v bloku generujeme samostatný SearchResult,
        // ale kontext se přizpůsobí bloku (sloučené matche sdílejí kontext)
        let block_first_line = match_entries[block[0]].0;
        let block_last_line = match_entries[*block.last().unwrap()].0;

        let ctx_start = block_first_line.saturating_sub(context_lines);
        let ctx_end = (block_last_line + context_lines).min(total.saturating_sub(1));

        for &entry_idx in block {
            let (line_idx, ref ranges) = match_entries[entry_idx];

            let context_before: Vec<String> = (ctx_start..line_idx)
                .filter(|&l| !block.iter().any(|&bi| match_entries[bi].0 == l) || l == line_idx)
                .take_while(|&l| l < line_idx)
                .map(|l| lines[l].to_string())
                .collect();

            let context_after: Vec<String> = ((line_idx + 1)..=ctx_end)
                .filter(|&l| !block.iter().any(|&bi| match_entries[bi].0 == l))
                .map(|l| lines[l].to_string())
                .collect();

            results.push(SearchResult {
                file: path.to_path_buf(),
                line: line_idx + 1,
                text: lines[line_idx].to_string(),
                match_ranges: ranges.clone(),
                context_before,
                context_after,
            });
        }
    }

    Ok(results)
}

pub fn run_project_search(
    root: PathBuf,
    files: Arc<Vec<PathBuf>>,
    query: String,
    options: SearchOptions,
    epoch: u64,
    cancel_epoch: Arc<std::sync::atomic::AtomicU64>,
) -> mpsc::Receiver<SearchBatch> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        // Sestav regex
        let regex = match build_regex(&query, &options) {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(SearchBatch::Error(e));
                return;
            }
        };

        // File filter přes globset
        let glob_matcher = if options.file_filter.trim().is_empty() {
            None
        } else {
            match globset::Glob::new(options.file_filter.trim()) {
                Ok(g) => Some(g.compile_matcher()),
                Err(e) => {
                    let _ = tx.send(SearchBatch::Error(format!("Neplatný file filter: {}", e)));
                    return;
                }
            }
        };

        let mut total_results = 0usize;

        for path in files.iter() {
            if cancel_epoch.load(Ordering::Relaxed) > epoch {
                return;
            }

            // Aplikuj file filter
            if let Some(ref matcher) = glob_matcher {
                let file_name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let path_str = path.to_string_lossy();
                if !matcher.is_match(&*file_name) && !matcher.is_match(&*path_str) {
                    continue;
                }
            }

            let full_path = root.join(path);
            match search_file_with_context(&full_path, &regex, 2) {
                Ok(file_results) => {
                    if !file_results.is_empty() {
                        total_results += file_results.len();
                        // Přepiš cesty na relativní
                        let relative_results: Vec<SearchResult> = file_results
                            .into_iter()
                            .map(|mut r| {
                                r.file = path.clone();
                                r
                            })
                            .collect();
                        let _ = tx.send(SearchBatch::Results(relative_results));
                    }
                }
                Err(e) => {
                    // I/O chyba — loguj, ale pokračuj
                    eprintln!("Search: chyba čtení {}: {}", full_path.display(), e);
                }
            }

            if total_results > 1000 {
                break;
            }
        }
        let _ = tx.send(SearchBatch::Done);
    });
    rx
}

pub fn collect_project_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let mut dirs = vec![root.to_path_buf()];

    while let Some(dir) = dirs.pop() {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }
                if path.is_dir() {
                    dirs.push(path);
                } else if let Ok(rel) = path.strip_prefix(root) {
                    files.push(rel.to_path_buf());
                }
            }
        }
        if files.len() > 5000 {
            break;
        }
    }
    files
}

// ---------------------------------------------------------------------------
// Replace engine
// ---------------------------------------------------------------------------

use crate::app::ui::workspace::state::ReplacePreview;

/// Spočítá replace preview pro každý unikátní soubor z výsledků vyhledávání.
///
/// Pro každý soubor načte obsah z disku, provede `regex.replace_all()`,
/// a pokud se obsah liší, vygeneruje `ReplacePreview`. Capture groups ($1, $2)
/// fungují automaticky díky `Regex::replace_all`.
pub fn compute_replace_previews(
    root: &Path,
    results: &[SearchResult],
    regex: &regex::Regex,
    replace_text: &str,
) -> std::io::Result<Vec<ReplacePreview>> {
    // Deduplikovat cesty — zachovat pořadí prvního výskytu
    let mut seen = std::collections::HashSet::new();
    let mut unique_files: Vec<&PathBuf> = Vec::new();
    for r in results {
        if seen.insert(&r.file) {
            unique_files.push(&r.file);
        }
    }

    let mut previews = Vec::new();
    for rel_path in &unique_files {
        let full_path = root.join(rel_path);
        let original_content = std::fs::read_to_string(&full_path)?;
        let new_content = regex
            .replace_all(&original_content, replace_text)
            .into_owned();
        if new_content != original_content {
            // Spočítej počet matchů v souboru
            let match_count = regex.find_iter(&original_content).count();
            previews.push(ReplacePreview {
                file: full_path,
                original_content,
                new_content,
                match_count,
                selected: true,
            });
        }
    }

    Ok(previews)
}

/// Aplikuje nahrazení — zapíše `new_content` do souborů kde `selected == true`.
///
/// Vrací vektor výsledků per-file. Selhání jednoho souboru neblokuje ostatní.
pub fn apply_replacements(previews: &[ReplacePreview]) -> Vec<(PathBuf, Result<(), String>)> {
    let mut results = Vec::new();
    for preview in previews {
        if !preview.selected {
            continue;
        }
        let result = std::fs::write(&preview.file, &preview.new_content)
            .map_err(|e| format!("{}: {}", preview.file.display(), e));
        results.push((preview.file.clone(), result));
    }
    results
}

// ---------------------------------------------------------------------------
// Unit testy — search engine
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn default_opts() -> SearchOptions {
        SearchOptions::default()
    }

    // -----------------------------------------------------------------------
    // build_regex — 8 toggle kombinací + error + prázdný query
    // -----------------------------------------------------------------------

    #[test]
    fn build_regex_plain_case_insensitive() {
        let opts = SearchOptions {
            use_regex: false,
            case_sensitive: false,
            whole_word: false,
            ..default_opts()
        };
        let re = build_regex("Hello", &opts).unwrap();
        assert!(re.is_match("hello world"));
        assert!(re.is_match("HELLO WORLD"));
        assert!(re.is_match("say hello"));
    }

    #[test]
    fn build_regex_plain_case_sensitive() {
        let opts = SearchOptions {
            use_regex: false,
            case_sensitive: true,
            whole_word: false,
            ..default_opts()
        };
        let re = build_regex("Hello", &opts).unwrap();
        assert!(re.is_match("Hello world"));
        assert!(!re.is_match("hello world"));
    }

    #[test]
    fn build_regex_plain_whole_word() {
        let opts = SearchOptions {
            use_regex: false,
            case_sensitive: false,
            whole_word: true,
            ..default_opts()
        };
        let re = build_regex("test", &opts).unwrap();
        assert!(re.is_match("run test now"));
        assert!(!re.is_match("testing now"));
    }

    #[test]
    fn build_regex_plain_case_sensitive_whole_word() {
        let opts = SearchOptions {
            use_regex: false,
            case_sensitive: true,
            whole_word: true,
            ..default_opts()
        };
        let re = build_regex("Test", &opts).unwrap();
        assert!(re.is_match("run Test now"));
        assert!(!re.is_match("run test now"));
        assert!(!re.is_match("Testing now"));
    }

    #[test]
    fn build_regex_regex_mode_case_insensitive() {
        let opts = SearchOptions {
            use_regex: true,
            case_sensitive: false,
            whole_word: false,
            ..default_opts()
        };
        let re = build_regex(r"fn\s+\w+", &opts).unwrap();
        assert!(re.is_match("fn main()"));
        assert!(re.is_match("FN Main()"));
    }

    #[test]
    fn build_regex_regex_mode_case_sensitive() {
        let opts = SearchOptions {
            use_regex: true,
            case_sensitive: true,
            whole_word: false,
            ..default_opts()
        };
        let re = build_regex(r"fn\s+\w+", &opts).unwrap();
        assert!(re.is_match("fn main()"));
        assert!(!re.is_match("FN Main()"));
    }

    #[test]
    fn build_regex_regex_mode_whole_word() {
        let opts = SearchOptions {
            use_regex: true,
            case_sensitive: false,
            whole_word: true,
            ..default_opts()
        };
        let re = build_regex("test", &opts).unwrap();
        assert!(re.is_match("run test now"));
        assert!(!re.is_match("testing now"));
    }

    #[test]
    fn build_regex_regex_mode_case_sensitive_whole_word() {
        let opts = SearchOptions {
            use_regex: true,
            case_sensitive: true,
            whole_word: true,
            ..default_opts()
        };
        let re = build_regex("Test", &opts).unwrap();
        assert!(re.is_match("run Test now"));
        assert!(!re.is_match("run test now"));
    }

    #[test]
    fn build_regex_invalid_pattern() {
        let opts = SearchOptions {
            use_regex: true,
            case_sensitive: false,
            whole_word: false,
            ..default_opts()
        };
        let result = build_regex("[invalid", &opts);
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(!err_msg.is_empty(), "Chybová hláška nesmí být prázdná");
        assert!(
            err_msg.contains("Neplatný regex"),
            "Chybová hláška by měla obsahovat prefix: {}",
            err_msg
        );
    }

    #[test]
    fn build_regex_empty_query() {
        let result = build_regex("", &default_opts());
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // search_file_with_context — match + kontext + sloučení + žádný match
    // -----------------------------------------------------------------------

    fn create_temp_file(content: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f.flush().unwrap();
        f
    }

    #[test]
    fn search_file_simple_match_with_context() {
        let content = "řádek 1\nřádek 2\nřádek 3\nhledaný text\nřádek 5\nřádek 6\nřádek 7\n";
        let f = create_temp_file(content);
        let re = regex::Regex::new("hledaný").unwrap();
        let results = search_file_with_context(f.path(), &re, 2).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line, 4);
        assert!(results[0].text.contains("hledaný text"));
        assert_eq!(results[0].match_ranges.len(), 1);
        // Kontext: 2 řádky před a 2 za
        assert_eq!(results[0].context_before.len(), 2);
        assert_eq!(results[0].context_after.len(), 2);
        assert!(results[0].context_before[0].contains("řádek 2"));
        assert!(results[0].context_before[1].contains("řádek 3"));
        assert!(results[0].context_after[0].contains("řádek 5"));
        assert!(results[0].context_after[1].contains("řádek 6"));
    }

    #[test]
    fn search_file_close_matches_merged() {
        // Dva matche na řádcích 3 a 5 (vzdálenost 2 ≤ 2*2=4) → sloučí se do jednoho bloku
        let content = "a\nb\nmatch1\nd\nmatch2\nf\ng\n";
        let f = create_temp_file(content);
        let re = regex::Regex::new("match").unwrap();
        let results = search_file_with_context(f.path(), &re, 2).unwrap();

        assert_eq!(results.len(), 2, "Dva matche v sloučeném bloku");
        // Obě výsledky by měly sdílet rozšířený kontext
        // match1 na řádku 3 — kontext začíná od řádku 1 (3-2=1)
        assert!(results[0].context_before.len() >= 2);
        // match2 na řádku 5 — kontext končí řádkem 7 (5+2=7), ale soubor má 7 řádků (0-6)
        // match2.context_after by měl zahrnovat řádky po match2 co nejsou match
        assert!(!results[1].context_after.is_empty() || results[1].line == 5);
    }

    #[test]
    fn search_file_no_match() {
        let content = "aaa\nbbb\nccc\n";
        let f = create_temp_file(content);
        let re = regex::Regex::new("xyz").unwrap();
        let results = search_file_with_context(f.path(), &re, 2).unwrap();
        assert!(results.is_empty());
    }

    // -----------------------------------------------------------------------
    // File filter — glob matching
    // -----------------------------------------------------------------------

    #[test]
    fn file_filter_glob_matches() {
        let matcher = globset::Glob::new("*.rs").unwrap().compile_matcher();
        assert!(matcher.is_match("main.rs"));
        assert!(matcher.is_match("lib.rs"));
        assert!(!matcher.is_match("readme.md"));
    }

    #[test]
    fn file_filter_glob_no_match() {
        let matcher = globset::Glob::new("*.py").unwrap().compile_matcher();
        assert!(!matcher.is_match("main.rs"));
        assert!(!matcher.is_match("lib.rs"));
    }

    // -----------------------------------------------------------------------
    // Replace engine — compute_replace_previews + apply_replacements
    // -----------------------------------------------------------------------

    #[test]
    fn test_compute_replace_previews_basic() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // Vytvoř 2 soubory s "foo"
        std::fs::write(root.join("a.txt"), "foo bar foo\n").unwrap();
        std::fs::write(root.join("b.txt"), "hello foo world\n").unwrap();

        let re = regex::Regex::new("foo").unwrap();

        // Simuluj SearchResult — stačí file + dummy data
        let results = vec![
            SearchResult {
                file: PathBuf::from("a.txt"),
                line: 1,
                text: "foo bar foo".to_string(),
                match_ranges: vec![(0, 3), (8, 11)],
                context_before: vec![],
                context_after: vec![],
            },
            SearchResult {
                file: PathBuf::from("b.txt"),
                line: 1,
                text: "hello foo world".to_string(),
                match_ranges: vec![(6, 9)],
                context_before: vec![],
                context_after: vec![],
            },
        ];

        let previews = compute_replace_previews(root, &results, &re, "bar").unwrap();
        assert_eq!(previews.len(), 2);

        assert_eq!(previews[0].new_content, "bar bar bar\n");
        assert_eq!(previews[0].match_count, 2);
        assert!(previews[0].selected);

        assert_eq!(previews[1].new_content, "hello bar world\n");
        assert_eq!(previews[1].match_count, 1);
        assert!(previews[1].selected);
    }

    #[test]
    fn test_compute_replace_previews_regex_capture() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        std::fs::write(root.join("c.txt"), "fn old_name() {}\nfn other_name() {}\n").unwrap();

        // Regex s capture group: zachytí jméno funkce
        let re = regex::Regex::new(r"fn (\w+)_name").unwrap();

        let results = vec![SearchResult {
            file: PathBuf::from("c.txt"),
            line: 1,
            text: "fn old_name() {}".to_string(),
            match_ranges: vec![(0, 11)],
            context_before: vec![],
            context_after: vec![],
        }];

        // Replace s $1 — zachová captured group
        let previews = compute_replace_previews(root, &results, &re, "fn ${1}_func").unwrap();
        assert_eq!(previews.len(), 1);
        assert_eq!(
            previews[0].new_content,
            "fn old_func() {}\nfn other_func() {}\n"
        );
        assert_eq!(previews[0].match_count, 2);
    }

    #[test]
    fn test_apply_replacements_success() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        let file_path = root.join("test.txt");
        std::fs::write(&file_path, "original content").unwrap();

        let previews = vec![ReplacePreview {
            file: file_path.clone(),
            original_content: "original content".to_string(),
            new_content: "replaced content".to_string(),
            match_count: 1,
            selected: true,
        }];

        let results = apply_replacements(&previews);
        assert_eq!(results.len(), 1);
        assert!(results[0].1.is_ok());

        // Ověřit obsah souboru na disku
        let actual = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(actual, "replaced content");
    }

    #[test]
    fn test_apply_replacements_partial_skip() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        let file_a = root.join("a.txt");
        let file_b = root.join("b.txt");
        std::fs::write(&file_a, "original A").unwrap();
        std::fs::write(&file_b, "original B").unwrap();

        let previews = vec![
            ReplacePreview {
                file: file_a.clone(),
                original_content: "original A".to_string(),
                new_content: "replaced A".to_string(),
                match_count: 1,
                selected: true,
            },
            ReplacePreview {
                file: file_b.clone(),
                original_content: "original B".to_string(),
                new_content: "replaced B".to_string(),
                match_count: 1,
                selected: false, // Přeskočit
            },
        ];

        let results = apply_replacements(&previews);
        // Jen vybrané soubory se zapisují
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, file_a);
        assert!(results[0].1.is_ok());

        // file_a — nahrazeno
        assert_eq!(std::fs::read_to_string(&file_a).unwrap(), "replaced A");
        // file_b — nezměněno
        assert_eq!(std::fs::read_to_string(&file_b).unwrap(), "original B");
    }

    #[test]
    fn test_apply_replacements_nonexistent_file_error() {
        // Neexistující soubor vrací Err, ale neblokuje ostatní soubory
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        let good_file = root.join("good.txt");
        std::fs::write(&good_file, "original").unwrap();
        let bad_file = root.join("nonexistent").join("bad.txt");

        let previews = vec![
            ReplacePreview {
                file: bad_file.clone(),
                original_content: "x".to_string(),
                new_content: "y".to_string(),
                match_count: 1,
                selected: true,
            },
            ReplacePreview {
                file: good_file.clone(),
                original_content: "original".to_string(),
                new_content: "replaced".to_string(),
                match_count: 1,
                selected: true,
            },
        ];

        let results = apply_replacements(&previews);
        assert_eq!(results.len(), 2, "Oba soubory se zpracují");

        // bad_file → Err s popisnou zprávou
        assert!(results[0].1.is_err());
        let err_msg = results[0].1.as_ref().unwrap_err();
        assert!(
            err_msg.contains("bad.txt"),
            "Chyba by měla obsahovat název souboru: {}",
            err_msg
        );

        // good_file → Ok
        assert!(results[1].1.is_ok());
        assert_eq!(std::fs::read_to_string(&good_file).unwrap(), "replaced");
    }
}
