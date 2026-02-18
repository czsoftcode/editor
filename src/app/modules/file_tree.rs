use std::path::{Path, PathBuf};

const IGNORED_DIRS: &[&str] = &[".git", "target", "node_modules", ".idea", ".vscode", "__pycache__"];

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
        }
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
            Self::show_node(ui, root, &mut selected, &mut action, has_clipboard, &expand_to);
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
    ) {
        let text_color = eframe::egui::Color32::from_rgb(230, 230, 230);
        let font_size = 16.0;

        if node.is_dir {
            let force_open = expand_to
                .as_ref()
                .is_some_and(|target| target.starts_with(&node.path));

            let header_text = eframe::egui::RichText::new(format!("\u{1F4C1} {}", &node.name))
                .size(font_size)
                .color(text_color);
            let mut header = eframe::egui::CollapsingHeader::new(header_text)
                .default_open(node.expanded);
            if force_open {
                header = header.open(Some(true));
            }
            let response = header.show(ui, |ui| {
                    node.load_children();
                    for child in &mut node.children {
                        Self::show_node(ui, child, selected, action, has_clipboard, expand_to);
                    }
                });

            let header_response = response.header_response;
            header_response.context_menu(|ui| {
                let menu_size = 15.0;
                if ui.button(eframe::egui::RichText::new("Nový soubor").size(menu_size)).clicked() {
                    *action = Some(ContextAction::NewFile(node.path.clone()));
                    ui.close_menu();
                }
                if ui.button(eframe::egui::RichText::new("Nový adresář").size(menu_size)).clicked() {
                    *action = Some(ContextAction::NewDir(node.path.clone()));
                    ui.close_menu();
                }
                if ui.button(eframe::egui::RichText::new("Přejmenovat").size(menu_size)).clicked() {
                    *action = Some(ContextAction::Rename(node.path.clone()));
                    ui.close_menu();
                }
                if ui.button(eframe::egui::RichText::new("Kopírovat").size(menu_size)).clicked() {
                    *action = Some(ContextAction::Copy(node.path.clone()));
                    ui.close_menu();
                }
                if has_clipboard {
                    if ui.button(eframe::egui::RichText::new("Vložit").size(menu_size)).clicked() {
                        *action = Some(ContextAction::Paste(node.path.clone()));
                        ui.close_menu();
                    }
                }
                if ui.button(eframe::egui::RichText::new("Smazat").size(menu_size)).clicked() {
                    *action = Some(ContextAction::Delete(node.path.clone()));
                    ui.close_menu();
                }
            });
        } else {
            let file_text = eframe::egui::RichText::new(format!("\u{1F4C4} {}", &node.name))
                .size(font_size)
                .color(text_color);
            let label = ui.selectable_label(false, file_text);
            if label.clicked() {
                *selected = Some(node.path.clone());
            }

            label.context_menu(|ui| {
                let menu_size = 15.0;
                if ui.button(eframe::egui::RichText::new("Přejmenovat").size(menu_size)).clicked() {
                    *action = Some(ContextAction::Rename(node.path.clone()));
                    ui.close_menu();
                }
                if ui.button(eframe::egui::RichText::new("Kopírovat").size(menu_size)).clicked() {
                    *action = Some(ContextAction::Copy(node.path.clone()));
                    ui.close_menu();
                }
                if ui.button(eframe::egui::RichText::new("Smazat").size(menu_size)).clicked() {
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
                    if source.is_dir() {
                        let _ = copy_dir_recursive(&source, &dest);
                    } else {
                        let _ = std::fs::copy(&source, &dest);
                    }
                    self.needs_reload = true;
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
                if response.lost_focus()
                    && ui.input(|i| i.key_pressed(eframe::egui::Key::Enter))
                {
                    should_create = true;
                }
                if !response.has_focus() {
                    response.request_focus();
                }
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button(eframe::egui::RichText::new("Vytvořit").size(dlg_size)).clicked() {
                    should_create = true;
                }
                if ui.button(eframe::egui::RichText::new("Zrušit").size(dlg_size)).clicked() {
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
                    let _ = std::fs::create_dir(&new_path);
                    self.expand_to = Some(new_path);
                } else {
                    let _ = std::fs::write(&new_path, "");
                    self.pending_created_file = Some(new_path.clone());
                    self.expand_to = Some(new_path);
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
                if response.lost_focus()
                    && ui.input(|i| i.key_pressed(eframe::egui::Key::Enter))
                {
                    should_rename = true;
                }
                if !response.has_focus() {
                    response.request_focus();
                }
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button(eframe::egui::RichText::new("Přejmenovat").size(dlg_size)).clicked() {
                    should_rename = true;
                }
                if ui.button(eframe::egui::RichText::new("Zrušit").size(dlg_size)).clicked() {
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
                    let _ = std::fs::rename(target, &new_path);
                    self.needs_reload = true;
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
            ui.label(eframe::egui::RichText::new(format!("Opravdu smazat?\n{}", path_display)).size(dlg_size));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button(eframe::egui::RichText::new("Ano").size(dlg_size)).clicked() {
                    should_delete = true;
                }
                if ui.button(eframe::egui::RichText::new("Ne").size(dlg_size)).clicked() {
                    self.delete_confirm = None;
                }
            });
        });

        if modal_response.should_close() {
            self.delete_confirm = None;
        }

        if should_delete {
            if let Some(path) = self.delete_confirm.take() {
                if path.is_dir() {
                    let _ = std::fs::remove_dir_all(&path);
                } else {
                    let _ = std::fs::remove_file(&path);
                }
                self.pending_deleted = Some(path);
                self.needs_reload = true;
            }
        }
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
