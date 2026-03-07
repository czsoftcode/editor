// GSD Frontmatter Parser — YAML-like frontmatter with round-trip fidelity
// Custom parser (zero new dependencies) for .planning/ markdown files.

/// A single value in frontmatter.
#[derive(Debug, Clone, PartialEq)]
pub enum FmValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    List(Vec<FmValue>),
    Map(Vec<(String, FmValue)>),
    Null,
}

/// A node in the frontmatter AST preserving source info for round-trip.
#[derive(Debug, Clone)]
pub struct FmNode {
    pub key: String,
    pub value: FmValue,
    /// Raw source lines (for round-trip reconstruction of unchanged nodes).
    pub raw_lines: Vec<String>,
    /// Comment lines immediately above this node.
    pub leading_comments: Vec<String>,
    /// Whether this node was modified via `set()`.
    pub modified: bool,
}

/// Parsed frontmatter document.
pub struct FmDocument {
    pub nodes: Vec<FmNode>,
    pub trailing_comments: Vec<String>,
    /// The body content after the closing `---`.
    pub body: String,
    /// Warnings from tolerant parsing.
    pub warnings: Vec<String>,
    /// Raw opening delimiter line.
    raw_open: String,
    /// Raw closing delimiter line.
    raw_close: String,
    /// Whether the source had a trailing newline.
    trailing_newline: bool,
}

impl FmDocument {
    /// Parse a full markdown file with optional frontmatter.
    pub fn parse(_content: &str) -> Self {
        todo!("RED phase: parse not yet implemented")
    }

    /// Get a value by dot-notation key (e.g., "progress.completed_phases").
    pub fn get(&self, _path: &str) -> Option<&FmValue> {
        todo!("RED phase: get not yet implemented")
    }

    /// Set a value by dot-notation key, creating intermediate maps if needed.
    pub fn set(&mut self, _path: &str, _value: FmValue) {
        todo!("RED phase: set not yet implemented")
    }

    /// Serialize back to string, preserving raw source for unchanged nodes.
    pub fn to_string_content(&self) -> String {
        todo!("RED phase: to_string_content not yet implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // CORE-01: Basic parsing
    // =========================================================================

    #[test]
    fn parse_simple_key_value() {
        let input = "---\nkey: value\n---\nbody text";
        let doc = FmDocument::parse(input);
        assert_eq!(doc.nodes.len(), 1);
        assert_eq!(doc.nodes[0].key, "key");
        assert_eq!(doc.nodes[0].value, FmValue::String("value".to_string()));
        assert_eq!(doc.body, "body text");
    }

    #[test]
    fn parse_integer_value() {
        let input = "---\ncount: 42\n---\n";
        let doc = FmDocument::parse(input);
        assert_eq!(doc.get("count"), Some(&FmValue::Integer(42)));
    }

    #[test]
    fn parse_float_value() {
        let input = "---\npi: 3.14\n---\n";
        let doc = FmDocument::parse(input);
        assert_eq!(doc.get("pi"), Some(&FmValue::Float(3.14)));
    }

    #[test]
    fn parse_boolean_values() {
        let input = "---\nenabled: true\ndisabled: false\n---\n";
        let doc = FmDocument::parse(input);
        assert_eq!(doc.get("enabled"), Some(&FmValue::Boolean(true)));
        assert_eq!(doc.get("disabled"), Some(&FmValue::Boolean(false)));
    }

    #[test]
    fn parse_quoted_strings() {
        let input = "---\ntitle: \"hello world\"\nalt: 'single quoted'\n---\n";
        let doc = FmDocument::parse(input);
        assert_eq!(
            doc.get("title"),
            Some(&FmValue::String("hello world".to_string()))
        );
        assert_eq!(
            doc.get("alt"),
            Some(&FmValue::String("single quoted".to_string()))
        );
    }

    #[test]
    fn parse_list() {
        let input = "---\nitems:\n  - alpha\n  - beta\n  - gamma\n---\n";
        let doc = FmDocument::parse(input);
        let items = doc.get("items").unwrap();
        match items {
            FmValue::List(list) => {
                assert_eq!(list.len(), 3);
                assert_eq!(list[0], FmValue::String("alpha".to_string()));
                assert_eq!(list[1], FmValue::String("beta".to_string()));
                assert_eq!(list[2], FmValue::String("gamma".to_string()));
            }
            _ => panic!("Expected List, got {:?}", items),
        }
    }

    #[test]
    fn parse_inline_list() {
        let input = "---\ntags: [a, b, c]\n---\n";
        let doc = FmDocument::parse(input);
        let tags = doc.get("tags").unwrap();
        match tags {
            FmValue::List(list) => {
                assert_eq!(list.len(), 3);
                assert_eq!(list[0], FmValue::String("a".to_string()));
            }
            _ => panic!("Expected List, got {:?}", tags),
        }
    }

    #[test]
    fn parse_nested_map() {
        let input = "---\nprogress:\n  total: 5\n  done: 3\n---\n";
        let doc = FmDocument::parse(input);
        assert_eq!(doc.get("progress.total"), Some(&FmValue::Integer(5)));
        assert_eq!(doc.get("progress.done"), Some(&FmValue::Integer(3)));
    }

    #[test]
    fn parse_inline_map() {
        let input = "---\nmetadata: {a: 1, b: 2}\n---\n";
        let doc = FmDocument::parse(input);
        assert_eq!(doc.get("metadata.a"), Some(&FmValue::Integer(1)));
        assert_eq!(doc.get("metadata.b"), Some(&FmValue::Integer(2)));
    }

    #[test]
    fn parse_null_value() {
        let input = "---\nempty:\n---\n";
        let doc = FmDocument::parse(input);
        assert_eq!(doc.get("empty"), Some(&FmValue::Null));
    }

    // =========================================================================
    // CORE-01: STATE.md format
    // =========================================================================

    #[test]
    fn parse_state_md_frontmatter() {
        let input = r#"---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: executing
last_updated: "2026-03-07T01:32:50.893Z"
last_activity: 2026-03-07 — Completed 19-03 code-fence fix
progress:
  total_phases: 5
  completed_phases: 1
  total_plans: 4
  completed_plans: 4
  percent: 75
---

# Project State

Body content here.
"#;
        let doc = FmDocument::parse(input);
        assert_eq!(doc.get("gsd_state_version"), Some(&FmValue::Float(1.0)));
        assert_eq!(
            doc.get("milestone"),
            Some(&FmValue::String("v1.0".to_string()))
        );
        assert_eq!(
            doc.get("status"),
            Some(&FmValue::String("executing".to_string()))
        );
        assert_eq!(
            doc.get("progress.total_phases"),
            Some(&FmValue::Integer(5))
        );
        assert_eq!(
            doc.get("progress.completed_plans"),
            Some(&FmValue::Integer(4))
        );
        assert_eq!(doc.get("progress.percent"), Some(&FmValue::Integer(75)));
        assert!(doc.body.contains("# Project State"));
    }

    // =========================================================================
    // CORE-02: Round-trip fidelity
    // =========================================================================

    #[test]
    fn round_trip_simple() {
        let input = "---\nkey: value\nanother: 42\n---\nbody text";
        let doc = FmDocument::parse(input);
        assert_eq!(doc.to_string_content(), input);
    }

    #[test]
    fn round_trip_state_md() {
        let input = r#"---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: executing
last_updated: "2026-03-07T01:32:50.893Z"
last_activity: 2026-03-07 — Completed 19-03 code-fence fix
progress:
  total_phases: 5
  completed_phases: 1
  total_plans: 4
  completed_plans: 4
  percent: 75
---

# Project State

Body content here.
"#;
        let doc = FmDocument::parse(input);
        assert_eq!(doc.to_string_content(), input);
    }

    #[test]
    fn round_trip_with_comments() {
        let input = "---\n# A comment\nkey: value\n---\nbody";
        let doc = FmDocument::parse(input);
        assert_eq!(doc.to_string_content(), input);
    }

    #[test]
    fn round_trip_with_lists() {
        let input = "---\nitems:\n  - one\n  - two\n  - three\n---\n";
        let doc = FmDocument::parse(input);
        assert_eq!(doc.to_string_content(), input);
    }

    #[test]
    fn round_trip_with_nested_map() {
        let input = "---\nprogress:\n  total: 5\n  done: 3\n---\nbody";
        let doc = FmDocument::parse(input);
        assert_eq!(doc.to_string_content(), input);
    }

    #[test]
    fn round_trip_inline_list() {
        let input = "---\ntags: [a, b, c]\n---\n";
        let doc = FmDocument::parse(input);
        assert_eq!(doc.to_string_content(), input);
    }

    // =========================================================================
    // Dot-notation get/set
    // =========================================================================

    #[test]
    fn get_top_level() {
        let input = "---\nkey: value\n---\n";
        let doc = FmDocument::parse(input);
        assert_eq!(doc.get("key"), Some(&FmValue::String("value".to_string())));
    }

    #[test]
    fn get_nested() {
        let input = "---\nprogress:\n  total: 5\n---\n";
        let doc = FmDocument::parse(input);
        assert_eq!(doc.get("progress.total"), Some(&FmValue::Integer(5)));
    }

    #[test]
    fn get_nonexistent_returns_none() {
        let input = "---\nkey: value\n---\n";
        let doc = FmDocument::parse(input);
        assert_eq!(doc.get("nonexistent"), None);
        assert_eq!(doc.get("key.nested"), None);
    }

    #[test]
    fn set_top_level_existing() {
        let input = "---\nstatus: pending\n---\nbody";
        let mut doc = FmDocument::parse(input);
        doc.set("status", FmValue::String("executing".to_string()));
        assert_eq!(
            doc.get("status"),
            Some(&FmValue::String("executing".to_string()))
        );
        let output = doc.to_string_content();
        assert!(output.contains("status: executing"));
        assert!(output.contains("body"));
    }

    #[test]
    fn set_nested_existing() {
        let input = "---\nprogress:\n  done: 3\n  total: 5\n---\n";
        let mut doc = FmDocument::parse(input);
        doc.set("progress.done", FmValue::Integer(4));
        assert_eq!(doc.get("progress.done"), Some(&FmValue::Integer(4)));
    }

    #[test]
    fn set_creates_new_top_level() {
        let input = "---\nexisting: yes\n---\n";
        let mut doc = FmDocument::parse(input);
        doc.set("new_key", FmValue::String("new_value".to_string()));
        assert_eq!(
            doc.get("new_key"),
            Some(&FmValue::String("new_value".to_string()))
        );
    }

    // =========================================================================
    // Tolerant parsing
    // =========================================================================

    #[test]
    fn tolerant_skips_malformed_lines() {
        let input = "---\ngood: value\n::: bad line\nanother: ok\n---\n";
        let doc = FmDocument::parse(input);
        assert_eq!(
            doc.get("good"),
            Some(&FmValue::String("value".to_string()))
        );
        assert_eq!(
            doc.get("another"),
            Some(&FmValue::String("ok".to_string()))
        );
        assert!(!doc.warnings.is_empty());
    }

    #[test]
    fn no_frontmatter_returns_body_only() {
        let input = "# Just a markdown file\n\nNo frontmatter here.";
        let doc = FmDocument::parse(input);
        assert!(doc.nodes.is_empty());
        assert_eq!(doc.body, input);
        assert!(doc.warnings.is_empty());
    }

    #[test]
    fn empty_frontmatter() {
        let input = "---\n---\nbody";
        let doc = FmDocument::parse(input);
        assert!(doc.nodes.is_empty());
        assert_eq!(doc.body, "body");
    }

    #[test]
    fn no_panic_on_empty_input() {
        let doc = FmDocument::parse("");
        assert!(doc.nodes.is_empty());
    }

    // =========================================================================
    // Multi-line block scalars
    // =========================================================================

    #[test]
    fn literal_block_scalar() {
        let input = "---\ndesc: |\n  line one\n  line two\n---\n";
        let doc = FmDocument::parse(input);
        let val = doc.get("desc").unwrap();
        match val {
            FmValue::String(s) => {
                assert!(s.contains("line one"));
                assert!(s.contains("line two"));
            }
            _ => panic!("Expected String for literal block, got {:?}", val),
        }
    }

    #[test]
    fn folded_block_scalar() {
        let input = "---\ndesc: >\n  line one\n  line two\n---\n";
        let doc = FmDocument::parse(input);
        let val = doc.get("desc").unwrap();
        match val {
            FmValue::String(s) => {
                assert!(s.contains("line one"));
                assert!(s.contains("line two"));
            }
            _ => panic!("Expected String for folded block, got {:?}", val),
        }
    }

    // =========================================================================
    // Edge cases
    // =========================================================================

    #[test]
    fn colon_in_value() {
        let input = "---\nurl: http://example.com:8080\n---\n";
        let doc = FmDocument::parse(input);
        assert_eq!(
            doc.get("url"),
            Some(&FmValue::String("http://example.com:8080".to_string()))
        );
    }

    #[test]
    fn value_with_special_chars() {
        let input = "---\nactivity: 2026-03-07 — Completed phase\n---\n";
        let doc = FmDocument::parse(input);
        assert_eq!(
            doc.get("activity"),
            Some(&FmValue::String(
                "2026-03-07 — Completed phase".to_string()
            ))
        );
    }

    #[test]
    fn list_of_quoted_strings() {
        let input = "---\nreqs: [\"AUTH-01\", \"AUTH-02\"]\n---\n";
        let doc = FmDocument::parse(input);
        let reqs = doc.get("reqs").unwrap();
        match reqs {
            FmValue::List(list) => {
                assert_eq!(list.len(), 2);
                assert_eq!(list[0], FmValue::String("AUTH-01".to_string()));
            }
            _ => panic!("Expected List, got {:?}", reqs),
        }
    }

    #[test]
    fn list_with_nested_key_value_items() {
        let input = "---\nprovides:\n  - phase: 19\n    provides: slash infra\n---\n";
        let doc = FmDocument::parse(input);
        let provides = doc.get("provides").unwrap();
        match provides {
            FmValue::List(list) => {
                assert_eq!(list.len(), 1);
                match &list[0] {
                    FmValue::Map(pairs) => {
                        assert!(pairs.iter().any(|(k, _)| k == "phase"));
                    }
                    _ => panic!("Expected Map item in list"),
                }
            }
            _ => panic!("Expected List, got {:?}", provides),
        }
    }

    #[test]
    fn multiple_nested_maps() {
        let input = "---\na:\n  x: 1\nb:\n  y: 2\n---\n";
        let doc = FmDocument::parse(input);
        assert_eq!(doc.get("a.x"), Some(&FmValue::Integer(1)));
        assert_eq!(doc.get("b.y"), Some(&FmValue::Integer(2)));
    }

    #[test]
    fn empty_string_value() {
        let input = "---\nkey: \"\"\n---\n";
        let doc = FmDocument::parse(input);
        assert_eq!(doc.get("key"), Some(&FmValue::String(String::new())));
    }

    #[test]
    fn preserves_trailing_newline() {
        let input = "---\nkey: val\n---\nbody\n";
        let doc = FmDocument::parse(input);
        assert_eq!(doc.to_string_content(), input);
    }
}
