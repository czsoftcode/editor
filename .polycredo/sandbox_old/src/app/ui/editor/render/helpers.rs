use crate::app::ui::editor::*;
use crate::config;
use std::collections::HashMap;

pub fn editor_line_count(content: &str) -> usize {
    content.lines().count().max(1) + usize::from(content.ends_with('\n'))
}

pub fn goto_centered_scroll_offset(
    line: usize,
    total_lines: usize,
    row_height: f32,
    viewport_height: f32,
) -> f32 {
    if row_height <= 0.0 || viewport_height <= 0.0 || total_lines == 0 {
        return 0.0;
    }

    let max_line_index = total_lines.saturating_sub(1) as f32;
    let line_index = (line.saturating_sub(1) as f32).min(max_line_index);
    let line_y = line_index * row_height;
    let centered = line_y - (viewport_height - row_height) * 0.5;

    let doc_height = total_lines as f32 * row_height;
    let max_scroll = (doc_height - viewport_height).max(0.0);
    centered.clamp(0.0, max_scroll)
}

pub fn restore_saved_cursor(
    ctx: &egui::Context,
    edit_id: egui::Id,
    cursor_range: Option<egui::text::CursorRange>,
) {
    if let Some(saved) = cursor_range {
        let mut state = egui::text_edit::TextEditState::load(ctx, edit_id).unwrap_or_default();
        state.cursor.set_char_range(Some(saved.as_ccursor_range()));
        state.store(ctx, edit_id);
    }
}

// --- Tab bar ---
impl Editor {
    // --- Font & gutter helpers ---

    pub fn current_editor_font_size(ui: &egui::Ui) -> f32 {
        ui.style()
            .text_styles
            .get(&egui::TextStyle::Monospace)
            .map(|f| f.size)
            .unwrap_or(config::EDITOR_FONT_SIZE)
    }

    pub fn gutter_width(ui: &egui::Ui, line_count: usize) -> f32 {
        let font_size = Self::current_editor_font_size(ui);
        let font_id = egui::FontId::monospace(font_size);
        let digits = ((line_count.max(1) as f64).log10().floor() as usize) + 1;
        let char_width = ui.fonts(|f| f.glyph_width(&font_id, '0'));
        (digits as f32) * char_width + 12.0
    }

    /// Paints colored underlines under lines that have LSP diagnostics.
    /// ERROR = červená, WARNING = oranžová, INFO = modrá, HINT = zelená.
    pub fn paint_squiggles(
        ui: &mut egui::Ui,
        output: &egui::text_edit::TextEditOutput,
        diagnostics_for_file: Option<&Vec<async_lsp::lsp_types::Diagnostic>>,
    ) {
        let Some(diagnostics) = diagnostics_for_file else {
            return;
        };
        if diagnostics.is_empty() {
            return;
        }

        // Build map: 0-based logical line → (priority, color)
        // Lower priority number = higher severity (ERROR=0 wins over WARNING=1, etc.)
        let mut diag_by_line: HashMap<usize, (u8, egui::Color32)> = HashMap::new();
        for diag in diagnostics {
            let line = diag.range.start.line as usize;
            let (color, priority) = match diag.severity {
                Some(async_lsp::lsp_types::DiagnosticSeverity::ERROR) => {
                    (egui::Color32::from_rgba_unmultiplied(220, 50, 50, 160), 0u8)
                }
                Some(async_lsp::lsp_types::DiagnosticSeverity::WARNING) => {
                    (egui::Color32::from_rgba_unmultiplied(220, 160, 0, 130), 1)
                }
                Some(async_lsp::lsp_types::DiagnosticSeverity::INFORMATION) => {
                    (egui::Color32::from_rgba_unmultiplied(30, 150, 220, 100), 2)
                }
                Some(async_lsp::lsp_types::DiagnosticSeverity::HINT) => {
                    (egui::Color32::from_rgba_unmultiplied(80, 200, 80, 80), 3)
                }
                _ => continue,
            };
            diag_by_line
                .entry(line)
                .and_modify(|(prev_prio, prev_color)| {
                    if priority < *prev_prio {
                        *prev_prio = priority;
                        *prev_color = color;
                    }
                })
                .or_insert((priority, color));
        }

        let galley_pos = output.galley_pos;
        let galley = &output.galley;

        let mut logical_line: usize = 0;
        let mut is_new_line = true;

        for row in galley.rows.iter() {
            if is_new_line {
                if let Some((_, color)) = diag_by_line.get(&logical_line) {
                    let y = galley_pos.y + row.rect.min.y;
                    let row_h = row.rect.height();
                    let x_start = galley_pos.x + row.rect.min.x;
                    // Extend at least a bit so empty lines are also visible
                    let x_end = (galley_pos.x + row.rect.max.x).max(x_start + 40.0);

                    // 2px underline at the bottom of the row
                    let underline_rect = egui::Rect::from_min_max(
                        egui::pos2(x_start, y + row_h - 2.0),
                        egui::pos2(x_end, y + row_h),
                    );
                    ui.painter().rect_filled(underline_rect, 0.0, *color);
                }
                logical_line += 1;
            }
            is_new_line = row.ends_with_newline;
        }
    }

    pub fn paint_line_numbers(
        ui: &mut egui::Ui,
        output: &egui::text_edit::TextEditOutput,
        gutter_rect: egui::Rect,
        diagnostics_for_file: Option<&Vec<async_lsp::lsp_types::Diagnostic>>,
    ) {
        let font_size = Self::current_editor_font_size(ui);
        let font_id = egui::FontId::monospace(font_size);
        let gutter_color = egui::Color32::from_rgb(130, 130, 130);
        let highlight_color = egui::Color32::from_rgba_unmultiplied(80, 65, 15, 50);

        let galley_pos = output.galley_pos;
        let galley = &output.galley;

        let cursor_row = output.cursor_range.map(|cr| cr.primary.rcursor.row);

        let mut line_num: usize = 1;
        let mut is_new_line = true;

        let diagnostic_map: HashMap<usize, Vec<&async_lsp::lsp_types::Diagnostic>> =
            diagnostics_for_file
                .map(|diagnostics| {
                    let mut map = HashMap::new();
                    for diag in diagnostics {
                        // LSP lines are 0-indexed, UI is 1-indexed
                        map.entry(diag.range.start.line as usize + 1)
                            .or_insert_with(Vec::new)
                            .push(diag);
                    }
                    map
                })
                .unwrap_or_default();

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
                ui.painter()
                    .rect_filled(highlight_rect, 0.0, highlight_color);
            }

            if is_new_line {
                let current_line_num = line_num;
                let text_color = gutter_color;
                let mut dot_color = None;
                let mut tooltip_text = Vec::new();

                if let Some(diagnostics_on_line) = diagnostic_map.get(&current_line_num) {
                    for diag in diagnostics_on_line {
                        let severity_color = match diag.severity {
                            Some(async_lsp::lsp_types::DiagnosticSeverity::ERROR) => {
                                egui::Color32::from_rgb(255, 60, 60)
                            }
                            Some(async_lsp::lsp_types::DiagnosticSeverity::WARNING) => {
                                egui::Color32::from_rgb(255, 180, 0)
                            }
                            Some(async_lsp::lsp_types::DiagnosticSeverity::INFORMATION) => {
                                egui::Color32::from_rgb(0, 180, 255)
                            }
                            Some(async_lsp::lsp_types::DiagnosticSeverity::HINT) => {
                                egui::Color32::from_rgb(100, 255, 100)
                            }
                            _ => gutter_color,
                        };
                        // Use the highest severity color for the dot
                        if dot_color.is_none()
                            || diag.severity
                                == Some(async_lsp::lsp_types::DiagnosticSeverity::ERROR)
                        {
                            dot_color = Some(severity_color);
                        }
                        tooltip_text.push(diag.message.clone());
                    }
                }

                let line_number_text = format!("{}", current_line_num);
                let text_galley =
                    ui.fonts(|f| f.layout_no_wrap(line_number_text, font_id.clone(), text_color));
                let text_pos = egui::pos2(gutter_rect.right() - 4.0 - text_galley.rect.width(), y);

                // Hover area: full gutter row width for easy tooltip triggering
                let line_number_rect = egui::Rect::from_min_max(
                    egui::pos2(gutter_rect.left(), y),
                    egui::pos2(gutter_rect.right(), y + row_height),
                );
                let response = ui.allocate_rect(line_number_rect, egui::Sense::hover());

                ui.painter().galley(text_pos, text_galley, text_color);

                // Dot on the LEFT side of the gutter — no overlap with line numbers
                if let Some(color) = dot_color {
                    ui.painter().circle_filled(
                        egui::pos2(gutter_rect.left() + 6.0, y + row_height / 2.0),
                        3.5,
                        color,
                    );
                }

                if response.hovered() && !tooltip_text.is_empty() {
                    response.on_hover_ui_at_pointer(|ui| {
                        for msg in &tooltip_text {
                            ui.label(msg);
                        }
                    });
                }

                line_num += 1;
            }
            is_new_line = row.ends_with_newline;
        }
    }
}
