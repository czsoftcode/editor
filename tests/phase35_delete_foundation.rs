use std::fs;

#[test]
fn phase35_delete_foundation_is_fail_closed() {
    let dialogs = fs::read_to_string("src/app/ui/file_tree/dialogs.rs")
        .expect("failed to read src/app/ui/file_tree/dialogs.rs");
    assert!(
        dialogs.contains("DeleteJobResult::Error(format!(\"trash move failed: {err}\"))"),
        "delete flow must propagate foundation errors"
    );

    let file_tree = fs::read_to_string("src/app/ui/file_tree/mod.rs")
        .expect("failed to read src/app/ui/file_tree/mod.rs");
    assert!(
        file_tree.contains("self.pending_error = Some(err);"),
        "file tree must surface async delete errors to toast pipeline"
    );

    let trash = fs::read_to_string("src/app/trash.rs").expect("failed to read src/app/trash.rs");
    assert!(
        trash.contains("std::fs::rename(&source_abs, &destination_abs)")
            && trash.contains("puvodni polozka zustava beze zmeny"),
        "trash move failure must be fail-closed and keep original item in place"
    );
    assert!(
        !trash.contains("std::fs::remove_file")
            && !trash.contains("std::fs::remove_dir_all"),
        "trash foundation must not fall back to hard delete"
    );
}

#[test]
fn phase35_delete_foundation_scope_guard_has_no_restore_foundation_symbols() {
    let plan = fs::read_to_string(".planning/phases/35-trash-foundation-async-safety/35-03-PLAN.md")
        .expect("failed to read 35-03 plan");
    let forbidden_prepare = format!("prepare_restore_{}", "foundation");
    let forbidden_phase = format!("phase35_restore_{}", "foundation");
    let forbidden_phrase = format!("{} {}", "restore", "foundation");
    assert!(
        !plan.contains(&forbidden_prepare)
            && !plan.contains(&forbidden_phase)
            && !plan.contains(&forbidden_phrase),
        "phase 35-03 must remain delete-only without restore-foundation symbols"
    );
}
