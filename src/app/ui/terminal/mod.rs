pub mod ai_chat;
pub mod bottom;
pub mod instance;
pub mod right;
pub mod window;

pub use instance::Terminal;
pub use window::StandardTerminalWindow;

fn shorten_project_path(project_root: &std::path::Path) -> String {
    let parts: Vec<String> = project_root
        .components()
        .filter_map(|part| {
            let value = part.as_os_str().to_string_lossy().to_string();
            if value.is_empty() || value == "/" {
                None
            } else {
                Some(value)
            }
        })
        .collect();

    if parts.len() < 2 {
        project_root.display().to_string()
    } else {
        format!(".../{}/{}", parts[parts.len() - 2], parts[parts.len() - 1])
    }
}

pub fn terminal_mode_label(sandbox_mode_enabled: bool, _project_root: &std::path::Path) -> String {
    if sandbox_mode_enabled {
        "Sandbox".to_string()
    } else {
        "Terminál".to_string()
    }
}

pub fn terminal_mode_label_for_workdir(
    working_dir: &std::path::Path,
    sandbox_root: &std::path::Path,
    _project_root: &std::path::Path,
) -> String {
    if working_dir.starts_with(sandbox_root) {
        "Sandbox".to_string()
    } else {
        "Terminál".to_string()
    }
}

pub fn terminal_working_dir<'a>(
    sandbox_mode_enabled: bool,
    sandbox_root: &'a std::path::Path,
    project_root: &'a std::path::Path,
) -> &'a std::path::Path {
    if sandbox_mode_enabled {
        sandbox_root
    } else {
        project_root
    }
}

#[cfg(test)]
mod tests {
    use super::terminal_mode_label_for_workdir;

    #[test]
    fn test_terminal_mode_label_for_workdir_uses_sandbox_label() {
        let project_root = std::path::Path::new("/home/user/project");
        let sandbox_root = project_root.join(".polycredo").join("sandbox");
        let label = terminal_mode_label_for_workdir(&sandbox_root, &sandbox_root, project_root);
        assert_eq!(label, "Sandbox");
    }

    #[test]
    fn test_terminal_mode_label_for_workdir_uses_project_label() {
        let project_root = std::path::Path::new("/home/user/project");
        let sandbox_root = project_root.join(".polycredo").join("sandbox");
        let label = terminal_mode_label_for_workdir(project_root, &sandbox_root, project_root);
        assert_eq!(label, "Terminál");
    }
}
