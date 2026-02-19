use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config;

const IGNORED_DIRS: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    ".idea",
    ".vscode",
    "__pycache__",
];

pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
    pub expanded: bool,
    pub children_loaded: bool,
}

impl FileNode {
    fn new(path: PathBuf, is_dir: bool) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        Self {
            name,
            path,
            is_dir,
            children: Vec::new(),
            expanded: false,
            children_loaded: !is_dir,
        }
    }

    fn load_children(&mut self) {
        if self.children_loaded || !self.is_dir {
            return;
        }
        self.children_loaded = true;

        let entries = match std::fs::read_dir(&self.path) {
            Ok(e) => e,
            Err(_) => return,
        };

        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if IGNORED_DIRS.contains(&name.as_str()) {
                continue;
            }

            let is_dir = path.is_dir();
            let node = FileNode::new(path, is_dir);
            if is_dir {
                dirs.push(node);
            } else {
                files.push(node);
            }
        }

        dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        self.children = dirs;
        self.children.append(&mut files);
    }
}

enum ContextAction {
    NewFile(PathBuf),
    NewDir(PathBuf),
    Rename(PathBuf),
    Copy(PathBuf),
    Paste(PathBuf),
    Delete(PathBuf),
}

#[derive(Default)]
pub struct FileTreeResult {
    pub selected: Option<PathBuf>,
    pub created_file: Option<PathBuf>,
    pub deleted: Option<PathBuf>,
}

pub struct FileTree {
    pub root: Option<FileNode>,
    root_path: PathBuf,
    clipboard: Option<PathBuf>,
    rename_target: Option<PathBuf>,
    rename_buffer: String,
    new_item_parent: Option<PathBuf>,
    new_item_buffer: String,
    new_item_is_dir: bool,
    delete_confirm: Option<PathBuf>,
    needs_reload: bool,
    pending_created_file: Option<PathBuf>,
    pending_deleted: Option<PathBuf>,
    expand_to: Option<PathBuf>,
    pending_error: Option<String>,
    /// Barvy souborů podle git stavu (absolutní cesty → barva)
    git_colors: HashMap<PathBuf, eframe::egui::Color32>,
}

impl FileTree {
    pub fn has_open_dialog(&self) -> bool {
        self.new_item_parent.is_some()
            || self.rename_target.is_some()
            || self.delete_confirm.is_some()
    }

    pub fn new() -> Self {
        Self {
            root: None,
            root_path: PathBuf::new(),
            clipboard: None,
            rename_target: None,
            rename_buffer: String::new(),
            new_item_parent: None,
            new_item_buffer: String::new(),
            new_item_is_dir: false,
            delete_confirm: None,
            needs_reload: false,
            pending_created_file: None,
            pending_deleted: None,
            expand_to: None,
            pending_error: None,
            git_colors: HashMap::new(),
        }
    }

    /// Nastaví mapování absolutních cest na barvy z git status.
    pub fn set_git_colors(&mut self, colors: HashMap<PathBuf, eframe::egui::Color32>) {
        self.git_colors = colors;
    }

    /// Vyzvedne případnou chybu I/O operace (pro zobrazení v toast notifikaci).
    pub fn take_error(&mut self) -> Option<String> {
        self.pending_error.take()
    }

    pub fn request_reload(&mut self) {
        self.needs_reload = true;
    }

    pub fn request_reload_and_expand(&mut self, target: &Path) {
        self.needs_reload = true;
        self.expand_to = Some(target.to_path_buf());
    }

    pub fn load(&mut self, path: &Path) {
        self.root_path = path.to_path_buf();
        let mut root = FileNode::new(path.to_path_buf(), true);
        root.expanded = true;
        root.load_children();
        self.root = Some(root);
    }

    pub fn ui(&mut self, ui: &mut eframe::egui::Ui) -> FileTreeResult {
        let mut result = FileTreeResult::default();

        if self.needs_reload {
            self.needs_reload = false;
            let path = self.root_path.clone();
            self.load(&path);
        }

        // Sebrání pending výsledků z předchozího framu
        result.created_file = self.pending_created_file.take();
        result.deleted = self.pending_deleted.take();

        let mut selected = None;
        let mut action = None;

        let expand_to = self.expand_to.take();
        if let Some(root) = &mut self.root {
            let has_clipboard = self.clipboard.is_some();
            Self::show_node(
                ui,
                root,
                &mut selected,
                &mut action,
                has_clipboard,
                &expand_to,
                &self.git_colors,
            );
        }

        if let Some(act) = action {
            self.handle_action(act);
        }

        self.show_dialogs(ui);

        result.selected = selected;
        result
    }

    fn show_node(
        ui: &mut eframe::egui::Ui,
        node: &mut FileNode,
        selected: &mut Option<PathBuf>,
        action: &mut Option<ContextAction>,
        has_clipboard: bool,
        expand_to: &Option<PathBuf>,
        git_colors: &HashMap<PathBuf, eframe::egui::Color32>,
    ) {
        let dark_mode = ui.visuals().dark_mode;
        let text_color = ui.visuals().text_color();
        let font_size = config::FILE_TREE_FONT_SIZE;

        // Git barvy jsou navrženy pro tmavé pozadí — v light mode je ztmavíme
        let adapt_git_color = |c: eframe::egui::Color32| -> eframe::egui::Color32 {
            if dark_mode {
                c
            } else {
                eframe::egui::Color32::from_rgb(
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

            let header_text = eframe::egui::RichText::new(format!("\u{1F4C1} {}", &node.name))
                .size(font_size)
                .color(text_color);
            let mut header =
                eframe::egui::CollapsingHeader::new(header_text).default_open(node.expanded);
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
                    );
                }
            });

            let header_response = response.header_response;
            header_response.context_menu(|ui| {
                let menu_size = 15.0;
                if ui
                    .button(eframe::egui::RichText::new("Nový soubor").size(menu_size))
                    .clicked()
                {
                    *action = Some(ContextAction::NewFile(node.path.clone()));
                    ui.close_menu();
                }
                if ui
                    .button(eframe::egui::RichText::new("Nový adresář").size(menu_size))
                    .clicked()
                {
                    *action = Some(ContextAction::NewDir(node.path.clone()));
                    ui.close_menu();
                }
                if ui
                    .button(eframe::egui::RichText::new("Přejmenovat").size(menu_size))
                    .clicked()
                {
                    *action = Some(ContextAction::Rename(node.path.clone()));
                    ui.close_menu();
                }
                if ui
                    .button(eframe::egui::RichText::new("Kopírovat").size(menu_size))
                    .clicked()
                {
                    *action = Some(ContextAction::Copy(node.path.clone()));
                    ui.close_menu();
                }
                if has_clipboard {
                    if ui
                        .button(eframe::egui::RichText::new("Vložit").size(menu_size))
                        .clicked()
                    {
                        *action = Some(ContextAction::Paste(node.path.clone()));
                        ui.close_menu();
                    }
                }
                if ui
                    .button(eframe::egui::RichText::new("Smazat").size(menu_size))
                    .clicked()
                {
                    *action = Some(ContextAction::Delete(node.path.clone()));
                    ui.close_menu();
                }
            });
        } else {
            let file_color = git_colors
                .get(&node.path)
                .copied()
                .map(adapt_git_color)
                .unwrap_or(text_color);
            let file_text = eframe::egui::RichText::new(format!("\u{1F4C4} {}", &node.name))
                .size(font_size)
                .color(file_color);
            let label = ui.selectable_label(false, file_text);
            if label.clicked() {
                *selected = Some(node.path.clone());
            }

            label.context_menu(|ui| {
                let menu_size = 15.0;
                if ui
                    .button(eframe::egui::RichText::new("Přejmenovat").size(menu_size))
                    .clicked()
                {
                    *action = Some(ContextAction::Rename(node.path.clone()));
                    ui.close_menu();
                }
                if ui
                    .button(eframe::egui::RichText::new("Kopírovat").size(menu_size))
                    .clicked()
                {
                    *action = Some(ContextAction::Copy(node.path.clone()));
                    ui.close_menu();
                }
                if ui
                    .button(eframe::egui::RichText::new("Smazat").size(menu_size))
                    .clicked()
                {
                    *action = Some(ContextAction::Delete(node.path.clone()));
                    ui.close_menu();
                }
            });
        }
    }

    fn handle_action(&mut self, action: ContextAction) {
        match action {
            ContextAction::NewFile(parent) => {
                self.new_item_parent = Some(parent);
                self.new_item_buffer = String::new();
                self.new_item_is_dir = false;
            }
            ContextAction::NewDir(parent) => {
                self.new_item_parent = Some(parent);
                self.new_item_buffer = String::new();
                self.new_item_is_dir = true;
            }
            ContextAction::Rename(path) => {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                self.rename_buffer = name;
                self.rename_target = Some(path);
            }
            ContextAction::Copy(path) => {
                self.clipboard = Some(path);
            }
            ContextAction::Paste(target_dir) => {
                if let Some(source) = &self.clipboard {
                    let source = source.clone();
                    let file_name = source
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let dest = target_dir.join(&file_name);
                    let result = if source.is_dir() {
                        copy_dir_recursive(&source, &dest)
                    } else {
                        std::fs::copy(&source, &dest).map(|_| ())
                    };
                    match result {
                        Ok(()) => {
                            self.needs_reload = true;
                        }
                        Err(e) => {
                            self.pending_error = Some(format!("Nelze vložit: {e}"));
                        }
                    }
                }
            }
            ContextAction::Delete(path) => {
                self.delete_confirm = Some(path);
            }
        }
    }

    fn show_dialogs(&mut self, ui: &mut eframe::egui::Ui) {
        self.show_new_item_dialog(ui);
        self.show_rename_dialog(ui);
        self.show_delete_dialog(ui);
    }

    fn show_new_item_dialog(&mut self, ui: &mut eframe::egui::Ui) {
        if self.new_item_parent.is_none() {
            return;
        }

        let title = if self.new_item_is_dir {
            "Nový adresář"
        } else {
            "Nový soubor"
        };

        let mut should_create = false;

        let modal = eframe::egui::Modal::new(eframe::egui::Id::new("new_item_modal"));
        let modal_response = modal.show(ui.ctx(), |ui| {
            let dlg_size = 15.0;
            ui.heading(title);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(eframe::egui::RichText::new("Název:").size(dlg_size));
                let response = ui.add(
                    eframe::egui::TextEdit::singleline(&mut self.new_item_buffer)
                        .font(eframe::egui::TextStyle::Body)
                        .desired_width(200.0),
                );
                if response.lost_focus() && ui.input(|i| i.key_pressed(eframe::egui::Key::Enter)) {
                    should_create = true;
                }
                if !response.has_focus() {
                    response.request_focus();
                }
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui
                    .button(eframe::egui::RichText::new("Vytvořit").size(dlg_size))
                    .clicked()
                {
                    should_create = true;
                }
                if ui
                    .button(eframe::egui::RichText::new("Zrušit").size(dlg_size))
                    .clicked()
                {
                    self.new_item_parent = None;
                }
            });
        });

        if modal_response.should_close() {
            self.new_item_parent = None;
        }

        if should_create && !self.new_item_buffer.trim().is_empty() {
            if let Some(parent) = &self.new_item_parent {
                let new_path = parent.join(self.new_item_buffer.trim());
                if self.new_item_is_dir {
                    match std::fs::create_dir(&new_path) {
                        Ok(()) => {
                            self.expand_to = Some(new_path);
                        }
                        Err(e) => {
                            self.pending_error = Some(format!("Nelze vytvořit adresář: {e}"));
                        }
                    }
                } else {
                    match std::fs::write(&new_path, "") {
                        Ok(()) => {
                            self.pending_created_file = Some(new_path.clone());
                            self.expand_to = Some(new_path);
                        }
                        Err(e) => {
                            self.pending_error = Some(format!("Nelze vytvořit soubor: {e}"));
                        }
                    }
                }
                self.needs_reload = true;
            }
            self.new_item_parent = None;
        }
    }

    fn show_rename_dialog(&mut self, ui: &mut eframe::egui::Ui) {
        if self.rename_target.is_none() {
            return;
        }

        let mut should_rename = false;

        let modal = eframe::egui::Modal::new(eframe::egui::Id::new("rename_modal"));
        let modal_response = modal.show(ui.ctx(), |ui| {
            let dlg_size = 15.0;
            ui.heading("Přejmenovat");
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(eframe::egui::RichText::new("Nový název:").size(dlg_size));
                let response = ui.add(
                    eframe::egui::TextEdit::singleline(&mut self.rename_buffer)
                        .font(eframe::egui::TextStyle::Body)
                        .desired_width(200.0),
                );
                if response.lost_focus() && ui.input(|i| i.key_pressed(eframe::egui::Key::Enter)) {
                    should_rename = true;
                }
                if !response.has_focus() {
                    response.request_focus();
                }
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui
                    .button(eframe::egui::RichText::new("Přejmenovat").size(dlg_size))
                    .clicked()
                {
                    should_rename = true;
                }
                if ui
                    .button(eframe::egui::RichText::new("Zrušit").size(dlg_size))
                    .clicked()
                {
                    self.rename_target = None;
                }
            });
        });

        if modal_response.should_close() {
            self.rename_target = None;
        }

        if should_rename && !self.rename_buffer.trim().is_empty() {
            if let Some(target) = &self.rename_target {
                if let Some(parent) = target.parent() {
                    let new_path = parent.join(self.rename_buffer.trim());
                    match std::fs::rename(target, &new_path) {
                        Ok(()) => {
                            self.needs_reload = true;
                        }
                        Err(e) => {
                            self.pending_error = Some(format!("Nelze přejmenovat: {e}"));
                        }
                    }
                }
            }
            self.rename_target = None;
        }
    }

    fn show_delete_dialog(&mut self, ui: &mut eframe::egui::Ui) {
        if self.delete_confirm.is_none() {
            return;
        }

        let path_display = self
            .delete_confirm
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let mut should_delete = false;
        let dlg_size = 15.0;

        let modal = eframe::egui::Modal::new(eframe::egui::Id::new("delete_modal"));
        let modal_response = modal.show(ui.ctx(), |ui| {
            ui.heading("Potvrdit smazání");
            ui.add_space(8.0);
            ui.label(
                eframe::egui::RichText::new(format!("Opravdu smazat?\n{}", path_display))
                    .size(dlg_size),
            );
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui
                    .button(eframe::egui::RichText::new("Ano").size(dlg_size))
                    .clicked()
                {
                    should_delete = true;
                }
                if ui
                    .button(eframe::egui::RichText::new("Ne").size(dlg_size))
                    .clicked()
                {
                    self.delete_confirm = None;
                }
            });
        });

        if modal_response.should_close() {
            self.delete_confirm = None;
        }

        if should_delete {
            if let Some(path) = self.delete_confirm.take() {
                let result = if path.is_dir() {
                    std::fs::remove_dir_all(&path)
                } else {
                    std::fs::remove_file(&path)
                };
                match result {
                    Ok(()) => {
                        self.pending_deleted = Some(path);
                        self.needs_reload = true;
                    }
                    Err(e) => {
                        self.pending_error = Some(format!("Nelze smazat: {e}"));
                    }
                }
            }
        }
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    let src_meta = std::fs::symlink_metadata(src)?;
    if src_meta.file_type().is_symlink() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "symbolické odkazy se nekopírují",
        ));
    }
    std::fs::create_dir(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let meta = std::fs::symlink_metadata(&src_path)?;
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
