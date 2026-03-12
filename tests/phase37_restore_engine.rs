use std::fs;
use std::path::{Path, PathBuf};

use tempfile::tempdir;

mod app {
    pub mod project_config {
        use std::path::{Path, PathBuf};

        pub fn project_trash_dir(project_root: &Path) -> PathBuf {
            project_root.join(".polycredo").join("trash")
        }

        pub fn trash_meta_path(entry_path: &Path) -> PathBuf {
            let file_name = entry_path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "unknown".to_string());
            entry_path.with_file_name(format!("{file_name}.meta.json"))
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

#[test]
fn phase37_restore_to_original_path() {
    let temp = tempdir().expect("temp dir");
    let project_root = temp.path();
    let source = project_root.join("docs").join("guide.txt");
    fs::create_dir_all(source.parent().expect("source parent")).expect("create source parent");
    fs::write(&source, "restore me").expect("write source file");

    let moved = trash::move_path_to_trash(project_root, &source).expect("move to trash");
    assert!(!source.exists(), "source must be moved into trash");
    assert!(moved.moved_to.exists(), "trash copy must exist before restore");

    let restored = trash::restore_from_trash(project_root, &moved.moved_to).expect("restore entry");
    assert_eq!(restored.restored_from, moved.moved_to);
    assert_eq!(restored.restored_to, source);
    assert_eq!(restored.original_target, source);
    assert!(restored.restored_to.exists(), "restored target must exist");
    assert!(!restored.restored_from.exists(), "trash entry must be removed after restore");
    assert!(
        !metadata_path_for(&restored.restored_from).exists(),
        "metadata sidecar must be removed after successful restore"
    );
}

#[test]
fn phase37_restore_creates_parent_dirs() {
    let temp = tempdir().expect("temp dir");
    let project_root = temp.path();
    let source = project_root.join("deep").join("nested").join("restore.txt");
    fs::create_dir_all(source.parent().expect("source parent")).expect("create source parent");
    fs::write(&source, "restore parent").expect("write source file");

    let moved = trash::move_path_to_trash(project_root, &source).expect("move to trash");
    fs::remove_dir_all(project_root.join("deep")).expect("remove source parent tree");
    assert!(
        !source.parent().expect("source parent").exists(),
        "parent tree must be missing before restore"
    );

    let restored = trash::restore_from_trash(project_root, &moved.moved_to).expect("restore entry");
    assert!(
        restored.restored_to.parent().expect("restored parent").exists(),
        "restore must recreate missing parent directories"
    );
    assert!(restored.restored_to.exists(), "restored file must exist");
}

#[test]
fn phase37_restore_fail_closed() {
    let temp = tempdir().expect("temp dir");
    let project_root = temp.path();

    let source_invalid_meta = project_root.join("invalid").join("meta.txt");
    fs::create_dir_all(source_invalid_meta.parent().expect("source parent"))
        .expect("create source parent");
    fs::write(&source_invalid_meta, "invalid").expect("write source");
    let moved_invalid_meta =
        trash::move_path_to_trash(project_root, &source_invalid_meta).expect("move to trash");
    fs::write(metadata_path_for(&moved_invalid_meta.moved_to), "{broken-json")
        .expect("break metadata json");

    let invalid_meta_err = trash::restore_from_trash(project_root, &moved_invalid_meta.moved_to)
        .expect_err("invalid metadata restore must fail");
    assert!(
        invalid_meta_err.to_string().starts_with("restore selhal:"),
        "restore errors must use consistent `restore selhal:` prefix for toast mapping"
    );
    assert!(
        moved_invalid_meta.moved_to.exists(),
        "invalid metadata failure must keep source data in trash"
    );
    assert!(
        !source_invalid_meta.exists(),
        "invalid metadata failure must not materialize original target"
    );

    let source_missing = project_root.join("missing").join("source.txt");
    fs::create_dir_all(source_missing.parent().expect("source parent")).expect("create parent");
    fs::write(&source_missing, "missing").expect("write source");
    let moved_missing = trash::move_path_to_trash(project_root, &source_missing).expect("move");
    fs::remove_file(&moved_missing.moved_to).expect("remove trash source");

    let missing_source_err = trash::restore_from_trash(project_root, &moved_missing.moved_to)
        .expect_err("missing source restore must fail");
    assert!(
        missing_source_err.to_string().starts_with("restore selhal:"),
        "missing source error must stay in restore error contract"
    );

    let source_io = project_root.join("io").join("source.txt");
    fs::create_dir_all(source_io.parent().expect("source parent")).expect("create io parent");
    fs::write(&source_io, "io").expect("write io source");
    let moved_io = trash::move_path_to_trash(project_root, &source_io).expect("move io");
    write_metadata(&moved_io.moved_to, 999, "blocked/target.txt");
    fs::write(project_root.join("blocked"), "not-a-dir").expect("create blocking file");

    let io_err = trash::restore_from_trash(project_root, &moved_io.moved_to)
        .expect_err("parent creation failure must fail");
    assert!(
        io_err.to_string().starts_with("restore selhal:"),
        "I/O restore failure must stay in restore error contract"
    );
    assert!(
        moved_io.moved_to.exists(),
        "I/O restore failure must keep source data in trash"
    );
    assert!(
        !project_root.join("blocked").join("target.txt").exists(),
        "I/O failure must not create destination file"
    );
}
