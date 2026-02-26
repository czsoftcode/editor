use crate::app::ui::file_tree::FileTree;
use crate::app::ui::file_tree::node::FileNode;
use crate::app::ui::file_tree::ops::ContextAction;
use crate::config;
use eframe::egui;
use std::collections::HashMap;
use std::path::PathBuf;

impl FileTree {
    #[allow(clippy::too_many_arguments)]
    pub fn show_node(
        ui: &mut egui::Ui,
        node: &mut FileNode,
        selected: &mut Option<PathBuf>,
        action: &mut Option<ContextAction>,
        has_clipboard: bool,
        expand_to: &Option<PathBuf>,
        git_colors: &HashMap<PathBuf, egui::Color32>,
        i18n: &crate::i18n::I18n,
        is_sandbox: bool,
    ) {
        let dark_mode = ui.visuals().dark_mode;
        let text_color = ui.visuals().text_color();
        let font_size = config::FILE_TREE_FONT_SIZE;

        // Git colors are designed for dark backgrounds — we darken them in light mode
        let adapt_git_color = |c: egui::Color32| -> egui::Color32 {
            if dark_mode {
                c
            } else {
                egui::Color32::from_rgb(
                    (c.r() as f32 * 0.55) as u8,
                    (c.g() as f32 * 0.55) as u8,
                    (c.b() as f32 * 0.55) as u8,
                )
            }
        };

        if node.is_dir {
            let force_open = expand_to
                .as_ref()
                .is_some_and(|target| target.starts_with(&node.path));

            let header_text = egui::RichText::new(format!("\u{1F4C1} {}", &node.name))
                .size(font_size)
                .color(text_color);
            let mut header = egui::CollapsingHeader::new(header_text).default_open(node.expanded);
            if force_open {
                header = header.open(Some(true));
            }
            let response = header.show(ui, |ui| {
                node.load_children();
                for child in &mut node.children {
                    Self::show_node(
                        ui,
                        child,
                        selected,
                        action,
                        has_clipboard,
                        expand_to,
                        git_colors,
                        i18n,
                        is_sandbox,
                    );
                }
            });

            let header_response = response.header_response;
            header_response.context_menu(|ui| {
                let menu_size = 15.0;
                if ui
                    .button(egui::RichText::new(i18n.get("file-tree-new-file")).size(menu_size))
                    .clicked()
                {
                    *action = Some(ContextAction::NewFile(node.path.clone()));
                    ui.close_menu();
                }
                if ui
                    .button(egui::RichText::new(i18n.get("file-tree-new-dir")).size(menu_size))
                    .clicked()
                {
                    *action = Some(ContextAction::NewDir(node.path.clone()));
                    ui.close_menu();
                }
                if ui
                    .button(egui::RichText::new(i18n.get("file-tree-rename")).size(menu_size))
                    .clicked()
                {
                    *action = Some(ContextAction::Rename(node.path.clone()));
                    ui.close_menu();
                }
                if ui
                    .button(egui::RichText::new(i18n.get("file-tree-copy")).size(menu_size))
                    .clicked()
                {
                    *action = Some(ContextAction::Copy(node.path.clone()));
                    ui.close_menu();
                }
                if has_clipboard
                    && ui
                        .button(egui::RichText::new(i18n.get("file-tree-paste")).size(menu_size))
                        .clicked()
                {
                    *action = Some(ContextAction::Paste(node.path.clone()));
                    ui.close_menu();
                }
                if ui
                    .button(egui::RichText::new(i18n.get("file-tree-delete")).size(menu_size))
                    .clicked()
                {
                    *action = Some(ContextAction::Delete(node.path.clone()));
                    ui.close_menu();
                }
            });
        } else {
            // Lazy calculate line count for files in sandbox
            if is_sandbox && node.line_count.is_none() {
                if let Ok(content) = std::fs::read_to_string(&node.path) {
                    node.line_count = Some(content.lines().count());
                } else {
                    node.line_count = Some(0); // Mark as tried
                }
            }

            let mut file_color = git_colors
                .get(&node.path)
                .copied()
                .map(adapt_git_color)
                .unwrap_or(text_color);

            let mut is_large = false;
            let mut is_very_large = false;
            let mut rounded_count = 0;
            if is_sandbox
                && let Some(count) = node.line_count
                && count >= 500
            {
                file_color = egui::Color32::WHITE;
                is_large = true;
                rounded_count = ((count as f32 / 10.0).round() * 10.0) as usize;
                if count >= 1000 {
                    is_very_large = true;
                }
            }

            let label = if is_large {
                let mut job = egui::text::LayoutJob::default();
                let main_font = egui::FontId::proportional(font_size);

                let stroke_width = if is_very_large { 2.0 } else { 1.0 };

                job.append(
                    &format!("\u{1F4C4} {}", &node.name),
                    0.0,
                    egui::TextFormat {
                        font_id: main_font,
                        color: file_color,
                        underline: egui::Stroke::new(stroke_width, file_color),
                        ..Default::default()
                    },
                );

                job.append(
                    &format!(" ({})", rounded_count),
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::proportional(font_size * 0.9),
                        color: file_color.linear_multiply(0.7),
                        italics: true,
                        ..Default::default()
                    },
                );
                ui.selectable_label(false, job)
            } else {
                let file_text = egui::RichText::new(format!("\u{1F4C4} {}", &node.name))
                    .size(font_size)
                    .color(file_color);
                ui.selectable_label(false, file_text)
            };

            if label.clicked() {
                *selected = Some(node.path.clone());
            }

            label.context_menu(|ui| {
                let menu_size = 15.0;
                if ui
                    .button(egui::RichText::new(i18n.get("file-tree-rename")).size(menu_size))
                    .clicked()
                {
                    *action = Some(ContextAction::Rename(node.path.clone()));
                    ui.close_menu();
                }
                if ui
                    .button(egui::RichText::new(i18n.get("file-tree-copy")).size(menu_size))
                    .clicked()
                {
                    *action = Some(ContextAction::Copy(node.path.clone()));
                    ui.close_menu();
                }
                if ui
                    .button(egui::RichText::new(i18n.get("file-tree-delete")).size(menu_size))
                    .clicked()
                {
                    *action = Some(ContextAction::Delete(node.path.clone()));
                    ui.close_menu();
                }
            });
        }
    }
}
