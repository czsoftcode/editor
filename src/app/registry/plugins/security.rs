use crate::app::registry::plugins::types::HostContext;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct Blacklist {
    pub patterns: Vec<String>,
    pub regexes: Vec<regex::Regex>,
}

impl Blacklist {
    pub fn is_blacklisted(&self, path: &str) -> bool {
        for re in &self.regexes {
            if re.is_match(path) {
                return true;
            }
        }
        // Fallback for cases where regex compilation might have failed or simple patterns
        for pattern in &self.patterns {
            if path.contains(pattern) {
                return true;
            }
        }
        false
    }
}

pub fn compile_glob(pattern: &str) -> Option<regex::Regex> {
    let regex_pattern = pattern
        .replace('.', r"\.")
        .replace('*', ".*")
        .replace('?', ".");
    regex::Regex::new(&format!("^{}$", regex_pattern)).ok()
}

#[derive(Clone)]
pub struct HostState {
    pub plugin_id: String,
    pub sandbox_root: PathBuf,
    pub blacklist: Arc<Mutex<Blacklist>>,
    pub context: Arc<Mutex<HostContext>>,
    pub action_sender: Option<std::sync::mpsc::Sender<crate::app::types::AppAction>>,
    pub egui_ctx: Option<eframe::egui::Context>,
}

impl HostState {
    pub fn is_allowed(&self, rel_path: &Path) -> bool {
        let path_str = rel_path.to_string_lossy();

        // Allow absolute paths ONLY if they are within the PolyCredo config directory.
        // This enables cross-project memory and shared configuration for plugins.
        if rel_path.is_absolute() {
            let config_dir = crate::ipc::plugins_dir().parent().unwrap().to_path_buf();
            if rel_path.starts_with(&config_dir) {
                return true;
            }
            return false;
        }

        if path_str.contains("..") {
            return false;
        }

        let blacklist = self.blacklist.lock().expect("lock");
        if blacklist.is_blacklisted(&path_str) {
            return false;
        }

        true
    }
}
