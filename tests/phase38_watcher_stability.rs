#[path = "../src/watcher.rs"]
mod watcher;

use std::path::PathBuf;

use watcher::{
    FsChange, PROJECT_WATCHER_BATCH_WINDOW_MS, build_project_watcher_batch_for_tests,
};

fn path(s: &str) -> PathBuf {
    PathBuf::from(s)
}

#[test]
fn phase38_dedupe_path_kind() {
    let batch = build_project_watcher_batch_for_tests(
        vec![
            FsChange::Modified(path("/tmp/a.txt")),
            FsChange::Modified(path("/tmp/a.txt")),
            FsChange::Created(path("/tmp/a.txt")),
            FsChange::Created(path("/tmp/a.txt")),
            FsChange::Modified(path("/tmp/b.txt")),
        ],
        500,
    );

    assert!(
        !batch.overflowed,
        "batch nesmi byt overflow pri malem poctu eventu"
    );
    assert_eq!(
        batch.changes.len(),
        2,
        "dedupe/merge ma vratit jen finalni stavy pro dve cesty"
    );
    assert!(
        batch
            .changes
            .iter()
            .any(|c| matches!(c, FsChange::Created(p) if *p == path("/tmp/a.txt"))),
        "kolize modify+create na stejne ceste ma zanechat deterministicky Created"
    );
    assert!(
        batch
            .changes
            .iter()
            .any(|c| matches!(c, FsChange::Modified(p) if *p == path("/tmp/b.txt"))),
        "nezavisla cesta musi zustat v batchi"
    );
}

#[test]
fn phase38_batch_window_locked() {
    assert!(
        (100..=150).contains(&PROJECT_WATCHER_BATCH_WINDOW_MS),
        "batch window musi zustat v intervalu 100-150 ms"
    );
    assert_eq!(
        PROJECT_WATCHER_BATCH_WINDOW_MS, 120,
        "batch window musi byt fixne zamceny na 120 ms"
    );
}
