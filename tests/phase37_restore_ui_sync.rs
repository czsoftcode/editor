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

#[test]
fn phase37_restore_triggers_reload_highlight() {
    let file_tree_mod =
        fs::read_to_string("src/app/ui/file_tree/mod.rs").expect("read file tree module");
    let panels = fs::read_to_string("src/app/ui/panels.rs").expect("read panels module");

    assert!(
        file_tree_mod.contains("self.request_reload_and_expand(&path);"),
        "restore result must trigger file-tree reload + expand path"
    );
    assert!(
        file_tree_mod.contains("self.refresh_trash_preview();"),
        "restore success must refresh preview list after handoff"
    );
    assert!(
        panels.contains("if let Some(restored) = result.restored"),
        "panel orchestration must process pending restored handoff from file tree"
    );
}

#[test]
fn phase37_restore_no_auto_open_tab() {
    let panels = fs::read_to_string("src/app/ui/panels.rs").expect("read panels module");
    let tabs = fs::read_to_string("src/app/ui/editor/tabs.rs").expect("read tabs module");

    assert!(
        panels.contains("ws.editor.sync_tabs_for_restored_path(&restored);"),
        "restore handoff must sync existing tabs only"
    );
    assert!(
        !panels.contains("open_file_in_ws(ws, restored"),
        "restore completion must not auto-open a newly restored file"
    );
    assert!(
        tabs.contains("pub fn sync_tabs_for_restored_path(&mut self, restored_path: &PathBuf)"),
        "editor tabs module must expose explicit restore tab sync helper"
    );
}

#[test]
fn phase37_preview_restore_roundtrip() {
    let file_tree_mod =
        fs::read_to_string("src/app/ui/file_tree/mod.rs").expect("read file tree module");
    let panels = fs::read_to_string("src/app/ui/panels.rs").expect("read panels module");

    assert!(
        file_tree_mod.contains("self.pending_restored = Some(path.clone());")
            && file_tree_mod
                .contains("\"restore selhal: restore worker disconnected; zkuste akci znovu\""),
        "restore worker must map success and disconnect into pending handoff/error pipeline"
    );
    assert!(
        panels.contains("if let Some(err) = ws.file_tree.take_error()")
            && panels.contains("ws.toasts.push(Toast::error(err));"),
        "pending restore error must surface as toast in panel pipeline"
    );
}
