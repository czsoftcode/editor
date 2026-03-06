# Architecture Patterns

**Domain:** Slash Command System + GSD Tools Integration into PolyCredo Editor
**Researched:** 2026-03-07
**Overall confidence:** HIGH (direct analysis of existing codebase + GSD Node.js source)

## Recommended Architecture

### High-Level Overview

```
User Input (chat prompt)
    |
    v
[SlashCommandDispatcher]  <-- intercepts before send_query_to_agent()
    |
    +-- starts with "/" ? -----> SlashCommand execution pipeline
    |                               |
    |                               +-- Pure commands (/clear, /help, /model)
    |                               |       -> immediate result -> chat as system message
    |                               |
    |                               +-- GSD commands (/gsd state, /gsd phase, ...)
    |                               |       -> GsdEngine -> result -> chat as system message
    |                               |
    |                               +-- AI-integrated GSD (/gsd research, /gsd plan)
    |                                       -> GsdEngine -> spawn AI via send_query_to_agent()
    |                                          with injected system context
    |
    +-- no "/" prefix ----------> send_query_to_agent() (existing flow, unchanged)
```

### Component Boundaries

| Component | Responsibility | Communicates With | New/Modified |
|-----------|---------------|-------------------|--------------|
| `SlashCommandDispatcher` | Parse "/" prefix, route to handler | ChatState, GsdEngine, WorkspaceState | **NEW** |
| `SlashCommandRegistry` | Register/lookup command handlers | SlashCommandDispatcher | **NEW** |
| `GsdEngine` | Port of gsd-tools.cjs logic (state, phase, roadmap, etc.) | Filesystem (.planning/), SlashCommandDispatcher | **NEW** |
| `logic.rs::send_query_to_agent()` | Existing AI query entry point | OllamaProvider, ChatState | **MODIFIED** (minor -- 15 lines added at top) |
| `background.rs::process_background_events()` | Stream polling, tool call handling | ChatState, ToolExecutor | **MODIFIED** (5 lines -- consume gsd_injected_context) |
| `ChatState` | Prompt, conversation history, streaming | UI render, logic.rs | **UNCHANGED** (output flows through existing conversation vec) |
| `WorkspaceState` | Central state container | Everything | **MODIFIED** (add 2 optional fields) |

### Data Flow

**Pure Slash Command (e.g. `/clear`, `/help`):**
```
1. User types "/clear" + Enter
2. ai_chat/logic.rs:send_query_to_agent() detects "/" prefix
3. SlashCommandDispatcher::dispatch("/clear", args, &mut ws)
4. Handler executes immediately (clears conversation)
5. Result injected as conversation entry: ("/clear", "[output markdown]")
6. No AI call made, prompt cleared, return early
```

**GSD Slash Command (e.g. `/gsd state`):**
```
1. User types "/gsd state" + Enter
2. SlashCommandDispatcher routes to GsdCommand handler
3. GsdCommand delegates to GsdEngine::cmd_state()
4. GsdEngine reads .planning/STATE.md, parses frontmatter
5. Returns formatted markdown string
6. Dispatcher pushes ("/gsd state", "[markdown table]") to conversation
7. No AI call made
```

**AI-Integrated GSD Command (e.g. `/gsd research "How to do X"`):**
```
1. User types "/gsd research 'How to implement X'" + Enter
2. SlashCommandDispatcher routes to GsdCommand handler
3. GsdEngine prepares context (project state, phase info, templates)
4. Returns SlashResult::DelegateToAi with enhanced system context
5. Dispatcher stores context in ws.gsd_injected_context
6. Dispatcher sets ws.ai.chat.prompt to the research question
7. Falls through to normal send_query_to_agent() flow
8. send_query_to_agent() checks ws.gsd_injected_context, prepends to system message
9. AI streams response normally through existing pipeline
```

## Interception Point: Where Slash Commands Enter

**Location:** `src/app/ui/terminal/ai_chat/logic.rs` at the top of `send_query_to_agent()`.

**Rationale:** This is the single entry point for all user chat input. The function already:
- Checks for empty prompts (line 12-14)
- Validates Ollama connection (line 16-19)
- Has mutable `&mut WorkspaceState` access
- Precedes all AI provider calls

Slash interception belongs as the first meaningful check after empty-prompt guard:

```rust
pub fn send_query_to_agent(ws: &mut WorkspaceState, i18n: &crate::i18n::I18n) {
    let prompt = ws.ai.chat.prompt.trim().to_string();
    if prompt.is_empty() {
        return;
    }

    // --- NEW: Slash command interception ---
    if prompt.starts_with('/') {
        let registry = ws.ensure_slash_registry();
        let result = registry.dispatch(&prompt, ws, i18n);
        match result {
            SlashResult::Handled(output) => {
                ws.ai.chat.conversation.push((prompt.clone(), output));
                ws.ai.chat.prompt.clear();
                ws.ai.chat.auto_scroll = true;
                if ws.ai.chat.history.last() != Some(&prompt) {
                    ws.ai.chat.history.push(prompt);
                }
                ws.ai.chat.history_index = None;
                return; // Skip AI call entirely
            }
            SlashResult::DelegateToAi { system_context, user_prompt } => {
                ws.ai.chat.prompt = user_prompt;
                ws.gsd_injected_context = Some(system_context);
                // Fall through to existing AI flow below
            }
            SlashResult::Error(msg) => {
                ws.ai.chat.conversation.push((prompt.clone(), format!("*Error: {}*", msg)));
                ws.ai.chat.prompt.clear();
                return;
            }
            SlashResult::Unknown => {
                ws.ai.chat.conversation.push((
                    prompt.clone(),
                    i18n.get("slash-unknown-command"),
                ));
                ws.ai.chat.prompt.clear();
                return;
            }
        }
    }

    // --- Existing flow continues unchanged from here ---
    if ws.ai.ollama.status != OllamaConnectionStatus::Connected { ... }
    // ...
}
```

**Where gsd_injected_context is consumed:** In `send_query_to_agent()` itself, when building the system message (around line 52-60 of current code):

```rust
// Existing code builds system_content from expertise + reasoning mandates
let full_system = if context_str.is_empty() {
    system_content
} else {
    format!("{}\n\n{}", system_content, context_str)
};

// NEW: Inject GSD context if present
let full_system = if let Some(gsd_ctx) = ws.gsd_injected_context.take() {
    format!("{}\n\n{}", full_system, gsd_ctx)
} else {
    full_system
};
```

## Module Structure

**Recommended:** `src/app/cli/slash/` for dispatch + `src/app/cli/gsd/` for GSD engine.

**Rationale:** Both belong under `src/app/cli/` because:
- They are part of the CLI/AI subsystem (not general editor functionality)
- They share types with `cli/types.rs` (AiMessage, conversation tuples)
- GSD commands that delegate to AI need the same provider infrastructure
- Separate `src/gsd/` at project root would create cross-module coupling

```
src/app/cli/
  mod.rs                  -- add: pub mod slash; pub mod gsd;
  slash/
    mod.rs                -- SlashResult enum, SlashHandler trait, CommandRegistry
    dispatch.rs           -- dispatch() function, "/" prefix parsing
    builtin.rs            -- /help, /clear, /new, /model, /git, /build, /settings
  gsd/
    mod.rs                -- GsdEngine struct, public API, GsdCommand (SlashHandler impl)
    state.rs              -- STATE.md parser/writer (port of state.cjs ~120 LOC)
    config.rs             -- .planning/config.json management (port of config.cjs ~80 LOC)
    phase.rs              -- Phase operations: add, insert, remove, complete (port of phase.cjs ~200 LOC)
    roadmap.rs            -- ROADMAP.md parse/update (port of roadmap.cjs ~150 LOC)
    frontmatter.rs        -- Markdown frontmatter CRUD (port of frontmatter.cjs ~100 LOC)
    verify.rs             -- Verification suite: plan structure, completeness, refs (port of verify.cjs ~250 LOC)
    template.rs           -- Template fill: summary, plan, verification (port of template.cjs ~150 LOC)
    milestone.rs          -- Milestone archival (port of milestone.cjs ~100 LOC)
    core.rs               -- Shared utilities: slug, timestamp, path helpers (port of core.cjs ~80 LOC)
```

**LOC estimates:** Based on gsd-tools.cjs (592 lines) + 11 lib modules. Rust port will be ~2,500-3,500 LOC total (Rust is more verbose than JS but the logic is straightforward file I/O + string manipulation).

## Patterns to Follow

### Pattern 1: SlashResult Enum (Command Return Protocol)

**What:** Every slash command returns a typed result that tells the dispatcher how to handle output.

**When:** Always. Every command handler must return SlashResult.

```rust
pub enum SlashResult {
    /// Command handled locally. String is markdown output for chat display.
    Handled(String),
    /// Command needs AI. Provides enhanced system context + modified user prompt.
    DelegateToAi {
        system_context: String,
        user_prompt: String,
    },
    /// Command execution failed.
    Error(String),
    /// Not a recognized command.
    Unknown,
}
```

### Pattern 2: Trait-Based Command Registration

**What:** Commands implement a trait, registered in a HashMap. GSD registers as one top-level command with subcommands.

**When:** For extensibility without modifying dispatcher logic.

```rust
pub trait SlashHandler: Send + Sync {
    /// Primary command name (e.g. "clear", "gsd")
    fn name(&self) -> &str;
    /// Subcommand names for autocomplete (e.g. "gsd" -> ["state", "phase", ...])
    fn subcommands(&self) -> Vec<&str> { vec![] }
    /// One-line help text
    fn help(&self) -> &str;
    /// Execute the command. `args` is everything after the command name.
    fn execute(&self, args: &str, ws: &mut WorkspaceState, i18n: &I18n) -> SlashResult;
}

pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn SlashHandler>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut reg = Self { commands: HashMap::new() };
        reg.register(Box::new(ClearCommand));
        reg.register(Box::new(HelpCommand));
        reg.register(Box::new(ModelCommand));
        reg.register(Box::new(NewCommand));
        reg.register(Box::new(GitCommand));
        reg.register(Box::new(BuildCommand));
        reg.register(Box::new(SettingsCommand));
        reg.register(Box::new(GsdCommand::new()));
        reg
    }

    pub fn dispatch(&self, input: &str, ws: &mut WorkspaceState, i18n: &I18n) -> SlashResult {
        let input = input.strip_prefix('/').unwrap_or(input);
        let (cmd, args) = input.split_once(' ').unwrap_or((input, ""));
        match self.commands.get(cmd) {
            Some(handler) => handler.execute(args, ws, i18n),
            None => SlashResult::Unknown,
        }
    }
}
```

### Pattern 3: GSD Output as Conversation Entries

**What:** GSD command output appears as a conversation pair (user command, system response), rendered with existing markdown pipeline.

**When:** For all slash commands that produce output.

**Rationale:** The chat UI already renders `ws.ai.chat.conversation` entries as markdown via `egui_commonmark`. No new UI components needed. Users see command + result inline in the conversation flow.

```rust
// In dispatcher:
SlashResult::Handled(output) => {
    ws.ai.chat.conversation.push((prompt.clone(), output));
    // ^ "user" side shows the command, "assistant" side shows formatted output
}
```

### Pattern 4: GsdEngine as Stateless Processor

**What:** GsdEngine functions read from disk per invocation, compute, and return results. No persistent in-memory GSD state cache.

**When:** For all GSD operations.

**Rationale:**
- `.planning/` files are the source of truth (may be edited by git, CLI tools, other editors)
- Node.js gsd-tools.cjs works the same way -- each invocation reads fresh state
- Files are small (STATE.md < 1KB, config.json < 500B, ROADMAP.md < 5KB)
- Disk I/O cost is negligible for local SSD reads
- Eliminates cache invalidation complexity

```rust
pub struct GsdEngine;

impl GsdEngine {
    pub fn load_state(root: &Path) -> Result<GsdProjectState, String> {
        let state_path = root.join(".planning/STATE.md");
        let content = std::fs::read_to_string(&state_path)
            .map_err(|e| format!("Cannot read STATE.md: {}", e))?;
        parse_state_frontmatter(&content)
    }

    pub fn execute(subcommand: &str, args: &str, root: &Path) -> Result<String, String> {
        match subcommand {
            "state" => Self::cmd_state(args, root),
            "phase" => Self::cmd_phase(args, root),
            "roadmap" => Self::cmd_roadmap(args, root),
            "commit" => Self::cmd_commit(args, root),
            "verify" => Self::cmd_verify(args, root),
            "progress" => Self::cmd_progress(args, root),
            _ => Err(format!("Unknown GSD subcommand: {}", subcommand)),
        }
    }
}
```

### Pattern 5: AI Delegation with Injected Context

**What:** AI-integrated GSD commands prepare rich system context, then delegate to the normal AI flow without duplicating provider setup logic.

**When:** For commands like `/gsd research`, `/gsd plan`, `/gsd commit-msg`.

```rust
// In GsdCommand handler:
"research" => {
    let question = args_after_subcommand.to_string();
    let state = GsdEngine::load_state(&ws.root_path).ok();
    let roadmap = GsdEngine::load_roadmap(&ws.root_path).ok();
    let context = format!(
        "## GSD Project Context\n\
         Current phase: {}\n\
         Status: {}\n\n\
         ## Research Task\n\
         Investigate and document: {}",
        state.as_ref().map(|s| &s.current_phase).unwrap_or(&"unknown".to_string()),
        state.as_ref().map(|s| &s.status).unwrap_or(&"unknown".to_string()),
        question
    );
    SlashResult::DelegateToAi {
        system_context: context,
        user_prompt: question,
    }
}
```

## State Management Decision

**GSD state does NOT get its own persistent state struct. Only 2 transient fields added to WorkspaceState.**

**Rationale:**
1. GsdEngine reads from disk per invocation (stateless pattern)
2. Only the AI delegation flow needs a transient bridge field (`gsd_injected_context`)
3. The slash registry is lazily initialized and lives for the workspace lifetime
4. All command output flows through existing `ws.ai.chat.conversation`

**Additions to WorkspaceState:**

```rust
pub struct WorkspaceState {
    // ... existing fields ...

    /// Slash command registry (lazily initialized on first "/" input)
    pub slash_registry: Option<crate::app::cli::slash::CommandRegistry>,

    /// Injected GSD context for AI-delegated commands (consumed once by send_query_to_agent)
    pub gsd_injected_context: Option<String>,
}
```

**What NOT to add:**
- No `GsdState` struct with cached STATE.md -- read fresh each time
- No `gsd_history` -- output is already in chat conversation
- No `gsd_config_cache` -- config.json is <500B, read on demand
- No new background task infrastructure -- GSD ops are sync disk I/O

## Anti-Patterns to Avoid

### Anti-Pattern 1: Separate Input Widget for Slash Commands

**What:** Creating a command palette or separate text input for slash commands.

**Why bad:** Splits user attention, duplicates input handling, breaks the "everything through chat" paradigm established in v1.2.0.

**Instead:** Intercept "/" prefix in existing chat prompt. All output appears in the same conversation.

### Anti-Pattern 2: GSD State as Persistent Cache

**What:** Loading .planning/ state into memory and keeping it synchronized with disk.

**Why bad:** External tools (git, CLI, other editors) modify .planning/ files. Cache invalidation becomes a separate problem. Node.js version reads fresh on each invocation.

**Instead:** Read from disk on each command. Files are small, I/O is negligible.

### Anti-Pattern 3: Async Runtime for GSD Operations

**What:** Introducing tokio or async-std for GSD file I/O.

**Why bad:** Codebase uses `ureq + std::thread` (KEY DECISION from PROJECT.md). All GSD operations are local disk reads (<1ms). Async runtime contradicts established threading model.

**Instead:** Synchronous file I/O for reads. For slow operations (git commit), use existing `spawn_task()` from `background.rs`.

### Anti-Pattern 4: Full YAML Parser Dependency for Frontmatter

**What:** Adding `serde_yaml` or similar for STATE.md frontmatter parsing.

**Why bad:** GSD frontmatter is flat key-value pairs (`key: value`), never nested YAML. A full YAML parser adds dependency weight for no benefit.

**Instead:** Simple line-based parser: split on `---` delimiters, parse `key: value` lines. The Node.js version uses basic string splitting.

### Anti-Pattern 5: Direct ChatState Mutation from Command Handlers

**What:** GSD command handlers directly pushing to `ws.ai.chat.conversation` or modifying streaming state.

**Why bad:** Breaks encapsulation. If chat internals change (e.g., conversation format), all handlers break.

**Instead:** Return `SlashResult` and let the dispatcher handle chat state mutations uniformly through a single code path.

## Integration Points Summary

### Files to CREATE (New)

| File | Purpose | Est. LOC |
|------|---------|----------|
| `src/app/cli/slash/mod.rs` | SlashResult, SlashHandler trait, CommandRegistry | ~100 |
| `src/app/cli/slash/dispatch.rs` | dispatch() function, "/" parsing | ~40 |
| `src/app/cli/slash/builtin.rs` | /help, /clear, /new, /model, /git, /build, /settings | ~200 |
| `src/app/cli/gsd/mod.rs` | GsdEngine, GsdCommand (SlashHandler impl) | ~150 |
| `src/app/cli/gsd/core.rs` | Slugs, timestamps, path helpers | ~80 |
| `src/app/cli/gsd/frontmatter.rs` | Markdown frontmatter CRUD | ~120 |
| `src/app/cli/gsd/state.rs` | STATE.md parser/writer | ~150 |
| `src/app/cli/gsd/config.rs` | .planning/config.json management | ~80 |
| `src/app/cli/gsd/phase.rs` | Phase add/insert/remove/complete | ~250 |
| `src/app/cli/gsd/roadmap.rs` | ROADMAP.md parse/update | ~200 |
| `src/app/cli/gsd/verify.rs` | Verification suite | ~300 |
| `src/app/cli/gsd/template.rs` | Template fill operations | ~200 |
| `src/app/cli/gsd/milestone.rs` | Milestone archival | ~120 |

**Total new code:** ~2,000 LOC

### Files to MODIFY (Existing)

| File | Change | Risk |
|------|--------|------|
| `src/app/cli/mod.rs` | Add `pub mod slash; pub mod gsd;` | LOW |
| `src/app/ui/terminal/ai_chat/logic.rs` | Add 25-line slash interception at top of `send_query_to_agent()` + 5-line context injection | LOW -- additive, existing flow unchanged |
| `src/app/ui/workspace/state/mod.rs` | Add 2 optional fields: `slash_registry`, `gsd_injected_context` | LOW |
| `src/app/ui/workspace/state/init.rs` | Initialize new fields to `None` | LOW |
| `locales/*.ftl` (5 files) | Add ~30 i18n keys for slash command messages | LOW |

### Files UNCHANGED

| File | Why |
|------|-----|
| `src/app/cli/executor.rs` | Tool executor untouched -- GSD doesn't use AI tools |
| `src/app/cli/provider.rs` | AiProvider trait unchanged |
| `src/app/cli/tools.rs` | Tool declarations unchanged |
| `src/app/cli/state.rs` | AiState unchanged |
| `src/app/ui/background.rs` | Stream processing unchanged (gsd_injected_context consumed in logic.rs) |
| `src/app/ui/terminal/ai_chat/render.rs` | Conversation rendering handles GSD output as normal markdown |
| `src/app/ui/terminal/right/mod.rs` | Claude panel unchanged |

## Build Order (Dependency-Aware)

**Phase 1: Slash Infrastructure (foundation, no GSD yet)**
1. `SlashResult` enum + `SlashHandler` trait + `CommandRegistry` -- standalone types
2. `dispatch.rs` -- "/" prefix parsing, HashMap lookup
3. Interception in `logic.rs::send_query_to_agent()` -- 25 lines at function top
4. `WorkspaceState` additions -- 2 `Option<>` fields + init

**Phase 2: Built-in Commands (validates infrastructure)**
5. `/clear` -- simplest handler, clears `ws.ai.chat.conversation`
6. `/help` -- iterates registry, builds help text
7. `/new` -- resets conversation (like existing `AiChatAction::NewQuery`)
8. `/model` -- displays/switches model via `ws.ai.ollama`
9. `/git`, `/build`, `/settings` -- workspace state inspection

**Phase 3: GSD Core (standalone, no slash integration yet)**
10. `gsd/core.rs` -- slug generation, timestamps, path utilities
11. `gsd/frontmatter.rs` -- `---` delimited key-value parsing
12. `gsd/config.rs` -- `.planning/config.json` read/write
13. `gsd/state.rs` -- STATE.md parse/update (depends on 11)

**Phase 4: GSD Operations (depends on Phase 3)**
14. `gsd/phase.rs` -- add, insert, remove, complete (depends on 11, 12, 13)
15. `gsd/roadmap.rs` -- ROADMAP.md parse/update (depends on 11)
16. `gsd/verify.rs` -- plan structure, completeness checks (depends on 11, 13)
17. `gsd/template.rs` -- template fill (depends on 11, 12)
18. `gsd/milestone.rs` -- milestone archival (depends on 13, 14, 15)

**Phase 5: GSD Slash Integration (wires everything together)**
19. `gsd/mod.rs` -- `GsdCommand` implementing `SlashHandler`, routes to subcommands
20. Register `GsdCommand` in `CommandRegistry::new()`

**Phase 6: AI Delegation (final integration)**
21. `SlashResult::DelegateToAi` handling in dispatcher
22. `gsd_injected_context` consumption in `send_query_to_agent()`
23. AI-integrated GSD commands: `/gsd research`, `/gsd plan`, `/gsd commit-msg`

**Phase 7: Polish**
24. i18n keys for all slash command messages (5 locales)
25. "/" prefix autocomplete hints in chat prompt UI (optional)
26. MIT attribution for GSD in About dialog

## Scalability Considerations

| Concern | v1.2.1 (initial) | v2.0+ (future) |
|---------|-------------------|----------------|
| Command count | ~15 (8 builtin + 7 GSD subcommands) | Registry pattern scales without code changes |
| GSD file parsing | Sync read per invocation | Add mtime-based cache only if profiling shows need |
| AI delegation | Ollama only | AiProvider trait already supports multiple providers |
| Autocomplete | Static list from registry.keys() | Could add fuzzy matching, history-based suggestions |
| GSD operations | Single project | GSD already scopes everything to ws.root_path |

## Sources

- Direct code analysis: `src/app/cli/` (mod.rs, types.rs, executor.rs, provider.rs, tools.rs, state.rs) -- HIGH confidence
- Direct code analysis: `src/app/ui/terminal/ai_chat/logic.rs` -- send_query_to_agent() entry point -- HIGH confidence
- Direct code analysis: `src/app/ui/background.rs` -- stream event processing, tool call cycle -- HIGH confidence
- Direct code analysis: `src/app/ui/workspace/state/mod.rs` -- WorkspaceState structure, 143 lines -- HIGH confidence
- GSD Node.js source: `~/.claude/get-shit-done/bin/gsd-tools.cjs` (592 lines) + 11 lib modules in `bin/lib/` -- HIGH confidence
- `.planning/PROJECT.md` -- key decisions, constraints, tech debt -- HIGH confidence
