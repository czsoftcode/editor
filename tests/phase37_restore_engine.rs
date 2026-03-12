use std::fs;
use std::path::{Path, PathBuf};

use tempfile::tempdir;

mod app {
    pub mod project_config {
        use std::path::{Path, PathBuf};

        pub fn project_trash_dir(project_root: &Path) -> PathBuf {
            project_root.join(".polycredo").join("trash")
        }
    }
}

#[path = "../src/app/trash.rs"]
mod trash;

use trash::{TrashEntryKind, TrashEntryMeta};

fn metadata_path_for(entry_path: &Path) -> PathBuf {
    let file_name = entry_path
        .file_name()
        .expect("entry must have filename")
        .to_string_lossy();
    entry_path.with_file_name(format!("{file_name}.meta.json"))
}

fn write_metadata(entry_path: &Path, deleted_at: u128, original: &str) {
    let meta = TrashEntryMeta {
        trash_name: entry_path
            .file_name()
            .expect("entry must have filename")
            .to_string_lossy()
            .to_string(),
        original_relative_path: PathBuf::from(original),
        deleted_at,
        entry_kind: TrashEntryKind::File,
    };
    let raw = serde_json::to_string_pretty(&meta).expect("metadata json");
    fs::write(metadata_path_for(entry_path), raw).expect("write metadata");
}

#[test]
fn phase37_list_trash_entries() {
    let temp = tempdir().expect("temp dir");
    let project_root = temp.path();
    let trash_root = app::project_config::project_trash_dir(project_root);
    fs::create_dir_all(&trash_root).expect("create trash root");

    let older = trash_root.join("older.txt.trash-100");
    let newer = trash_root.join("newer.txt.trash-200");
    fs::write(&older, "old").expect("write older entry");
    fs::write(&newer, "new").expect("write newer entry");
    write_metadata(&older, 100, "src/older.txt");
    write_metadata(&newer, 200, "src/newer.txt");

    let entries = trash::list_trash_entries(project_root).expect("list entries");
    assert_eq!(entries.len(), 2, "must return two preview entries");
    assert_eq!(entries[0].name, "newer.txt.trash-200");
    assert_eq!(entries[1].name, "older.txt.trash-100");
    assert_eq!(
        entries[0].original_relative_path.as_deref(),
        Some(Path::new("src/newer.txt"))
    );
}

#[test]
fn phase37_list_invalid_metadata() {
    let temp = tempdir().expect("temp dir");
    let project_root = temp.path();
    let trash_root = app::project_config::project_trash_dir(project_root);
    fs::create_dir_all(&trash_root).expect("create trash root");

    let broken = trash_root.join("broken.txt.trash-300");
    fs::write(&broken, "bad").expect("write entry");
    fs::write(metadata_path_for(&broken), "{invalid-json").expect("write broken metadata");

    let entries = trash::list_trash_entries(project_root).expect("list entries");
    assert_eq!(entries.len(), 1, "broken entry must still be visible");
    assert_eq!(entries[0].name, "broken.txt.trash-300");
    assert!(
        matches!(
            entries[0].metadata_status,
            trash::TrashMetadataStatus::Invalid
        ),
        "invalid metadata must be explicit state for UI"
    );
}
