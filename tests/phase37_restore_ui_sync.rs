use std::fs;

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

#[test]
fn phase37_restore_conflict_as_copy() {
    let temp = tempdir().expect("temp dir");
    let project_root = temp.path();
    let source = project_root.join("docs").join("guide.txt");
    fs::create_dir_all(source.parent().expect("source parent")).expect("create source parent");
    fs::write(&source, "trash copy").expect("write source");

    let moved = trash::move_path_to_trash(project_root, &source).expect("move to trash");
    fs::write(&source, "existing target").expect("create conflicting target");

    let restored = trash::restore_from_trash_with_policy(
        project_root,
        &moved.moved_to,
        trash::RestoreConflictPolicy::RestoreAsCopy,
    )
    .expect("restore as copy must succeed");

    assert!(
        source.exists(),
        "conflict policy must never overwrite existing original target"
    );
    assert_eq!(
        fs::read_to_string(&source).expect("read original"),
        "existing target"
    );
    assert!(
        restored.restored_to.exists(),
        "restored copy target must exist"
    );
    assert_ne!(
        restored.restored_to, source,
        "copy restore target must differ from occupied original path"
    );
}
