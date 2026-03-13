use std::path::PathBuf;

use eframe::egui;
use similar::{ChangeTag, TextDiff};
use xxhash_rust::xxh3::xxh3_64;

use crate::app::local_history::{HistoryEntry, LocalHistory};
use crate::app::ui::widgets::modal::{ModalResult, show_modal};
use crate::highlighter::Highlighter;
use crate::i18n::I18n;

/// Převede UNIX timestamp (sekundy) na lokální čas přes libc::localtime_r.
/// Vrací (rok, měsíc, den, hodiny, minuty, sekundy).
fn unix_ts_to_local(ts: u64) -> (i32, u32, u32, u32, u32, u32) {
    let time_t = ts as libc::time_t;
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    unsafe {
        libc::localtime_r(&time_t, &mut tm);
    }
    (
        tm.tm_year + 1900,
        (tm.tm_mon + 1) as u32,
        tm.tm_mday as u32,
        tm.tm_hour as u32,
        tm.tm_min as u32,
        tm.tm_sec as u32,
    )
}

// ── Datové struktury ──────────────────────────────────────────────

/// Jeden řádek diff výstupu s tagem operace a vlastněným textem.
#[derive(Debug, Clone)]
pub struct DiffLine {
    pub tag: ChangeTag,
    pub text: String,
}

/// Barvy pro diff rendering — dark/light mode.
pub struct DiffColors {
    pub bg_added: egui::Color32,
    pub bg_removed: egui::Color32,
    pub fg_added: egui::Color32,
    pub fg_removed: egui::Color32,
    pub fg_normal: egui::Color32,
}

/// Zdroj posledního scroll eventu — pro synchronizaci panelů.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollSource {
    Left,
    Right,
    None,
}

/// Výsledek renderování history split view.
pub struct HistorySplitResult {
    /// Uživatel zavřel history view.
    pub close: bool,
    /// Obsah levého panelu se změnil (editací).
    pub content_changed: bool,
    /// Uživatel potvrdil obnovení vybrané historické verze.
    pub restore_confirmed: bool,
}

/// Výstup funkce `build_panel_texts` — texty a diff mapy pro oba panely.
pub struct PanelTexts {
    /// Text levého panelu (Equal + Insert řádky).
    pub left_text: String,
    /// Text pravého panelu (Equal + Delete řádky).
    pub right_text: String,
    /// Per-řádek diff tag pro levý panel.
    pub left_diff_map: Vec<ChangeTag>,
    /// Per-řádek diff tag pro pravý panel.
    pub right_diff_map: Vec<ChangeTag>,
}

/// Stav otevřeného history split view pro konkrétní soubor.
pub struct HistoryViewState {
    /// Absolutní cesta k souboru.
    pub file_path: PathBuf,
    /// Relativní cesta vůči kořeni projektu.
    pub relative_path: PathBuf,
    /// Seznam historických verzí (nejnovější první).
    pub entries: Vec<HistoryEntry>,
    /// Index aktuálně vybrané verze v seznamu.
    pub selected_index: Option<usize>,
    /// Obsah aktuální verze (načtený jednou při otevření).
    pub current_content: String,
    /// UNIX timestamp poslední modifikace aktuálního souboru (mtime).
    pub current_file_mtime: Option<u64>,
    /// Cachovaný diff výsledek.
    pub cached_diff: Option<Vec<DiffLine>>,
    /// Index verze, pro kterou je diff cachovaný (pro invalidaci).
    pub diff_for_index: Option<usize>,
    /// Poměr šířky levého/pravého panelu (0.0–1.0).
    pub split_ratio: f32,
    /// Hash obsahu levého panelu pro invalidaci diff cache.
    pub content_hash: u64,
    /// Scroll offset levého panelu (vertikální).
    pub left_scroll_y: f32,
    /// Scroll offset pravého panelu (vertikální).
    pub right_scroll_y: f32,
    /// Zdroj posledního scroll eventu (pro sync).
    pub scroll_source: ScrollSource,
    /// Obsah pravého panelu (Equal+Delete řádky ze history).
    pub right_panel_text: String,
    /// Mapa: řádek (0-based) → DiffTag pro levý panel.
    pub left_diff_map: Vec<ChangeTag>,
    /// Mapa: řádek (0-based) → DiffTag pro pravý panel.
    pub right_diff_map: Vec<ChangeTag>,
    /// Zobrazit potvrzovací dialog pro obnovení historické verze.
    pub show_restore_confirm: bool,
}

// ── Pomocné funkce ────────────────────────────────────────────────

/// Formátuje UNIX timestamp jako datum a čas v lokálním čase.
fn format_timestamp(ts: u64) -> String {
    let (year, month, day, hours, minutes, seconds) = unix_ts_to_local(ts);
    format!(
        "{:02}.{:02}.{} {:02}:{:02}:{:02}",
        day, month, year, hours, minutes, seconds
    )
}

/// Spočítá řádkový diff mezi historickou a aktuální verzí.
/// `historical` je "old", `current` je "new" — takže Insert = přidáno v aktuální,
/// Delete = odebráno (existovalo jen v historické).
pub fn compute_diff(current: &str, historical: &str) -> Vec<DiffLine> {
    let diff = TextDiff::from_lines(historical, current);
    diff.iter_all_changes()
        .map(|change| DiffLine {
            tag: change.tag(),
            text: change.value().to_string(),
        })
        .collect()
}

/// Vrací barvy pro diff rendering podle dark/light mode.
pub fn diff_colors(dark_mode: bool) -> DiffColors {
    if dark_mode {
        DiffColors {
            bg_added: egui::Color32::from_rgba_unmultiplied(40, 100, 40, 100),
            bg_removed: egui::Color32::from_rgba_unmultiplied(120, 30, 30, 100),
            fg_added: egui::Color32::from_rgb(150, 255, 150),
            fg_removed: egui::Color32::from_rgb(255, 150, 150),
            fg_normal: egui::Color32::from_rgb(220, 220, 220),
        }
    } else {
        DiffColors {
            bg_added: egui::Color32::from_rgba_unmultiplied(200, 240, 200, 255),
            bg_removed: egui::Color32::from_rgba_unmultiplied(255, 210, 210, 255),
            fg_added: egui::Color32::from_rgb(0, 100, 0),
            fg_removed: egui::Color32::from_rgb(150, 0, 0),
            fg_normal: egui::Color32::from_rgb(30, 30, 30),
        }
    }
}

/// Sestaví texty a diff mapy pro oba panely z diff výstupu.
///
/// Levý panel obsahuje Equal + Insert řádky, pravý Equal + Delete.
/// Diff mapy mapují index řádku v příslušném panelu na `ChangeTag`.
pub fn build_panel_texts(diff_lines: &[DiffLine]) -> PanelTexts {
    let mut left_text = String::new();
    let mut right_text = String::new();
    let mut left_diff_map: Vec<ChangeTag> = Vec::new();
    let mut right_diff_map: Vec<ChangeTag> = Vec::new();

    for line in diff_lines {
        match line.tag {
            ChangeTag::Equal => {
                left_text.push_str(&line.text);
                right_text.push_str(&line.text);
                left_diff_map.push(ChangeTag::Equal);
                right_diff_map.push(ChangeTag::Equal);
            }
            ChangeTag::Insert => {
                // Insert = nový řádek v aktuální verzi → levý panel
                left_text.push_str(&line.text);
                left_diff_map.push(ChangeTag::Insert);
            }
            ChangeTag::Delete => {
                // Delete = řádek existoval jen v historické verzi → pravý panel
                right_text.push_str(&line.text);
                right_diff_map.push(ChangeTag::Delete);
            }
        }
    }

    PanelTexts {
        left_text,
        right_text,
        left_diff_map,
        right_diff_map,
    }
}

/// Vrací byte offset začátku každého řádku v textu.
///
/// Řádek 0 začíná na offset 0. Řádek N začíná na byte za (N-1)-tým `\n`.
/// Pro prázdný text vrátí `vec![0]` (jeden řádek začínající na offsetu 0).
pub fn compute_line_offsets(text: &str) -> Vec<usize> {
    let mut offsets = vec![0usize];
    for (i, byte) in text.as_bytes().iter().enumerate() {
        if *byte == b'\n' {
            offsets.push(i + 1);
        }
    }
    offsets
}

/// Pro daný byte offset najde index řádku (0-based) pomocí binary search v line_offsets.
fn byte_offset_to_line(line_offsets: &[usize], byte_offset: usize) -> usize {
    match line_offsets.binary_search(&byte_offset) {
        Ok(idx) => idx,
        // partition_point vrací index prvního prvku > byte_offset, takže řádek je idx - 1
        Err(idx) => idx.saturating_sub(1),
    }
}

/// Aplikuje diff background barvy na sections v `LayoutJob`.
///
/// Pro každou section zjistí řádek z `byte_range.start` (binary search v line_offsets),
/// a pokud diff_map pro ten řádek != Equal, nastaví `section.format.background`
/// na příslušnou diff barvu. Equal řádky ponechá beze změny.
pub fn apply_diff_backgrounds(
    job: &mut egui::text::LayoutJob,
    text: &str,
    diff_map: &[ChangeTag],
    colors: &DiffColors,
) {
    if diff_map.is_empty() {
        return;
    }

    let line_offsets = compute_line_offsets(text);

    for section in &mut job.sections {
        let byte_start = section.byte_range.start;
        let line_idx = byte_offset_to_line(&line_offsets, byte_start);

        if let Some(&tag) = diff_map.get(line_idx) {
            match tag {
                ChangeTag::Insert => {
                    section.format.background = colors.bg_added;
                }
                ChangeTag::Delete => {
                    section.format.background = colors.bg_removed;
                }
                ChangeTag::Equal => {
                    // Beze změny — ponechat existující background (syntax highlighting)
                }
            }
        }
    }
}

/// Spočítá content hash pro invalidaci diff cache.
pub fn content_hash(text: &str) -> u64 {
    xxh3_64(text.as_bytes())
}

// ── Hlavní renderovací funkce ─────────────────────────────────────

/// Renderuje history split view s toolbar a dvěma diff panely.
///
/// Levý panel je editovatelný `TextEdit` se syntax highlighting + diff background.
/// Pravý panel je read-only `Label` s `LayoutJob` se syntax highlighting + diff background.
/// Scroll je synchronizovaný proportionálně.
#[allow(clippy::too_many_arguments)]
pub fn render_history_split_view(
    history_view: &mut HistoryViewState,
    local_history: &LocalHistory,
    ui: &mut egui::Ui,
    i18n: &I18n,
    highlighter: &Highlighter,
    theme_name: &str,
    ext: &str,
    fname: &str,
    font_size: f32,
) -> HistorySplitResult {
    let mut result = HistorySplitResult {
        close: false,
        content_changed: false,
        restore_confirmed: false,
    };

    let file_name = history_view
        .file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "???".to_string());

    let entry_count = history_view.entries.len();

    // ── Toolbar ───────────────────────────────────────────────────
    ui.horizontal(|ui| {
        // Nadpis souboru
        let mut args = fluent_bundle::FluentArgs::new();
        args.set("name", file_name.as_str());
        ui.heading(i18n.get_args("history-panel-title", &args));

        ui.add_space(16.0);

        // Info o vybrané verzi
        if let Some(idx) = history_view.selected_index
            && let Some(entry) = history_view.entries.get(idx)
        {
            let date_str = format_timestamp(entry.timestamp);
            let mut ver_args = fluent_bundle::FluentArgs::new();
            ver_args.set("date", date_str.as_str());
            ui.label(i18n.get_args("history-version-info", &ver_args));
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Zavírací tlačítko
            if ui.button(i18n.get("history-panel-close")).clicked() {
                result.close = true;
            }

            ui.add_space(8.0);

            // Navigační šipky: → = novější (index-1), ← = starší (index+1)
            let selected = history_view.selected_index.unwrap_or(0);

            // → Novější
            let newer_enabled = selected > 0;
            let newer_btn = ui
                .add_enabled(
                    newer_enabled,
                    egui::Button::new("→").min_size(egui::vec2(28.0, 20.0)),
                )
                .on_hover_text(i18n.get("history-nav-newer"));
            if newer_btn.clicked() && newer_enabled {
                history_view.selected_index = Some(selected - 1);
            }

            // ← Starší
            let older_enabled = selected + 1 < entry_count;
            let older_btn = ui
                .add_enabled(
                    older_enabled,
                    egui::Button::new("←").min_size(egui::vec2(28.0, 20.0)),
                )
                .on_hover_text(i18n.get("history-nav-older"));
            if older_btn.clicked() && older_enabled {
                history_view.selected_index = Some(selected + 1);
            }

            ui.add_space(8.0);

            // Tlačítko "Obnovit" — aktivní jen pokud je vybraná verze
            if ui
                .add_enabled(
                    history_view.selected_index.is_some(),
                    egui::Button::new(i18n.get("history-restore-btn")),
                )
                .clicked()
            {
                history_view.show_restore_confirm = true;
            }
        });
    });
    ui.separator();

    // ── Confirm dialog pro obnovení historické verze ──────────
    if history_view.show_restore_confirm {
        let modal_result = show_modal(
            ui.ctx(),
            "history_restore_confirm",
            &i18n.get("history-restore-confirm-title"),
            &i18n.get("history-restore-confirm-ok"),
            &i18n.get("history-restore-confirm-cancel"),
            |ui| {
                ui.label(i18n.get("history-restore-confirm-text"));
                Some(())
            },
        );
        match modal_result {
            ModalResult::Confirmed(()) => {
                result.restore_confirmed = true;
                history_view.show_restore_confirm = false;
            }
            ModalResult::Cancelled => {
                history_view.show_restore_confirm = false;
            }
            ModalResult::Pending => {}
        }
    }

    if history_view.entries.is_empty() {
        ui.label(i18n.get("history-panel-no-versions"));
        return result;
    }

    // ── Diff cache ────────────────────────────────────────────────
    // Invalidace při změně vybrané verze NEBO změně obsahu (editace).
    let selected_idx = history_view.selected_index.unwrap_or(0);
    let current_hash = content_hash(&history_view.current_content);
    let need_recompute = history_view.diff_for_index != Some(selected_idx)
        || history_view.content_hash != current_hash;

    if need_recompute {
        // Načíst obsah historické verze
        let historical_content = if let Some(entry) = history_view.entries.get(selected_idx) {
            match local_history.get_snapshot_content(&history_view.relative_path, entry) {
                Ok(content) => content,
                Err(e) => format!("Chyba čtení: {}", e),
            }
        } else {
            String::new()
        };

        let diff = compute_diff(&history_view.current_content, &historical_content);
        let panels = build_panel_texts(&diff);

        // Uložit výsledky — current_content slouží přímo jako levý panel text
        history_view.right_panel_text = panels.right_text;
        history_view.left_diff_map = panels.left_diff_map;
        history_view.right_diff_map = panels.right_diff_map;
        history_view.cached_diff = Some(diff);
        history_view.diff_for_index = Some(selected_idx);
        history_view.content_hash = current_hash;
    }

    let dark_mode = ui.visuals().dark_mode;
    let colors = diff_colors(dark_mode);
    let bg = highlighter.background_color(theme_name);

    // ── Split view ────────────────────────────────────────────────
    let available = ui.available_size();
    let handle_size = 6.0_f32;
    let usable_width = (available.x - handle_size).max(100.0);
    let left_w = (usable_width * history_view.split_ratio).clamp(50.0, usable_width - 50.0);
    let right_w = usable_width - left_w;
    let panel_height = available.y;

    // Labely panelů s časem úpravy
    let current_label = {
        let base = i18n.get("history-current-label");
        if let Some(mtime) = history_view.current_file_mtime {
            format!("{} — {}", base, format_timestamp(mtime))
        } else {
            base.to_string()
        }
    };
    let historical_label = {
        let base = i18n.get("history-historical-label");
        if let Some(entry) = history_view
            .selected_index
            .and_then(|idx| history_view.entries.get(idx))
        {
            format!("{} — {}", base, format_timestamp(entry.timestamp))
        } else {
            base.to_string()
        }
    };

    // Label + separator zaberou cca 26px výšky
    let label_h = 26.0_f32;
    let scroll_height = (panel_height - label_h).max(50.0);

    // Alokujeme celý zbývající prostor a renderujeme manuálně
    let (split_rect, _) =
        ui.allocate_exact_size(egui::vec2(available.x, panel_height), egui::Sense::hover());

    // Rect pro levý panel
    let left_rect = egui::Rect::from_min_size(split_rect.min, egui::vec2(left_w, panel_height));
    // Rect pro handle
    let handle_rect = egui::Rect::from_min_size(
        egui::pos2(split_rect.min.x + left_w, split_rect.min.y),
        egui::vec2(handle_size, panel_height),
    );
    // Rect pro pravý panel
    let right_rect = egui::Rect::from_min_size(
        egui::pos2(split_rect.min.x + left_w + handle_size, split_rect.min.y),
        egui::vec2(right_w, panel_height),
    );

    // Snapshot diff map a right_panel_text pro closure capture (closure nemůže
    // borrowovat &mut history_view a zároveň ho layouter potřebuje immutable).
    let left_diff_map = history_view.left_diff_map.clone();
    let right_diff_map = history_view.right_diff_map.clone();
    let right_panel_text = history_view.right_panel_text.clone();

    // ── Levý panel (aktuální verze — editovatelný, Equal + Insert) ──
    let mut left_ui = ui.new_child(egui::UiBuilder::new().max_rect(left_rect));
    left_ui.set_clip_rect(left_rect);
    left_ui.label(egui::RichText::new(&*current_label).strong().small());
    left_ui.separator();

    let left_colors = DiffColors {
        bg_added: colors.bg_added,
        bg_removed: colors.bg_removed,
        fg_added: colors.fg_added,
        fg_removed: colors.fg_removed,
        fg_normal: colors.fg_normal,
    };

    let left_scroll_output = egui::ScrollArea::both()
        .id_salt("history_split_left")
        .auto_shrink([false, false])
        .max_height(scroll_height)
        .vertical_scroll_offset(history_view.left_scroll_y)
        .show(&mut left_ui, |ui| {
            let frame = egui::Frame::new()
                .fill(bg)
                .inner_margin(egui::Margin::same(4));
            frame.show(ui, |ui| {
                let ldm = &left_diff_map;
                let lc = &left_colors;
                let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                    let job_arc = highlighter.highlight(text, ext, fname, font_size, theme_name);
                    let mut job = (*job_arc).clone();
                    job.wrap.max_width = wrap_width;
                    apply_diff_backgrounds(&mut job, text, ldm, lc);
                    ui.fonts(|f| f.layout_job(job))
                };

                let te_output = egui::TextEdit::multiline(&mut history_view.current_content)
                    .id(egui::Id::new("history_left_edit"))
                    .font(egui::TextStyle::Monospace)
                    .code_editor()
                    .desired_width(f32::INFINITY)
                    .layouter(&mut layouter)
                    .show(ui);

                if te_output.response.changed() {
                    // Obsah se změnil — invalidace hash (diff se přepočítá příští frame)
                    history_view.content_hash = content_hash(&history_view.current_content);
                    result.content_changed = true;
                }
            });
        });

    // ── Resize handle ─────────────────────────────────────────
    let handle_response = ui.interact(
        handle_rect,
        ui.id().with("history_handle"),
        egui::Sense::drag(),
    );
    let handle_color = if handle_response.hovered() || handle_response.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        egui::Color32::from_rgb(100, 140, 200)
    } else {
        egui::Color32::from_rgb(55, 60, 70)
    };
    ui.painter().rect_filled(handle_rect, 0.0, handle_color);
    // Tři tečky na handle
    let dot_x = handle_rect.center().x;
    for dy in [-6.0_f32, 0.0, 6.0] {
        ui.painter().circle_filled(
            egui::pos2(dot_x, handle_rect.center().y + dy),
            1.5,
            egui::Color32::from_rgb(160, 170, 190),
        );
    }
    if handle_response.dragged() {
        let delta = handle_response.drag_delta().x;
        let usable = (available.x - handle_size).max(1.0);
        history_view.split_ratio =
            ((history_view.split_ratio * usable + delta) / usable).clamp(0.1, 0.9);
    }

    // ── Pravý panel (historická verze — read-only, Equal + Delete) ──
    let mut right_ui = ui.new_child(egui::UiBuilder::new().max_rect(right_rect));
    right_ui.set_clip_rect(right_rect);
    right_ui.label(egui::RichText::new(&*historical_label).strong().small());
    right_ui.separator();

    let right_scroll_output = egui::ScrollArea::both()
        .id_salt("history_split_right")
        .auto_shrink([false, false])
        .max_height(scroll_height)
        .vertical_scroll_offset(history_view.right_scroll_y)
        .show(&mut right_ui, |ui| {
            let frame = egui::Frame::new()
                .fill(bg)
                .inner_margin(egui::Margin::same(4));
            frame.show(ui, |ui| {
                if history_view.selected_index.is_some() && !right_panel_text.is_empty() {
                    // Syntax highlighting + diff background pro pravý panel
                    let job_arc =
                        highlighter.highlight(&right_panel_text, ext, fname, font_size, theme_name);
                    let mut job = (*job_arc).clone();
                    job.wrap.max_width = f32::INFINITY;
                    apply_diff_backgrounds(&mut job, &right_panel_text, &right_diff_map, &colors);
                    ui.add(egui::Label::new(job).wrap_mode(egui::TextWrapMode::Extend));
                } else {
                    // Žádná verze vybraná (1 verze → prázdný pravý panel)
                    ui.label(
                        egui::RichText::new(i18n.get("history-panel-no-versions"))
                            .color(egui::Color32::GRAY)
                            .italics(),
                    );
                }
            });
        });

    // ── Scroll sync ───────────────────────────────────────────────
    // Epsilon tolerance pro detekci změny (zabraňuje feedback loop).
    let epsilon = 1.0_f32;
    let new_left_y = left_scroll_output.state.offset.y;
    let new_right_y = right_scroll_output.state.offset.y;

    let left_changed = (new_left_y - history_view.left_scroll_y).abs() > epsilon;
    let right_changed = (new_right_y - history_view.right_scroll_y).abs() > epsilon;

    if left_changed && history_view.scroll_source != ScrollSource::Right {
        // Levý panel scrolloval → sync pravý proportionálně
        let left_max = left_scroll_output.content_size.y - left_scroll_output.inner_rect.height();
        let right_max =
            right_scroll_output.content_size.y - right_scroll_output.inner_rect.height();
        if left_max > 0.0 && right_max > 0.0 {
            let ratio = new_left_y / left_max;
            history_view.right_scroll_y = (ratio * right_max).max(0.0);
        }
        history_view.left_scroll_y = new_left_y;
        history_view.scroll_source = ScrollSource::Left;
    } else if right_changed && history_view.scroll_source != ScrollSource::Left {
        // Pravý panel scrolloval → sync levý proportionálně
        let left_max = left_scroll_output.content_size.y - left_scroll_output.inner_rect.height();
        let right_max =
            right_scroll_output.content_size.y - right_scroll_output.inner_rect.height();
        if left_max > 0.0 && right_max > 0.0 {
            let ratio = new_right_y / right_max;
            history_view.left_scroll_y = (ratio * left_max).max(0.0);
        }
        history_view.right_scroll_y = new_right_y;
        history_view.scroll_source = ScrollSource::Right;
    } else {
        // Žádná změna nebo oba stabilní → resetovat zdroj
        history_view.left_scroll_y = new_left_y;
        history_view.right_scroll_y = new_right_y;
        history_view.scroll_source = ScrollSource::None;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_diff_detects_insertions_and_deletions() {
        let current = "řádek1\nřádek2\nřádek3\n";
        let historical = "řádek1\nřádek3\n";
        let diff = compute_diff(current, historical);

        // Měl by obsahovat Equal(řádek1), Insert(řádek2), Equal(řádek3)
        assert!(!diff.is_empty());
        let tags: Vec<ChangeTag> = diff.iter().map(|d| d.tag).collect();
        assert!(tags.contains(&ChangeTag::Equal));
        assert!(tags.contains(&ChangeTag::Insert));
    }

    #[test]
    fn compute_diff_identical_texts_all_equal() {
        let text = "ahoj\nsvět\n";
        let diff = compute_diff(text, text);
        assert!(diff.iter().all(|d| d.tag == ChangeTag::Equal));
    }

    #[test]
    fn diff_colors_dark_mode_has_semitransparent_backgrounds() {
        let c = diff_colors(true);
        assert!(c.bg_added.a() < 255);
        assert!(c.bg_removed.a() < 255);
    }

    #[test]
    fn diff_colors_light_mode_has_opaque_backgrounds() {
        let c = diff_colors(false);
        assert_eq!(c.bg_added.a(), 255);
        assert_eq!(c.bg_removed.a(), 255);
    }

    #[test]
    fn format_timestamp_produces_correct_format() {
        // 2024-01-15 11:30:45 UTC = 1705318245
        // V lokálním čase se hodina může lišit, ale formát DD.MM.YYYY HH:MM:SS musí sedět.
        let s = format_timestamp(1705318245);
        assert!(s.contains("2024"), "Rok 2024 chybí v: {}", s);
        // Ověříme formát: DD.MM.YYYY HH:MM:SS (přesně 19 znaků)
        assert_eq!(s.len(), 19, "Nesprávná délka formátu: '{}'", s);
        assert_eq!(&s[2..3], ".", "Chybí tečka za dnem v: {}", s);
        assert_eq!(&s[5..6], ".", "Chybí tečka za měsícem v: {}", s);
        assert_eq!(&s[13..14], ":", "Chybí dvojtečka za hodinami v: {}", s);
        assert_eq!(&s[16..17], ":", "Chybí dvojtečka za minutami v: {}", s);
    }

    // ── Testy pro build_panel_texts ───────────────────────────

    #[test]
    fn build_panel_texts_splits_diff_correctly() {
        // Simulujeme diff: "řádek1\n" (Equal), "nový\n" (Insert), "starý\n" (Delete), "řádek3\n" (Equal)
        let diff_lines = vec![
            DiffLine {
                tag: ChangeTag::Equal,
                text: "řádek1\n".to_string(),
            },
            DiffLine {
                tag: ChangeTag::Insert,
                text: "nový\n".to_string(),
            },
            DiffLine {
                tag: ChangeTag::Delete,
                text: "starý\n".to_string(),
            },
            DiffLine {
                tag: ChangeTag::Equal,
                text: "řádek3\n".to_string(),
            },
        ];

        let panels = build_panel_texts(&diff_lines);

        // Levý panel: Equal + Insert řádky
        assert_eq!(panels.left_text, "řádek1\nnový\nřádek3\n");
        // Pravý panel: Equal + Delete řádky
        assert_eq!(panels.right_text, "řádek1\nstarý\nřádek3\n");

        // Diff mapy
        assert_eq!(
            panels.left_diff_map,
            vec![ChangeTag::Equal, ChangeTag::Insert, ChangeTag::Equal]
        );
        assert_eq!(
            panels.right_diff_map,
            vec![ChangeTag::Equal, ChangeTag::Delete, ChangeTag::Equal]
        );
    }

    #[test]
    fn build_panel_texts_empty_diff() {
        let panels = build_panel_texts(&[]);

        assert!(panels.left_text.is_empty());
        assert!(panels.right_text.is_empty());
        assert!(panels.left_diff_map.is_empty());
        assert!(panels.right_diff_map.is_empty());
    }

    #[test]
    fn build_panel_texts_identical_text_all_equal() {
        let diff_lines = vec![
            DiffLine {
                tag: ChangeTag::Equal,
                text: "ahoj\n".to_string(),
            },
            DiffLine {
                tag: ChangeTag::Equal,
                text: "svět\n".to_string(),
            },
        ];

        let panels = build_panel_texts(&diff_lines);

        assert_eq!(panels.left_text, "ahoj\nsvět\n");
        assert_eq!(panels.right_text, "ahoj\nsvět\n");
        assert!(panels.left_diff_map.iter().all(|t| *t == ChangeTag::Equal));
        assert!(panels.right_diff_map.iter().all(|t| *t == ChangeTag::Equal));
    }

    // ── Testy pro compute_line_offsets ────────────────────────

    #[test]
    fn compute_line_offsets_utf8_text() {
        // "příliš\nžluťoučký\n" — UTF-8 multi-byte znaky
        let text = "příliš\nžluťoučký\n";
        let offsets = compute_line_offsets(text);

        // Řádek 0 začíná na 0
        assert_eq!(offsets[0], 0);
        // "příliš\n" v UTF-8: p(1) ř(2) í(2) l(1) i(1) š(2) \n(1) = 10 bytů
        assert_eq!(offsets[1], 10);
        // "žluťoučký\n" v UTF-8: ž(2) l(1) u(1) ť(2) o(1) u(1) č(2) k(1) ý(2) \n(1) = 14 bytů
        assert_eq!(offsets[2], 24);
        assert_eq!(offsets.len(), 3);
    }

    #[test]
    fn compute_line_offsets_empty_text() {
        let offsets = compute_line_offsets("");
        assert_eq!(offsets, vec![0]);
    }

    #[test]
    fn compute_line_offsets_single_line_no_newline() {
        let offsets = compute_line_offsets("hello");
        assert_eq!(offsets, vec![0]);
    }

    // ── Testy pro apply_diff_backgrounds ──────────────────────

    #[test]
    fn apply_diff_backgrounds_sets_correct_colors() {
        let colors = diff_colors(true);
        let text = "equal line\ninserted line\ndeleted line\n";
        let diff_map = vec![ChangeTag::Equal, ChangeTag::Insert, ChangeTag::Delete];
        let font_id = egui::FontId::monospace(13.0);

        // Sestavíme LayoutJob s jednou section na řádek
        let mut job = egui::text::LayoutJob::default();
        job.append(
            "equal line\n",
            0.0,
            egui::text::TextFormat {
                font_id: font_id.clone(),
                ..Default::default()
            },
        );
        job.append(
            "inserted line\n",
            0.0,
            egui::text::TextFormat {
                font_id: font_id.clone(),
                ..Default::default()
            },
        );
        job.append(
            "deleted line\n",
            0.0,
            egui::text::TextFormat {
                font_id: font_id.clone(),
                ..Default::default()
            },
        );

        apply_diff_backgrounds(&mut job, text, &diff_map, &colors);

        // Section 0 (Equal) — background beze změny (default = transparent)
        assert_eq!(
            job.sections[0].format.background,
            egui::Color32::TRANSPARENT
        );
        // Section 1 (Insert) — bg_added
        assert_eq!(job.sections[1].format.background, colors.bg_added);
        // Section 2 (Delete) — bg_removed
        assert_eq!(job.sections[2].format.background, colors.bg_removed);
    }

    #[test]
    fn apply_diff_backgrounds_empty_diff_map_noop() {
        let colors = diff_colors(true);
        let text = "hello\n";
        let font_id = egui::FontId::monospace(13.0);

        let mut job = egui::text::LayoutJob::default();
        job.append(
            "hello\n",
            0.0,
            egui::text::TextFormat {
                font_id: font_id.clone(),
                ..Default::default()
            },
        );

        let bg_before = job.sections[0].format.background;
        apply_diff_backgrounds(&mut job, text, &[], &colors);
        // Žádná změna — diff mapa prázdná
        assert_eq!(job.sections[0].format.background, bg_before);
    }

    // ── Test content_hash ─────────────────────────────────────

    #[test]
    fn content_hash_differs_for_different_texts() {
        let h1 = content_hash("ahoj svět");
        let h2 = content_hash("ahoj světe");
        assert_ne!(h1, h2);
    }

    #[test]
    fn content_hash_same_for_same_text() {
        let h1 = content_hash("test");
        let h2 = content_hash("test");
        assert_eq!(h1, h2);
    }
}
