use std::path::PathBuf;
use std::sync::mpsc;

use super::types::path_env;

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
// run_build_check — spustí `cargo build` v pozadí, vrátí Receiver s výsledky
// ---------------------------------------------------------------------------

pub(crate) fn run_build_check(root_path: PathBuf) -> mpsc::Receiver<Vec<BuildError>> {
    let (tx, rx) = mpsc::channel();
    let env = path_env();
    std::thread::spawn(move || {
        let output = std::process::Command::new("cargo")
            .args(["build", "--color=never"])
            .current_dir(&root_path)
            .env("PATH", &env)
            .output();
        if let Ok(output) = output {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let _ = tx.send(parse_build_errors(&stderr));
        }
    });
    rx
}

// ---------------------------------------------------------------------------
// parse_build_errors — parsuje stderr z cargo build
// ---------------------------------------------------------------------------

pub(crate) fn parse_build_errors(stderr: &str) -> Vec<BuildError> {
    let mut errors = Vec::new();
    let mut current_message: Option<(String, bool)> = None;

    for line in stderr.lines() {
        if line.starts_with("error") || line.starts_with("warning") {
            let is_warning = line.starts_with("warning");
            current_message = Some((line.to_string(), is_warning));
        } else if let Some(location) = line.trim_start().strip_prefix("--> ") {
            if let Some((msg, is_warning)) = current_message.take() {
                let parts: Vec<&str> = location.rsplitn(3, ':').collect();
                if parts.len() >= 3 {
                    if let (Ok(line_num), Ok(col)) =
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
        }
    }
    errors
}
