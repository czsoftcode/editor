# Project Research Summary

**Project:** PolyCredo Editor v1.2.1 -- GSD Integration + Slash Commands
**Domain:** Node.js-to-Rust port of project management CLI tools into a GUI code editor
**Researched:** 2026-03-07
**Confidence:** HIGH

## Executive Summary

PolyCredo Editor v1.2.1 adds a slash command system and ports the GSD (Get Shit Done) project management toolkit from Node.js (5,421 LOC across 11 modules) into the existing Rust/egui editor. The critical finding is that **zero new dependencies are required** -- every GSD capability maps to crates already in Cargo.toml (serde_json, regex, walkdir, globset) or the Rust standard library (std::process::Command for git, std::fs for file I/O, std::time for dates). This is a pure code addition, not a dependency expansion.

The recommended approach is a layered build: first establish slash command dispatch infrastructure (intercept "/" prefix in the existing chat input flow), then build the GSD engine as a stateless processor that reads `.planning/` files on demand. The architecture avoids in-memory state caching -- files are small (<5KB each), disk I/O is negligible, and external tools may modify these files at any time. AI-integrated GSD commands delegate to the existing Ollama provider by injecting enhanced system context into the normal chat flow, requiring only ~30 lines of modification to the existing `send_query_to_agent()` function.

The primary risks are: (1) the frontmatter parser must handle JavaScript's dynamic object model without introducing `unwrap()` panics -- a custom `FrontmatterValue` enum with two-pass parsing eliminates this; (2) git operations must never block the GUI thread -- the existing `spawn_task` async pattern in background.rs must be used; (3) file-based state concurrency (file watcher storms, torn reads) requires atomic writes and watcher debouncing for `.planning/` paths. All risks have known mitigations grounded in patterns already present in the codebase.

## Key Findings

### Recommended Stack

No new crate dependencies. The entire GSD port uses the existing stack.

**Core technologies (all existing):**
- `serde_json` + `serde`: GSD config.json management, frontmatter-to-JSON conversion
- `regex`: Section extraction from markdown, field patterns in STATE.md (use `OnceLock` for static patterns)
- `walkdir`: Phase directory scanning, file discovery in `.planning/`
- `std::process::Command`: Git operations (add, commit, check-ignore, rev-parse) -- 4 commands total
- `std::time::SystemTime`: ISO date/datetime formatting (2 formats needed, no chrono)
- `ureq` + existing `AiProvider` trait: AI-integrated GSD commands via Ollama

**Explicitly rejected:** serde_yaml/serde_yml (overkill for constrained frontmatter), git2 (native C dep for 4 commands), chrono (2 date formats only), reqwest/tokio (contradicts sync threading model).

### Expected Features

**Must have (table stakes):**
- Slash command dispatch infrastructure (intercept "/" in chat, route to handlers)
- Basic commands: `/help`, `/clear`, `/new`, `/model`
- YAML frontmatter parser (custom, ~150 LOC, handles GSD's constrained format)
- `/gsd state`, `/gsd progress` -- most frequently used GSD commands
- `/gsd phase` subcommands (add, complete, list) -- core GSD workflow
- `/gsd roadmap` subcommands (get-phase, analyze) -- project navigation
- `/gsd commit` -- standardized planning doc commits
- Graceful error handling when `.planning/` is missing

**Should have (differentiators):**
- `/gsd init` compound workflow commands (context aggregation for AI)
- `/gsd verify` suite (plan structure, completeness, reference checks)
- `/gsd template fill` and `/gsd scaffold` (reduce boilerplate)
- `/gsd milestone complete` (lifecycle management)
- Tab completion for slash commands
- `/git`, `/build`, `/settings` convenience wrappers

**Defer (v2+):**
- AI-integrated init commands (AI generation on top of context aggregation)
- `/gsd frontmatter` direct CRUD (power-user feature)
- Tab completion with fuzzy matching
- Command history filtering for slash commands

### Architecture Approach

The architecture follows a clean layered design: slash commands intercept at the top of `send_query_to_agent()` in logic.rs before the AI provider is invoked. Pure commands return immediately with markdown output rendered in the existing chat UI. GSD commands delegate to a stateless `GsdEngine` that reads `.planning/` files per invocation. AI-integrated commands prepare enhanced system context and fall through to the normal Ollama flow. Only 2 fields are added to WorkspaceState (slash_registry, gsd_injected_context), and 5 existing files need minor modification.

**Major components:**
1. `SlashCommandDispatcher` + `CommandRegistry` -- parse "/" prefix, trait-based handler lookup, ~140 LOC
2. `GsdEngine` -- stateless processor porting 11 Node.js modules, ~2,000-3,500 LOC total
3. `SlashResult` enum -- typed return protocol (Handled, DelegateToAi, Error, Unknown) ensuring uniform output handling

**Module location:** `src/app/cli/slash/` (dispatch) + `src/app/cli/gsd/` (engine), both under the existing CLI subsystem.

### Critical Pitfalls

1. **Dynamic JS object model in frontmatter parser** -- JavaScript's object-to-array "promotion" pattern violates Rust borrowing rules. Use a `FrontmatterValue` enum with index-based arena, not nested references. Two-pass parsing eliminates retroactive type changes.

2. **Blocking git operations freeze GUI** -- `std::process::Command::output()` on main thread causes 1-30s freezes. All git ops MUST use existing `spawn_task` + `mpsc::channel` pattern from background.rs. Create a reusable `GitExecutor` wrapper.

3. **Slash command input conflicts** -- `/home/user/path` looks like a command. Use strict routing: "/" + registered command name + (space or end-of-string). Unknown `/word` falls through to AI, not error.

4. **File-based state concurrency** -- GSD writes trigger file watcher storms and torn reads. Use atomic writes (tempfile + rename), debounce watcher for `.planning/` with 200ms+ delay, never read state files in the UI render loop.

5. **Regex compilation cost** -- 20+ patterns in state.cjs, recompiled per call in naive port. Replace most regex with `str::starts_with` + `str::find` line-by-line parsing. Use `OnceLock<Regex>` for genuine regex needs.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Core Infrastructure + Types
**Rationale:** Everything depends on these foundational types. Errors, output format, path helpers, and the FrontmatterValue enum must exist before any command implementation.
**Delivers:** `GsdError` enum, `GsdOutput` enum, `SlashResult` enum, `SlashHandler` trait, `CommandRegistry`, `GsdPaths` helper, date/time utilities
**Addresses:** Slash command dispatch, error handling
**Avoids:** Pitfall 7 (error handling mismatch), Pitfall 13 (path handling), Pitfall 14 (output mapping)

### Phase 2: Slash Command Dispatch + Built-in Commands
**Rationale:** Validates the dispatch infrastructure with simple commands before adding GSD complexity. Delivers immediate user value.
**Delivers:** "/" interception in logic.rs, `/help`, `/clear`, `/new`, `/model`, `/git`, `/build`, `/settings`
**Addresses:** Basic commands (table stakes), chat input routing
**Avoids:** Pitfall 3 (input conflicts), Pitfall 9 (dispatch complexity)

### Phase 3: Frontmatter Parser + GSD Core
**Rationale:** The frontmatter parser is the foundation of all GSD modules. Must be built with a test-first approach to handle edge cases. GSD core (config, slugs, timestamps) provides shared utilities.
**Delivers:** `FrontmatterValue` type, frontmatter extract/reconstruct/splice, config.json management, slug generation, phase number comparator
**Addresses:** YAML frontmatter parsing (table stakes), GSD config
**Avoids:** Pitfall 1 (dynamic object model), Pitfall 6 (encoding edge cases), Pitfall 8 (JS patterns)

### Phase 4: GSD State + Progress
**Rationale:** Most frequently used GSD commands. Validates the full pipeline from slash dispatch through frontmatter parsing to markdown output in chat.
**Delivers:** `/gsd state` (snapshot, patch, advance), `/gsd progress` (visual progress report)
**Addresses:** State commands (table stakes), progress display
**Avoids:** Pitfall 2 (regex-heavy parsing), Pitfall 4 (state concurrency), Pitfall 11 (write round-trip)

### Phase 5: GSD Phase + Roadmap + Commit
**Rationale:** The operational core of GSD workflow management. Phase operations depend on state and frontmatter modules. Git commit must use async wrapper.
**Delivers:** `/gsd phase` (add, insert, remove, complete, list), `/gsd roadmap` (get-phase, analyze, update-progress), `/gsd commit`
**Addresses:** Phase management (table stakes), roadmap operations, git integration
**Avoids:** Pitfall 5 (blocking git), Pitfall 15 (directory scanning performance)

### Phase 6: GSD Verify + Template + Milestone
**Rationale:** Quality assurance and file generation layer. Depends on all prior GSD modules being stable.
**Delivers:** `/gsd verify` (6 verification subcommands), `/gsd validate`, `/gsd template fill`, `/gsd scaffold`, `/gsd milestone complete`
**Addresses:** Verification suite (differentiator), template filling, milestone lifecycle

### Phase 7: AI-Integrated GSD + Init Commands
**Rationale:** The most complex feature, aggregating all modules. Requires stable GSD engine + working AI delegation. This is the capstone phase.
**Delivers:** `/gsd init` compound commands (new-project, plan-phase, execute-phase, research), AI context injection via `gsd_injected_context`
**Addresses:** AI-integrated commands (differentiator), workflow aggregation

### Phase 8: i18n + Polish
**Rationale:** Batch all i18n keys after commands are stable. Add tab completion and UX polish.
**Delivers:** 50+ i18n keys across 5 locales, tab completion hints, command history filtering
**Addresses:** i18n (table stakes for this project), discoverability

### Phase Ordering Rationale

- **Dependency chain:** Types (P1) -> Dispatch (P2) -> Parser (P3) -> State (P4) -> Operations (P5) -> Quality (P6) -> AI (P7) -> Polish (P8). Each phase depends on the previous.
- **Risk front-loading:** The hardest problems (frontmatter parser, state concurrency, git async) are in phases 3-5. Solving them early prevents late-stage rewrites.
- **Incremental value:** After phase 2, users already have working slash commands. After phase 4, the most-used GSD commands work. Each phase is independently shippable.
- **The chat model question (Pitfall 10):** The research suggests extending `Vec<(String, String)>` to `Vec<ChatEntry>` enum. This is a breaking change. Recommendation: defer this to a v2 concern. For v1.2.1, render command output as the "assistant" side of the conversation pair with a `> /command` prefix for visual distinction. This avoids a risky refactor of the chat data model while GSD features are still being validated.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 3 (Frontmatter Parser):** The two-pass parsing approach and FrontmatterValue arena design need detailed specification. The object-to-array promotion edge case requires careful test case design from the JS source.
- **Phase 4 (State Operations):** The `writeStateMd` round-trip complexity (Pitfall 11) needs a concrete caching strategy decision. Read the full state.cjs (721 LOC) during phase planning.
- **Phase 7 (AI-Integrated GSD):** The init command context aggregation is complex (710 LOC in init.cjs). Needs research on which commands are relevant for Ollama vs Claude-specific.

Phases with standard patterns (skip research-phase):
- **Phase 1 (Types):** Standard Rust enum + trait patterns. No ambiguity.
- **Phase 2 (Built-in Commands):** Simple dispatch + existing codebase patterns. Well-documented in ARCHITECTURE.md.
- **Phase 5 (Phase/Roadmap):** Direct port of JS logic with established patterns from phases 3-4.
- **Phase 8 (i18n):** Existing i18n infrastructure, just adding keys.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Direct analysis of all 11 GSD modules + existing Cargo.toml. Zero ambiguity on dependency needs. |
| Features | HIGH | Complete mapping of 40+ JS commands to Rust equivalents. Clear table stakes vs differentiator separation. |
| Architecture | HIGH | Interception point verified in existing logic.rs. Module structure follows established codebase patterns. ~30 lines of modification to existing files. |
| Pitfalls | HIGH | Based on 58,187 LOC existing codebase analysis + 5,421 LOC GSD source. All critical pitfalls have concrete prevention strategies grounded in existing code patterns. |

**Overall confidence:** HIGH

### Gaps to Address

- **Chat model extension (Pitfall 10):** The research recommends extending the conversation model to `Vec<ChatEntry>`, but this is a breaking change. The pragmatic recommendation is to defer this. However, if command output rendering proves inadequate with the (command, output) tuple approach, this will need to be revisited mid-implementation.
- **Estimated LOC variance:** STACK.md estimates ~4,700 LOC, ARCHITECTURE.md estimates ~2,000 LOC. The discrepancy is due to different scoping (ARCHITECTURE.md counts core logic only, STACK.md includes tests and boilerplate). Actual LOC will be determined during implementation. Plan for ~3,000-4,000 LOC.
- **init.cjs command relevance:** Several init subcommands (research-phase, plan-phase, execute-phase) are designed for Claude Code's agent spawning model. Need to determine which are meaningful in an Ollama-backed editor during Phase 7 planning.
- **GSD template compatibility:** GSD templates reference Claude model tiers and agent roles. Templates may need adaptation for the Ollama context. Evaluate during Phase 6.

## Sources

### Primary (HIGH confidence)
- GSD tools source: `~/.claude/get-shit-done/bin/lib/*.cjs` (11 modules, 5,421 LOC) -- direct line-by-line analysis
- GSD entry point: `~/.claude/get-shit-done/bin/gsd-tools.cjs` (592 LOC) -- dispatch logic analysis
- PolyCredo Editor source: `src/app/cli/` (9 modules), `src/app/ui/` (terminal, background, workspace) -- direct code analysis
- Project Cargo.toml -- verified all existing dependencies

### Secondary (MEDIUM confidence)
- Rust `regex` crate compilation cost -- documented in crate docs, `OnceLock` pattern recommended
- POSIX `rename()` atomicity -- established guarantee for same-filesystem operations

### Tertiary (LOW confidence)
- Estimated Rust LOC for port -- based on module analysis, actual will vary with error handling verbosity and test coverage

---
*Research completed: 2026-03-07*
*Ready for roadmap: yes*
