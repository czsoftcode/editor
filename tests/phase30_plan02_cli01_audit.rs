use std::fs;
use std::path::Path;

#[test]
fn cli01_audit_artifact_exists_with_pass_markers() {
    let path = Path::new(".planning/phases/30-cli-namespace-removal-foundation/30-02-AUDIT.md");
    assert!(path.exists(), "missing audit artifact: {}", path.display());

    let content = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    assert!(
        content.contains("CLI-01: PASS"),
        "audit must declare CLI-01 PASS"
    );
    assert!(
        content.contains("cargo check: PASS"),
        "audit must include cargo check PASS"
    );
    assert!(
        content.contains("rg -n \"app::cli|src/app/cli\" src: PASS"),
        "audit must include namespace grep PASS"
    );
}
