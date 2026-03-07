// GSD State & Progress commands — /gsd state, /gsd progress
// Reads and updates .planning/STATE.md via frontmatter parser.

use std::path::Path;

use super::frontmatter::{FmDocument, FmValue};
use super::paths;
use super::super::slash::SlashResult;
use crate::app::ui::workspace::state::WorkspaceState;

/// Format a Unicode progress bar: [████████░░] 80%
pub fn format_progress_bar(percent: u32, width: usize) -> String {
    let clamped = percent.min(100);
    let filled = (width as u32 * clamped / 100) as usize;
    let empty = width - filled;
    let bar: String = "\u{2588}".repeat(filled) + &"\u{2591}".repeat(empty);
    format!("[{}] {}%", bar, clamped)
}

/// Extract an i64 from FmValue (Integer or Float).
fn fm_i64(val: Option<&FmValue>) -> i64 {
    match val {
        Some(FmValue::Integer(n)) => *n,
        Some(FmValue::Float(f)) => *f as i64,
        Some(FmValue::String(s)) => s.parse::<i64>().unwrap_or(0),
        _ => 0,
    }
}

/// Extract a string from FmValue.
fn fm_str<'a>(val: Option<&'a FmValue>) -> &'a str {
    match val {
        Some(FmValue::String(s)) => s.as_str(),
        _ => "",
    }
}

/// Format the full state display from a parsed STATE.md document.
pub fn format_state_display(doc: &FmDocument) -> String {
    let mut out = String::from("## GSD State\n\n");

    // Milestone
    let milestone = fm_str(doc.get("milestone"));
    if !milestone.is_empty() {
        let milestone_name = fm_str(doc.get("milestone_name"));
        if !milestone_name.is_empty() {
            out.push_str(&format!("**Milestone:** {} ({})\n", milestone, milestone_name));
        } else {
            out.push_str(&format!("**Milestone:** {}\n", milestone));
        }
    }

    // Phase
    let total_phases = fm_i64(doc.get("progress.total_phases"));
    let completed_phases = fm_i64(doc.get("progress.completed_phases"));
    if total_phases > 0 {
        out.push_str(&format!("**Phase:** {}/{}\n", completed_phases, total_phases));
    }

    // Status
    let status = fm_str(doc.get("status"));
    if !status.is_empty() {
        out.push_str(&format!("**Status:** {}\n", status));
    }

    // Last activity
    let last_activity = fm_str(doc.get("last_activity"));
    if !last_activity.is_empty() {
        out.push_str(&format!("**Last activity:** {}\n", last_activity));
    }

    // Progress section
    out.push_str("\n### Progress\n\n");
    let percent = fm_i64(doc.get("progress.percent")) as u32;
    out.push_str(&format_progress_bar(percent, 10));
    out.push('\n');

    let total_plans = fm_i64(doc.get("progress.total_plans"));
    let done_plans = fm_i64(doc.get("progress.completed_plans"));
    out.push_str(&format!(
        "Plans: {}/{} | Phases: {}/{}\n",
        done_plans, total_plans, completed_phases, total_phases
    ));

    // Extract body sections: Velocity (from "## Performance Metrics" or "### Velocity")
    let velocity_section = extract_body_section(&doc.body, "Velocity");
    if !velocity_section.is_empty() {
        out.push_str("\n### Velocity\n\n");
        out.push_str(&velocity_section);
    }

    // Blockers
    let blockers_section = extract_body_section(&doc.body, "Blockers/Concerns");
    if !blockers_section.is_empty() {
        out.push_str("\n### Blockers\n\n");
        out.push_str(&blockers_section);
    }

    out
}

/// Extract content of a ### section from the body.
/// Returns lines between `### {heading}` and the next heading (## or ###) or EOF.
fn extract_body_section(body: &str, heading: &str) -> String {
    let target = format!("### {}", heading);
    let mut in_section = false;
    let mut lines = Vec::new();

    for line in body.lines() {
        if in_section {
            // Stop at next heading (## or ###)
            if (line.starts_with("### ") || line.starts_with("## ")) && !line.starts_with(&target)
            {
                break;
            }
            lines.push(line);
        } else if line.trim().starts_with(&target) {
            in_section = true;
        }
    }

    // Trim leading/trailing empty lines
    while lines.first().map_or(false, |l| l.trim().is_empty()) {
        lines.remove(0);
    }
    while lines.last().map_or(false, |l| l.trim().is_empty()) {
        lines.pop();
    }

    if lines.is_empty() {
        String::new()
    } else {
        let mut result = lines.join("\n");
        result.push('\n');
        result
    }
}

/// /gsd state [args] — show state or update/patch
pub fn cmd_state(ws: &mut WorkspaceState, args: &str) -> SlashResult {
    let root = &ws.root_path;
    let state_file = paths::state_path(root);

    if args.is_empty() {
        // Read and display state
        match std::fs::read_to_string(&state_file) {
            Ok(content) => {
                let doc = FmDocument::parse(&content);
                SlashResult::Immediate(format_state_display(&doc))
            }
            Err(_) => SlashResult::Immediate(
                "No STATE.md found. Create `.planning/STATE.md` to track project state."
                    .to_string(),
            ),
        }
    } else if args.starts_with("update ") {
        handle_state_update(root, &args["update ".len()..])
    } else if args.starts_with("patch ") {
        handle_state_patch(root, &args["patch ".len()..])
    } else {
        SlashResult::Immediate(format!(
            "Unknown state subcommand: `{}`. Use `/gsd state`, `/gsd state update <field> <value>`, or `/gsd state patch <key=val> ...`.",
            args
        ))
    }
}

/// /gsd progress — compact progress bar + counts
pub fn cmd_progress(ws: &mut WorkspaceState) -> SlashResult {
    let state_file = paths::state_path(&ws.root_path);

    match std::fs::read_to_string(&state_file) {
        Ok(content) => {
            let doc = FmDocument::parse(&content);
            let percent = fm_i64(doc.get("progress.percent")) as u32;
            let total_plans = fm_i64(doc.get("progress.total_plans"));
            let done_plans = fm_i64(doc.get("progress.completed_plans"));
            let total_phases = fm_i64(doc.get("progress.total_phases"));
            let done_phases = fm_i64(doc.get("progress.completed_phases"));

            let output = format!(
                "## Progress\n\n{}\nPlans: {}/{} | Phases: {}/{}",
                format_progress_bar(percent, 10),
                done_plans,
                total_plans,
                done_phases,
                total_phases
            );
            SlashResult::Immediate(output)
        }
        Err(_) => SlashResult::Immediate(
            "No STATE.md found. Create `.planning/STATE.md` to track project state.".to_string(),
        ),
    }
}

/// Parse a value string to FmValue: try bool, then i64, then f64, then string.
fn parse_value_string(s: &str) -> FmValue {
    match s {
        "true" => FmValue::Boolean(true),
        "false" => FmValue::Boolean(false),
        _ => {
            if let Ok(n) = s.parse::<i64>() {
                FmValue::Integer(n)
            } else if let Ok(f) = s.parse::<f64>() {
                FmValue::Float(f)
            } else {
                FmValue::String(s.to_string())
            }
        }
    }
}

/// /gsd state update <field> <value>
fn handle_state_update(root: &Path, args: &str) -> SlashResult {
    let parts: Vec<&str> = args.splitn(2, char::is_whitespace).collect();
    if parts.len() < 2 {
        return SlashResult::Immediate(
            "Usage: `/gsd state update <field> <value>`\nExample: `/gsd state update status executing`"
                .to_string(),
        );
    }
    let field = parts[0].trim();
    let value_str = parts[1].trim();

    let state_file = paths::state_path(root);
    let content = match std::fs::read_to_string(&state_file) {
        Ok(c) => c,
        Err(_) => {
            return SlashResult::Immediate(
                "No STATE.md found. Cannot update.".to_string(),
            );
        }
    };

    let mut doc = FmDocument::parse(&content);
    let value = parse_value_string(value_str);
    doc.set(field, value);

    // Also update last_updated timestamp
    let now = chrono_iso_now();
    doc.set("last_updated", FmValue::String(now));

    let serialized = doc.to_string_content();
    if let Err(e) = std::fs::write(&state_file, &serialized) {
        return SlashResult::Immediate(format!("Failed to write STATE.md: {}", e));
    }

    SlashResult::Immediate(format!("Updated `{}` to `{}`", field, value_str))
}

/// /gsd state patch key=val key2=val2 ...
fn handle_state_patch(root: &Path, args: &str) -> SlashResult {
    let pairs: Vec<&str> = args.split_whitespace().collect();
    if pairs.is_empty() {
        return SlashResult::Immediate(
            "Usage: `/gsd state patch <key=value> [key2=value2] ...`".to_string(),
        );
    }

    let state_file = paths::state_path(root);
    let content = match std::fs::read_to_string(&state_file) {
        Ok(c) => c,
        Err(_) => {
            return SlashResult::Immediate(
                "No STATE.md found. Cannot patch.".to_string(),
            );
        }
    };

    let mut doc = FmDocument::parse(&content);
    let mut updated_fields = Vec::new();

    for pair in &pairs {
        if let Some(eq_pos) = pair.find('=') {
            let key = &pair[..eq_pos];
            let val_str = &pair[eq_pos + 1..];
            let value = parse_value_string(val_str);
            doc.set(key, value);
            updated_fields.push(format!("`{}` = `{}`", key, val_str));
        } else {
            return SlashResult::Immediate(format!(
                "Invalid pair: `{}`. Expected format: `key=value`.",
                pair
            ));
        }
    }

    // Update last_updated
    let now = chrono_iso_now();
    doc.set("last_updated", FmValue::String(now));

    let serialized = doc.to_string_content();
    if let Err(e) = std::fs::write(&state_file, &serialized) {
        return SlashResult::Immediate(format!("Failed to write STATE.md: {}", e));
    }

    SlashResult::Immediate(format!("Patched {} field(s):\n{}", updated_fields.len(), updated_fields.join("\n")))
}

/// Append a bullet item to a ### section in STATE.md body.
pub fn append_to_section(root: &Path, section: &str, text: &str) -> Result<(), String> {
    let state_file = paths::state_path(root);
    let content = std::fs::read_to_string(&state_file)
        .map_err(|e| format!("Cannot read STATE.md: {}", e))?;

    let mut doc = FmDocument::parse(&content);
    let target = format!("### {}", section);
    let bullet = format!("- {}", text);

    // Find the section in body and append bullet
    let body_lines: Vec<&str> = doc.body.lines().collect();
    let mut new_body = String::new();
    let mut found = false;
    let mut inserted = false;

    let mut i = 0;
    while i < body_lines.len() {
        let line = body_lines[i];
        new_body.push_str(line);
        new_body.push('\n');

        if !found && line.trim().starts_with(&target) {
            found = true;
            // Skip past section content until next heading or EOF
            i += 1;
            // Collect existing lines in this section
            let mut section_lines = Vec::new();
            while i < body_lines.len() {
                let next = body_lines[i];
                if next.starts_with("### ") || next.starts_with("## ") {
                    break;
                }
                section_lines.push(next);
                i += 1;
            }
            // Write section lines
            for sl in &section_lines {
                new_body.push_str(sl);
                new_body.push('\n');
            }
            // Append bullet at end of section
            // If last line of section was non-empty, we're fine. If section was empty, add bullet.
            new_body.push_str(&bullet);
            new_body.push('\n');
            inserted = true;
            continue; // Don't increment i again
        }
        i += 1;
    }

    if !found {
        // Section not found — append at end of body
        if !new_body.ends_with('\n') {
            new_body.push('\n');
        }
        new_body.push_str(&format!("\n{}\n\n{}\n", target, bullet));
        inserted = true;
    }

    if !inserted {
        return Err(format!("Could not insert into section '{}'", section));
    }

    // Preserve trailing newline behavior
    if doc.body.ends_with('\n') && !new_body.ends_with('\n') {
        new_body.push('\n');
    }

    doc.body = new_body;

    let serialized = doc.to_string_content();
    std::fs::write(&state_file, &serialized)
        .map_err(|e| format!("Cannot write STATE.md: {}", e))?;

    Ok(())
}

/// Record a decision to STATE.md ### Decisions section.
pub fn record_decision(root: &Path, decision: &str) -> Result<(), String> {
    append_to_section(root, "Decisions", decision)
}

/// Record a blocker to STATE.md ### Blockers/Concerns section.
pub fn record_blocker(root: &Path, blocker: &str) -> Result<(), String> {
    append_to_section(root, "Blockers/Concerns", blocker)
}

/// Returns current ISO 8601 timestamp.
fn chrono_iso_now() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    // Simple ISO 8601 without external crate
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Calculate year/month/day from days since epoch (1970-01-01)
    let (year, month, day) = days_to_ymd(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

/// Convert days since epoch to (year, month, day).
fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_STATE: &str = r#"---
gsd_state_version: 1.0
milestone: v1.2.1-dev
milestone_name: GSD Integration
status: executing
last_updated: "2026-03-07T02:17:35Z"
last_activity: 2026-03-07
progress:
  total_phases: 23
  completed_phases: 19
  total_plans: 49
  completed_plans: 45
  percent: 80
---

# Project State

## Performance Metrics

**Velocity:**
- v1.0.2: 17 plans (5 phases)
- v1.2.0: 19 plans (6 phases)

### Velocity

- v1.0.2: 17 plans completed (5 phases)
- v1.2.0: 19 plans completed (6 phases)

### Decisions

Key decisions logged in PROJECT.md.

- [v1.2.0]: ureq + std::thread

### Blockers/Concerns

- Research: frontmatter parser
- Research: state operations

### Known Tech Debt

- Warning text contrast
"#;

    #[test]
    fn test_format_progress_bar_zero() {
        let bar = format_progress_bar(0, 10);
        assert_eq!(bar, "[░░░░░░░░░░] 0%");
    }

    #[test]
    fn test_format_progress_bar_50() {
        let bar = format_progress_bar(50, 10);
        assert_eq!(bar, "[█████░░░░░] 50%");
    }

    #[test]
    fn test_format_progress_bar_100() {
        let bar = format_progress_bar(100, 10);
        assert_eq!(bar, "[██████████] 100%");
    }

    #[test]
    fn test_format_progress_bar_80() {
        let bar = format_progress_bar(80, 10);
        assert_eq!(bar, "[████████░░] 80%");
    }

    #[test]
    fn test_format_progress_bar_clamp_over_100() {
        let bar = format_progress_bar(150, 10);
        assert_eq!(bar, "[██████████] 100%");
    }

    #[test]
    fn test_format_state_display() {
        let doc = FmDocument::parse(SAMPLE_STATE);
        let output = format_state_display(&doc);

        assert!(output.contains("## GSD State"), "Should have heading");
        assert!(output.contains("**Milestone:** v1.2.1-dev (GSD Integration)"), "Milestone with name");
        assert!(output.contains("**Phase:** 19/23"), "Phase counts");
        assert!(output.contains("**Status:** executing"), "Status");
        assert!(output.contains("**Last activity:** 2026-03-07"), "Last activity");
        assert!(output.contains("[████████░░] 80%"), "Progress bar");
        assert!(output.contains("Plans: 45/49 | Phases: 19/23"), "Plan/phase counts");
        assert!(output.contains("### Velocity"), "Velocity section");
        assert!(output.contains("v1.0.2"), "Velocity content");
        assert!(output.contains("### Blockers"), "Blockers section");
        assert!(output.contains("frontmatter parser"), "Blocker content");
    }

    #[test]
    fn test_format_state_display_missing_fields() {
        let minimal = "---\nstatus: planning\n---\n\nBody text.\n";
        let doc = FmDocument::parse(minimal);
        let output = format_state_display(&doc);

        assert!(output.contains("**Status:** planning"));
        assert!(output.contains("[░░░░░░░░░░] 0%"), "Zero progress when fields missing");
    }

    #[test]
    fn test_cmd_progress_parsing() {
        let tmp = tempfile::tempdir().unwrap();
        let planning = tmp.path().join(".planning");
        std::fs::create_dir_all(&planning).unwrap();
        std::fs::write(planning.join("STATE.md"), SAMPLE_STATE).unwrap();

        // We can't easily call cmd_progress without WorkspaceState,
        // but we can test the core logic directly
        let doc = FmDocument::parse(SAMPLE_STATE);
        let percent = fm_i64(doc.get("progress.percent")) as u32;
        let total_plans = fm_i64(doc.get("progress.total_plans"));
        let done_plans = fm_i64(doc.get("progress.completed_plans"));

        assert_eq!(percent, 80);
        assert_eq!(total_plans, 49);
        assert_eq!(done_plans, 45);
    }

    #[test]
    fn test_extract_body_section_velocity() {
        let section = extract_body_section(SAMPLE_STATE.split("---").nth(2).unwrap_or(""), "Velocity");
        assert!(section.contains("v1.0.2"));
        assert!(section.contains("v1.2.0"));
    }

    #[test]
    fn test_extract_body_section_missing() {
        let section = extract_body_section("Some body text\n", "Nonexistent");
        assert!(section.is_empty());
    }

    #[test]
    fn test_parse_value_string() {
        assert_eq!(parse_value_string("true"), FmValue::Boolean(true));
        assert_eq!(parse_value_string("false"), FmValue::Boolean(false));
        assert_eq!(parse_value_string("42"), FmValue::Integer(42));
        assert_eq!(parse_value_string("3.14"), FmValue::Float(3.14));
        assert_eq!(parse_value_string("hello"), FmValue::String("hello".to_string()));
        assert_eq!(parse_value_string("executing"), FmValue::String("executing".to_string()));
    }

    #[test]
    fn test_handle_state_update() {
        let tmp = tempfile::tempdir().unwrap();
        let planning = tmp.path().join(".planning");
        std::fs::create_dir_all(&planning).unwrap();
        std::fs::write(planning.join("STATE.md"), SAMPLE_STATE).unwrap();

        let result = handle_state_update(tmp.path(), "status planning");
        match result {
            SlashResult::Immediate(msg) => {
                assert!(msg.contains("Updated"), "Should confirm update: {}", msg);
                assert!(msg.contains("status"), "Should mention field");
            }
            _ => panic!("Expected Immediate result"),
        }

        // Verify file was updated
        let content = std::fs::read_to_string(planning.join("STATE.md")).unwrap();
        let doc = FmDocument::parse(&content);
        assert_eq!(doc.get("status"), Some(&FmValue::String("planning".to_string())));
    }

    #[test]
    fn test_handle_state_update_nested() {
        let tmp = tempfile::tempdir().unwrap();
        let planning = tmp.path().join(".planning");
        std::fs::create_dir_all(&planning).unwrap();
        std::fs::write(planning.join("STATE.md"), SAMPLE_STATE).unwrap();

        let result = handle_state_update(tmp.path(), "progress.completed_phases 20");
        match result {
            SlashResult::Immediate(msg) => {
                assert!(msg.contains("Updated"), "{}", msg);
            }
            _ => panic!("Expected Immediate result"),
        }

        let content = std::fs::read_to_string(planning.join("STATE.md")).unwrap();
        let doc = FmDocument::parse(&content);
        assert_eq!(doc.get("progress.completed_phases"), Some(&FmValue::Integer(20)));
    }

    #[test]
    fn test_handle_state_patch() {
        let tmp = tempfile::tempdir().unwrap();
        let planning = tmp.path().join(".planning");
        std::fs::create_dir_all(&planning).unwrap();
        std::fs::write(planning.join("STATE.md"), SAMPLE_STATE).unwrap();

        let result = handle_state_patch(tmp.path(), "status=done progress.completed_phases=23");
        match result {
            SlashResult::Immediate(msg) => {
                assert!(msg.contains("Patched 2 field"), "{}", msg);
            }
            _ => panic!("Expected Immediate result"),
        }

        let content = std::fs::read_to_string(planning.join("STATE.md")).unwrap();
        let doc = FmDocument::parse(&content);
        assert_eq!(doc.get("status"), Some(&FmValue::String("done".to_string())));
        assert_eq!(doc.get("progress.completed_phases"), Some(&FmValue::Integer(23)));
    }

    #[test]
    fn test_append_to_section_decisions() {
        let tmp = tempfile::tempdir().unwrap();
        let planning = tmp.path().join(".planning");
        std::fs::create_dir_all(&planning).unwrap();
        std::fs::write(planning.join("STATE.md"), SAMPLE_STATE).unwrap();

        let result = record_decision(tmp.path(), "[v2.0]: New architecture chosen");
        assert!(result.is_ok(), "record_decision should succeed: {:?}", result);

        let content = std::fs::read_to_string(planning.join("STATE.md")).unwrap();
        assert!(content.contains("- [v2.0]: New architecture chosen"), "Decision should be appended");
        // Existing decisions still present
        assert!(content.contains("- [v1.2.0]: ureq + std::thread"));
    }

    #[test]
    fn test_append_to_section_blockers() {
        let tmp = tempfile::tempdir().unwrap();
        let planning = tmp.path().join(".planning");
        std::fs::create_dir_all(&planning).unwrap();
        std::fs::write(planning.join("STATE.md"), SAMPLE_STATE).unwrap();

        let result = record_blocker(tmp.path(), "New blocker: need auth token");
        assert!(result.is_ok());

        let content = std::fs::read_to_string(planning.join("STATE.md")).unwrap();
        assert!(content.contains("- New blocker: need auth token"));
        // Existing blockers still present
        assert!(content.contains("- Research: frontmatter parser"));
    }

    #[test]
    fn test_append_to_missing_section() {
        let minimal = "---\nstatus: ok\n---\n\n# Project\n\nSome text.\n";
        let tmp = tempfile::tempdir().unwrap();
        let planning = tmp.path().join(".planning");
        std::fs::create_dir_all(&planning).unwrap();
        std::fs::write(planning.join("STATE.md"), minimal).unwrap();

        let result = append_to_section(tmp.path(), "New Section", "First item");
        assert!(result.is_ok());

        let content = std::fs::read_to_string(planning.join("STATE.md")).unwrap();
        assert!(content.contains("### New Section"));
        assert!(content.contains("- First item"));
    }

    #[test]
    fn test_roundtrip_after_update() {
        let tmp = tempfile::tempdir().unwrap();
        let planning = tmp.path().join(".planning");
        std::fs::create_dir_all(&planning).unwrap();
        std::fs::write(planning.join("STATE.md"), SAMPLE_STATE).unwrap();

        // Update a single field
        handle_state_update(tmp.path(), "status done");

        // Verify unchanged fields are preserved
        let content = std::fs::read_to_string(planning.join("STATE.md")).unwrap();
        let doc = FmDocument::parse(&content);

        // Changed field
        assert_eq!(doc.get("status"), Some(&FmValue::String("done".to_string())));
        // Unchanged fields preserved
        assert_eq!(doc.get("milestone"), Some(&FmValue::String("v1.2.1-dev".to_string())));
        assert_eq!(doc.get("progress.total_phases"), Some(&FmValue::Integer(23)));
        // Body preserved
        assert!(content.contains("### Velocity"), "Body sections preserved");
        assert!(content.contains("### Decisions"), "Body sections preserved");
    }
}
