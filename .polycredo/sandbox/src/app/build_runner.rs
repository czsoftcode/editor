use std::path::{Path, PathBuf};
use std::sync::mpsc;

use serde_json::Value;

use super::types::{BuildProfile, path_env};
use super::ui::terminal::Terminal;

// ---------------------------------------------------------------------------
// Build / Runner Execution
// ---------------------------------------------------------------------------

pub(crate) fn run_profile(
    ctx: &eframe::egui::Context,
    project_root: &Path,
    profile: &BuildProfile,
    id_counter: &mut u64,
) -> Terminal {
    let mut full_command = profile.command.clone();
    for arg in &profile.args {
        full_command.push(' ');
        full_command.push_str(arg);
    }

    let working_dir = if let Some(sub) = &profile.working_dir {
        project_root.join(sub)
    } else {
        project_root.to_path_buf()
    };

    *id_counter += 1;
    Terminal::new(*id_counter, ctx, &working_dir, Some(&full_command))
}

// ---------------------------------------------------------------------------
// BuildError
// ---------------------------------------------------------------------------

pub struct BuildError {
    pub file: PathBuf,
    pub line: usize,
    pub _column: usize,
    pub message: String,
    pub is_warning: bool,
}

// ---------------------------------------------------------------------------
// run_build_check — starts `cargo build` in background, returns Receiver with results
// ---------------------------------------------------------------------------

pub(crate) fn run_build_check(root_path: PathBuf) -> mpsc::Receiver<Vec<BuildError>> {
    let (tx, rx) = mpsc::channel();
    let env = path_env();
    std::thread::spawn(move || {
        let output = std::process::Command::new("cargo")
            .args(["build", "--color=never", "--message-format=json"])
            .current_dir(&root_path)
            .env("PATH", &env)
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut errors = parse_build_messages_json(&stdout);
            if errors.is_empty() {
                // Fallback for cases where cargo/rustc only sends text output.
                let stderr = String::from_utf8_lossy(&output.stderr);
                errors = parse_build_errors_legacy(&stderr);
            }
            let _ = tx.send(errors);
        } else {
            let _ = tx.send(Vec::new());
        }
    });
    rx
}

fn parse_build_messages_json(stdout: &str) -> Vec<BuildError> {
    let mut errors = Vec::new();

    for line in stdout.lines() {
        let Ok(msg) = serde_json::from_str::<Value>(line) else {
            continue;
        };

        if msg
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or_default()
            != "compiler-message"
        {
            continue;
        }

        let Some(diagnostic) = msg.get("message") else {
            continue;
        };

        let level = diagnostic
            .get("level")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let is_warning = matches!(level, "warning");
        let is_error = matches!(level, "error");
        if !is_warning && !is_error {
            continue;
        }

        let spans = diagnostic
            .get("spans")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        let span = spans
            .iter()
            .find(|s| {
                s.get("is_primary")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            })
            .or_else(|| spans.first());
        let Some(span) = span else {
            continue;
        };

        let Some(file_name) = span.get("file_name").and_then(Value::as_str) else {
            continue;
        };
        if file_name.is_empty() || file_name.starts_with('<') {
            continue;
        }

        let line_num = span
            .get("line_start")
            .and_then(Value::as_u64)
            .unwrap_or(1)
            .max(1) as usize;
        let col = span
            .get("column_start")
            .and_then(Value::as_u64)
            .unwrap_or(1)
            .max(1) as usize;

        let message = diagnostic
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if message.is_empty() {
            continue;
        }

        errors.push(BuildError {
            file: PathBuf::from(file_name),
            line: line_num,
            _column: col,
            message,
            is_warning,
        });
    }

    errors
}

// ---------------------------------------------------------------------------
// parse_build_errors_legacy — fallback parser for text stderr
// ---------------------------------------------------------------------------

fn parse_build_errors_legacy(stderr: &str) -> Vec<BuildError> {
    let mut errors = Vec::new();
    let mut current_message: Option<(String, bool)> = None;

    for line in stderr.lines() {
        if line.starts_with("error") || line.starts_with("warning") {
            let is_warning = line.starts_with("warning");
            current_message = Some((line.to_string(), is_warning));
        } else if let Some(location) = line.trim_start().strip_prefix("--> ")
            && let Some((msg, is_warning)) = current_message.take()
        {
            let parts: Vec<&str> = location.rsplitn(3, ':').collect();
            if parts.len() >= 3
                && let (Ok(line_num), Ok(col)) =
                    (parts[1].parse::<usize>(), parts[0].parse::<usize>())
            {
                errors.push(BuildError {
                    file: PathBuf::from(parts[2]),
                    line: line_num,
                    _column: col,
                    message: msg,
                    is_warning,
                });
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_json_compiler_message_error() {
        let input = r#"{"reason":"compiler-message","message":{"level":"error","message":"cannot find value `x` in this scope","spans":[{"file_name":"src/main.rs","line_start":3,"column_start":5,"is_primary":true}]}}"#;
        let parsed = parse_build_messages_json(input);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].file, PathBuf::from("src/main.rs"));
        assert_eq!(parsed[0].line, 3);
        assert!(!parsed[0].is_warning);
    }

    #[test]
    fn parse_json_ignores_non_compiler_messages() {
        let input = r#"{"reason":"build-finished","success":false}"#;
        let parsed = parse_build_messages_json(input);
        assert!(parsed.is_empty());
    }
}
