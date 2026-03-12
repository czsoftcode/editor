#[allow(dead_code)]
#[path = "../src/watcher.rs"]
mod watcher;

use std::path::PathBuf;

use watcher::{FsChange, PROJECT_WATCHER_BATCH_WINDOW_MS, build_project_watcher_batch_for_tests};

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

#[test]
fn phase38_remove_precedence() {
    let batch = build_project_watcher_batch_for_tests(
        vec![
            FsChange::Created(path("/tmp/c.txt")),
            FsChange::Modified(path("/tmp/c.txt")),
            FsChange::Removed(path("/tmp/c.txt")),
            FsChange::Created(path("/tmp/d.txt")),
            FsChange::Removed(path("/tmp/d.txt")),
            FsChange::Modified(path("/tmp/e.txt")),
            FsChange::Removed(path("/tmp/e.txt")),
        ],
        500,
    );

    assert!(
        batch
            .changes
            .iter()
            .all(|c| !matches!(c, FsChange::Created(p) | FsChange::Modified(p) if *p == path("/tmp/c.txt"))),
        "remove musi mit prednost pred create/modify na stejne ceste"
    );
    assert!(
        batch
            .changes
            .iter()
            .any(|c| matches!(c, FsChange::Removed(p) if *p == path("/tmp/c.txt"))),
        "finalni stav kolizni cesty c.txt musi byt Removed"
    );
    assert!(
        batch
            .changes
            .iter()
            .any(|c| matches!(c, FsChange::Removed(p) if *p == path("/tmp/d.txt"))),
        "create nasledovane remove musi skoncit jako Removed"
    );
    assert!(
        batch
            .changes
            .iter()
            .any(|c| matches!(c, FsChange::Removed(p) if *p == path("/tmp/e.txt"))),
        "modify nasledovane remove musi skoncit jako Removed"
    );
}

#[test]
fn phase38_overflow_sets_fallback() {
    let events: Vec<FsChange> = (0..501)
        .map(|i| FsChange::Modified(path(&format!("/tmp/overflow-{i}.txt"))))
        .collect();

    let batch = build_project_watcher_batch_for_tests(events, 500);

    assert!(
        batch.overflowed,
        "burst nad limitem musi nastavit overflow fallback signal"
    );
    assert!(
        batch.changes.is_empty(),
        "overflow path nesmi vracet granularni replay syrovych eventu"
    );
}
