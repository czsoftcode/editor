use std::path::PathBuf;
use std::sync::LazyLock;

use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::Regex;

// ---------------------------------------------------------------------------
// PathSandbox
// ---------------------------------------------------------------------------

/// Validates that file paths stay within the project root directory.
/// Blocks path traversal attacks (../, symlinks, absolute paths outside project).
pub struct PathSandbox {
    project_root: PathBuf,
}

impl PathSandbox {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    /// Validates a relative path against the project root.
    /// Returns the canonical absolute path if valid, or an error message.
    pub fn validate_path(&self, relative: &str) -> Result<PathBuf, String> {
        // Reject absolute paths that don't start with project root
        let path = std::path::Path::new(relative);
        if path.is_absolute() {
            let canonical = std::fs::canonicalize(path)
                .map_err(|e| format!("Cannot resolve absolute path '{}': {}", relative, e))?;
            let root_canonical = std::fs::canonicalize(&self.project_root)
                .map_err(|e| format!("Cannot resolve project root: {}", e))?;
            if !canonical.starts_with(&root_canonical) {
                return Err(format!("Path '{}' is outside project root", relative));
            }
            return Ok(canonical);
        }

        // Block obvious traversal attempts early
        if relative.contains("..") {
            // Check if the joined path would escape
            let joined = self.project_root.join(relative);
            // Try to canonicalize (file must exist for this to work)
            match std::fs::canonicalize(&joined) {
                Ok(canonical) => {
                    let root_canonical = std::fs::canonicalize(&self.project_root)
                        .map_err(|e| format!("Cannot resolve project root: {}", e))?;
                    if !canonical.starts_with(&root_canonical) {
                        return Err(format!(
                            "Path traversal blocked: '{}' resolves outside project root",
                            relative
                        ));
                    }
                    Ok(canonical)
                }
                Err(_) => {
                    // File doesn't exist — check parent
                    self.validate_nonexistent_path(relative)
                }
            }
        } else {
            let joined = self.project_root.join(relative);
            match std::fs::canonicalize(&joined) {
                Ok(canonical) => {
                    let root_canonical = std::fs::canonicalize(&self.project_root)
                        .map_err(|e| format!("Cannot resolve project root: {}", e))?;
                    if !canonical.starts_with(&root_canonical) {
                        return Err(format!("Path '{}' resolves outside project root", relative));
                    }
                    Ok(canonical)
                }
                Err(_) => {
                    // File doesn't exist yet (write_file for new file)
                    self.validate_nonexistent_path(relative)
                }
            }
        }
    }

    /// For non-existing files, canonicalize the parent directory and verify it.
    fn validate_nonexistent_path(&self, relative: &str) -> Result<PathBuf, String> {
        let joined = self.project_root.join(relative);
        let parent = joined
            .parent()
            .ok_or_else(|| format!("Cannot determine parent of '{}'", relative))?;
        let filename = joined
            .file_name()
            .ok_or_else(|| format!("Cannot determine filename of '{}'", relative))?;

        let parent_canonical = std::fs::canonicalize(parent)
            .map_err(|_| format!("Parent directory of '{}' does not exist", relative))?;
        let root_canonical = std::fs::canonicalize(&self.project_root)
            .map_err(|e| format!("Cannot resolve project root: {}", e))?;

        if !parent_canonical.starts_with(&root_canonical) {
            return Err(format!(
                "Path traversal blocked: '{}' resolves outside project root",
                relative
            ));
        }

        Ok(parent_canonical.join(filename))
    }
}

// ---------------------------------------------------------------------------
// SecretsFilter
// ---------------------------------------------------------------------------

/// Scrubs secret values from text before sending to AI.
pub struct SecretsFilter;

static SECRETS_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)(API_KEY|SECRET|TOKEN|PASSWORD|PASSWD|DB_PASSWORD|PRIVATE_KEY|ACCESS_KEY|AUTH_TOKEN|CLIENT_SECRET)\s*[=:]\s*['"]?([^\s'"}\]]+)"#
    ).expect("Invalid secrets regex")
});

impl SecretsFilter {
    /// Replaces secret values with [REDACTED] in the given text.
    pub fn scrub(text: &str) -> String {
        SECRETS_REGEX
            .replace_all(text, "$1=[REDACTED]")
            .into_owned()
    }
}

// ---------------------------------------------------------------------------
// CommandBlacklist
// ---------------------------------------------------------------------------

/// Classification of a shell command for security purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandClassification {
    /// Command is dangerous and must be rejected.
    Blocked,
    /// Command involves network access — show extra warning.
    NetworkWarning,
    /// Command needs user approval before execution.
    NeedsApproval,
}

/// Classifies shell commands into security categories.
pub struct CommandBlacklist;

const BLACKLISTED_COMMANDS: &[&str] = &[
    "rm -rf /", "sudo ", "shutdown", "reboot", "mkfs", "dd if=", "format ",
];

const NETWORK_COMMANDS: &[&str] = &["curl", "wget", "nc ", "ssh ", "scp ", "rsync", "telnet"];

impl CommandBlacklist {
    /// Classifies a command string into Blocked, NetworkWarning, or NeedsApproval.
    pub fn classify(cmd: &str) -> CommandClassification {
        let lower = cmd.to_lowercase();

        for pattern in BLACKLISTED_COMMANDS {
            if lower.contains(pattern) {
                return CommandClassification::Blocked;
            }
        }

        for pattern in NETWORK_COMMANDS {
            if lower.contains(pattern) {
                return CommandClassification::NetworkWarning;
            }
        }

        CommandClassification::NeedsApproval
    }
}

// ---------------------------------------------------------------------------
// FileBlacklist
// ---------------------------------------------------------------------------

/// Blocks access to sensitive files based on glob patterns.
pub struct FileBlacklist {
    glob_set: GlobSet,
}

const DEFAULT_BLOCKED_PATTERNS: &[&str] = &[
    ".env*",
    "*.pem",
    "*.key",
    "id_rsa*",
    "credentials.*",
    "secrets.*",
    "*.pfx",
    "*.p12",
];

impl FileBlacklist {
    /// Creates a new FileBlacklist with default patterns, optional user patterns,
    /// and optional .gitignore patterns.
    pub fn new(
        user_patterns: Option<Vec<String>>,
        gitignore_patterns: Option<Vec<String>>,
    ) -> Self {
        let mut builder = GlobSetBuilder::new();

        for pattern in DEFAULT_BLOCKED_PATTERNS {
            if let Ok(glob) = Glob::new(pattern) {
                builder.add(glob);
            }
        }

        if let Some(patterns) = user_patterns {
            for pattern in &patterns {
                if let Ok(glob) = Glob::new(pattern) {
                    builder.add(glob);
                }
            }
        }

        if let Some(patterns) = gitignore_patterns {
            for pattern in &patterns {
                if let Ok(glob) = Glob::new(pattern) {
                    builder.add(glob);
                }
            }
        }

        let glob_set = builder.build().unwrap_or_else(|_| GlobSet::empty());

        Self { glob_set }
    }

    /// Checks if a file path matches any blocked pattern.
    pub fn is_blocked(&self, path: &str) -> bool {
        let p = std::path::Path::new(path);
        // Check against the full path and just the filename
        self.glob_set.is_match(p)
            || p.file_name()
                .map(|f| self.glob_set.is_match(f))
                .unwrap_or(false)
    }
}

// ---------------------------------------------------------------------------
// RateLimiter
// ---------------------------------------------------------------------------

/// Per-conversation rate limiter for AI tool calls.
pub struct RateLimiter {
    write_count: u32,
    exec_count: u32,
    max_writes: u32,
    max_execs: u32,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            write_count: 0,
            exec_count: 0,
            max_writes: 50,
            max_execs: 20,
        }
    }

    /// Checks and increments the write counter. Returns Err if limit exceeded.
    pub fn check_write(&mut self) -> Result<(), String> {
        self.write_count += 1;
        if self.write_count > self.max_writes {
            return Err(format!(
                "Write rate limit exceeded: {}/{}",
                self.write_count, self.max_writes
            ));
        }
        Ok(())
    }

    /// Checks and increments the exec counter. Returns Err if limit exceeded.
    pub fn check_exec(&mut self) -> Result<(), String> {
        self.exec_count += 1;
        if self.exec_count > self.max_execs {
            return Err(format!(
                "Exec rate limit exceeded: {}/{}",
                self.exec_count, self.max_execs
            ));
        }
        Ok(())
    }

    /// Read operations are unlimited — always Ok.
    pub fn check_read(&self) -> Result<(), String> {
        Ok(())
    }

    /// Resets all counters (new conversation).
    pub fn reset(&mut self) {
        self.write_count = 0;
        self.exec_count = 0;
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // --- PathSandbox tests ---

    #[test]
    fn path_sandbox_valid_relative_path() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        fs::write(root.join("main.rs"), "fn main() {}").unwrap();

        let sandbox = PathSandbox::new(root.clone());
        let result = sandbox.validate_path("main.rs");
        assert!(
            result.is_ok(),
            "Valid relative path should succeed: {:?}",
            result
        );
        let canonical = result.unwrap();
        assert!(canonical.ends_with("main.rs"));
    }

    #[test]
    fn path_sandbox_blocks_traversal() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();

        let sandbox = PathSandbox::new(root);
        let result = sandbox.validate_path("../../../etc/passwd");
        assert!(result.is_err(), "Path traversal should be blocked");
    }

    #[test]
    fn path_sandbox_blocks_absolute_outside() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();

        let sandbox = PathSandbox::new(root);
        let result = sandbox.validate_path("/etc/passwd");
        assert!(
            result.is_err(),
            "Absolute path outside project should be blocked"
        );
    }

    #[test]
    fn path_sandbox_allows_new_file_in_project() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        // Parent dir exists, file doesn't
        let sandbox = PathSandbox::new(root);
        let result = sandbox.validate_path("new_file.rs");
        assert!(
            result.is_ok(),
            "New file in project root should be allowed: {:?}",
            result
        );
    }

    // --- SecretsFilter tests ---

    #[test]
    fn secrets_filter_scrubs_api_key() {
        let result = SecretsFilter::scrub("API_KEY=sk-123abc");
        assert_eq!(result, "API_KEY=[REDACTED]");
    }

    #[test]
    fn secrets_filter_scrubs_password_with_quotes() {
        let result = SecretsFilter::scrub("PASSWORD = 'mypass'");
        // The closing quote remains since regex captures up to whitespace/quote boundary
        assert!(
            result.starts_with("PASSWORD=[REDACTED]"),
            "Password should be scrubbed, got: {}",
            result
        );
        assert!(!result.contains("mypass"), "Secret value should not appear");
    }

    #[test]
    fn secrets_filter_leaves_normal_text() {
        let input = "normal code without secrets";
        let result = SecretsFilter::scrub(input);
        assert_eq!(result, input);
    }

    #[test]
    fn secrets_filter_scrubs_multiline() {
        let input = "TOKEN: abc123\nDATA=ok";
        let result = SecretsFilter::scrub(input);
        assert!(result.contains("TOKEN=[REDACTED]"));
        assert!(result.contains("DATA=ok"));
    }

    // --- CommandBlacklist tests ---

    #[test]
    fn command_blacklist_cargo_build() {
        assert_eq!(
            CommandBlacklist::classify("cargo build"),
            CommandClassification::NeedsApproval
        );
    }

    #[test]
    fn command_blacklist_rm_rf_root() {
        assert_eq!(
            CommandBlacklist::classify("rm -rf /"),
            CommandClassification::Blocked
        );
    }

    #[test]
    fn command_blacklist_sudo() {
        assert_eq!(
            CommandBlacklist::classify("sudo apt install"),
            CommandClassification::Blocked
        );
    }

    #[test]
    fn command_blacklist_curl() {
        assert_eq!(
            CommandBlacklist::classify("curl https://example.com"),
            CommandClassification::NetworkWarning
        );
    }

    #[test]
    fn command_blacklist_wget() {
        assert_eq!(
            CommandBlacklist::classify("wget file.zip"),
            CommandClassification::NetworkWarning
        );
    }

    // --- FileBlacklist tests ---

    #[test]
    fn file_blacklist_blocks_env() {
        let bl = FileBlacklist::new(None, None);
        assert!(bl.is_blocked(".env"));
    }

    #[test]
    fn file_blacklist_allows_source() {
        let bl = FileBlacklist::new(None, None);
        assert!(!bl.is_blocked("src/main.rs"));
    }

    #[test]
    fn file_blacklist_blocks_id_rsa() {
        let bl = FileBlacklist::new(None, None);
        assert!(bl.is_blocked("id_rsa"));
    }

    #[test]
    fn file_blacklist_blocks_credentials_json() {
        let bl = FileBlacklist::new(None, None);
        assert!(bl.is_blocked("credentials.json"));
    }

    #[test]
    fn file_blacklist_user_patterns() {
        let bl = FileBlacklist::new(Some(vec!["*.secret".to_string()]), None);
        assert!(bl.is_blocked("config.secret"));
    }

    // --- RateLimiter tests ---

    #[test]
    fn rate_limiter_write_limit() {
        let mut rl = RateLimiter::new();
        for _ in 0..50 {
            assert!(rl.check_write().is_ok());
        }
        assert!(rl.check_write().is_err(), "51st write should fail");
    }

    #[test]
    fn rate_limiter_exec_limit() {
        let mut rl = RateLimiter::new();
        for _ in 0..20 {
            assert!(rl.check_exec().is_ok());
        }
        assert!(rl.check_exec().is_err(), "21st exec should fail");
    }

    #[test]
    fn rate_limiter_read_unlimited() {
        let rl = RateLimiter::new();
        for _ in 0..1000 {
            assert!(rl.check_read().is_ok());
        }
    }

    #[test]
    fn rate_limiter_reset() {
        let mut rl = RateLimiter::new();
        for _ in 0..50 {
            rl.check_write().unwrap();
        }
        rl.reset();
        assert!(
            rl.check_write().is_ok(),
            "After reset, write should succeed"
        );
    }
}
