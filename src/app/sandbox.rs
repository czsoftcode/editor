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

    /// Synchronizes the sandbox with the project root.
    /// Only copies files that are missing or different in the sandbox.
    /// Skips .git, target and other ignored directories.
    pub fn sync_from_project(&self) -> Result<(), String> {
        for entry in WalkDir::new(&self.project_root)
            .into_iter()
            .filter_entry(|e| !Self::is_ignored(e.path()))
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

                // Copy only if size or mtime differs (simple check for speed)
                let should_copy = if !sandbox_path.exists() {
                    true
                } else {
                    let m1 = entry.metadata().map_err(|e| e.to_string())?;
                    let m2 = sandbox_path.metadata().map_err(|e| e.to_string())?;
                    m1.len() != m2.len()
                };

                if should_copy {
                    fs::copy(entry.path(), &sandbox_path).map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    }

    /// Checks if a path should be ignored during sandbox sync.
    fn is_ignored(path: &Path) -> bool {
        let s = path.to_string_lossy();
        s.contains("/.git/")
            || s.ends_with("/.git")
            || s.contains("/target/")
            || s.ends_with("/target")
            || s.contains("/.polycredo/")
            || s.ends_with("/.polycredo")
            || s.contains("/node_modules/")
            || s.ends_with("/node_modules")
    }

    /// Promotes a file from sandbox back to the real project.
    #[allow(dead_code)]
    pub fn promote_file(&self, relative_path: &Path) -> Result<(), String> {
        let src = self.root.join(relative_path);
        let dst = self.project_root.join(relative_path);

        if src.exists() {
            fs::copy(src, dst).map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Source file does not exist in sandbox".to_string())
        }
    }
}
