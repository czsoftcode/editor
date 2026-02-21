use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use xxhash_rust::xxh64::xxh64;

type HistoryIndex = HashMap<String, String>;

/// Handles versioning of local files to provide a Git-independent safety net.
pub struct LocalHistory {
    base_dir: PathBuf,
    index_path: PathBuf,
    index: HistoryIndex,
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub timestamp: u64,
    pub hash: u64,
}

impl LocalHistory {
    pub fn new(project_root: &Path) -> Self {
        let base_dir = project_root.join(".polycredo").join("history");
        if !base_dir.exists() {
            let _ = fs::create_dir_all(&base_dir);
        }
        let index_path = base_dir.join("index.json");
        let index = if index_path.exists() {
            fs::read_to_string(&index_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            HashMap::new()
        };

        Self {
            base_dir,
            index_path,
            index,
        }
    }

    /// Saves the current in-memory index to `index.json`.
    fn save_index(&self) {
        if let Ok(s) = serde_json::to_string_pretty(&self.index) {
            let _ = fs::write(&self.index_path, s);
        }
    }

    /// Generates a safe folder name based on the hash of the original file path.
    fn encode_path(project_path: &Path) -> String {
        let hash = xxh64(project_path.to_string_lossy().as_bytes(), 0);
        format!("{:x}", hash)
    }

    /// Computes a fast xxHash64 of a string.
    fn compute_hash(content: &str) -> u64 {
        xxh64(content.as_bytes(), 0)
    }

    /// Takes a snapshot of the current content if it differs from the last saved snapshot.
    /// Returns the Path to the snapshot if it was newly created, or None if skipped (unmodified).
    pub fn take_snapshot(&mut self, relative_file_path: &Path, content: &str) -> Option<PathBuf> {
        // Never take snapshots of the internal .polycredo directory (history, sandbox, etc.)
        if relative_file_path
            .components()
            .any(|c| c.as_os_str() == ".polycredo")
        {
            return None;
        }

        let new_hash = Self::compute_hash(content);
        let safe_name = Self::encode_path(relative_file_path);
        let file_history_dir = self.base_dir.join(&safe_name);

        // Update index if needed
        if !self.index.contains_key(&safe_name) {
            self.index.insert(
                safe_name.clone(),
                relative_file_path.to_string_lossy().to_string(),
            );
            self.save_index();
        }

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
    pub fn cleanup(&self, max_versions: usize) {
        if let Ok(read_dir) = fs::read_dir(&self.base_dir) {
            for entry in read_dir.flatten() {
                if entry.path().is_dir() {
                    // This is a directory for a specific file (e.g. "src_main.rs")
                    let mut versions = Vec::new();
                    if let Ok(file_dir) = fs::read_dir(entry.path()) {
                        for f_entry in file_dir.flatten() {
                            let path = f_entry.path();
                            if path.is_file()
                                && let Some(file_name) =
                                    path.file_name().map(|n| n.to_string_lossy())
                            {
                                // Parse timestamp from filename (timestamp_hash.txt)
                                if let Some(ts_str) = file_name.split('_').next()
                                    && let Ok(ts) = ts_str.parse::<u64>()
                                {
                                    versions.push((path, ts));
                                }
                            }
                        }
                    }

                    // Sort newest first (by timestamp)
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
