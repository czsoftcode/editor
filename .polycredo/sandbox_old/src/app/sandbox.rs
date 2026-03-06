use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use xxhash_rust::xxh3::xxh3_64;

pub struct Sandbox {
    pub root: PathBuf,
    project_root: PathBuf,
}

#[derive(Default, Clone)]
pub struct SyncPlan {
    pub to_sandbox: Vec<PathBuf>, // Newer in Project
    pub to_project: Vec<PathBuf>, // Newer in Sandbox
}

impl SyncPlan {
    pub fn is_empty(&self) -> bool {
        self.to_sandbox.is_empty() && self.to_project.is_empty()
    }
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

    pub fn project_root(&self) -> &std::path::Path {
        &self.project_root
    }

    fn calculate_file_hash(path: &Path) -> Option<u64> {
        let bytes = fs::read(path).ok()?;
        Some(xxh3_64(&bytes))
    }

    /// Analyzes differences and returns a plan for bidirectional sync.
    pub fn get_sync_plan(&self) -> SyncPlan {
        let mut plan = SyncPlan::default();

        // 1. Check Project -> Sandbox (Newer in Project)
        for entry in WalkDir::new(&self.project_root)
            .into_iter()
            .filter_entry(|e| !Self::is_ignored_in_project(e.path()))
        {
            let Ok(entry) = entry else { continue };
            if entry.file_type().is_file() {
                let rel_path = entry.path().strip_prefix(&self.project_root).unwrap();
                let sandbox_path = self.root.join(rel_path);

                if !sandbox_path.exists() {
                    plan.to_sandbox.push(rel_path.to_path_buf());
                } else {
                    let Ok(m_project) = entry.metadata() else {
                        continue;
                    };
                    let Ok(m_sandbox) = sandbox_path.metadata() else {
                        continue;
                    };

                    // Only consider if size or hash differs
                    let different = if m_project.len() != m_sandbox.len() {
                        true
                    } else {
                        let h_project = Self::calculate_file_hash(entry.path());
                        let h_sandbox = Self::calculate_file_hash(&sandbox_path);
                        h_project != h_sandbox
                    };

                    if different {
                        let Ok(t_project) = m_project.modified() else {
                            continue;
                        };
                        let Ok(t_sandbox) = m_sandbox.modified() else {
                            continue;
                        };
                        if t_project > t_sandbox {
                            plan.to_sandbox.push(rel_path.to_path_buf());
                        }
                    }
                }
            }
        }

        // 2. Check Sandbox -> Project (Newer in Sandbox)
        for entry in WalkDir::new(&self.root)
            .into_iter()
            .filter_entry(|e| !Self::is_ignored_common(e.path()))
        {
            let Ok(entry) = entry else { continue };
            if entry.file_type().is_file() {
                let rel_path = entry.path().strip_prefix(&self.root).unwrap();
                let project_path = self.project_root.join(rel_path);

                if project_path.exists() {
                    let Ok(m_sandbox) = entry.metadata() else {
                        continue;
                    };
                    let Ok(m_project) = project_path.metadata() else {
                        continue;
                    };

                    // Only consider if size or hash differs
                    let different = if m_sandbox.len() != m_project.len() {
                        true
                    } else {
                        let h_sandbox = Self::calculate_file_hash(entry.path());
                        let h_project = Self::calculate_file_hash(&project_path);
                        h_sandbox != h_project
                    };

                    if different {
                        let Ok(t_sandbox) = m_sandbox.modified() else {
                            continue;
                        };
                        let Ok(t_project) = m_project.modified() else {
                            continue;
                        };
                        if t_sandbox > t_project {
                            plan.to_project.push(rel_path.to_path_buf());
                        }
                    }
                } else {
                    // New files in sandbox are handled by the standard "promote" UI,
                    // but we can include them here if we want full auto-sync.
                    // For now, let's keep it to modified files only to be safe.
                }
            }
        }

        plan
    }

    /// Synchronizes the sandbox with the project root.
    /// Only copies files that are missing or different in the sandbox.
    /// Skips .git, target and other ignored directories.
    #[allow(dead_code)]
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

    /// Applies a precomputed sync plan from project to sandbox.
    pub fn sync_plan_to_sandbox(&self, plan: &SyncPlan) -> Result<usize, String> {
        let mut copied = 0;
        for rel_path in &plan.to_sandbox {
            let src = self.project_root.join(rel_path);
            let dst = self.root.join(rel_path);
            if !src.exists() {
                return Err(format!(
                    "Missing source file for sync: {}",
                    rel_path.display()
                ));
            }
            if let Some(parent) = dst.parent()
                && !parent.exists()
            {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::copy(&src, &dst).map_err(|e| e.to_string())?;
            copied += 1;
        }
        Ok(copied)
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
                // SAFETY: We no longer automatically delete files in the real project
                Ok(())
            }
            (false, false) => Err(format!(
                "Promotion failed: File {} does not exist in sandbox or project",
                relative_path.display()
            )),
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

                    // Metadata first: if sizes differ, they are definitely different.
                    if m_sandbox.len() != m_project.len() {
                        true
                    } else {
                        // For identical sizes, use fast xxh3 hashing for 100% accuracy.
                        let h_sandbox = Self::calculate_file_hash(abs_sandbox_path);
                        let h_project = Self::calculate_file_hash(&abs_project_path);
                        h_sandbox != h_project
                    }
                };

                if is_staged {
                    staged_set.insert(rel_path.to_path_buf());
                }
            }
        }

        let mut staged: Vec<PathBuf> = staged_set.into_iter().collect();
        staged.sort();
        staged
    }
}

#[cfg(test)]
mod tests {
    use super::{Sandbox, SyncPlan};
    use std::fs;
    use std::path::PathBuf;

    fn create_temp_dir(label: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        dir.push(format!("polycredo_sandbox_test_{}_{}", label, stamp));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn test_sync_plan_to_sandbox_copies_files() {
        let project_root = create_temp_dir("project");
        let sandbox_root = create_temp_dir("sandbox");

        let src_path = project_root.join("src").join("main.rs");
        if let Some(parent) = src_path.parent() {
            fs::create_dir_all(parent).expect("create project dir");
        }
        fs::write(&src_path, "fn main() {}").expect("write source");

        let mut plan = SyncPlan::default();
        plan.to_sandbox.push(PathBuf::from("src/main.rs"));

        let sandbox = Sandbox::new_with_roots(project_root.clone(), sandbox_root.clone());
        let copied = sandbox.sync_plan_to_sandbox(&plan).expect("sync plan");

        let dst_path = sandbox_root.join("src").join("main.rs");
        assert!(dst_path.exists());
        assert_eq!(copied, 1);
    }
}
