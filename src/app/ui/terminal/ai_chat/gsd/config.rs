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
        let path = root.join(".planning").join("config.json");
        let value = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| Value::Object(serde_json::Map::new()));
        Self { value, path }
    }

    /// Get a value by dot-notation key (e.g., "workflow.auto_advance").
    pub fn get(&self, dot_path: &str) -> Option<&Value> {
        let parts: Vec<&str> = dot_path.split('.').collect();
        let mut current = &self.value;
        for part in parts {
            current = current.get(part)?;
        }
        Some(current)
    }

    /// Set a value by dot-notation key, creating intermediate objects if needed.
    pub fn set(&mut self, dot_path: &str, val: Value) {
        let parts: Vec<&str> = dot_path.split('.').collect();
        let mut current = &mut self.value;
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                current[*part] = val;
                return;
            }
            if !current.get(*part).map_or(false, |v| v.is_object()) {
                current[*part] = Value::Object(serde_json::Map::new());
            }
            current = &mut current[*part];
        }
    }

    /// Write config to disk.
    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create directory: {e}"))?;
        }
        let json = serde_json::to_string_pretty(&self.value)
            .map_err(|e| format!("JSON serialization error: {e}"))?;
        std::fs::write(&self.path, json)
            .map_err(|e| format!("Cannot write config: {e}"))?;
        Ok(())
    }
}

/// Parse a string value: try bool, then int, then float, fallback to string.
fn parse_value_str(s: &str) -> Value {
    if s == "true" {
        return Value::Bool(true);
    }
    if s == "false" {
        return Value::Bool(false);
    }
    if let Ok(n) = s.parse::<i64>() {
        return Value::Number(serde_json::Number::from(n));
    }
    if let Ok(f) = s.parse::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            return Value::Number(n);
        }
    }
    Value::String(s.to_string())
}

/// Format a JSON value for display.
fn format_value(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Null => "null".to_string(),
        _ => serde_json::to_string_pretty(v).unwrap_or_else(|_| format!("{v:?}")),
    }
}

/// Slash handler for /gsd config.
pub fn cmd_config(ws: &mut WorkspaceState, args: &str) -> SlashResult {
    let root = &ws.root_path;
    let parts: Vec<&str> = args.splitn(3, char::is_whitespace).collect();
    let sub = parts.first().map(|s| *s).unwrap_or("");

    match sub {
        "" => {
            // Show all config
            let cfg = GsdConfig::load(root);
            let pretty = serde_json::to_string_pretty(&cfg.value)
                .unwrap_or_else(|_| "{}".to_string());
            SlashResult::Immediate(format!("## GSD Config\n\n```json\n{}\n```", pretty))
        }
        "get" => {
            let key = parts.get(1).unwrap_or(&"").trim();
            if key.is_empty() {
                return SlashResult::Immediate(
                    "Usage: `/gsd config get <key>` (e.g., `/gsd config get workflow.auto_advance`)".to_string()
                );
            }
            let cfg = GsdConfig::load(root);
            match cfg.get(key) {
                Some(v) => SlashResult::Immediate(format!("`{}` = `{}`", key, format_value(v))),
                None => SlashResult::Immediate(format!("Key `{}` not found in config.", key)),
            }
        }
        "set" => {
            let key = parts.get(1).unwrap_or(&"").trim();
            let val_str = parts.get(2).unwrap_or(&"").trim();
            if key.is_empty() || val_str.is_empty() {
                return SlashResult::Immediate(
                    "Usage: `/gsd config set <key> <value>` (e.g., `/gsd config set workflow.auto_advance true`)".to_string()
                );
            }
            let mut cfg = GsdConfig::load(root);
            let val = parse_value_str(val_str);
            cfg.set(key, val.clone());
            match cfg.save() {
                Ok(()) => SlashResult::Immediate(format!("Set `{}` = `{}`", key, format_value(&val))),
                Err(e) => SlashResult::Immediate(format!("Error saving config: {}", e)),
            }
        }
        _ => {
            SlashResult::Immediate(
                "Unknown config subcommand. Usage: `/gsd config`, `/gsd config get <key>`, `/gsd config set <key> <value>`".to_string()
            )
        }
    }
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

    #[test]
    fn test_parse_value_str() {
        assert_eq!(parse_value_str("true"), json!(true));
        assert_eq!(parse_value_str("false"), json!(false));
        assert_eq!(parse_value_str("42"), json!(42));
        assert_eq!(parse_value_str("3.14"), json!(3.14));
        assert_eq!(parse_value_str("hello"), json!("hello"));
    }
}
