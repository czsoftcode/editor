use crate::app::types::ProjectProfiles;
use std::path::{Path, PathBuf};

const POLYCREDO_DIR: &str = ".polycredo";
const PROFILES_FILE: &str = "profiles.toml";
const TRASH_DIR: &str = "trash";

pub(crate) fn project_config_dir(project_root: &Path) -> PathBuf {
    project_root.join(POLYCREDO_DIR)
}

pub(crate) fn profiles_path(project_root: &Path) -> PathBuf {
    project_config_dir(project_root).join(PROFILES_FILE)
}

pub(crate) fn trash_dir_path(project_root: &Path) -> PathBuf {
    project_config_dir(project_root).join(TRASH_DIR)
}

pub(crate) fn project_trash_dir(project_root: &Path) -> PathBuf {
    trash_dir_path(project_root)
}

pub(crate) fn trash_meta_path(entry_path: &Path) -> PathBuf {
    let file_name = entry_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".to_string());
    entry_path.with_file_name(format!("{file_name}.meta.json"))
}

pub(crate) fn load_profiles(project_root: &Path) -> ProjectProfiles {
    let path = profiles_path(project_root);
    if let Ok(content) = std::fs::read_to_string(&path) {
        toml::from_str(&content).unwrap_or_default()
    } else {
        // If not found, we might want to return some default runners based on project type
        // For now, return empty.
        ProjectProfiles::default()
    }
}

#[allow(dead_code)]
pub(crate) fn save_profiles(
    project_root: &Path,
    profiles: &ProjectProfiles,
) -> std::io::Result<()> {
    let config_dir = project_config_dir(project_root);
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
    }

    let path = profiles_path(project_root);
    let toml_str = toml::to_string_pretty(profiles).map_err(std::io::Error::other)?;

    std::fs::write(path, toml_str)
}
