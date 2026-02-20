/// Safe filename or directory name: no path separators,
/// traversal components (`.` / `..`) and null bytes.
pub(crate) fn is_safe_filename(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains('\0')
}

/// Valid project name: only ASCII alphanumerics, underscores,
/// and hyphens are allowed. Name must not be empty or start with a hyphen.
pub(crate) fn is_valid_project_name(name: &str) -> bool {
    if name.is_empty() || name.starts_with('-') {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Safe project name = must satisfy both rules simultaneously.
/// Use everywhere where a name becomes a directory name on disk.
#[allow(dead_code)]
pub(crate) fn is_safe_project_name(name: &str) -> bool {
    is_valid_project_name(name) && is_safe_filename(name)
}
