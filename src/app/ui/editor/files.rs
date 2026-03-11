use super::*;
use std::path::PathBuf;

impl Editor {
    // --- File operations ---

    /// Returns true if a tab for the given path exists and has unsaved changes.
    pub fn is_path_modified(&self, path: &PathBuf) -> bool {
        self.tabs.iter().any(|t| t.path == *path && t.modified)
    }

    /// Finds the tab path whose canonicalized path matches `canonical`.
    /// Returns the original (non-canonicalized) tab path if it exists.
    pub fn tab_path_for_canonical(&self, canonical: &PathBuf) -> Option<PathBuf> {
        self.tabs
            .iter()
            .find(|t| t.canonical_path == *canonical)
            .map(|t| t.path.clone())
    }

    /// Reloads a specific tab (by path) from disk — regardless of the active tab.
    pub fn reload_path_from_disk(&mut self, path: &PathBuf) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.path == *path) {
            if tab.is_binary {
                if let Ok(bytes) = std::fs::read(&tab.path) {
                    tab.binary_data = Some(bytes);
                    tab.image_texture = None;
                    tab.modified = false;
                    tab.last_edit = None;
                    tab.save_status = SaveStatus::Saved;
                }
            } else if let Ok(content) = std::fs::read_to_string(&tab.path) {
                tab.content = content.clone();
                tab.last_saved_content = content;
                tab.modified = false;
                tab.last_edit = None;
                tab.save_status = SaveStatus::Saved;
            }
        }
        self.update_search();
    }

    /// Attempts to autosave the active tab. Returns an error message if writing fails.
    pub fn try_autosave(
        &mut self,
        i18n: &crate::i18n::I18n,
        is_internal_save: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Option<String> {
        let should_save = self.should_autosave();
        if should_save {
            // Mark attempt time *before* saving (or right now)
            if let Some(tab) = self.active_mut() {
                tab.last_autosave_attempt = Some(std::time::Instant::now());
            }

            self.save(i18n, is_internal_save)
        } else {
            None
        }
    }

    /// Vrací true, pokud má aktivní tab projít autosave právě teď.
    pub fn should_autosave(&self) -> bool {
        self.active().is_some_and(|t| {
            !t.deleted
                && t.modified
                && t.last_edit
                    .is_some_and(|e| e.elapsed().as_millis() >= AUTOSAVE_DELAY_MS)
                // Prevent infinite retry loops on save error:
                // Only try to autosave if there was a NEW edit since the last attempt.
                && (t.last_autosave_attempt.is_none()
                    || t.last_edit.unwrap() > t.last_autosave_attempt.unwrap())
        })
    }

    /// Saves the active tab. Returns an error message if writing fails, otherwise None.
    pub fn save(
        &mut self,
        i18n: &crate::i18n::I18n,
        is_internal_save: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Option<String> {
        let tab = self.active_mut()?;

        if tab.read_error {
            let mut args = fluent_bundle::FluentArgs::new();
            args.set(
                "name",
                tab.path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            );
            return Some(i18n.get_args("error-file-read-only-error", &args));
        }

        tab.save_status = SaveStatus::Saving;

        is_internal_save.store(true, std::sync::atomic::Ordering::SeqCst);
        let res = if tab.is_binary {
            if let Some(bytes) = &tab.binary_data {
                std::fs::write(&tab.path, bytes)
            } else {
                Ok(())
            }
        } else {
            std::fs::write(&tab.path, &tab.content)
        };
        is_internal_save.store(false, std::sync::atomic::Ordering::SeqCst);

        match res {
            Ok(()) => {
                if !tab.is_binary {
                    tab.last_saved_content = tab.content.clone();
                }
                tab.modified = false;
                tab.last_edit = None;
                tab.save_status = SaveStatus::Saved;
                None
            }
            Err(e) => {
                tab.save_status = SaveStatus::Modified;
                let name = tab
                    .path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                let mut args = fluent_bundle::FluentArgs::new();
                args.set("name", name);
                args.set("reason", e.to_string());
                Some(i18n.get_args("error-file-save", &args))
            }
        }
    }

    /// Saves a specific tab identified by path (regardless of the active tab).
    /// Returns an error message if writing fails, otherwise None.
    pub fn save_path(
        &mut self,
        path: &PathBuf,
        i18n: &crate::i18n::I18n,
        is_internal_save: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Option<String> {
        let tab = self.tabs.iter_mut().find(|t| t.path == *path)?;

        if tab.read_error {
            let mut args = fluent_bundle::FluentArgs::new();
            args.set(
                "name",
                tab.path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            );
            return Some(i18n.get_args("error-file-read-only-error", &args));
        }

        tab.save_status = SaveStatus::Saving;

        is_internal_save.store(true, std::sync::atomic::Ordering::SeqCst);
        let res = if tab.is_binary {
            if let Some(bytes) = &tab.binary_data {
                std::fs::write(&tab.path, bytes)
            } else {
                Ok(())
            }
        } else {
            std::fs::write(&tab.path, &tab.content)
        };
        is_internal_save.store(false, std::sync::atomic::Ordering::SeqCst);

        match res {
            Ok(()) => {
                if !tab.is_binary {
                    tab.last_saved_content = tab.content.clone();
                }
                tab.modified = false;
                tab.last_edit = None;
                tab.save_status = SaveStatus::Saved;
                None
            }
            Err(e) => {
                tab.save_status = SaveStatus::Modified;
                let name = tab
                    .path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                let mut args = fluent_bundle::FluentArgs::new();
                args.set("name", name);
                args.set("reason", e.to_string());
                Some(i18n.get_args("error-file-save", &args))
            }
        }
    }

    pub fn remap_tabs_for_root_change(
        &mut self,
        from_root: &std::path::Path,
        to_root: &std::path::Path,
    ) -> TabRemapSummary {
        let mut remapped = 0;
        let mut missing = 0;
        let mut reload_paths: Vec<PathBuf> = Vec::new();

        for tab in &mut self.tabs {
            let Ok(rel_path) = tab.path.strip_prefix(from_root) else {
                continue;
            };
            let new_path = to_root.join(rel_path);
            let exists = new_path.exists();

            tab.path = new_path.clone();
            tab.canonical_path = new_path.canonicalize().unwrap_or_else(|_| new_path.clone());
            tab.deleted = !exists;

            if exists && !tab.modified {
                reload_paths.push(new_path);
            } else if !exists {
                missing += 1;
            }

            remapped += 1;
        }

        for path in reload_paths {
            self.reload_path_from_disk(&path);
        }

        let expand_to = self
            .active_tab
            .and_then(|idx| self.tabs.get(idx))
            .map(|tab| tab.path.clone())
            .filter(|path| path.exists());

        self.update_search();
        self.scroll_to_active = true;

        TabRemapSummary {
            remapped,
            missing,
            expand_to,
        }
    }
}

pub struct TabRemapSummary {
    pub remapped: usize,
    pub missing: usize,
    pub expand_to: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(label: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        dir.push(format!(
            "polycredo_editor_test_{}_{}_{}",
            label,
            stamp,
            std::process::id()
        ));
        dir
    }

    fn write_file(path: &PathBuf, content: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    #[test]
    fn test_remap_tabs_reloads_when_target_exists() {
        let old_root = temp_root("old");
        let new_root = temp_root("new");
        let rel = PathBuf::from("src/lib.rs");
        let old_path = old_root.join(&rel);
        let new_path = new_root.join(&rel);

        write_file(&old_path, "old");
        write_file(&new_path, "new");

        let mut editor = Editor::new();
        editor.open_file(&old_path);

        let summary = editor.remap_tabs_for_root_change(&old_root, &new_root);
        let tab = editor.tabs.first().unwrap();

        assert_eq!(summary.remapped, 1);
        assert_eq!(summary.missing, 0);
        assert_eq!(tab.path, new_path);
        assert!(!tab.deleted);
        assert_eq!(tab.content, "new");
    }

    #[test]
    fn test_remap_tabs_marks_missing_when_target_absent() {
        let old_root = temp_root("old_missing");
        let new_root = temp_root("new_missing");
        let rel = PathBuf::from("src/main.rs");
        let old_path = old_root.join(&rel);

        write_file(&old_path, "old");

        let mut editor = Editor::new();
        editor.open_file(&old_path);

        let summary = editor.remap_tabs_for_root_change(&old_root, &new_root);
        let tab = editor.tabs.first().unwrap();

        assert_eq!(summary.remapped, 1);
        assert_eq!(summary.missing, 1);
        assert!(tab.deleted);
    }
}
