use crate::app::ui::file_tree::FileTree;
use std::path::{Path, PathBuf};

pub enum ContextAction {
    NewFile(PathBuf),
    NewDir(PathBuf),
    Rename(PathBuf),
    Copy(PathBuf),
    Paste(PathBuf),
    Delete(PathBuf),
}

impl FileTree {
    pub fn handle_action(&mut self, action: ContextAction, i18n: &crate::i18n::I18n) {
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
                            let mut args = fluent_bundle::FluentArgs::new();
                            args.set("reason", e.to_string());
                            self.pending_error =
                                Some(i18n.get_args("file-tree-paste-error", &args));
                        }
                    }
                }
            }
            ContextAction::Delete(path) => {
                self.delete_confirm = Some(path);
            }
        }
    }
}

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    let src_meta = std::fs::symlink_metadata(src)?;
    if src_meta.file_type().is_symlink() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "symbolic links are not copied",
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
