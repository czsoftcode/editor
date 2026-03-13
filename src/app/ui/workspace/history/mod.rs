use std::path::PathBuf;

use eframe::egui;
use similar::{ChangeTag, TextDiff};

use crate::app::local_history::{HistoryEntry, LocalHistory};
use crate::i18n::I18n;

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
    /// Cachovaný diff výsledek.
    pub cached_diff: Option<Vec<DiffLine>>,
    /// Index verze, pro kterou je diff cachovaný (pro invalidaci).
    pub diff_for_index: Option<usize>,
    /// Poměr šířky levého/pravého panelu (0.0–1.0).
    pub split_ratio: f32,
}

// ── Pomocné funkce ────────────────────────────────────────────────

/// Formátuje UNIX timestamp jako datum a čas (UTC, bez závislosti na chrono).
fn format_timestamp(ts: u64) -> String {
    let secs = ts;
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    let (year, month, day) = days_to_date(days);
    format!(
        "{:02}.{:02}.{} {:02}:{:02}:{:02}",
        day, month, year, hours, minutes, seconds
    )
}

/// Převede počet dní od UNIX epochy na (rok, měsíc, den).
fn days_to_date(days: u64) -> (u64, u64, u64) {
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
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

// ── Hlavní renderovací funkce ─────────────────────────────────────

/// Renderuje history split view s toolbar a dvěma diff panely.
/// Vrací `true` pokud byl view zavřen (uživatel klikl na ✕).
pub fn render_history_split_view(
    history_view: &mut HistoryViewState,
    local_history: &LocalHistory,
    ui: &mut egui::Ui,
    i18n: &I18n,
) -> bool {
    let mut close_requested = false;

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
                close_requested = true;
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
        });
    });
    ui.separator();

    if history_view.entries.is_empty() {
        ui.label(i18n.get("history-panel-no-versions"));
        return close_requested;
    }

    // ── Diff cache ────────────────────────────────────────────────
    let selected_idx = history_view.selected_index.unwrap_or(0);
    let need_recompute = history_view.diff_for_index != Some(selected_idx);

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
        history_view.cached_diff = Some(diff);
        history_view.diff_for_index = Some(selected_idx);
    }

    let diff_lines = history_view.cached_diff.as_deref().unwrap_or(&[]);
    let dark_mode = ui.visuals().dark_mode;
    let colors = diff_colors(dark_mode);
    let font_id = egui::FontId::monospace(13.0);

    // ── Split view ────────────────────────────────────────────────
    let available = ui.available_size();
    let handle_size = 6.0_f32;
    let usable_width = (available.x - handle_size).max(100.0);
    let left_w = (usable_width * history_view.split_ratio).clamp(50.0, usable_width - 50.0);
    let right_w = usable_width - left_w;
    let panel_height = available.y;

    // Labely panelů
    let current_label = i18n.get("history-current-label");
    let historical_label = i18n.get("history-historical-label");

    ui.horizontal(|ui| {
        // ── Levý panel (aktuální verze — Equal + Insert) ──────────
        ui.vertical(|ui| {
            ui.set_width(left_w);
            ui.label(egui::RichText::new(&*current_label).strong().small());
            egui::ScrollArea::both()
                .id_salt("history_split_left")
                .auto_shrink([false, false])
                .max_height(panel_height - 20.0)
                .show(ui, |ui| {
                    let mut job = egui::text::LayoutJob::default();
                    for line in diff_lines {
                        match line.tag {
                            ChangeTag::Equal => {
                                job.append(
                                    &line.text,
                                    0.0,
                                    egui::text::TextFormat {
                                        font_id: font_id.clone(),
                                        color: colors.fg_normal,
                                        ..Default::default()
                                    },
                                );
                            }
                            ChangeTag::Insert => {
                                job.append(
                                    &line.text,
                                    0.0,
                                    egui::text::TextFormat {
                                        font_id: font_id.clone(),
                                        color: colors.fg_added,
                                        background: colors.bg_added,
                                        ..Default::default()
                                    },
                                );
                            }
                            ChangeTag::Delete => {
                                // Přeskočit — Delete řádky se zobrazují jen v pravém panelu
                            }
                        }
                    }
                    ui.add(egui::Label::new(job).wrap_mode(egui::TextWrapMode::Extend));
                });
        });

        // ── Resize handle ─────────────────────────────────────────
        let (handle_rect, handle_response) =
            ui.allocate_exact_size(egui::vec2(handle_size, panel_height), egui::Sense::drag());
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

        // ── Pravý panel (historická verze — Equal + Delete) ───────
        ui.vertical(|ui| {
            ui.set_width(right_w);
            ui.label(egui::RichText::new(&*historical_label).strong().small());
            egui::ScrollArea::both()
                .id_salt("history_split_right")
                .auto_shrink([false, false])
                .max_height(panel_height - 20.0)
                .show(ui, |ui| {
                    let mut job = egui::text::LayoutJob::default();
                    for line in diff_lines {
                        match line.tag {
                            ChangeTag::Equal => {
                                job.append(
                                    &line.text,
                                    0.0,
                                    egui::text::TextFormat {
                                        font_id: font_id.clone(),
                                        color: colors.fg_normal,
                                        ..Default::default()
                                    },
                                );
                            }
                            ChangeTag::Delete => {
                                job.append(
                                    &line.text,
                                    0.0,
                                    egui::text::TextFormat {
                                        font_id: font_id.clone(),
                                        color: colors.fg_removed,
                                        background: colors.bg_removed,
                                        ..Default::default()
                                    },
                                );
                            }
                            ChangeTag::Insert => {
                                // Přeskočit — Insert řádky se zobrazují jen v levém panelu
                            }
                        }
                    }
                    ui.add(egui::Label::new(job).wrap_mode(egui::TextWrapMode::Extend));
                });
        });
    });

    close_requested
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
        let s = format_timestamp(1705318245);
        assert!(s.contains("2024"), "Rok 2024 chybí v: {}", s);
        assert!(s.contains("11:30:45"), "Čas chybí v: {}", s);
    }
}
