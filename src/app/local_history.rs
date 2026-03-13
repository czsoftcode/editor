use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use xxhash_rust::xxh3::xxh3_64;

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
        let hash = xxh3_64(project_path.to_string_lossy().as_bytes());
        format!("{:x}", hash)
    }

    /// Computes a fast xxHash64 of a string.
    fn compute_hash(content: &str) -> u64 {
        xxh3_64(content.as_bytes())
    }

    /// Vytvoří snapshot aktuálního obsahu, pokud se liší od posledního uloženého snímku.
    /// Vrátí cestu ke snapshotovému souboru, nebo `Ok(None)` pokud je přeskočen (duplicitní obsah,
    /// `.polycredo` cesta). I/O chyby se propagují nahoru.
    pub fn take_snapshot(
        &mut self,
        relative_file_path: &Path,
        content: &str,
    ) -> Result<Option<PathBuf>, io::Error> {
        // Nikdy nesnapshotovat interní .polycredo adresář (historie atd.)
        if relative_file_path
            .components()
            .any(|c| c.as_os_str() == ".polycredo")
        {
            return Ok(None);
        }

        let new_hash = Self::compute_hash(content);
        let safe_name = Self::encode_path(relative_file_path);
        let file_history_dir = self.base_dir.join(&safe_name);

        // Aktualizovat index pokud je potřeba
        if !self.index.contains_key(&safe_name) {
            self.index.insert(
                safe_name.clone(),
                relative_file_path.to_string_lossy().to_string(),
            );
            self.save_index();
        }

        if !file_history_dir.exists() {
            fs::create_dir_all(&file_history_dir)?;
        }

        // Najít nejnovější snapshot — vyhnout se duplikaci identického obsahu
        let entries = self.get_history(relative_file_path);
        if let Some(latest) = entries.first()
            && latest.hash == new_hash
        {
            return Ok(None); // Žádná změna od posledního snapshotů
        }

        // Uložit nový snapshot
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Formát: timestamp_hash.txt
        let snapshot_name = format!("{}_{}.txt", timestamp, new_hash);
        let snapshot_path = file_history_dir.join(snapshot_name);

        fs::write(&snapshot_path, content)?;
        Ok(Some(snapshot_path))
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

    /// Načte obsah historické verze souboru z disku.
    pub fn get_snapshot_content(
        &self,
        relative_file_path: &Path,
        entry: &HistoryEntry,
    ) -> io::Result<String> {
        let safe_name = Self::encode_path(relative_file_path);
        let snapshot_name = format!("{}_{}.txt", entry.timestamp, entry.hash);
        let snapshot_path = self.base_dir.join(safe_name).join(snapshot_name);
        fs::read_to_string(snapshot_path)
    }

    /// Vyčistí staré verze historie — deleguje na standalone `cleanup_history_dir()`.
    /// `max_versions`: maximální počet verzí na soubor.
    /// `max_age_secs`: pokud Some, smaže verze starší než daný počet sekund.
    pub fn cleanup(&self, max_versions: usize, max_age_secs: Option<u64>) {
        cleanup_history_dir(&self.base_dir, max_versions, max_age_secs);
    }
}

/// Standalone cleanup funkce — `Send`-safe (žádný `&self`).
/// Iteruje adresáře v `base_dir`, pro každý soubor ponechá max `max_versions` nejnovějších verzí
/// a volitelně smaže verze starší než `max_age_secs` sekund od aktuálního času.
pub fn cleanup_history_dir(base_dir: &Path, max_versions: usize, max_age_secs: Option<u64>) {
    let current_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let read_dir = match fs::read_dir(base_dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };

    for entry in read_dir.flatten() {
        if !entry.path().is_dir() {
            continue;
        }

        let mut versions = Vec::new();
        if let Ok(file_dir) = fs::read_dir(entry.path()) {
            for f_entry in file_dir.flatten() {
                let path = f_entry.path();
                if path.is_file()
                    && let Some(file_name) = path.file_name().map(|n| n.to_string_lossy())
                {
                    // Parsovat timestamp z filename (timestamp_hash.txt)
                    if let Some(ts_str) = file_name.split('_').next()
                        && let Ok(ts) = ts_str.parse::<u64>()
                    {
                        versions.push((path, ts));
                    }
                }
            }
        }

        // Seřadit od nejnovějšího (sestupně podle timestamp)
        versions.sort_by(|a, b| b.1.cmp(&a.1));

        // Jednoduchý průchod: ponechat max_versions nejnovějších,
        // ze zbytku smazat vše. Z ponechaných navíc smazat ty, co překročily max_age.
        for (idx, (path, ts)) in versions.into_iter().enumerate() {
            if idx >= max_versions {
                // Přes limit počtu verzí — smazat bezpodmínečně
                let _ = fs::remove_file(path);
            } else if let Some(max_age) = max_age_secs {
                // V limitu verzí, ale kontrola stáří
                if current_ts.saturating_sub(ts) > max_age {
                    let _ = fs::remove_file(path);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    /// Vytvoří LocalHistory v dočasném adresáři.
    fn setup() -> (TempDir, LocalHistory) {
        let tmp = TempDir::new().expect("tmpdir");
        let lh = LocalHistory::new(tmp.path());
        (tmp, lh)
    }

    #[test]
    fn take_snapshot_creates_file_on_fs() {
        let (_tmp, mut lh) = setup();
        let rel = Path::new("src/main.rs");
        let content = "fn main() {}";

        let result = lh.take_snapshot(rel, content);
        assert!(result.is_ok(), "take_snapshot by neměl selhat");
        let path = result.unwrap();
        assert!(path.is_some(), "snapshot by měl být vytvořen");
        let snapshot_path = path.unwrap();
        assert!(snapshot_path.exists(), "soubor by měl existovat na disku");

        let stored = fs::read_to_string(&snapshot_path).unwrap();
        assert_eq!(stored, content);
    }

    #[test]
    fn duplicate_content_is_skipped() {
        let (_tmp, mut lh) = setup();
        let rel = Path::new("src/lib.rs");
        let content = "pub fn hello() {}";

        let first = lh.take_snapshot(rel, content).unwrap();
        assert!(first.is_some(), "první snapshot by měl být vytvořen");

        let second = lh.take_snapshot(rel, content).unwrap();
        assert!(second.is_none(), "duplikovaný obsah by měl být přeskočen");
    }

    #[test]
    fn polycredo_path_is_skipped() {
        let (_tmp, mut lh) = setup();
        let rel = Path::new(".polycredo/history/index.json");
        let content = "{}";

        let result = lh.take_snapshot(rel, content).unwrap();
        assert!(result.is_none(), ".polycredo cesta by měla být přeskočena");
    }

    #[test]
    fn get_snapshot_content_returns_correct_data() {
        let (_tmp, mut lh) = setup();
        let rel = Path::new("test.txt");
        let content = "Obsah testovacího souboru";

        lh.take_snapshot(rel, content).unwrap();

        let entries = lh.get_history(rel);
        assert_eq!(entries.len(), 1);

        let read_back = lh.get_snapshot_content(rel, &entries[0]).unwrap();
        assert_eq!(read_back, content);
    }

    #[test]
    fn get_history_returns_sorted_entries() {
        let (_tmp, mut lh) = setup();
        let rel = Path::new("multi.txt");

        // Vytvořit 3 snapshoty s různým obsahem
        lh.take_snapshot(rel, "verze 1").unwrap();
        // Malá pauza, aby se lišil timestamp (nebo alespoň hash)
        lh.take_snapshot(rel, "verze 2").unwrap();
        lh.take_snapshot(rel, "verze 3").unwrap();

        let entries = lh.get_history(rel);
        assert_eq!(entries.len(), 3, "měly by existovat 3 snapshoty");

        // Záznamy by měly být seřazeny od nejnovějšího
        for window in entries.windows(2) {
            assert!(
                window[0].timestamp >= window[1].timestamp,
                "záznamy by měly být seřazeny sestupně"
            );
        }
    }

    #[test]
    fn error_on_readonly_directory() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = TempDir::new().expect("tmpdir");
        let mut lh = LocalHistory::new(tmp.path());

        // Vytvořit base_dir a pak ji nastavit jako readonly
        fs::create_dir_all(&lh.base_dir).unwrap();
        fs::set_permissions(&lh.base_dir, fs::Permissions::from_mode(0o444)).unwrap();

        let rel = Path::new("readonly_test.txt");
        let result = lh.take_snapshot(rel, "obsah");

        // Vrátit zpět oprávnění pro cleanup
        fs::set_permissions(&lh.base_dir, fs::Permissions::from_mode(0o755)).unwrap();

        assert!(result.is_err(), "zápis do readonly adresáře by měl selhat");
    }

    #[test]
    fn cleanup_removes_old_versions_by_age() {
        let tmp = TempDir::new().expect("tmpdir");
        let base_dir = tmp.path().join(".polycredo").join("history");
        let file_dir = base_dir.join("test_file_hash");
        fs::create_dir_all(&file_dir).unwrap();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let sixty_days_ago = now - 60 * 24 * 3600;
        let ten_days_ago = now - 10 * 24 * 3600;
        let max_age_secs = 30 * 24 * 3600_u64; // 30 dní

        // Starý snapshot (60 dní) — má být smazán
        let old_path = file_dir.join(format!("{}_111.txt", sixty_days_ago));
        fs::write(&old_path, "starý obsah").unwrap();

        // Čerstvý snapshot (10 dní) — má zůstat
        let recent_path = file_dir.join(format!("{}_222.txt", ten_days_ago));
        fs::write(&recent_path, "čerstvý obsah").unwrap();

        // Aktuální snapshot — má zůstat
        let current_path = file_dir.join(format!("{}_333.txt", now));
        fs::write(&current_path, "aktuální obsah").unwrap();

        // Spustit cleanup: max 50 verzí, max_age 30 dní
        cleanup_history_dir(&base_dir, 50, Some(max_age_secs));

        assert!(
            !old_path.exists(),
            "starý snapshot (60 dní) by měl být smazán"
        );
        assert!(
            recent_path.exists(),
            "čerstvý snapshot (10 dní) by měl zůstat"
        );
        assert!(current_path.exists(), "aktuální snapshot by měl zůstat");
    }

    #[test]
    fn cleanup_respects_max_versions_before_age() {
        let tmp = TempDir::new().expect("tmpdir");
        let base_dir = tmp.path().join(".polycredo").join("history");
        let file_dir = base_dir.join("versions_test");
        fs::create_dir_all(&file_dir).unwrap();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Vytvořit 5 čerstvých verzí (všechny v rámci max_age)
        for i in 0..5 {
            let ts = now - i * 60; // každá o minutu starší
            let path = file_dir.join(format!("{}_{}.txt", ts, 100 + i));
            fs::write(&path, format!("verze {}", i)).unwrap();
        }

        // Cleanup s max_versions=3 — nejstarší 2 mají být smazány i když jsou v rámci age
        cleanup_history_dir(&base_dir, 3, Some(30 * 24 * 3600));

        let remaining: Vec<_> = fs::read_dir(&file_dir)
            .unwrap()
            .flatten()
            .filter(|e| e.path().is_file())
            .collect();
        assert_eq!(remaining.len(), 3, "měly by zůstat jen 3 nejnovější verze");
    }
}
