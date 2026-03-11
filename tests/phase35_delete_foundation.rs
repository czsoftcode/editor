use std::fs;

#[test]
fn phase35_delete_foundation_is_fail_closed() {
    let dialogs = fs::read_to_string("src/app/ui/file_tree/dialogs.rs")
        .expect("failed to read src/app/ui/file_tree/dialogs.rs");
    assert!(
        dialogs.contains("DeleteJobResult::Error(err.to_string())"),
        "delete flow must propagate foundation errors"
    );

    let file_tree = fs::read_to_string("src/app/ui/file_tree/mod.rs")
        .expect("failed to read src/app/ui/file_tree/mod.rs");
    assert!(
        file_tree.contains("self.pending_error = Some(err);"),
        "file tree must surface async delete errors to toast pipeline"
    );
}

#[test]
fn phase35_delete_foundation_scope_guard_has_no_restore_foundation_symbols() {
    let plan = fs::read_to_string(".planning/phases/35-trash-foundation-async-safety/35-03-PLAN.md")
        .expect("failed to read 35-03 plan");
    assert!(
        !plan.contains("prepare_restore_foundation")
            && !plan.contains("phase35_restore_foundation")
            && !plan.contains("restore foundation"),
        "phase 35-03 must remain delete-only without restore-foundation symbols"
    );
}
