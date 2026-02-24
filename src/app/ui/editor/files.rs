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
        read_only: bool,
    ) -> Option<String> {
        let should_save = self.active().is_some_and(|t| {
            !t.deleted
                && t.modified
                && t.last_edit
                    .is_some_and(|e| e.elapsed().as_millis() >= AUTOSAVE_DELAY_MS)
                // Prevent infinite retry loops on save error:
                // Only try to autosave if there was a NEW edit since the last attempt.
                && (t.last_autosave_attempt.is_none()
                    || t.last_edit.unwrap() > t.last_autosave_attempt.unwrap())
        });
        if should_save {
            // Prevent infinite error loops in Safe Mode:
            // If strictly read-only and outside sandbox, do not attempt autosave.
            // Explicit save (Ctrl+S) will still trigger the error via save().
            if read_only
                && let Some(tab) = self.active()
                && !tab.path.to_string_lossy().contains(".polycredo/sandbox")
            {
                // Mark as "attempted" so we don't check again until next edit,
                // effectively silencing the check.
                if let Some(tab_mut) = self.active_mut() {
                    tab_mut.last_autosave_attempt = Some(std::time::Instant::now());
                }
                return None;
            }

            // Mark attempt time *before* saving (or right now)
            if let Some(tab) = self.active_mut() {
                tab.last_autosave_attempt = Some(std::time::Instant::now());
            }

            self.save(i18n, is_internal_save, read_only)
        } else {
            None
        }
    }

    /// Saves the active tab. Returns an error message if writing fails, otherwise None.
    pub fn save(
        &mut self,
        i18n: &crate::i18n::I18n,
        is_internal_save: &std::sync::Arc<std::sync::atomic::AtomicBool>,
        read_only: bool,
    ) -> Option<String> {
        let tab = self.active_mut()?;

        // If Safe Mode is on, only allow saving files within the sandbox
        if read_only {
            let path_str = tab.path.to_string_lossy();
            if !path_str.contains(".polycredo/sandbox") {
                return Some(i18n.get("error-safe-mode-blocked"));
            }
        }

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
        read_only: bool,
    ) -> Option<String> {
        let tab = self.tabs.iter_mut().find(|t| t.path == *path)?;

        if read_only {
            let path_str = tab.path.to_string_lossy();
            if !path_str.contains(".polycredo/sandbox") {
                return Some(i18n.get("error-safe-mode-blocked"));
            }
        }

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
}
