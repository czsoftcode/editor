const AUDIT_PATH: &str =
    ".planning/phases/30-cli-namespace-removal-foundation/30-01-AUDIT.md";

#[test]
fn cli02_foundation_audit_artifact_exists_and_is_passed() {
    let content = std::fs::read_to_string(AUDIT_PATH)
        .unwrap_or_else(|err| panic!("failed to read {AUDIT_PATH}: {err}"));

    assert!(
        content.contains("CLI-02 foundation subset: PASS"),
        "audit artifact must contain PASS marker"
    );
}
