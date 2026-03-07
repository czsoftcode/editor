use std::path::{Path, PathBuf};
use serde_json::Value;

use crate::app::ui::terminal::ai_chat::slash::SlashResult;
use crate::app::ui::workspace::state::WorkspaceState;

/// GSD config wrapper around .planning/config.json.
pub struct GsdConfig {
    pub value: Value,
    pub path: PathBuf,
}

impl GsdConfig {
    /// Load config from root/.planning/config.json.
    /// Returns empty object if file doesn't exist.
    pub fn load(root: &Path) -> Self {
        todo!()
    }

    /// Get a value by dot-notation key (e.g., "workflow.auto_advance").
    pub fn get(&self, dot_path: &str) -> Option<&Value> {
        todo!()
    }

    /// Set a value by dot-notation key, creating intermediate objects if needed.
    pub fn set(&mut self, dot_path: &str, val: Value) {
        todo!()
    }

    /// Write config to disk.
    pub fn save(&self) -> Result<(), String> {
        todo!()
    }
}

/// Slash handler for /gsd config.
pub fn cmd_config(ws: &mut WorkspaceState, args: &str) -> SlashResult {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_load_missing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let cfg = GsdConfig::load(tmp.path());
        assert!(cfg.value.is_object());
        assert_eq!(cfg.value.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_load_existing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let planning = tmp.path().join(".planning");
        std::fs::create_dir_all(&planning).unwrap();
        std::fs::write(
            planning.join("config.json"),
            r#"{"mode":"yolo","workflow":{"auto_advance":true}}"#,
        )
        .unwrap();

        let cfg = GsdConfig::load(tmp.path());
        assert_eq!(cfg.value["mode"], json!("yolo"));
    }

    #[test]
    fn test_get_top_level() {
        let tmp = tempfile::tempdir().unwrap();
        let planning = tmp.path().join(".planning");
        std::fs::create_dir_all(&planning).unwrap();
        std::fs::write(
            planning.join("config.json"),
            r#"{"mode":"yolo","workflow":{"auto_advance":true}}"#,
        )
        .unwrap();

        let cfg = GsdConfig::load(tmp.path());
        assert_eq!(cfg.get("mode"), Some(&json!("yolo")));
    }

    #[test]
    fn test_get_nested() {
        let tmp = tempfile::tempdir().unwrap();
        let planning = tmp.path().join(".planning");
        std::fs::create_dir_all(&planning).unwrap();
        std::fs::write(
            planning.join("config.json"),
            r#"{"workflow":{"auto_advance":true}}"#,
        )
        .unwrap();

        let cfg = GsdConfig::load(tmp.path());
        assert_eq!(cfg.get("workflow.auto_advance"), Some(&json!(true)));
    }

    #[test]
    fn test_get_missing_key() {
        let tmp = tempfile::tempdir().unwrap();
        let cfg = GsdConfig::load(tmp.path());
        assert_eq!(cfg.get("nonexistent"), None);
        assert_eq!(cfg.get("a.b"), None);
    }

    #[test]
    fn test_set_and_save() {
        let tmp = tempfile::tempdir().unwrap();
        let mut cfg = GsdConfig::load(tmp.path());
        cfg.set("mode", json!("strict"));
        cfg.save().unwrap();

        // Reload and verify
        let cfg2 = GsdConfig::load(tmp.path());
        assert_eq!(cfg2.get("mode"), Some(&json!("strict")));
    }

    #[test]
    fn test_set_nested_creates_intermediate() {
        let tmp = tempfile::tempdir().unwrap();
        let mut cfg = GsdConfig::load(tmp.path());
        cfg.set("workflow.auto_advance", json!(true));
        assert_eq!(cfg.get("workflow.auto_advance"), Some(&json!(true)));
    }
}
