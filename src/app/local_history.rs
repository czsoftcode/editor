use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use xxhash_rust::xxh64::xxh64;

/// Handles versioning of local files to provide a Git-independent safety net
/// specifically tailored for guarding against unwanted AI modifications.
pub struct LocalHistory {
    base_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub timestamp: u64,
    #[allow(dead_code)]
    pub path: PathBuf,
    pub hash: u64,
}

impl LocalHistory {
    pub fn new(project_root: &Path) -> Self {
        let base_dir = project_root.join(".polycredo").join("history");
        if !base_dir.exists() {
            let _ = fs::create_dir_all(&base_dir);
        }
        Self { base_dir }
    }

    /// Generates a safe folder name based on the original file path.
    /// E.g., "src/main.rs" -> "src_main.rs"
    fn encode_path(project_path: &Path) -> String {
        project_path.to_string_lossy().replace(['/', '\\'], "_")
    }

    /// Computes a fast xxHash64 of a string.
    fn compute_hash(content: &str) -> u64 {
        xxh64(content.as_bytes(), 0)
    }

    /// Takes a snapshot of the current content if it differs from the last saved snapshot.
    /// Returns the Path to the snapshot if it was newly created, or None if skipped (unmodified).
    pub fn take_snapshot(&self, relative_file_path: &Path, content: &str) -> Option<PathBuf> {
        let new_hash = Self::compute_hash(content);
        let safe_name = Self::encode_path(relative_file_path);
        let file_history_dir = self.base_dir.join(&safe_name);

        if !file_history_dir.exists() {
            let _ = fs::create_dir_all(&file_history_dir);
        }

        // Find the most recent snapshot to avoid duplicating identical content
        let entries = self.get_history(relative_file_path);
        if let Some(latest) = entries.first()
            && latest.hash == new_hash
        {
            return None; // No changes since last snapshot
        }

        // Save new snapshot
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Format: timestamp_hash.txt
        let snapshot_name = format!("{}_{}.txt", timestamp, new_hash);
        let snapshot_path = file_history_dir.join(snapshot_name);

        if fs::write(&snapshot_path, content).is_ok() {
            Some(snapshot_path)
        } else {
            None
        }
    }

    /// Returns a list of all historical versions for a given file, sorted from newest to oldest.
    pub fn get_history(&self, relative_file_path: &Path) -> Vec<HistoryEntry> {
        let safe_name = Self::encode_path(relative_file_path);
        let file_history_dir = self.base_dir.join(&safe_name);

        let mut entries = Vec::new();

        if let Ok(read_dir) = fs::read_dir(file_history_dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                    // Parse timestamp_hash.txt
                    let parts: Vec<&str> = file_name.trim_end_matches(".txt").split('_').collect();
                    if parts.len() == 2
                        && let (Ok(ts), Ok(hash)) =
                            (parts[0].parse::<u64>(), parts[1].parse::<u64>())
                    {
                        entries.push(HistoryEntry {
                            timestamp: ts,
                            path,
                            hash,
                        });
                    }
                }
            }
        }

        // Sort descending (newest first)
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries
    }

    /// Cleans up old history files to prevent unbounded disk growth.
    /// Keeps only the `max_versions` newest versions for each tracked file.
    #[allow(dead_code)]
    pub fn cleanup(&self, max_versions: usize) {
        if let Ok(read_dir) = fs::read_dir(&self.base_dir) {
            for entry in read_dir.flatten() {
                if entry.path().is_dir() {
                    // This is a directory for a specific file (e.g. "src_main.rs")
                    let mut versions = Vec::new();
                    if let Ok(file_dir) = fs::read_dir(entry.path()) {
                        for f_entry in file_dir.flatten() {
                            if let Ok(meta) = f_entry.metadata()
                                && let Ok(modified) = meta.modified()
                            {
                                versions.push((f_entry.path(), modified));
                            }
                        }
                    }

                    // Sort newest first
                    versions.sort_by(|a, b| b.1.cmp(&a.1));

                    // Remove versions exceeding the limit
                    for (path, _) in versions.into_iter().skip(max_versions) {
                        let _ = fs::remove_file(path);
                    }
                }
            }
        }
    }
}
