use std::path::{Path, PathBuf};

/// Returns the .planning directory path.
pub fn planning_dir(root: &Path) -> PathBuf {
    root.join(".planning")
}

/// Returns the phase directory path: root/.planning/phases/{number}-{slug}
pub fn phase_dir(root: &Path, number: &str, slug: &str) -> PathBuf {
    root.join(".planning")
        .join("phases")
        .join(format!("{}-{}", number, slug))
}

/// Returns the STATE.md path.
pub fn state_path(root: &Path) -> PathBuf {
    root.join(".planning").join("STATE.md")
}

/// Returns the ROADMAP.md path.
pub fn roadmap_path(root: &Path) -> PathBuf {
    root.join(".planning").join("ROADMAP.md")
}

/// Converts a name to a URL-safe slug.
/// Lowercase, replace non-alphanum with '-', collapse multiple '-', trim '-'.
pub fn slugify(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else {
            slug.push('-');
        }
    }
    // Collapse multiple dashes
    let mut result = String::with_capacity(slug.len());
    let mut prev_dash = false;
    for ch in slug.chars() {
        if ch == '-' {
            if !prev_dash {
                result.push('-');
            }
            prev_dash = true;
        } else {
            result.push(ch);
            prev_dash = false;
        }
    }
    // Trim leading/trailing dashes
    result.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_planning_dir() {
        let root = Path::new("/home/user/project");
        assert_eq!(
            planning_dir(root),
            PathBuf::from("/home/user/project/.planning")
        );
    }

    #[test]
    fn test_phase_dir() {
        let root = Path::new("/home/user/project");
        assert_eq!(
            phase_dir(root, "20", "gsd-core"),
            PathBuf::from("/home/user/project/.planning/phases/20-gsd-core")
        );
    }

    #[test]
    fn test_state_path() {
        let root = Path::new("/home/user/project");
        assert_eq!(
            state_path(root),
            PathBuf::from("/home/user/project/.planning/STATE.md")
        );
    }

    #[test]
    fn test_roadmap_path() {
        let root = Path::new("/home/user/project");
        assert_eq!(
            roadmap_path(root),
            PathBuf::from("/home/user/project/.planning/ROADMAP.md")
        );
    }

    #[test]
    fn test_slugify_basic() {
        assert_eq!(slugify("GSD Core + State Engine"), "gsd-core-state-engine");
    }

    #[test]
    fn test_slugify_special_chars() {
        assert_eq!(slugify("Hello World!"), "hello-world");
        assert_eq!(slugify("  spaces  "), "spaces");
        assert_eq!(slugify("a--b--c"), "a-b-c");
    }

    #[test]
    fn test_slugify_numbers() {
        assert_eq!(slugify("Phase 20"), "phase-20");
    }
}
