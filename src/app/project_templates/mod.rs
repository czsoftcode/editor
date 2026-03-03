use crate::app::types::ProjectType;
use std::fs;
use std::path::Path;

mod expressjs;
mod fastapi;
mod laravel;
mod nette;
mod nextjs;
mod rust;
mod symfony;

pub(crate) fn generate_project(
    project_type: ProjectType,
    name: &str,
    target_path: &Path,
) -> Result<(), String> {
    if !target_path.exists() {
        fs::create_dir_all(target_path).map_err(|e| e.to_string())?;
    }

    match project_type {
        ProjectType::Rust => rust::generate(name, target_path),
        ProjectType::Symfony74 => symfony::generate(name, target_path, "7.4.*", "8.2"),
        ProjectType::Symfony80 => symfony::generate(name, target_path, "8.0.*", "8.4"),
        ProjectType::Laravel12 => laravel::generate(name, target_path),
        ProjectType::Nette32 => nette::generate(name, target_path, "3.2", "8.2"),
        ProjectType::Nette30 => nette::generate(name, target_path, "3.0", "7.4"),
        ProjectType::FastApi => fastapi::generate(name, target_path),
        ProjectType::NextJs => nextjs::generate(name, target_path),
        ProjectType::ExpressJs => expressjs::generate(name, target_path),
    }
}
