use crate::app::ui::file_tree::FileTree;
use crate::app::ui::file_tree::node::FileNode;
use crate::app::ui::file_tree::ops::ContextAction;
use crate::app::ui::git_status::{GitVisualStatus, git_color_for_visuals};
use crate::config;
use eframe::egui;
use std::collections::HashMap;
use std::path::PathBuf;

fn resolve_file_tree_git_color(
    status: Option<GitVisualStatus>,
    visuals: &egui::Visuals,
    text_color: egui::Color32,
) -> egui::Color32 {
    status
        .map(|status| git_color_for_visuals(status, visuals))
        .unwrap_or(text_color)
}

impl FileTree {
    #[allow(clippy::too_many_arguments)]
    pub fn show_node(
        ui: &mut egui::Ui,
        node: &mut FileNode,
        selected: &mut Option<PathBuf>,
        action: &mut Option<ContextAction>,
        has_clipboard: bool,
        expand_to: &Option<PathBuf>,
        git_statuses: &HashMap<PathBuf, GitVisualStatus>,
        i18n: &crate::i18n::I18n,
        is_sandbox: bool,
    ) {
        let visuals = ui.visuals();
        let text_color = ui.visuals().text_color();
        let font_size = config::FILE_TREE_FONT_SIZE;

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
                        git_statuses,
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

            let mut file_color = resolve_file_tree_git_color(
                git_statuses.get(&node.path).copied(),
                visuals,
                text_color,
            );

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

#[cfg(test)]
mod tests {
    use super::resolve_file_tree_git_color;
    use crate::app::ui::git_status::{GitVisualStatus, git_color_for_visuals};
    use crate::settings::{LightVariant, Settings};
    use eframe::egui::{Color32, Visuals};
    use std::collections::HashSet;

    fn light_visuals(variant: LightVariant) -> Visuals {
        Settings {
            dark_theme: false,
            light_variant: variant,
            ..Default::default()
        }
        .to_egui_visuals()
    }

    #[test]
    fn file_tree_git_resolve_uses_light_palette_for_untracked_when_not_dark() {
        let text_color = Color32::WHITE;
        let visuals = Visuals::light();
        let resolved =
            resolve_file_tree_git_color(Some(GitVisualStatus::Untracked), &visuals, text_color);

        assert_eq!(
            resolved,
            git_color_for_visuals(GitVisualStatus::Untracked, &visuals)
        );
    }

    #[test]
    fn file_tree_git_resolve_falls_back_to_text_color_without_status() {
        let text_color = Color32::from_rgb(25, 26, 27);
        let visuals = Visuals::light();
        let resolved = resolve_file_tree_git_color(None, &visuals, text_color);

        assert_eq!(resolved, text_color);
    }

    #[test]
    fn file_tree_git_resolve_light_palette_statuses_are_distinct() {
        let text_color = Color32::WHITE;
        let visuals = Visuals::light();
        let statuses = [
            GitVisualStatus::Modified,
            GitVisualStatus::Added,
            GitVisualStatus::Deleted,
            GitVisualStatus::Untracked,
        ];

        let colors: HashSet<Color32> = statuses
            .into_iter()
            .map(|status| resolve_file_tree_git_color(Some(status), &visuals, text_color))
            .collect();

        assert_eq!(colors.len(), 4);
    }

    #[test]
    fn file_tree_git_resolve_light_variant_tones_differ_for_modified() {
        let text_color = Color32::WHITE;
        let warm_visuals = light_visuals(LightVariant::WarmIvory);
        let sepia_visuals = light_visuals(LightVariant::Sepia);
        let warm = resolve_file_tree_git_color(
            Some(GitVisualStatus::Modified),
            &warm_visuals,
            text_color,
        );
        let sepia = resolve_file_tree_git_color(
            Some(GitVisualStatus::Modified),
            &sepia_visuals,
            text_color,
        );

        assert_ne!(
            warm, sepia,
            "light variants must produce different file tree tones"
        );
    }
}
