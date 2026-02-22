use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct Sandbox {
    pub root: PathBuf,
    project_root: PathBuf,
}

impl Sandbox {
    pub fn new(project_root: &Path) -> Self {
        let root = project_root.join(".polycredo").join("sandbox");
        if !root.exists() {
            let _ = fs::create_dir_all(&root);
        }
        Self {
            root,
            project_root: project_root.to_path_buf(),
        }
    }

    pub fn new_with_roots(project_root: PathBuf, root: PathBuf) -> Self {
        Self { root, project_root }
    }

    /// Synchronizes the sandbox with the project root.
    /// Only copies files that are missing or different in the sandbox.
    /// Skips .git, target and other ignored directories.
    pub fn sync_from_project(&self) -> Result<(), String> {
        for entry in WalkDir::new(&self.project_root)
            .into_iter()
            .filter_entry(|e| !Self::is_ignored_in_project(e.path()))
        {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.file_type().is_file() {
                let rel_path = entry.path().strip_prefix(&self.project_root).unwrap();
                let sandbox_path = self.root.join(rel_path);

                if let Some(parent) = sandbox_path.parent()
                    && !parent.exists()
                {
                    let _ = fs::create_dir_all(parent);
                }

                // Copy if missing or project version is newer
                let should_copy = if !sandbox_path.exists() {
                    true
                } else {
                    let m_project = entry.metadata().map_err(|e| e.to_string())?;
                    let m_sandbox = sandbox_path.metadata().map_err(|e| e.to_string())?;

                    let t_project = m_project.modified().map_err(|e| e.to_string())?;
                    let t_sandbox = m_sandbox.modified().map_err(|e| e.to_string())?;

                    t_project > t_sandbox || m_project.len() != m_sandbox.len()
                };

                if should_copy {
                    fs::copy(entry.path(), &sandbox_path).map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    }

    /// Checks if a path should be ignored during sandbox sync from project.
    fn is_ignored_in_project(path: &Path) -> bool {
        path.components().any(|c| {
            let s = c.as_os_str().to_string_lossy();
            matches!(
                s.as_ref(),
                ".git" | "target" | "node_modules" | ".polycredo"
            )
        })
    }

    /// Common ignores for both project and sandbox (build artifacts, git, etc).
    fn is_ignored_common(path: &Path) -> bool {
        path.components().any(|c| {
            let s = c.as_os_str().to_string_lossy();
            matches!(s.as_ref(), ".git" | "target" | "node_modules")
        })
    }

    /// Promotes a file from sandbox back to the real project.
    /// If the file exists in the project but NOT in the sandbox, it will be DELETED from the project.
    pub fn promote_file(&self, relative_path: &Path) -> Result<(), String> {
        let src = self.root.join(relative_path);
        let dst = self.project_root.join(relative_path);

        match (src.exists(), dst.exists()) {
            (true, _) => {
                // Ensure destination directory exists
                if let Some(parent) = dst.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create destination directory: {}", e))?;
                }
                fs::copy(&src, &dst)
                    .map_err(|e| format!("Failed to copy file from sandbox: {}", e))?;
                Ok(())
            }
            (false, true) => {
                // File was deleted in sandbox, so delete it in project too
                fs::remove_file(&dst)
                    .map_err(|e| format!("Failed to remove file from project: {}", e))?;
                Ok(())
            }
            (false, false) => {
                // Neither exists - might have been deleted externally while we were promoting
                Err(format!(
                    "Promotion failed: File {} does not exist in sandbox or project",
                    relative_path.display()
                ))
            }
        }
    }

    /// Returns a list of relative paths of files that are different in the sandbox
    /// compared to the project root.
    pub fn get_staged_files(&self) -> Vec<PathBuf> {
        use std::collections::HashSet;
        let mut staged_set = HashSet::new();

        // 1. Detect New and Modified files (present in sandbox)
        for entry in WalkDir::new(&self.root)
            .into_iter()
            .filter_entry(|e| !Self::is_ignored_common(e.path()))
        {
            let Ok(entry) = entry else { continue };
            if entry.file_type().is_file() {
                let abs_sandbox_path = entry.path();
                let Ok(rel_path) = abs_sandbox_path.strip_prefix(&self.root) else {
                    continue;
                };
                let abs_project_path = self.project_root.join(rel_path);

                let is_staged = if !abs_project_path.exists() {
                    true
                } else {
                    let Ok(m_sandbox) = abs_sandbox_path.metadata() else {
                        continue;
                    };
                    let Ok(m_project) = abs_project_path.metadata() else {
                        continue;
                    };

                    let Ok(t_sandbox) = m_sandbox.modified() else {
                        continue;
                    };
                    let Ok(t_project) = m_project.modified() else {
                        continue;
                    };

                    if t_sandbox > t_project {
                        let s_content =
                            std::fs::read_to_string(abs_sandbox_path).unwrap_or_default();
                        let p_content =
                            std::fs::read_to_string(&abs_project_path).unwrap_or_default();
                        s_content != p_content
                    } else {
                        false
                    }
                };

                if is_staged {
                    staged_set.insert(rel_path.to_path_buf());
                }
            }
        }

        // 2. Detect Deleted files (present in project, missing in sandbox)
        for entry in WalkDir::new(&self.project_root)
            .into_iter()
            .filter_entry(|e| !Self::is_ignored_in_project(e.path()))
        {
            let Ok(entry) = entry else { continue };
            if entry.file_type().is_file() {
                let rel_path = entry.path().strip_prefix(&self.project_root).unwrap();
                let sandbox_path = self.root.join(rel_path);

                if !sandbox_path.exists() {
                    staged_set.insert(rel_path.to_path_buf());
                }
            }
        }

        let mut staged: Vec<PathBuf> = staged_set.into_iter().collect();
        staged.sort();
        staged
    }
}
