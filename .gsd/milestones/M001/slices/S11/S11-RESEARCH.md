# Phase 20: GSD Core + State Engine - Research

**Researched:** 2026-03-07
**Domain:** YAML-like frontmatter parsing, config management, state/progress slash commands in Rust/egui
**Confidence:** HIGH

## Summary

Phase 20 builds the foundational GSD module that all subsequent GSD phases (21-23) depend on. The core deliverables are: (1) a custom YAML-like frontmatter parser with round-trip fidelity, (2) a config.json reader/writer with dot-notation, (3) path helpers for phase numbering and slug generation, (4) `/gsd state` and `/gsd progress` slash commands, and (5) graceful handling of missing `.planning/` directory.

The project already has `serde_json` as a dependency, so config.json management is straightforward. The frontmatter parser is the most complex piece -- it must handle a full YAML subset (strings, integers, floats, booleans, lists, nested maps, multi-line strings, quoted strings, inline lists/maps) while preserving comments, whitespace, and key ordering for round-trip fidelity. This requires an AST-based approach with source positions.

**Primary recommendation:** Build the GSD module in `src/app/ui/terminal/ai_chat/gsd/` (adjacent to `slash.rs`) with one file per concern: `mod.rs` (dispatch), `frontmatter.rs` (parser), `config.rs` (config.json), `state.rs` (state/progress commands), `paths.rs` (path helpers). All commands are synchronous (small file I/O). Integrate via a single `"gsd" => cmd_gsd(ws, args)` branch in slash.rs dispatch.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- GSD subcommand routing: standalone module, space-separated subcommands (`/gsd state`, `/gsd progress`)
- `/gsd` without args = `/gsd help`, detail in `/gsd help` not `/help`
- Custom frontmatter parser (no serde_yaml dependency) -- full YAML subset with round-trip fidelity
- Tolerant error handling: parse what's valid, ignore invalid lines, return partial result + warnings
- Full round-trip: preserve comments, whitespace, inline formatting, key ordering via AST with positions
- `/gsd state` shows detailed overview; `/gsd progress` shows only progress bar + phase table
- `/gsd state update` synchronous with dot-notation arguments
- `/gsd state patch` for batch updates
- Config file is `.planning/config.json` with graceful fallback and auto-create on first write
- `/gsd config get <key>` and `/gsd config set <key> <value>` for CRUD
- Dot-notation depth: 2 levels

### Claude's Discretion
- Internal architecture of GSD dispatch module (enum vs match vs trait)
- Progress bar format (Unicode block chars, length)
- Frontmatter AST internal representation
- Path helpers API design (slug generation, phase numbering)
- Error message wording for graceful fallback

### Deferred Ideas (OUT OF SCOPE)
- GSD update system (version check from GitHub)
- Compilation of new GSD versions from editor
- About dialog version check for GSD
- Evaluation of Rust compilation benefits
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CORE-01 | GSD frontmatter parser can parse YAML-like frontmatter from `.planning/` markdown files | Custom two-pass parser in frontmatter.rs with FrontmatterNode AST |
| CORE-02 | GSD frontmatter serializer can write frontmatter back preserving content | Round-trip via AST reconstruction -- preserve raw source spans for unchanged nodes |
| CORE-03 | GSD config module can load, read, and update `.planning/config.json` with dot-notation | serde_json::Value traversal with split('.') path navigation |
| CORE-04 | GSD core utilities provide path helpers, phase numbering, slug generation | paths.rs with functions for phase dir resolution, slug from name, decimal phase numbering |
| CORE-05 | GSD handles missing `.planning/` directory gracefully | check_planning_dir() guard returning friendly SlashResult::Immediate message |
| STATE-01 | `/gsd state` shows current project state | Parse STATE.md frontmatter + format as markdown table/sections |
| STATE-02 | `/gsd state update <field> <value>` updates STATE.md | Frontmatter parse -> modify AST node -> serialize back |
| STATE-03 | `/gsd state patch` batch-updates multiple fields | Parse args as key=value pairs, apply all mutations, single write |
| STATE-04 | `/gsd progress` shows visual progress bar | Read frontmatter progress fields, render Unicode block bar + phase table |
| STATE-05 | State module can record metrics, decisions, blockers | Append to markdown body sections (below frontmatter) via string manipulation |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde_json | 1.x (already in Cargo.toml) | config.json read/write | Already a project dependency, zero new deps policy |
| std::fs | stable | File I/O for STATE.md, config.json, frontmatter | Synchronous I/O matches project pattern (small files, sub-ms) |
| std::path | stable | Path manipulation for .planning/ directory | Standard library |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| (none) | - | - | Zero new dependencies policy -- everything uses std + existing deps |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Custom frontmatter parser | serde_yaml | Violates zero-new-deps policy; overkill for limited YAML subset |
| serde_json::Value traversal | Typed config struct | Typed struct is rigid; Value allows arbitrary keys and dot-notation without schema changes |
| String manipulation for body sections | pulldown-cmark roundtrip | pulldown-cmark is lossy on roundtrip; string find+insert is safer for preserving formatting |

## Architecture Patterns

### Recommended Project Structure
```
src/app/ui/terminal/ai_chat/
  slash.rs                    -- existing dispatch (add "gsd" branch)
  gsd/
    mod.rs                    -- GSD subcommand dispatch, help text
    frontmatter.rs            -- YAML-like parser + serializer (AST-based)
    config.rs                 -- config.json load/save/get/set
    state.rs                  -- /gsd state, /gsd state update, /gsd progress
    paths.rs                  -- path helpers, slug generation, phase numbering
```

### Pattern 1: GSD Subcommand Dispatch
**What:** Match-based dispatch mirroring slash.rs pattern but one level deeper.
**When to use:** For all `/gsd <subcommand>` routing.
**Example:**
```rust
// src/app/ui/terminal/ai_chat/gsd/mod.rs
use super::slash::SlashResult;
use crate::app::ui::workspace::state::WorkspaceState;

pub fn cmd_gsd(ws: &mut WorkspaceState, args: &str) -> SlashResult {
    let parts: Vec<&str> = args.splitn(2, char::is_whitespace).collect();
    let sub = parts.first().map(|s| s.to_lowercase()).unwrap_or_default();
    let sub_args = parts.get(1).unwrap_or(&"").trim();

    // Guard: check .planning/ exists
    if sub != "help" && !sub.is_empty() {
        if let Some(err) = check_planning_dir(&ws.root_path) {
            return err;
        }
    }

    match sub.as_str() {
        "" | "help" => cmd_gsd_help(),
        "state" => state::cmd_state(ws, sub_args),
        "progress" => state::cmd_progress(ws),
        "config" => config::cmd_config(ws, sub_args),
        _ => SlashResult::Immediate(format!(
            "Unknown GSD command: `{}`. Type `/gsd help` for available commands.", sub
        )),
    }
}

fn check_planning_dir(root: &std::path::Path) -> Option<SlashResult> {
    let planning = root.join(".planning");
    if !planning.is_dir() {
        Some(SlashResult::Immediate(
            "No `.planning/` directory found in this project.\n\n\
             To get started with GSD, create a `.planning/` directory with STATE.md and config.json."
                .to_string(),
        ))
    } else {
        None
    }
}
```

### Pattern 2: Frontmatter AST with Round-Trip
**What:** Parse `---` delimited frontmatter into an ordered AST that preserves source text for unchanged nodes.
**When to use:** For CORE-01 and CORE-02.
**Example:**
```rust
// src/app/ui/terminal/ai_chat/gsd/frontmatter.rs

/// A single value in frontmatter
#[derive(Debug, Clone)]
pub enum FmValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    List(Vec<FmValue>),
    Map(Vec<(String, FmValue)>),  // ordered pairs
    Null,
}

/// A node in the frontmatter AST preserving source info
#[derive(Debug, Clone)]
pub struct FmNode {
    pub key: String,
    pub value: FmValue,
    /// Raw source lines (for round-trip reconstruction of unchanged nodes)
    pub raw_lines: Vec<String>,
    /// Comment lines above this node
    pub leading_comments: Vec<String>,
}

/// Parsed frontmatter document
pub struct FmDocument {
    pub nodes: Vec<FmNode>,
    pub trailing_comments: Vec<String>,
    /// The body content after the closing `---`
    pub body: String,
    /// Warnings from tolerant parsing
    pub warnings: Vec<String>,
}

impl FmDocument {
    /// Parse a full markdown file with frontmatter
    pub fn parse(content: &str) -> Self { /* ... */ }

    /// Get a value by dot-notation key (e.g., "progress.completed_phases")
    pub fn get(&self, path: &str) -> Option<&FmValue> { /* ... */ }

    /// Set a value by dot-notation key, creating intermediate maps if needed
    pub fn set(&mut self, path: &str, value: FmValue) { /* ... */ }

    /// Serialize back to string, preserving raw source for unchanged nodes
    pub fn to_string(&self) -> String { /* ... */ }
}
```

### Pattern 3: Config JSON with Dot-Notation
**What:** Thin wrapper around serde_json::Value with dot-notation get/set.
**When to use:** For CORE-03.
**Example:**
```rust
// src/app/ui/terminal/ai_chat/gsd/config.rs
use serde_json::Value;
use std::path::Path;

pub struct GsdConfig {
    pub value: Value,
    pub path: std::path::PathBuf,
}

impl GsdConfig {
    pub fn load(root: &Path) -> Self {
        let path = root.join(".planning/config.json");
        let value = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| Value::Object(serde_json::Map::new()));
        Self { value, path }
    }

    pub fn get(&self, dot_path: &str) -> Option<&Value> {
        let parts: Vec<&str> = dot_path.split('.').collect();
        let mut current = &self.value;
        for part in parts {
            current = current.get(part)?;
        }
        Some(current)
    }

    pub fn set(&mut self, dot_path: &str, val: Value) {
        let parts: Vec<&str> = dot_path.split('.').collect();
        let mut current = &mut self.value;
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                current[part] = val.clone();
                return;
            }
            if !current.get(part).map_or(false, |v| v.is_object()) {
                current[part] = Value::Object(serde_json::Map::new());
            }
            current = &mut current[part];
        }
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create directory: {e}"))?;
        }
        let json = serde_json::to_string_pretty(&self.value)
            .map_err(|e| format!("JSON serialization error: {e}"))?;
        std::fs::write(&self.path, json)
            .map_err(|e| format!("Cannot write config: {e}"))?;
        Ok(())
    }
}
```

### Pattern 4: Synchronous Slash Command with File I/O
**What:** GSD commands read/write small files synchronously and return `SlashResult::Immediate`.
**When to use:** For all GSD state/config commands (files are <10KB, sub-ms I/O).
**Example:**
```rust
pub fn cmd_state(ws: &mut WorkspaceState, args: &str) -> SlashResult {
    let root = &ws.root_path;
    let state_path = root.join(".planning/STATE.md");

    if args.is_empty() {
        // Read and display state
        match std::fs::read_to_string(&state_path) {
            Ok(content) => {
                let doc = FmDocument::parse(&content);
                let output = format_state_display(&doc);
                SlashResult::Immediate(output)
            }
            Err(_) => SlashResult::Immediate(
                "No STATE.md found. Create `.planning/STATE.md` to track project state.".to_string()
            ),
        }
    } else if args.starts_with("update ") {
        // ... parse dot-notation field and value, update frontmatter
        todo!()
    } else if args.starts_with("patch ") {
        // ... batch update
        todo!()
    } else {
        SlashResult::Immediate(format!("Unknown state subcommand: `{args}`. Use `/gsd state`, `/gsd state update`, or `/gsd state patch`."))
    }
}
```

### Anti-Patterns to Avoid
- **Full YAML parser:** Don't try to handle all YAML spec (anchors, aliases, tags, flow sequences with nesting). The frontmatter subset is limited -- stick to the spec in CONTEXT.md.
- **Typed config struct:** Don't define a Rust struct for config.json. Use `serde_json::Value` -- it allows arbitrary keys and survives schema evolution across GSD versions.
- **Async file I/O for small files:** STATE.md and config.json are <10KB. Spawning threads + channels for these is over-engineering. Use synchronous `std::fs::read_to_string` / `std::fs::write`.
- **Modifying body via AST:** Don't try to parse the markdown body content into an AST. For STATE-05 (append decisions/blockers), use string find + insert at the right position.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON parsing/serialization | Custom JSON parser | `serde_json::Value` | Already a dependency; handles all edge cases |
| Unicode progress bar | ASCII approximation | Unicode block elements (U+2588 `█`, U+2591 `░`) | Clean visual, works in egui_commonmark |
| Config file creation | Manual directory/file creation | `std::fs::create_dir_all` + `serde_json::to_string_pretty` | Handles nested dirs, pretty output |
| Key ordering preservation | BTreeMap or HashMap | `Vec<(String, FmValue)>` | Preserves insertion order, trivial lookup for small maps |

**Key insight:** The frontmatter parser IS a hand-roll, but it's justified because the project has a zero-new-deps policy and the YAML subset is well-defined. Everything else should use existing standard library and serde_json.

## Common Pitfalls

### Pitfall 1: Frontmatter Delimiter Detection
**What goes wrong:** Parser fails on files with `---` inside code blocks, or files with no frontmatter.
**Why it happens:** Naive "split on `---`" doesn't account for markdown code fences.
**How to avoid:** First `---` must be at line 1 (or after optional BOM). Second `---` must be at start of line with nothing else (except optional trailing whitespace). Only look at the first two `---` lines, then stop.
**Warning signs:** Files with triple-dash in code blocks produce garbage.

### Pitfall 2: YAML Indentation Parsing
**What goes wrong:** Nested maps and lists have indentation-sensitive parsing that breaks on mixed tabs/spaces.
**Why it happens:** YAML uses indentation for nesting but different editors use different whitespace.
**How to avoid:** Normalize: treat each tab as 2 spaces. Count leading spaces to determine nesting depth. Use a stack-based approach where each level tracks its indent count.
**Warning signs:** Nested `progress:` section in STATE.md parses flat instead of hierarchical.

### Pitfall 3: Round-Trip Data Loss
**What goes wrong:** Writing back a parsed file loses comments, blank lines, or changes quoting style.
**Why it happens:** Parser discards "formatting" information and only keeps semantic data.
**How to avoid:** Store raw source lines per node. When serializing, if a node hasn't been modified, emit its raw lines verbatim. Only re-serialize nodes that were changed via `set()`.
**Warning signs:** `parse(content).to_string() != content` for unchanged documents.

### Pitfall 4: Dot-Notation Edge Cases
**What goes wrong:** `set("progress.total", 5)` on a document where `progress` doesn't exist yet, or where `progress` is a string instead of a map.
**Why it happens:** Intermediate path segments may not exist or may be wrong types.
**How to avoid:** For config.json (serde_json::Value), auto-create intermediate objects. For frontmatter, auto-create intermediate FmValue::Map nodes. When a path segment exists but isn't a map, replace it (config) or return an error (frontmatter -- don't silently destroy data).
**Warning signs:** Panic on `value["nonexistent"]["key"]`.

### Pitfall 5: Autocomplete for GSD Subcommands
**What goes wrong:** Typing `/gsd s` doesn't show autocomplete for GSD subcommands because the autocomplete system only handles top-level `/` commands.
**Why it happens:** Current autocomplete logic in input.rs checks `text.starts_with('/') && !text[1..].contains(char::is_whitespace)` -- this means `/gsd state` (with space) won't trigger autocomplete.
**How to avoid:** This is a Phase 20 concern. Either: (a) extend matching_commands to handle `/gsd <sub>` as a special case, or (b) defer GSD subcommand autocomplete. Recommend approach (a): when text starts with `/gsd `, pass the remainder to a `gsd::matching_subcommands()` function.
**Warning signs:** Users have to type full `/gsd state update` without any completion help.

### Pitfall 6: STATE.md Concurrent Access
**What goes wrong:** Two writes to STATE.md from different code paths corrupt the file.
**Why it happens:** Read-modify-write without locking.
**How to avoid:** All GSD operations are synchronous and run on the UI thread (egui immediate mode). There's no concurrent access -- the egui frame loop is single-threaded. No extra locking needed.
**Warning signs:** None expected -- this is a non-issue given the architecture.

## Code Examples

### Frontmatter Parsing (Two-Pass Approach)
```rust
// Pass 1: Extract raw frontmatter text between --- delimiters
// Pass 2: Parse YAML-like key-value pairs from extracted text

pub fn parse(content: &str) -> FmDocument {
    let lines: Vec<&str> = content.lines().collect();

    // Find frontmatter boundaries
    let (fm_start, fm_end) = match find_frontmatter_bounds(&lines) {
        Some(bounds) => bounds,
        None => return FmDocument::empty_with_body(content.to_string()),
    };

    // Extract frontmatter lines (between the two ---)
    let fm_lines = &lines[fm_start + 1..fm_end];

    // Parse into AST nodes
    let (nodes, warnings) = parse_yaml_lines(fm_lines);

    // Everything after closing --- is body
    let body_start = fm_end + 1;
    let body = if body_start < lines.len() {
        lines[body_start..].join("\n")
    } else {
        String::new()
    };

    FmDocument { nodes, trailing_comments: vec![], body, warnings }
}

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
```

### Progress Bar Rendering
```rust
fn format_progress_bar(percent: u32, width: usize) -> String {
    let filled = (width as u32 * percent / 100) as usize;
    let empty = width - filled;
    let bar: String = "\u{2588}".repeat(filled) + &"\u{2591}".repeat(empty);
    format!("[{}] {}%", bar, percent)
}

// Example output: [████████░░] 80%
```

### State Display Formatting
```rust
fn format_state_display(doc: &FmDocument) -> String {
    let mut out = String::from("## GSD State\n\n");

    if let Some(FmValue::String(m)) = doc.get("milestone") {
        out.push_str(&format!("**Milestone:** {}\n", m));
    }
    if let Some(FmValue::String(s)) = doc.get("status") {
        out.push_str(&format!("**Status:** {}\n", s));
    }
    if let Some(FmValue::String(a)) = doc.get("last_activity") {
        out.push_str(&format!("**Last activity:** {}\n", a));
    }

    // Progress section
    out.push_str("\n### Progress\n\n");
    let percent = doc.get("progress.percent")
        .and_then(|v| match v { FmValue::Integer(n) => Some(*n as u32), _ => None })
        .unwrap_or(0);
    out.push_str(&format_progress_bar(percent, 10));
    out.push('\n');

    // Plans and phases counts
    let total_plans = doc.get("progress.total_plans").and_then(|v| v.as_i64()).unwrap_or(0);
    let done_plans = doc.get("progress.completed_plans").and_then(|v| v.as_i64()).unwrap_or(0);
    let total_phases = doc.get("progress.total_phases").and_then(|v| v.as_i64()).unwrap_or(0);
    let done_phases = doc.get("progress.completed_phases").and_then(|v| v.as_i64()).unwrap_or(0);
    out.push_str(&format!("Plans: {}/{} | Phases: {}/{}\n", done_plans, total_plans, done_phases, total_phases));

    out
}
```

### Slash Integration Point
```rust
// In slash.rs, add to COMMANDS:
SlashCommand { name: "gsd", description: "GSD project management (type /gsd help for subcommands)" },

// In slash.rs dispatch match:
"gsd" => gsd::cmd_gsd(ws, args),
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| serde_yaml crate for YAML | Custom parser for limited subset | Project decision (v1.2.1-dev) | Zero new deps, tailored error handling |
| Async file I/O (tokio) | Synchronous std::fs | Project decision (v1.2.0) | Simpler code, no runtime dependency |
| HashMap for config | serde_json::Value | Standard Rust pattern | Flexible schema, dot-notation traversal |

**Deprecated/outdated:**
- serde_yaml 0.8 was the old standard, but serde_yaml 0.9+ has breaking changes and the crate is now "unsafe-libyaml" based. Custom parser avoids this entire dependency chain.

## Open Questions

1. **Frontmatter multi-line string format**
   - What we know: YAML supports `|` (literal block) and `>` (folded block) scalars
   - What's unclear: Does STATE.md actually use multi-line strings, or are all values single-line?
   - Recommendation: Check current STATE.md -- it uses quoted strings and simple values only. Implement `|` and `>` support but don't prioritize testing edge cases.

2. **GSD subcommand autocomplete scope**
   - What we know: Current autocomplete only handles top-level `/` commands
   - What's unclear: Should Phase 20 extend autocomplete for `/gsd <sub>` or defer to later?
   - Recommendation: Implement basic two-level autocomplete in Phase 20 -- it's a small change to input.rs and significantly improves UX. When text is `/gsd `, call `gsd::matching_subcommands()` instead of `slash::matching_commands()`.

3. **STATE-05 body section manipulation**
   - What we know: STATE-05 requires appending to "Decisions", "Blockers", "Pending Todos" sections
   - What's unclear: Exact format of section headers and bullet points
   - Recommendation: Use existing STATE.md format (seen in research). Find `### Decisions` header, locate end of section (next `###` or EOF), insert new bullet before that boundary.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[cfg(test)]` + `cargo test` |
| Config file | Cargo.toml (existing) |
| Quick run command | `cargo test --lib gsd -- --nocapture` |
| Full suite command | `cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CORE-01 | Frontmatter parse | unit | `cargo test --lib frontmatter -- --nocapture` | Wave 0 |
| CORE-02 | Frontmatter round-trip | unit | `cargo test --lib frontmatter::tests::round_trip -- --nocapture` | Wave 0 |
| CORE-03 | Config dot-notation get/set | unit | `cargo test --lib config -- --nocapture` | Wave 0 |
| CORE-04 | Path helpers, slug, numbering | unit | `cargo test --lib paths -- --nocapture` | Wave 0 |
| CORE-05 | Missing .planning/ graceful | unit | `cargo test --lib gsd::tests::missing_planning -- --nocapture` | Wave 0 |
| STATE-01 | /gsd state display | unit | `cargo test --lib state::tests::state_display -- --nocapture` | Wave 0 |
| STATE-02 | /gsd state update | unit | `cargo test --lib state::tests::state_update -- --nocapture` | Wave 0 |
| STATE-03 | /gsd state patch | unit | `cargo test --lib state::tests::state_patch -- --nocapture` | Wave 0 |
| STATE-04 | /gsd progress display | unit | `cargo test --lib state::tests::progress_display -- --nocapture` | Wave 0 |
| STATE-05 | Record metrics/decisions/blockers | unit | `cargo test --lib state::tests::append_section -- --nocapture` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --lib gsd -- --nocapture`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/app/ui/terminal/ai_chat/gsd/frontmatter.rs` -- `#[cfg(test)] mod tests` with parse, round-trip, tolerant parsing tests
- [ ] `src/app/ui/terminal/ai_chat/gsd/config.rs` -- `#[cfg(test)] mod tests` with get/set/dot-notation tests
- [ ] `src/app/ui/terminal/ai_chat/gsd/paths.rs` -- `#[cfg(test)] mod tests` with slug, numbering tests
- [ ] `src/app/ui/terminal/ai_chat/gsd/state.rs` -- `#[cfg(test)] mod tests` with state display, update, progress tests
- [ ] `src/app/ui/terminal/ai_chat/gsd/mod.rs` -- `#[cfg(test)] mod tests` with dispatch routing, missing .planning/ tests

## Sources

### Primary (HIGH confidence)
- Project source: `src/app/ui/terminal/ai_chat/slash.rs` -- existing slash dispatch pattern, SlashResult enum, COMMANDS registry
- Project source: `src/app/ui/terminal/ai_chat/logic.rs` -- slash intercept, send_query_to_agent flow
- Project source: `src/settings.rs` -- file I/O pattern (load/save with graceful fallback)
- Project source: `src/app/ui/widgets/ai/chat/input.rs` -- autocomplete system
- Project source: `.planning/STATE.md` -- actual frontmatter format to parse
- Project source: `.planning/config.json` -- actual config format to manage
- Project source: `Cargo.toml` -- serde_json already a dependency

### Secondary (MEDIUM confidence)
- CONTEXT.md decisions -- locked by user discussion

### Tertiary (LOW confidence)
- None -- all research based on project source code and locked decisions

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new deps, using existing serde_json + std::fs
- Architecture: HIGH -- follows established slash.rs dispatch pattern, module structure mirrors existing conventions
- Pitfalls: HIGH -- based on direct analysis of STATE.md format and codebase patterns
- Frontmatter parser: MEDIUM -- AST design is discretionary, two-pass approach is sound but implementation details need validation during coding

**Research date:** 2026-03-07
**Valid until:** 2026-04-07 (stable -- no external dependency changes)