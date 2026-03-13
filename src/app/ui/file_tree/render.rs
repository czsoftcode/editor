use crate::app::ui::file_tree::FileTree;
use crate::app::ui::file_tree::node::FileNode;
use crate::app::ui::file_tree::ops::ContextAction;
use crate::app::ui::git_status::{GitVisualStatus, git_color_for_visuals};
use crate::config;
use eframe::egui;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Vrátí ikonu souboru podle přípony. Adresáře řeší volající zvlášť.
fn file_icon(name: &str) -> &'static str {
    let ext = name.rsplit('.').next().unwrap_or("");
    match ext.to_ascii_lowercase().as_str() {
        // Rust
        "rs" => "🦀",
        // Web
        "html" | "htm" => "🌐",
        "css" | "scss" | "sass" | "less" => "🎨",
        "js" | "mjs" | "cjs" => "📜",
        "ts" | "tsx" | "jsx" => "📘",
        "json" => "📋",
        "xml" | "svg" => "📐",
        "wasm" => "⚙️",
        // Konfigurace / data
        "toml" | "yaml" | "yml" | "ini" | "cfg" | "conf" => "⚙️",
        "env" => "🔒",
        "lock" => "🔒",
        // Dokumentace
        "md" | "mdx" => "📝",
        "txt" | "text" | "rtf" => "📄",
        "pdf" => "📕",
        "doc" | "docx" | "odt" => "📕",
        // Obrázky
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "ico" | "tiff" | "tif" => "🖼️",
        // Audio / video
        "mp3" | "wav" | "ogg" | "flac" | "aac" => "🎵",
        "mp4" | "avi" | "mkv" | "webm" | "mov" => "🎬",
        // Archivy
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "zst" => "📦",
        // Skripty / shell
        "sh" | "bash" | "zsh" | "fish" | "bat" | "cmd" | "ps1" => "⚡",
        // Programovací jazyky
        "py" => "🐍",
        "rb" => "💎",
        "go" => "🔷",
        "java" | "kt" | "kts" => "☕",
        "c" | "h" => "🔧",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "🔧",
        "cs" => "🟣",
        "swift" => "🐦",
        "php" => "🐘",
        "lua" => "🌙",
        "sql" => "🗃️",
        "r" => "📊",
        // Databáze / binárky
        "db" | "sqlite" | "sqlite3" => "🗃️",
        "bin" | "exe" | "dll" | "so" | "dylib" | "a" => "⬛",
        // Docker / CI
        "dockerfile" => "🐳",
        // Fonty
        "ttf" | "otf" | "woff" | "woff2" => "🔤",
        // Licence, readme atp. (bez přípony nebo speciální)
        "license" | "licence" => "📜",
        // Git
        "gitignore" | "gitattributes" | "gitmodules" => "🔀",
        // FTL (Fluent lokalizace)
        "ftl" => "🌍",
        // Výchozí
        _ => match name.to_ascii_lowercase().as_str() {
            "dockerfile" => "🐳",
            "makefile" | "justfile" | "rakefile" => "⚡",
            "license" | "licence" => "📜",
            "readme" | "changelog" | "authors" | "contributors" => "📝",
            _ => "📄",
        },
    }
}

/// Ikona adresáře. Některé speciální adresáře mají vlastní ikonu.
fn dir_icon(name: &str) -> &'static str {
    match name.to_ascii_lowercase().as_str() {
        "src" | "lib" => "📦",
        "test" | "tests" | "spec" | "specs" => "🧪",
        "doc" | "docs" | "documentation" => "📚",
        "assets" | "static" | "public" | "resources" | "res" => "🖼️",
        "locales" | "locale" | "i18n" | "l10n" | "translations" => "🌍",
        "scripts" | "bin" => "⚡",
        "config" | "cfg" | "conf" | ".config" => "⚙️",
        ".git" => "🔀",
        ".github" | ".gitlab" => "🔀",
        "node_modules" => "📦",
        "target" | "build" | "dist" | "out" | "output" => "🏗️",
        "vendor" | "third_party" | "deps" | "dependencies" => "📦",
        "examples" | "example" => "💡",
        "benches" | "benchmarks" => "📊",
        _ => "📁",
    }
}

/// Virtuální uzel stromu změn – vytvořený z git statusů bez čtení z disku.
struct ChangesNode {
    name: String,
    abs_path: PathBuf,
    is_dir: bool,
    status: Option<GitVisualStatus>,
    children: Vec<ChangesNode>,
}

impl ChangesNode {
    fn new_dir(name: String, abs_path: PathBuf) -> Self {
        Self {
            name,
            abs_path,
            is_dir: true,
            status: None,
            children: Vec::new(),
        }
    }

    fn new_file(name: String, abs_path: PathBuf, status: GitVisualStatus) -> Self {
        Self {
            name,
            abs_path,
            is_dir: false,
            status: Some(status),
            children: Vec::new(),
        }
    }

    /// Vloží relativní cestu do stromu. Mezilehlé adresáře se vytváří automaticky.
    fn insert(&mut self, rel: &Path, abs_path: PathBuf, status: GitVisualStatus) {
        let components: Vec<_> = rel.components().collect();
        self.insert_components(&components, abs_path, status);
    }

    fn insert_components(
        &mut self,
        components: &[std::path::Component<'_>],
        abs_path: PathBuf,
        status: GitVisualStatus,
    ) {
        if components.is_empty() {
            return;
        }

        let name = components[0].as_os_str().to_string_lossy().to_string();

        if components.len() == 1 {
            // Listový uzel — soubor
            self.children
                .push(ChangesNode::new_file(name, abs_path, status));
        } else {
            // Adresářový uzel — najdi existující nebo vytvoř nový
            let dir_abs = self.abs_path.join(&name);
            let dir_pos = self
                .children
                .iter()
                .position(|c| c.is_dir && c.name == name);

            let idx = if let Some(pos) = dir_pos {
                pos
            } else {
                self.children.push(ChangesNode::new_dir(name, dir_abs));
                self.children.len() - 1
            };
            self.children[idx].insert_components(&components[1..], abs_path, status);
        }
    }

    /// Rekurzivně seřadí strom: adresáře první, pak soubory, abecedně.
    fn sort(&mut self) {
        self.children.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });
        for child in &mut self.children {
            child.sort();
        }
    }
}

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
    ) {
        let visuals = ui.visuals();
        let text_color = ui.visuals().text_color();
        let font_size = config::FILE_TREE_FONT_SIZE;

        if node.is_dir {
            let force_open = expand_to
                .as_ref()
                .is_some_and(|target| target.starts_with(&node.path));

            let icon = dir_icon(&node.name);
            let header_text = egui::RichText::new(format!("{icon} {}", &node.name))
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
            // Lazy calculate line count for files
            if node.line_count.is_none() {
                if let Ok(content) = std::fs::read_to_string(&node.path) {
                    node.line_count = Some(content.lines().count());
                } else {
                    node.line_count = Some(0); // Mark as tried
                }
            }

            let file_color = resolve_file_tree_git_color(
                git_statuses.get(&node.path).copied(),
                visuals,
                text_color,
            );

            let mut is_large = false;
            let mut is_very_large = false;
            let mut rounded_count = 0;
            if let Some(count) = node.line_count
                && count >= 500
            {
                is_large = true;
                rounded_count = ((count as f32 / 10.0).round() * 10.0) as usize;
                if count >= 1000 {
                    is_very_large = true;
                }
            }

            let icon = file_icon(&node.name);
            let label = if is_large {
                let mut job = egui::text::LayoutJob::default();
                let main_font = egui::FontId::proportional(font_size);

                let stroke_width = if is_very_large { 2.0 } else { 1.0 };

                job.append(
                    &format!("{icon} {}", &node.name),
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
                let file_text = egui::RichText::new(format!("{icon} {}", &node.name))
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

    /// Vykreslí stromovou strukturu změněných/přidaných/smazaných souborů z git statusů.
    /// Strom je vždy plně rozbalený (všechny adresáře otevřené).
    pub fn show_changes(
        ui: &mut egui::Ui,
        root_path: &Path,
        git_statuses: &HashMap<PathBuf, GitVisualStatus>,
        selected: &mut Option<PathBuf>,
    ) {
        if git_statuses.is_empty() {
            ui.label(
                egui::RichText::new("—")
                    .size(config::FILE_TREE_FONT_SIZE)
                    .weak(),
            );
            return;
        }

        // Sestavíme virtuální strom ze změněných cest
        let mut tree = ChangesNode::new_dir(String::new(), root_path.to_path_buf());
        for (abs_path, status) in git_statuses {
            let rel = abs_path.strip_prefix(root_path).unwrap_or(abs_path);
            tree.insert(rel, abs_path.clone(), *status);
        }
        tree.sort();

        // Vykreslíme obsah kořene (bez kořenového adresáře samotného)
        for child in &tree.children {
            Self::show_changes_node(ui, child, selected);
        }
    }

    /// Rekurzivní vykreslení jednoho uzlu stromu změn.
    fn show_changes_node(ui: &mut egui::Ui, node: &ChangesNode, selected: &mut Option<PathBuf>) {
        let font_size = config::FILE_TREE_FONT_SIZE;
        let visuals = ui.visuals();
        let text_color = visuals.text_color();

        if node.is_dir {
            let icon = dir_icon(&node.name);
            let header_text = egui::RichText::new(format!("{icon} {}", &node.name))
                .size(font_size)
                .color(text_color);
            // Vždy plně rozbalený
            egui::CollapsingHeader::new(header_text)
                .default_open(true)
                .open(Some(true))
                .show(ui, |ui| {
                    for child in &node.children {
                        Self::show_changes_node(ui, child, selected);
                    }
                });
        } else {
            let status = node.status.unwrap_or(GitVisualStatus::Modified);
            let color = git_color_for_visuals(status, visuals);

            let status_char = match status {
                GitVisualStatus::Modified => "M",
                GitVisualStatus::Added => "A",
                GitVisualStatus::Deleted => "D",
                GitVisualStatus::Untracked => "?",
            };

            let mut job = egui::text::LayoutJob::default();

            // Prefix se statusem
            job.append(
                &format!("{status_char} "),
                0.0,
                egui::TextFormat {
                    font_id: egui::FontId::monospace(font_size * 0.85),
                    color,
                    ..Default::default()
                },
            );

            // Ikona a název souboru
            let icon = file_icon(&node.name);
            job.append(
                &format!("{icon} {}", &node.name),
                0.0,
                egui::TextFormat {
                    font_id: egui::FontId::proportional(font_size),
                    color,
                    ..Default::default()
                },
            );

            let label = ui.selectable_label(false, job);
            if label.clicked() {
                *selected = Some(node.abs_path.clone());
            }
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
        let warm =
            resolve_file_tree_git_color(Some(GitVisualStatus::Modified), &warm_visuals, text_color);
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
