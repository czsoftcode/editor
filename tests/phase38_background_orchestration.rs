use std::fs;

#[test]
fn phase38_batch_applies_deduped_changes() {
    let source = fs::read_to_string("src/app/ui/background.rs")
        .unwrap_or_else(|err| panic!("failed to read src/app/ui/background.rs: {err}"));

    assert!(
        source.contains("dedupe_project_watcher_changes"),
        "background orchestrace musi mit dedupe helper pro watcher batch",
    );
    assert!(
        source.contains("let deduped_changes = dedupe_project_watcher_changes"),
        "process_background_events musi aplikovat jen dedupe vystup",
    );
}
