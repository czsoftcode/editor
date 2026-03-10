use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

/// File-based audit logger for AI tool calls.
/// Writes timestamped entries to `.polycredo/ai-audit.log`.
/// Never panics or propagates errors — audit must not break tool execution.
pub struct AuditLogger {
    log_path: PathBuf,
}

impl AuditLogger {
    /// Creates a new AuditLogger. The log file will be at `project_root/.polycredo/ai-audit.log`.
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            log_path: project_root.join(".polycredo").join("ai-audit.log"),
        }
    }

    /// Logs a tool call with the given tool name, status, and details.
    /// Format: `[{ISO 8601 timestamp}] [{tool_name}] [{status}] {details}`
    pub fn log_tool_call(&self, tool_name: &str, status: &str, details: &str) {
        let timestamp = Self::format_timestamp();
        let line = format!("[{}] [{}] [{}] {}\n", timestamp, tool_name, status, details);
        self.append_line(&line);
    }

    /// Logs a security event (path traversal blocks, rate limit hits, etc.).
    pub fn log_security_event(&self, event_type: &str, details: &str) {
        let timestamp = Self::format_timestamp();
        let line = format!("[{}] [SECURITY:{}] {}\n", timestamp, event_type, details);
        self.append_line(&line);
    }

    /// Appends a line to the audit log file. Creates `.polycredo/` if missing.
    /// On error, prints a warning to stderr — never panics.
    fn append_line(&self, line: &str) {
        if let Some(parent) = self.log_path.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!(
                        "[AuditLogger] Failed to create directory {:?}: {}",
                        parent, e
                    );
                    return;
                }
            }
        }

        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            Ok(mut file) => {
                if let Err(e) = file.write_all(line.as_bytes()) {
                    eprintln!(
                        "[AuditLogger] Failed to write to {:?}: {}",
                        self.log_path, e
                    );
                }
            }
            Err(e) => {
                eprintln!("[AuditLogger] Failed to open {:?}: {}", self.log_path, e);
            }
        }
    }

    /// Formats current time as ISO 8601 (UTC). Uses UNIX timestamp calculation
    /// to avoid chrono dependency.
    fn format_timestamp() -> String {
        let duration = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();

        let secs = duration.as_secs();

        // Calculate date/time components from UNIX timestamp
        let days = secs / 86400;
        let time_of_day = secs % 86400;
        let hours = time_of_day / 3600;
        let minutes = (time_of_day % 3600) / 60;
        let seconds = time_of_day % 60;

        // Days since epoch to year/month/day (simplified Gregorian)
        let (year, month, day) = Self::days_to_date(days);

        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            year, month, day, hours, minutes, seconds
        )
    }

    /// Converts days since UNIX epoch to (year, month, day).
    fn days_to_date(mut days: u64) -> (u64, u64, u64) {
        let mut year = 1970u64;

        loop {
            let days_in_year = if Self::is_leap(year) { 366 } else { 365 };
            if days < days_in_year {
                break;
            }
            days -= days_in_year;
            year += 1;
        }

        let months_days: &[u64] = if Self::is_leap(year) {
            &[31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };

        let mut month = 1u64;
        for &md in months_days {
            if days < md {
                break;
            }
            days -= md;
            month += 1;
        }

        (year, month, days + 1)
    }

    fn is_leap(year: u64) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn audit_logger_writes_tool_call() {
        let tmp = tempfile::tempdir().unwrap();
        let logger = AuditLogger::new(tmp.path().to_path_buf());

        logger.log_tool_call("read_project_file", "auto_approved", "path=src/main.rs");

        let log_path = tmp.path().join(".polycredo").join("ai-audit.log");
        assert!(log_path.exists(), "Audit log file should be created");

        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("[read_project_file]"));
        assert!(content.contains("[auto_approved]"));
        assert!(content.contains("path=src/main.rs"));
    }

    #[test]
    fn audit_logger_writes_denied_status() {
        let tmp = tempfile::tempdir().unwrap();
        let logger = AuditLogger::new(tmp.path().to_path_buf());

        logger.log_tool_call("exec", "denied", "command=rm -rf /");

        let content = fs::read_to_string(tmp.path().join(".polycredo/ai-audit.log")).unwrap();
        assert!(content.contains("[denied]"));
        assert!(content.contains("command=rm -rf /"));
    }

    #[test]
    fn audit_logger_creates_polycredo_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let polycredo_dir = tmp.path().join(".polycredo");
        assert!(!polycredo_dir.exists());

        let logger = AuditLogger::new(tmp.path().to_path_buf());
        logger.log_tool_call("test", "ok", "details");

        assert!(polycredo_dir.exists(), ".polycredo dir should be created");
    }

    #[test]
    fn audit_logger_appends_multiple() {
        let tmp = tempfile::tempdir().unwrap();
        let logger = AuditLogger::new(tmp.path().to_path_buf());

        logger.log_tool_call("tool1", "ok", "first");
        logger.log_tool_call("tool2", "ok", "second");

        let content = fs::read_to_string(tmp.path().join(".polycredo/ai-audit.log")).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2, "Should have 2 log lines");
    }

    #[test]
    fn audit_logger_security_event() {
        let tmp = tempfile::tempdir().unwrap();
        let logger = AuditLogger::new(tmp.path().to_path_buf());

        logger.log_security_event("PATH_TRAVERSAL", "blocked ../../../etc/passwd");

        let content = fs::read_to_string(tmp.path().join(".polycredo/ai-audit.log")).unwrap();
        assert!(content.contains("[SECURITY:PATH_TRAVERSAL]"));
    }

    #[test]
    fn audit_logger_timestamp_format() {
        let tmp = tempfile::tempdir().unwrap();
        let logger = AuditLogger::new(tmp.path().to_path_buf());

        logger.log_tool_call("test", "ok", "check timestamp");

        let content = fs::read_to_string(tmp.path().join(".polycredo/ai-audit.log")).unwrap();
        // Should match ISO 8601 pattern: [YYYY-MM-DDThh:mm:ssZ]
        assert!(
            regex::Regex::new(r"\[\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z\]")
                .unwrap()
                .is_match(&content),
            "Timestamp should be ISO 8601 format, got: {}",
            content
        );
    }
}
