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
