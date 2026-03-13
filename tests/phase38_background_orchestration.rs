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

#[test]
fn phase38_overflow_triggers_single_reload() {
    let source = fs::read_to_string("src/app/ui/background.rs")
        .unwrap_or_else(|err| panic!("failed to read src/app/ui/background.rs: {err}"));

    assert!(
        source.contains("trigger_project_watcher_reload_once"),
        "overflow fallback musi jit pres single-reload guard helper",
    );
    assert!(
        source.contains("if fs_batch.overflowed"),
        "background orchestrace musi mit explicitni overflow branch",
    );
}

#[test]
fn phase38_watcher_disconnect_handled_once() {
    let source = fs::read_to_string("src/app/ui/background.rs")
        .unwrap_or_else(|err| panic!("failed to read src/app/ui/background.rs: {err}"));

    assert!(
        source.contains("if fs_batch.disconnected"),
        "background orchestrace musi mit explicitni disconnect branch",
    );
    assert!(
        source.contains("project_watcher_disconnect_reported"),
        "disconnect branch musi drzet one-shot guard proti opakovanemu toast loopu",
    );
    assert!(
        source.contains("project_watcher_active = false"),
        "po disconnectu se musi ukoncit dalsi polling odpojeneho watcheru",
    );
}
