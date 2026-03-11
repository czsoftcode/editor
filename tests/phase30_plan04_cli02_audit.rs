const AUDIT_PATH: &str = ".planning/phases/30-cli-namespace-removal-foundation/30-04-AUDIT.md";

#[test]
fn cli02_ai_terminal_audit_artifact_exists_and_is_passed() {
    let content = std::fs::read_to_string(AUDIT_PATH)
        .unwrap_or_else(|err| panic!("failed to read {AUDIT_PATH}: {err}"));

    assert!(
        content.contains("CLI-02 AI terminal subset: PASS"),
        "audit artifact must contain PASS marker"
    );
}
