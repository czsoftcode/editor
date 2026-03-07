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

impl FmValue {
    /// Traverse into a Map by key name, returning a reference to the sub-value.
    fn map_get(&self, key: &str) -> Option<&FmValue> {
        match self {
            FmValue::Map(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// Traverse into a Map by key name (mutable).
    fn map_get_mut(&mut self, key: &str) -> Option<&mut FmValue> {
        match self {
            FmValue::Map(pairs) => pairs.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// Set a key in a Map, inserting if not present.
    fn map_set(&mut self, key: &str, value: FmValue) {
        if let FmValue::Map(pairs) = self {
            if let Some(existing) = pairs.iter_mut().find(|(k, _)| k == key) {
                existing.1 = value;
            } else {
                pairs.push((key.to_string(), value));
            }
        }
    }

    /// Serialize a value to a YAML-like string (single line).
    fn serialize(&self) -> String {
        match self {
            FmValue::String(s) => {
                // Quote if contains special chars that would be ambiguous
                if s.is_empty() {
                    "\"\"".to_string()
                } else if s.contains(": ")
                    || s.contains('#')
                    || s.starts_with('"')
                    || s.starts_with('\'')
                    || s.starts_with('{')
                    || s.starts_with('[')
                    || s == "true"
                    || s == "false"
                    || s == "null"
                {
                    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
                } else {
                    s.clone()
                }
            }
            FmValue::Integer(n) => n.to_string(),
            FmValue::Float(f) => {
                let s = f.to_string();
                if s.contains('.') {
                    s
                } else {
                    format!("{s}.0")
                }
            }
            FmValue::Boolean(b) => b.to_string(),
            FmValue::Null => "".to_string(),
            FmValue::List(items) => {
                let parts: Vec<String> = items.iter().map(|v| v.serialize()).collect();
                format!("[{}]", parts.join(", "))
            }
            FmValue::Map(pairs) => {
                let parts: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.serialize()))
                    .collect();
                format!("{{{}}}", parts.join(", "))
            }
        }
    }
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
    /// Whether the source had frontmatter at all.
    has_frontmatter: bool,
}

impl FmDocument {
    /// Create an empty document with body only (no frontmatter).
    fn empty_with_body(body: String) -> Self {
        Self {
            nodes: Vec::new(),
            trailing_comments: Vec::new(),
            body,
            warnings: Vec::new(),
            raw_open: String::new(),
            raw_close: String::new(),
            trailing_newline: false,
            has_frontmatter: false,
        }
    }

    /// Parse a full markdown file with optional frontmatter.
    pub fn parse(content: &str) -> Self {
        if content.is_empty() {
            return Self::empty_with_body(String::new());
        }

        let trailing_newline = content.ends_with('\n');
        let lines: Vec<&str> = content.lines().collect();

        // Pass 1: Find frontmatter boundaries
        let (fm_start, fm_end) = match find_frontmatter_bounds(&lines) {
            Some(bounds) => bounds,
            None => return Self::empty_with_body(content.to_string()),
        };

        let raw_open = format!("{}\n", lines[fm_start]);

        // Extract frontmatter lines (between the two ---)
        let fm_lines = &lines[fm_start + 1..fm_end];

        // Parse into AST nodes
        let (nodes, trailing_comments, warnings) = parse_yaml_lines(fm_lines);

        // Everything after closing --- is body
        let body_start = fm_end + 1;
        let body = if body_start < lines.len() {
            let body_lines = &lines[body_start..];
            let mut body_str = body_lines.join("\n");
            if trailing_newline {
                body_str.push('\n');
            }
            body_str
        } else {
            String::new()
        };

        // If file ends with `---\n` and no body content, include \n in raw_close
        let raw_close = if body.is_empty() && trailing_newline {
            format!("{}\n", lines[fm_end])
        } else {
            lines[fm_end].to_string()
        };

        Self {
            nodes,
            trailing_comments,
            body,
            warnings,
            raw_open,
            raw_close,
            trailing_newline,
            has_frontmatter: true,
        }
    }

    /// Get a value by dot-notation key (e.g., "progress.completed_phases").
    pub fn get(&self, path: &str) -> Option<&FmValue> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        // Find the top-level node
        let node = self.nodes.iter().find(|n| n.key == parts[0])?;

        if parts.len() == 1 {
            return Some(&node.value);
        }

        // Navigate into nested values
        let mut current = &node.value;
        for &part in &parts[1..] {
            current = current.map_get(part)?;
        }
        Some(current)
    }

    /// Set a value by dot-notation key, creating intermediate maps if needed.
    pub fn set(&mut self, path: &str, value: FmValue) {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return;
        }

        if parts.len() == 1 {
            // Top-level set
            if let Some(node) = self.nodes.iter_mut().find(|n| n.key == parts[0]) {
                node.value = value;
                node.modified = true;
            } else {
                // Create new top-level node
                self.nodes.push(FmNode {
                    key: parts[0].to_string(),
                    value,
                    raw_lines: Vec::new(),
                    leading_comments: Vec::new(),
                    modified: true,
                });
            }
            return;
        }

        // Multi-level: find or create the top-level node
        let top_key = parts[0];
        let node = if let Some(idx) = self.nodes.iter().position(|n| n.key == top_key) {
            &mut self.nodes[idx]
        } else {
            self.nodes.push(FmNode {
                key: top_key.to_string(),
                value: FmValue::Map(Vec::new()),
                raw_lines: Vec::new(),
                leading_comments: Vec::new(),
                modified: true,
            });
            self.nodes.last_mut().unwrap()
        };
        node.modified = true;

        // Navigate to the parent of the target, creating intermediate Maps
        let mut current = &mut node.value;
        for &part in &parts[1..parts.len() - 1] {
            if current.map_get(part).is_none() {
                current.map_set(part, FmValue::Map(Vec::new()));
            }
            current = current.map_get_mut(part).unwrap();
        }

        // Set the leaf value
        let leaf_key = parts[parts.len() - 1];
        current.map_set(leaf_key, value);
    }

    /// Serialize back to string, preserving raw source for unchanged nodes.
    pub fn to_string_content(&self) -> String {
        if !self.has_frontmatter {
            return self.body.clone();
        }

        let mut out = String::new();
        out.push_str(&self.raw_open);

        for node in &self.nodes {
            // Leading comments
            for c in &node.leading_comments {
                out.push_str(c);
                out.push('\n');
            }

            if !node.modified && !node.raw_lines.is_empty() {
                // Round-trip: emit raw lines verbatim
                for line in &node.raw_lines {
                    out.push_str(line);
                    out.push('\n');
                }
            } else {
                // Re-serialize from value
                serialize_node(&mut out, &node.key, &node.value, 0);
            }
        }

        for c in &self.trailing_comments {
            out.push_str(c);
            out.push('\n');
        }

        out.push_str(&self.raw_close);

        // Add body
        if !self.body.is_empty() {
            out.push('\n');
            out.push_str(&self.body);
        }

        out
    }
}

// =============================================================================
// Internal parsing functions
// =============================================================================

fn find_frontmatter_bounds(lines: &[&str]) -> Option<(usize, usize)> {
    if lines.is_empty() || lines[0].trim() != "---" {
        return None;
    }
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            return Some((0, i));
        }
    }
    None
}

/// Count leading spaces in a line (tab = 2 spaces).
fn indent_of(line: &str) -> usize {
    let mut count = 0;
    for ch in line.chars() {
        match ch {
            ' ' => count += 1,
            '\t' => count += 2,
            _ => break,
        }
    }
    count
}

/// Parse a scalar value string into the appropriate FmValue.
fn parse_scalar(s: &str) -> FmValue {
    let trimmed = s.trim();

    if trimmed.is_empty() {
        return FmValue::Null;
    }

    // Quoted strings
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        let inner = &trimmed[1..trimmed.len() - 1];
        return FmValue::String(inner.to_string());
    }

    // Inline list
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        return parse_inline_list(trimmed);
    }

    // Inline map
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return parse_inline_map(trimmed);
    }

    // Boolean
    match trimmed {
        "true" => return FmValue::Boolean(true),
        "false" => return FmValue::Boolean(false),
        "null" | "~" => return FmValue::Null,
        _ => {}
    }

    // Integer
    if let Ok(n) = trimmed.parse::<i64>() {
        return FmValue::Integer(n);
    }

    // Float
    if let Ok(f) = trimmed.parse::<f64>() {
        // Only if it has a dot or is scientific notation
        if trimmed.contains('.') || trimmed.contains('e') || trimmed.contains('E') {
            return FmValue::Float(f);
        }
    }

    // Default: string
    FmValue::String(trimmed.to_string())
}

/// Parse inline list: `[a, b, c]`
fn parse_inline_list(s: &str) -> FmValue {
    let inner = &s[1..s.len() - 1];
    if inner.trim().is_empty() {
        return FmValue::List(Vec::new());
    }
    let items: Vec<FmValue> = split_inline_items(inner)
        .iter()
        .map(|item| parse_scalar(item))
        .collect();
    FmValue::List(items)
}

/// Parse inline map: `{a: 1, b: 2}`
fn parse_inline_map(s: &str) -> FmValue {
    let inner = &s[1..s.len() - 1];
    if inner.trim().is_empty() {
        return FmValue::Map(Vec::new());
    }
    let mut pairs = Vec::new();
    for item in split_inline_items(inner) {
        let item = item.trim();
        if let Some(colon_pos) = item.find(':') {
            let key = item[..colon_pos].trim().to_string();
            let val_str = item[colon_pos + 1..].trim();
            let value = parse_scalar(val_str);
            pairs.push((key, value));
        }
    }
    FmValue::Map(pairs)
}

/// Split comma-separated items respecting nested brackets and quotes.
fn split_inline_items(s: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut depth_bracket = 0i32;
    let mut depth_brace = 0i32;
    let mut in_double_quote = false;
    let mut in_single_quote = false;

    for ch in s.chars() {
        match ch {
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
                current.push(ch);
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
                current.push(ch);
            }
            '[' if !in_double_quote && !in_single_quote => {
                depth_bracket += 1;
                current.push(ch);
            }
            ']' if !in_double_quote && !in_single_quote => {
                depth_bracket -= 1;
                current.push(ch);
            }
            '{' if !in_double_quote && !in_single_quote => {
                depth_brace += 1;
                current.push(ch);
            }
            '}' if !in_double_quote && !in_single_quote => {
                depth_brace -= 1;
                current.push(ch);
            }
            ',' if !in_double_quote
                && !in_single_quote
                && depth_bracket == 0
                && depth_brace == 0 =>
            {
                items.push(current.trim().to_string());
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        items.push(trimmed);
    }
    items
}

/// Represents a pending line during parsing.
struct PendingLine<'a> {
    indent: usize,
    content: &'a str,
    raw: &'a str,
}

/// Parse YAML-like frontmatter lines into FmNode list.
fn parse_yaml_lines(lines: &[&str]) -> (Vec<FmNode>, Vec<String>, Vec<String>) {
    let mut nodes = Vec::new();
    let mut warnings = Vec::new();
    let mut trailing_comments = Vec::new();
    let mut pending_comments: Vec<String> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Empty line
        if trimmed.is_empty() {
            pending_comments.push(line.to_string());
            i += 1;
            continue;
        }

        // Comment line
        if trimmed.starts_with('#') {
            pending_comments.push(line.to_string());
            i += 1;
            continue;
        }

        let indent = indent_of(line);

        // Top-level key: value pair (indent == 0)
        if indent == 0 {
            if let Some((key, val_str)) = parse_key_value(trimmed) {
                let val_trimmed = val_str.trim();

                // Check for block scalar
                if val_trimmed == "|" || val_trimmed == ">" {
                    let is_literal = val_trimmed == "|";
                    let mut raw_lines = vec![line.to_string()];
                    let mut block_lines: Vec<String> = Vec::new();
                    i += 1;
                    while i < lines.len() {
                        let next = lines[i];
                        let next_indent = indent_of(next);
                        let next_trimmed = next.trim();
                        if !next_trimmed.is_empty() && next_indent == 0 {
                            break;
                        }
                        raw_lines.push(next.to_string());
                        if !next_trimmed.is_empty() {
                            // Strip the common indent (assume 2 for block scalars)
                            let stripped = if next.len() >= 2 { &next[2..] } else { next };
                            block_lines.push(stripped.to_string());
                        } else {
                            block_lines.push(String::new());
                        }
                        i += 1;
                    }
                    let value = if is_literal {
                        FmValue::String(block_lines.join("\n"))
                    } else {
                        // Folded: join with space
                        FmValue::String(block_lines.join(" "))
                    };
                    let comments = std::mem::take(&mut pending_comments);
                    nodes.push(FmNode {
                        key,
                        value,
                        raw_lines,
                        leading_comments: comments,
                        modified: false,
                    });
                    continue;
                }

                // Check for nested content (next line indented more)
                if val_trimmed.is_empty() {
                    let mut raw_lines = vec![line.to_string()];
                    let child_start = i + 1;
                    let (child_end, child_value) =
                        parse_nested_block(lines, child_start, &mut raw_lines, &mut warnings);
                    i = child_end;
                    let comments = std::mem::take(&mut pending_comments);
                    nodes.push(FmNode {
                        key,
                        value: child_value,
                        raw_lines,
                        leading_comments: comments,
                        modified: false,
                    });
                    continue;
                }

                // Simple scalar value
                let value = parse_scalar(val_str);
                let comments = std::mem::take(&mut pending_comments);
                nodes.push(FmNode {
                    key,
                    value,
                    raw_lines: vec![line.to_string()],
                    leading_comments: comments,
                    modified: false,
                });
                i += 1;
            } else {
                // Malformed line
                warnings.push(format!("Skipped malformed line: {}", line));
                pending_comments.push(line.to_string());
                i += 1;
            }
        } else {
            // Indented line at top level — should not happen, skip
            warnings.push(format!("Unexpected indented line at top level: {}", line));
            i += 1;
        }
    }

    // Any remaining pending comments are trailing
    if !pending_comments.is_empty() {
        trailing_comments = pending_comments;
    }

    (nodes, trailing_comments, warnings)
}

/// Parse nested block content (indented below a key).
/// Returns (next_index, parsed_value).
fn parse_nested_block(
    lines: &[&str],
    start: usize,
    raw_lines: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> (usize, FmValue) {
    if start >= lines.len() {
        return (start, FmValue::Null);
    }

    // Determine the indent of the first content line
    let mut first_content = start;
    while first_content < lines.len() && lines[first_content].trim().is_empty() {
        raw_lines.push(lines[first_content].to_string());
        first_content += 1;
    }
    if first_content >= lines.len() {
        return (first_content, FmValue::Null);
    }

    let base_indent = indent_of(lines[first_content]);
    if base_indent == 0 {
        return (start, FmValue::Null);
    }

    let first_trimmed = lines[first_content].trim();

    // Check if it's a list (starts with -)
    if first_trimmed.starts_with("- ") || first_trimmed == "-" {
        return parse_list_block(lines, first_content, base_indent, raw_lines, warnings);
    }

    // Otherwise it's a map
    parse_map_block(lines, first_content, base_indent, raw_lines, warnings)
}

/// Parse an indented list block.
fn parse_list_block(
    lines: &[&str],
    start: usize,
    base_indent: usize,
    raw_lines: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> (usize, FmValue) {
    let mut items: Vec<FmValue> = Vec::new();
    let mut i = start;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        let indent = indent_of(line);

        if trimmed.is_empty() {
            raw_lines.push(line.to_string());
            i += 1;
            continue;
        }

        if indent < base_indent {
            break;
        }

        if indent == base_indent && trimmed.starts_with("- ") {
            raw_lines.push(line.to_string());
            let item_content = trimmed[2..].trim();

            // Check if the next lines are indented more (list item is a map)
            let next_i = i + 1;
            if next_i < lines.len() {
                let next_line = lines[next_i];
                let next_indent = indent_of(next_line);
                let next_trimmed = next_line.trim();
                // If next line is more indented and has key: value, it's a map item continuation
                if next_indent > base_indent
                    && !next_trimmed.is_empty()
                    && !next_trimmed.starts_with("- ")
                    && next_trimmed.contains(": ")
                {
                    // This list item is a map: parse the first kv from the `- ` line
                    // and subsequent indented kv lines
                    let mut map_pairs: Vec<(String, FmValue)> = Vec::new();
                    if let Some((k, v)) = parse_key_value(item_content) {
                        map_pairs.push((k, parse_scalar(v)));
                    }
                    i = next_i;
                    let map_indent = next_indent;
                    while i < lines.len() {
                        let ml = lines[i];
                        let mt = ml.trim();
                        let mi = indent_of(ml);
                        if mt.is_empty() {
                            raw_lines.push(ml.to_string());
                            i += 1;
                            continue;
                        }
                        if mi < map_indent {
                            break;
                        }
                        if mi == map_indent {
                            raw_lines.push(ml.to_string());
                            if let Some((k, v)) = parse_key_value(mt) {
                                map_pairs.push((k, parse_scalar(v)));
                            }
                            i += 1;
                        } else {
                            break;
                        }
                    }
                    items.push(FmValue::Map(map_pairs));
                    continue;
                }
            }

            // Simple list item
            items.push(parse_scalar(item_content));
            i += 1;
        } else if indent == base_indent && trimmed == "-" {
            raw_lines.push(line.to_string());
            items.push(FmValue::Null);
            i += 1;
        } else {
            break;
        }
    }

    (i, FmValue::List(items))
}

/// Parse an indented map block.
fn parse_map_block(
    lines: &[&str],
    start: usize,
    base_indent: usize,
    raw_lines: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> (usize, FmValue) {
    let mut pairs: Vec<(String, FmValue)> = Vec::new();
    let mut i = start;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        let indent = indent_of(line);

        if trimmed.is_empty() {
            raw_lines.push(line.to_string());
            i += 1;
            continue;
        }

        if indent < base_indent {
            break;
        }

        if indent == base_indent {
            raw_lines.push(line.to_string());
            if let Some((key, val_str)) = parse_key_value(trimmed) {
                let val_trimmed = val_str.trim();
                if val_trimmed.is_empty() {
                    // Nested sub-block
                    let child_start = i + 1;
                    let (child_end, child_value) =
                        parse_nested_block(lines, child_start, raw_lines, warnings);
                    pairs.push((key, child_value));
                    i = child_end;
                } else {
                    pairs.push((key, parse_scalar(val_str)));
                    i += 1;
                }
            } else {
                warnings.push(format!("Skipped malformed nested line: {}", line));
                i += 1;
            }
        } else {
            // More deeply indented — belongs to previous entry, skip for now
            raw_lines.push(line.to_string());
            i += 1;
        }
    }

    (i, FmValue::Map(pairs))
}

/// Try to parse a `key: value` pair from a line (after trimming).
/// Returns (key, rest_after_colon_space) or None.
fn parse_key_value(line: &str) -> Option<(String, &str)> {
    // Find the first `: ` or a trailing `:`
    if let Some(pos) = line.find(": ") {
        let key = line[..pos].trim();
        if key.is_empty()
            || (key.contains(' ') && !key.starts_with('"'))
            || key.contains(':')
        {
            return None;
        }
        let val = &line[pos + 2..];
        Some((key.to_string(), val))
    } else if line.ends_with(':') {
        let key = line[..line.len() - 1].trim();
        if key.is_empty() || key.contains(':') {
            return None;
        }
        Some((key.to_string(), ""))
    } else {
        None
    }
}

/// Serialize a key-value pair at a given indent level.
fn serialize_node(out: &mut String, key: &str, value: &FmValue, indent: usize) {
    let prefix = " ".repeat(indent);
    match value {
        FmValue::Map(pairs) if indent == 0 || !pairs.is_empty() => {
            // For top-level maps or non-empty maps, serialize as nested block
            if matches!(value, FmValue::Map(p) if !p.is_empty()) && indent == 0 {
                out.push_str(&format!("{}{}:\n", prefix, key));
                for (k, v) in pairs {
                    serialize_node(out, k, v, indent + 2);
                }
            } else if matches!(value, FmValue::Map(p) if !p.is_empty()) {
                out.push_str(&format!("{}{}:\n", prefix, key));
                for (k, v) in pairs {
                    serialize_node(out, k, v, indent + 2);
                }
            } else {
                out.push_str(&format!("{}{}: {}\n", prefix, key, value.serialize()));
            }
        }
        FmValue::List(items)
            if !items.is_empty()
                && items
                    .iter()
                    .any(|i| matches!(i, FmValue::Map(_) | FmValue::List(_))) =>
        {
            // Complex list — use block format
            out.push_str(&format!("{}{}:\n", prefix, key));
            let item_prefix = " ".repeat(indent + 2);
            for item in items {
                out.push_str(&format!("{}- {}\n", item_prefix, item.serialize()));
            }
        }
        FmValue::List(items) if !items.is_empty() => {
            // Simple list — use block format
            out.push_str(&format!("{}{}:\n", prefix, key));
            let item_prefix = " ".repeat(indent + 2);
            for item in items {
                out.push_str(&format!("{}- {}\n", item_prefix, item.serialize()));
            }
        }
        FmValue::Null => {
            out.push_str(&format!("{}{}:\n", prefix, key));
        }
        _ => {
            out.push_str(&format!("{}{}: {}\n", prefix, key, value.serialize()));
        }
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
