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
