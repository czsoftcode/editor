# Phase 19: Slash Command Infrastructure - Research

**Researched:** 2026-03-07
**Domain:** Chat panel slash command dispatch system (Rust / egui)
**Confidence:** HIGH

## Summary

Phase 19 implements a slash command dispatch system in the existing AI chat panel. When the user types `/command` in the chat prompt, the system intercepts it before it reaches the AI model, executes the command locally, and displays the result as a system message in the conversation. Seven built-in commands are required: `/help`, `/clear`, `/new`, `/model`, `/git`, `/build`, `/settings`. Unknown commands that start with `/` pass through to AI as normal prompts, but if they closely match a registered command, a fuzzy suggestion is shown.

The implementation is well-scoped: all integration points exist in the codebase, all state needed by commands is accessible via `WorkspaceState`, and the existing `conversation: Vec<(String, String)>` structure supports the output pattern. The main design challenge is the system message rendering (distinct background color) and the async `/build` command with message updating.

**Primary recommendation:** Create a new `src/app/ui/terminal/ai_chat/slash.rs` module with a command registry (HashMap or match-based), a dispatch function called from `send_query_to_agent()`, and individual handler functions. Use the existing `conversation` vec for output with a special marker prefix for system messages.

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions
- Slash command output appears as a **system message in the conversation** (inserted into `conversation` vec)
- System messages have a **distinct background color** (different from AI responses) in both dark and light mode
- Both the user's command AND the system response are visible in conversation
- Output is **formatted as markdown** using the existing egui_commonmark renderer
- /build is **non-blocking** -- runs cargo build on a background thread; immediately shows "Building..." then updates to result
- /git shows a **full diff summary** similar to `git diff --stat` (branch, per-file +/-, total)
- `/model` (no args) lists models with active marked; `/model <name>` switches
- **Strict lowercase** -- only `/help` works, not `/Help`
- **No space after slash** -- `/ help` is NOT a command
- If not a registered command, the **entire input goes to AI** as a normal prompt
- Unknown commands matching a registered prefix show **fuzzy suggestions** using Levenshtein distance
- /clear: clears conversation + resets token counters, preserves prompt history
- /new: same as /clear + shows PolyCredo ASCII logo with version/model/rank
- /settings: opens settings modal dialog (existing `show_settings_dialog` flag)
- Slash commands **are recorded** in prompt history (arrow-up recalls them)

### Claude's Discretion
- Exact Levenshtein threshold and suggestion count for fuzzy matching
- Specific background colors for system messages (within dark/light theme constraints)
- Internal dispatch architecture (HashMap vs match-based vs enum)
- How /build polls for completion (channel pattern vs polling)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SLASH-01 | `/help` shows list of all available commands with descriptions | Command registry with description metadata; markdown table output |
| SLASH-02 | `/clear` clears current chat conversation | Reuse pattern from `AiChatAction::NewQuery` (conversation.clear, token reset) |
| SLASH-03 | `/new` resets conversation keeping prompt history | Clone `AiChatAction::NewQuery` logic including `AiManager::get_logo()` |
| SLASH-04 | `/model` lists or switches models | Access `ws.ai.ollama.models` for list, `ws.ai.ollama.selected_model` for switch |
| SLASH-05 | `/git` shows git status/diff summary | Run `git diff --stat` via `std::process::Command` on background thread |
| SLASH-06 | `/build` triggers cargo build from chat | Reuse `build_runner::run_build_check()`, mpsc channel for async result |
| SLASH-07 | `/settings` opens settings dialog | Set `ws.show_settings_dialog = true` |
| SLASH-08 | Slash dispatch intercepts `/` before AI model | Check `prompt.starts_with('/')` at top of `send_query_to_agent()` |
| SLASH-09 | Unknown slash commands show helpful error with suggestions | Hand-rolled Levenshtein distance, threshold ~2, top 1-2 suggestions |

</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| (no new crates) | - | All functionality uses existing dependencies | v1.2.1-dev constraint: zero new dependencies |

### Supporting (already in Cargo.toml)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `std::sync::mpsc` | std | Background task results (build, git) | /build and /git async commands |
| `std::process::Command` | std | Execute git and cargo commands | /git diff --stat, /build |
| `egui_commonmark` | 0.20 | Markdown rendering in conversation | System message output rendering |
| `regex` | 1 | Parsing slash command input | Optional, but `starts_with` + `split_whitespace` sufficient |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-rolled Levenshtein | `strsim` crate | Zero-dep constraint forbids new crates; Levenshtein is 15 lines |
| HashMap registry | match-based dispatch | HashMap more extensible for Phase 20+ GSD commands; match simpler for 7 commands |

## Architecture Patterns

### Recommended Module Structure
```
src/app/ui/terminal/ai_chat/
  slash.rs          # NEW: command registry, dispatch, handlers
  logic.rs          # MODIFIED: intercept at top of send_query_to_agent()
  mod.rs            # MODIFIED: add AiChatAction variants or use slash module directly
  render.rs         # MODIFIED: system message distinct rendering
```

### Pattern 1: Command Registry with Dispatch
**What:** A struct holding registered commands mapped to handler functions
**When to use:** When the command set will grow (Phase 20+ adds ~15 GSD commands)
**Example:**
```rust
pub struct SlashCommand {
    pub name: &'static str,
    pub description: &'static str, // For /help output
    pub handler: fn(&str, &mut WorkspaceState) -> SlashResult,
}

pub enum SlashResult {
    /// Immediate text response (markdown)
    Immediate(String),
    /// Background task started, response will be updated via channel
    Async { placeholder: String },
    /// No output to conversation (e.g., /settings just opens dialog)
    Silent,
    /// Not a slash command, pass to AI
    NotACommand,
}
```

### Pattern 2: System Message Marker in Conversation Vec
**What:** Prefix system responses with a marker string so the renderer can distinguish them
**When to use:** The conversation is `Vec<(String, String)>` -- no enum variant for message type
**Example:**
```rust
// When inserting system message:
const SYSTEM_MSG_MARKER: &str = "\x00SYS\x00";

// In slash handler:
ws.ai.chat.conversation.push((
    "/help".to_string(),
    format!("{}{}", SYSTEM_MSG_MARKER, markdown_output),
));

// In conversation renderer:
if response.starts_with(SYSTEM_MSG_MARKER) {
    let content = &response[SYSTEM_MSG_MARKER.len()..];
    // Render with system_bg color instead of AI response color
    render_system_message(ui, content, cache);
} else {
    // Normal AI response rendering
}
```

**Alternative:** Change `Vec<(String, String)>` to `Vec<ChatEntry>` with an enum field for message type. This is cleaner but touches more code. Claude's discretion -- marker is minimal-diff, enum is future-proof.

### Pattern 3: Async Build Command with Channel
**What:** Non-blocking build that updates conversation entry
**When to use:** `/build` command
**Example:**
```rust
// Start build
let rx = build_runner::run_build_check(ws.root_path.clone());
ws.ai.chat.conversation.push((
    "/build".to_string(),
    format!("{}Building...", SYSTEM_MSG_MARKER),
));
// Store receiver for polling
ws.slash_build_rx = Some(rx);

// In per-frame update (background.rs or ai_chat poll):
if let Some(rx) = &ws.slash_build_rx {
    if let Ok(errors) = rx.try_recv() {
        // Update last conversation entry
        let summary = format_build_summary(&errors);
        if let Some(last) = ws.ai.chat.conversation.last_mut() {
            last.1 = format!("{}{}", SYSTEM_MSG_MARKER, summary);
        }
        ws.slash_build_rx = None;
    }
}
```

### Pattern 4: Intercept Point in send_query_to_agent
**What:** Check for slash prefix before AI processing
**When to use:** SLASH-08 requirement
**Example:**
```rust
pub fn send_query_to_agent(ws: &mut WorkspaceState, i18n: &I18n) {
    if ws.ai.chat.prompt.trim().is_empty() {
        return;
    }

    // Slash command intercept -- before Ollama connection check
    if ws.ai.chat.prompt.starts_with('/') {
        slash::dispatch(ws, i18n);
        return;
    }

    // Existing Ollama check and AI flow...
    if ws.ai.ollama.status != OllamaConnectionStatus::Connected {
        // ...
    }
}
```

**Key insight:** Intercept BEFORE the Ollama connection check. Slash commands work even when Ollama is disconnected.

### Anti-Patterns to Avoid
- **Don't use Toast for slash output:** Decisions lock output to conversation, not toasts
- **Don't send slash commands to AI:** SLASH-08 explicitly requires interception before AI
- **Don't block UI thread for /build or /git:** Both must run on background threads
- **Don't forget prompt history:** CONTEXT.md explicitly states slash commands ARE recorded in history

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Markdown rendering | Custom formatter | `egui_commonmark::CommonMarkCache` | Already integrated in AiState |
| Build execution | Custom cargo runner | `build_runner::run_build_check()` | Existing, tested, handles JSON output |
| Logo display | Custom ASCII art | `AiManager::get_logo()` | Reuse for /new command |
| Git branch | Custom parser | Existing `fetch_git_branch` pattern | Already spawns thread, returns via channel |

**Key insight:** Levenshtein distance IS hand-rolled (trivially -- ~15 lines), because adding `strsim` violates the zero-new-deps constraint.

## Common Pitfalls

### Pitfall 1: Blocking UI on /git and /build
**What goes wrong:** Running `git diff --stat` or `cargo build` synchronously freezes the egui render loop
**Why it happens:** Both commands can take 1-10+ seconds
**How to avoid:** Always spawn on `std::thread`, return `mpsc::Receiver`, poll with `try_recv()` in the frame update
**Warning signs:** UI becomes unresponsive when user types `/build`

### Pitfall 2: System Message Rendering Regression
**What goes wrong:** System messages render with AI response styling (wrong colors, model name header)
**Why it happens:** The conversation renderer treats all `(q, a)` pairs identically
**How to avoid:** Use a marker prefix or enum to distinguish system messages, skip AI-specific rendering (model name, token counts) for system entries
**Warning signs:** System messages show "PolyCredo CLI" header or model name

### Pitfall 3: /build Message Update After Clear
**What goes wrong:** Build completes and tries to update a conversation entry that was cleared by /clear or /new
**Why it happens:** The index or reference to the conversation entry is stale
**How to avoid:** Store a generation counter or check conversation length; if entry was cleared, either append a new entry or drop the result
**Warning signs:** Index out of bounds panic or updating wrong conversation entry

### Pitfall 4: Slash Commands Failing When Ollama Disconnected
**What goes wrong:** User can't use /help, /clear, /settings when Ollama is offline
**Why it happens:** Intercept happens after the Ollama connection check in `send_query_to_agent()`
**How to avoid:** Move slash intercept BEFORE the Ollama connection check
**Warning signs:** Toast error "Ollama disconnected" when typing /help

### Pitfall 5: /model Switch Without Validation
**What goes wrong:** User types `/model nonexistent_model` and the model is set but doesn't exist
**Why it happens:** No validation against `ws.ai.ollama.models` list
**How to avoid:** Check if model name exists in the models list; show error if not found, with suggestion of closest match
**Warning signs:** Chat silently fails on next prompt because model is invalid

### Pitfall 6: Levenshtein False Positives
**What goes wrong:** `/hello` triggers "Did you mean /help?" even though user intended to send to AI
**Why it happens:** Levenshtein threshold too low or applied to all `/` prefixed input
**How to avoid:** Only show suggestions when the input looks like a command attempt (short, single word). If input is clearly a sentence (has spaces beyond args), pass to AI without suggestion
**Warning signs:** Normal messages starting with `/` get intercepted

## Code Examples

### Hand-rolled Levenshtein Distance
```rust
/// Computes Levenshtein edit distance between two strings.
fn levenshtein(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();
    let mut prev: Vec<usize> = (0..=b_len).collect();
    let mut curr = vec![0; b_len + 1];
    for (i, ca) in a.chars().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr[j + 1] = (prev[j] + cost)
                .min(prev[j + 1] + 1)
                .min(curr[j] + 1);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b_len]
}
```

### /git Output Formatting (git diff --stat)
```rust
fn run_git_diff_stat(root: &Path) -> String {
    let output = std::process::Command::new("git")
        .args(["diff", "--stat", "HEAD"])
        .current_dir(root)
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let branch = /* from ws.git_branch */;
            format!("**Branch:** `{}`\n\n```\n{}\n```", branch, stdout.trim())
        }
        Err(e) => format!("Git error: {}", e),
    }
}
```

### /help Output Formatting
```rust
fn format_help(commands: &[SlashCommand]) -> String {
    let mut out = String::from("## Available Commands\n\n| Command | Description |\n|---------|-------------|\n");
    for cmd in commands {
        out.push_str(&format!("| `/{name}` | {desc} |\n", name = cmd.name, desc = cmd.description));
    }
    out
}
```

### Fuzzy Suggestion for Unknown Commands
```rust
fn suggest_command(input: &str, commands: &[&str]) -> Option<String> {
    let threshold = 2; // max edit distance
    let mut best: Option<(&str, usize)> = None;
    for cmd in commands {
        let dist = levenshtein(input, cmd);
        if dist <= threshold {
            if best.is_none() || dist < best.unwrap().1 {
                best = Some((cmd, dist));
            }
        }
    }
    best.map(|(cmd, _)| cmd.to_string())
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| All input goes to AI | Slash prefix intercept | This phase | Commands work offline, no AI cost |
| Toast-based feedback | Conversation-based output | This phase | Persistent, scrollable output |

## Open Questions

1. **ChatEntry enum vs marker prefix?**
   - What we know: Marker prefix is minimal diff, enum is cleaner long-term
   - What's unclear: How many places read `conversation` vec directly?
   - Recommendation: Start with marker prefix (less refactoring), consider enum in Phase 20 if GSD commands need richer metadata

2. **Where to poll /build result?**
   - What we know: Existing build polling happens in `workspace/mod.rs` background tick
   - What's unclear: Whether to add a new field to `WorkspaceState` or reuse existing `build_check_rx`
   - Recommendation: Add `slash_build_rx: Option<mpsc::Receiver<Vec<BuildError>>>` to ChatState or WorkspaceState, poll in the same background tick

3. **show_settings_dialog field location**
   - What we know: CONTEXT.md says "existing `show_settings_dialog` flag"
   - Research found: The field is likely on WorkspaceState or AppShared, needs verification during implementation
   - Recommendation: Grep for exact field name during task execution

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[cfg(test)]` + `#[test]` |
| Config file | Cargo.toml (standard) |
| Quick run command | `cargo test slash` |
| Full suite command | `cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SLASH-01 | /help returns formatted command list | unit | `cargo test slash::tests::help_output -x` | No -- Wave 0 |
| SLASH-02 | /clear empties conversation | unit | `cargo test slash::tests::clear_resets -x` | No -- Wave 0 |
| SLASH-03 | /new resets with logo | unit | `cargo test slash::tests::new_shows_logo -x` | No -- Wave 0 |
| SLASH-04 | /model lists and switches | unit | `cargo test slash::tests::model_list -x` | No -- Wave 0 |
| SLASH-05 | /git runs diff stat | integration | `cargo test slash::tests::git_output -x` | No -- Wave 0 |
| SLASH-06 | /build triggers async build | unit | `cargo test slash::tests::build_starts -x` | No -- Wave 0 |
| SLASH-07 | /settings opens dialog | unit | `cargo test slash::tests::settings_opens -x` | No -- Wave 0 |
| SLASH-08 | Dispatch intercepts / prefix | unit | `cargo test slash::tests::dispatch_intercept -x` | No -- Wave 0 |
| SLASH-09 | Unknown command shows suggestions | unit | `cargo test slash::tests::fuzzy_suggest -x` | No -- Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test slash`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/app/ui/terminal/ai_chat/slash.rs` -- all slash command tests (SLASH-01 through SLASH-09)
- [ ] Levenshtein distance unit tests (edge cases: empty strings, identical strings, single char diff)

Note: Many commands interact with WorkspaceState which requires complex setup. Tests for dispatch logic, Levenshtein, help formatting, and model validation can be pure unit tests. Tests for /git and /build need mock or are integration-level.

## Sources

### Primary (HIGH confidence)
- Project source code: `src/app/ui/terminal/ai_chat/logic.rs` -- exact intercept point identified
- Project source code: `src/app/ui/terminal/ai_chat/mod.rs` -- AiChatAction::NewQuery reuse pattern
- Project source code: `src/app/build_runner.rs` -- existing async build with mpsc pattern
- Project source code: `src/app/ui/background.rs` -- git fetch patterns (fetch_git_branch, fetch_git_status)
- Project source code: `src/app/cli/state.rs` -- ChatState struct with conversation/history vecs
- Project source code: `src/app/ui/widgets/ai/chat/conversation.rs` -- conversation renderer

### Secondary (MEDIUM confidence)
- `similar` crate (v2.7.0) already in Cargo.toml -- provides TextDiff but NOT Levenshtein distance; hand-rolled needed
- [edit-distance crate](https://crates.io/crates/edit-distance) -- reference implementation, but zero-dep constraint prevents usage

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- zero new dependencies, all patterns exist in codebase
- Architecture: HIGH -- intercept point, conversation model, and async patterns all verified in source
- Pitfalls: HIGH -- derived from direct code analysis of existing patterns (blocking, stale references, connection checks)

**Research date:** 2026-03-07
**Valid until:** 2026-04-07 (stable domain, no external API changes expected)
