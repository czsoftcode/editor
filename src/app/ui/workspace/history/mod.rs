use std::path::PathBuf;

use eframe::egui;

use crate::app::local_history::{HistoryEntry, LocalHistory};
use crate::i18n::I18n;

/// Stav otevřeného history panelu pro konkrétní soubor.
pub struct HistoryViewState {
    /// Absolutní cesta k souboru.
    pub file_path: PathBuf,
    /// Relativní cesta vůči kořeni projektu.
    pub relative_path: PathBuf,
    /// Seznam historických verzí (nejnovější první).
    pub entries: Vec<HistoryEntry>,
    /// Index aktuálně vybrané verze v seznamu.
    pub selected_index: Option<usize>,
    /// Obsah vybrané verze pro náhled.
    pub preview_content: Option<String>,
    /// Příznak pro scroll k vybrané položce.
    pub scroll_to_selected: bool,
}

/// Formátuje UNIX timestamp jako datum a čas (UTC, bez závislosti na chrono).
fn format_timestamp(ts: u64) -> String {
    // Jednoduchý formátor: rozložíme UNIX timestamp na datum a čas v UTC.
    // Pro desktop editor je to dostatečné — uživatel vidí relativní pořadí verzí.
    let secs = ts;
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Převod dnů od epochy na datum (jednoduchý Gregorián)
    let (year, month, day) = days_to_date(days);
    format!(
        "{:02}.{:02}.{} {:02}:{:02}:{:02}",
        day, month, year, hours, minutes, seconds
    )
}

/// Převede počet dní od UNIX epochy na (rok, měsíc, den).
fn days_to_date(days: u64) -> (u64, u64, u64) {
    // Civilní kalendář z dní od epochy (1970-01-01)
    // Algoritmus z http://howardhinnant.github.io/date_algorithms.html
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

/// Renderuje history panel uvnitř workspace CentralPanel.
/// Vrací `true` pokud byl panel zavřen (uživatel klikl na zavírací tlačítko).
pub fn render_history_panel(
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

    // Horní bar: nadpis + zavírací tlačítko
    ui.horizontal(|ui| {
        let mut args = fluent_bundle::FluentArgs::new();
        args.set("name", file_name.as_str());
        ui.heading(i18n.get_args("history-panel-title", &args));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(i18n.get("history-panel-close")).clicked() {
                close_requested = true;
            }
        });
    });
    ui.separator();

    if history_view.entries.is_empty() {
        ui.label(i18n.get("history-panel-no-versions"));
        return close_requested;
    }

    // Horizontální split: levý 30% seznam verzí, pravý 70% náhled
    let available_width = ui.available_width();
    let list_width = (available_width * 0.3).max(150.0);

    ui.horizontal(|ui| {
        // Levý panel — seznam verzí
        ui.vertical(|ui| {
            ui.set_width(list_width);
            egui::ScrollArea::vertical()
                .id_salt("history_version_list")
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    for (idx, entry) in history_view.entries.iter().enumerate() {
                        let is_selected = history_view.selected_index == Some(idx);
                        let date_str = format_timestamp(entry.timestamp);
                        let mut args = fluent_bundle::FluentArgs::new();
                        args.set("date", date_str.as_str());
                        let label = i18n.get_args("history-panel-version-label", &args);

                        let response = ui.selectable_label(is_selected, &label);
                        if response.clicked() && !is_selected {
                            history_view.selected_index = Some(idx);
                            // Načíst obsah vybrané verze
                            match local_history
                                .get_snapshot_content(&history_view.relative_path, entry)
                            {
                                Ok(content) => {
                                    history_view.preview_content = Some(content);
                                }
                                Err(e) => {
                                    history_view.preview_content =
                                        Some(format!("Chyba čtení: {}", e));
                                }
                            }
                        }
                    }
                });
        });

        ui.separator();

        // Pravý panel — náhled textu
        ui.vertical(|ui| {
            egui::ScrollArea::both()
                .id_salt("history_preview")
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    if let Some(content) = &history_view.preview_content {
                        ui.add(
                            egui::TextEdit::multiline(&mut content.as_str())
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .interactive(false),
                        );
                    } else {
                        ui.colored_label(
                            ui.visuals().weak_text_color(),
                            i18n.get("history-panel-no-versions"),
                        );
                    }
                });
        });
    });

    close_requested
}
