const ACTIVE_AUDIT_PATH: &str = ".planning/phases/30-cli-namespace-removal-foundation/30-01-AUDIT.md";
const ARCHIVED_AUDIT_PATH: &str =
    ".planning/milestones/v1.3.0-phases/30-cli-namespace-removal-foundation/30-01-AUDIT.md";

fn resolve_audit_path() -> &'static str {
    if std::path::Path::new(ACTIVE_AUDIT_PATH).exists() {
        ACTIVE_AUDIT_PATH
    } else {
        ARCHIVED_AUDIT_PATH
    }
}

#[test]
fn cli02_foundation_audit_artifact_exists_and_is_passed() {
    let audit_path = resolve_audit_path();
    let content = std::fs::read_to_string(audit_path)
        .unwrap_or_else(|err| panic!("failed to read {audit_path}: {err}"));

    assert!(
        content.contains("CLI-02 foundation subset: PASS"),
        "audit artifact must contain PASS marker"
    );
}
