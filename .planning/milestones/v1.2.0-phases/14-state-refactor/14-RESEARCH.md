# Phase 14: State Refactor - Research

**Researched:** 2026-03-06
**Domain:** Rust struct refactoring, borrow checker patterns, module organization
**Confidence:** HIGH

## Summary

Phase 14 is a pure refactoring phase: extracting ~25 AI-related fields from `WorkspaceState` into a dedicated `AiState` sub-struct with three nested sub-structs (`ChatState`, `OllamaState`, `AiSettings`). The codebase already uses this sub-struct pattern (e.g., `ws.editor`, `ws.file_tree`, `ws.project_search`) so the approach is well-established.

The primary challenge is the volume of call sites (~156 occurrences across 16 files) that reference `ws.ai_*` and `ws.ollama_*` fields. Each occurrence must be renamed to the new nested path (`ws.ai.chat.*`, `ws.ai.ollama.*`, `ws.ai.settings.*`). The risk is low because the Rust compiler will catch every missed rename at compile time.

**Primary recommendation:** Execute as 3-4 incremental commits, each compiling cleanly. Define all new structs in `src/app/ai/state.rs`, use `#[derive(Default)]` where possible, and leverage `cargo check` after each step to ensure zero regressions.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Grouping: ai_* fields into AiState with sub-structs ChatState, OllamaState, AiSettings; specific field assignments defined
- Fields staying in WorkspaceState: claude_tabs, claude_active_tab, next_claude_tab_id, claude_float, pending_ask_user, pending_plugin_approval, ai_tool_available, ai_tool_check_rx, ai_tool_last_check, selected_agent_id, show_ai_chat, ai_viewport_open
- Access pattern: ws.ai.chat.prompt, ws.ai.ollama.status, ws.ai.settings.expertise
- Struct name: AiState with pub ai: AiState field in WorkspaceState
- Prefix stripping: ai_prompt -> prompt, ai_loading -> loading; ollama_* -> status, models etc.
- 3 sub-structs inside AiState:
  - ChatState: prompt, history, history_index, monologue, conversation, system_prompt, response, loading, focus_requested, last_payload, in_tokens, out_tokens
  - OllamaState: status, models, selected_model, check_rx, last_check, base_url, api_key
  - AiSettings: expertise, reasoning_depth, font_scale, language, selected_provider, show_settings
- Top-level in AiState: inspector_open, cancellation_token, markdown_cache
- Code location: src/app/ai/state.rs, re-export via src/app/ai/mod.rs
- OllamaConnectionStatus enum moves from workspace/state/mod.rs to src/app/ai/state.rs
- Incremental migration in 3-4 commits, each compiling without warnings
- Each step renames fields simultaneously (no intermediate step with old names)

### Claude's Discretion
- Exact field ordering inside sub-structs
- Default impl strategy for AiState and sub-structs
- Order of incremental steps (which sub-struct first)
- Handling borrow checker issues with nested sub-structs

### Deferred Ideas (OUT OF SCOPE)
None
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CLEN-01 | AiChatState sub-struct -- konsolidace ~30 ai_* poli z WorkspaceState | Full field mapping, file impact analysis, migration strategy, borrow checker patterns documented below |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust compiler | stable | Compile-time verification of all renames | Catches 100% of missed field renames |
| cargo check | - | Fast compile validation between steps | No need to run full build, ~5x faster than cargo build |

### Supporting
No new dependencies needed. This is a pure code reorganization.

## Architecture Patterns

### Recommended Structure
```
src/app/ai/
  mod.rs          -- re-exports AiState, ChatState, OllamaState, AiSettings + existing exports
  state.rs        -- NEW: AiState, ChatState, OllamaState, AiSettings, OllamaConnectionStatus
  types.rs        -- existing AI types (AiExpertiseRole, AiReasoningDepth, etc.)
  provider.rs     -- existing AiProvider trait
  ollama.rs       -- existing OllamaProvider
  tools.rs        -- existing tools
```

### Pattern 1: Sub-Struct with Default Derive
**What:** Use `#[derive(Default)]` on AiState and all sub-structs where field defaults are trivial (empty strings, false bools, 0 ints, None options).
**When to use:** When all field defaults align with Rust's Default trait behavior.
**Exceptions:** Fields needing non-default values (e.g., `focus_requested: true` in ChatState, `font_scale: 100` in AiSettings, `status: OllamaConnectionStatus::Checking` in OllamaState) require manual Default impl.

```rust
// ChatState needs manual Default because focus_requested starts as true
impl Default for ChatState {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            history: Vec::new(),
            history_index: None,
            monologue: Vec::new(),
            conversation: Vec::new(),
            system_prompt: String::new(),
            response: None,
            loading: false,
            focus_requested: true, // non-default
            last_payload: String::new(),
            in_tokens: 0,
            out_tokens: 0,
        }
    }
}

// OllamaState needs manual Default for last_check offset
impl Default for OllamaState {
    fn default() -> Self {
        Self {
            status: OllamaConnectionStatus::Checking, // already default via derive
            models: Vec::new(),
            selected_model: String::new(),
            check_rx: None,
            last_check: std::time::Instant::now(), // will be overridden in init
            base_url: String::new(),
            api_key: None,
        }
    }
}

// AiSettings needs manual Default for font_scale = 100
impl Default for AiSettings {
    fn default() -> Self {
        Self {
            expertise: AiExpertiseRole::default(),
            reasoning_depth: AiReasoningDepth::default(),
            font_scale: 100, // non-default
            language: String::new(),
            selected_provider: "gemini".to_string(), // non-default
            show_settings: false,
        }
    }
}
```

### Pattern 2: Init-time Construction
**What:** In `init_workspace()`, construct the AiState explicitly rather than relying on Default, since many fields come from PersistentState/Settings.
**Current pattern:** Each field is initialized individually in init_workspace(). After refactor, build sub-structs first, then compose AiState.

```rust
// In init_workspace:
let chat = ChatState {
    prompt: String::new(),
    conversation: vec![( /* logo */ )],
    system_prompt: panel_state.ai_system_prompt.clone().unwrap_or_else(|| ...),
    focus_requested: true,
    ..Default::default()
};
let ollama = OllamaState {
    selected_model: panel_state.ollama_selected_model.clone().unwrap_or_default(),
    last_check: std::time::Instant::now() - Duration::from_secs(OLLAMA_CHECK_INTERVAL_SECS),
    base_url: settings.plugins.get("ollama")...,
    api_key: settings.plugins.get("ollama")...,
    ..Default::default()
};
let ai_settings = AiSettings {
    font_scale: panel_state.ai_font_scale,
    selected_provider: selected_provider.clone(),
    expertise: panel_state.ai_expertise.unwrap_or_else(|| ...),
    ..Default::default()
};
// Then in WorkspaceState { ... }:
ai: AiState {
    chat,
    ollama,
    settings: ai_settings,
    inspector_open: false,
    cancellation_token: Arc::new(AtomicBool::new(false)),
    markdown_cache: egui_commonmark::CommonMarkCache::default(),
},
```

### Pattern 3: ws_to_panel_state Adaptation
**What:** Update `ws_to_panel_state()` to read from nested paths.
```rust
pub fn ws_to_panel_state(ws: &WorkspaceState) -> PersistentState {
    PersistentState {
        ai_font_scale: ws.ai.settings.font_scale,
        ai_selected_provider: Some(ws.ai.settings.selected_provider.clone()),
        ai_system_prompt: Some(ws.ai.chat.system_prompt.clone()),
        ai_language: Some(ws.ai.settings.language.clone()),
        ai_expertise: Some(ws.ai.settings.expertise),
        ai_reasoning_depth: Some(ws.ai.settings.reasoning_depth),
        ollama_selected_model: if ws.ai.ollama.selected_model.is_empty() {
            None
        } else {
            Some(ws.ai.ollama.selected_model.clone())
        },
        // ... other non-AI fields unchanged
    }
}
```

### Anti-Patterns to Avoid
- **Intermediate alias step:** Do NOT create temporary `pub fn ai_prompt(&self) -> &str` accessor methods as a transition. Rename directly -- the compiler catches everything.
- **Splitting across multiple files:** All 3 sub-structs belong in one `state.rs` file. They are tightly coupled and small.
- **Partial commit that doesn't compile:** Never commit a half-renamed state. Each commit must pass `cargo check` cleanly.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Field rename verification | Manual grep + review | `cargo check` | Compiler catches 100% of missed renames in Rust |
| Default values for structs | Manual constructor functions | `#[derive(Default)]` + manual impl where needed | Standard Rust pattern, less code |
| Re-exports | Explicit use statements everywhere | `pub use` in mod.rs | Single source of truth for imports |

## Common Pitfalls

### Pitfall 1: Borrow Checker with Nested Mutable Access
**What goes wrong:** Accessing `ws.ai.chat.prompt` and `ws.editor.tabs` simultaneously through `&mut ws` works fine because they are disjoint fields. BUT accessing `ws.ai.chat.prompt` and `ws.ai.ollama.status` through separate `&mut` borrows of `ws.ai` can cause issues if done through helper functions that take `&mut AiState`.
**Why it happens:** Rust's borrow checker operates at the field level for direct struct access, but at the reference level for function parameters.
**How to avoid:** Keep direct field access (`ws.ai.chat.prompt = ...`) rather than extracting helper methods that take `&mut AiState` during this refactor. The codebase already uses direct field access pattern throughout.
**Warning signs:** Compile errors like "cannot borrow `ws.ai` as mutable more than once".

### Pitfall 2: Missing OllamaConnectionStatus Move
**What goes wrong:** Forgetting to move `OllamaConnectionStatus` enum from `workspace/state/mod.rs` to `src/app/ai/state.rs` and updating all imports.
**How to avoid:** Move the enum in the same commit as OllamaState extraction. Update imports in: `background.rs`, `init.rs`, `ai_bar.rs`, `workspace/state/mod.rs`.

### Pitfall 3: PersistentState Field Names Unchanged
**What goes wrong:** The `PersistentState` struct fields (in `app/types.rs`) use `ai_*` prefixes for serialization. Renaming these would break existing saved state files.
**How to avoid:** Keep `PersistentState` field names as-is. Only the read/write mapping in `init_workspace()` and `ws_to_panel_state()` changes to reference nested paths.

### Pitfall 4: Conversation Init Logo Call
**What goes wrong:** The `ai_conversation` field in `init_workspace()` has a complex initialization that calls `AiManager::get_logo()` with values derived from `panel_state` and `ai_settings`. This logic must be preserved exactly in the new `ChatState` construction.
**How to avoid:** Keep this initialization code intact, just restructure it to build a `ChatState` value.

## Code Examples

### Field Mapping Reference (Complete)

```
OLD (ws.*)                    -> NEW (ws.ai.*)
-------------------------------------------
ai_prompt                    -> ai.chat.prompt
ai_history                   -> ai.chat.history
ai_history_index             -> ai.chat.history_index
ai_monologue                 -> ai.chat.monologue
ai_conversation              -> ai.chat.conversation
ai_system_prompt             -> ai.chat.system_prompt
ai_response                  -> ai.chat.response
ai_loading                   -> ai.chat.loading
ai_focus_requested           -> ai.chat.focus_requested
ai_last_payload              -> ai.chat.last_payload
ai_in_tokens                 -> ai.chat.in_tokens
ai_out_tokens                -> ai.chat.out_tokens
ollama_status                -> ai.ollama.status
ollama_models                -> ai.ollama.models
ollama_selected_model        -> ai.ollama.selected_model
ollama_check_rx              -> ai.ollama.check_rx
ollama_last_check            -> ai.ollama.last_check
ollama_base_url              -> ai.ollama.base_url
ollama_api_key               -> ai.ollama.api_key
ai_expertise                 -> ai.settings.expertise
ai_reasoning_depth           -> ai.settings.reasoning_depth
ai_font_scale                -> ai.settings.font_scale
ai_language                  -> ai.settings.language
ai_selected_provider         -> ai.settings.selected_provider
ai_show_settings             -> ai.settings.show_settings
ai_inspector_open            -> ai.inspector_open
ai_cancellation_token        -> ai.cancellation_token
markdown_cache               -> ai.markdown_cache

STAYS IN WorkspaceState (unchanged):
  claude_tabs, claude_active_tab, next_claude_tab_id, claude_float
  pending_ask_user, pending_plugin_approval
  ai_tool_available, ai_tool_check_rx, ai_tool_last_check
  selected_agent_id, show_ai_chat, ai_viewport_open
```

### Files Requiring Changes (Impact Analysis)

| File | Occurrences | Primary Fields |
|------|-------------|----------------|
| `src/app/ui/terminal/ai_chat/render.rs` | 27 | chat.*, settings.*, markdown_cache |
| `src/app/ui/background.rs` | 21 | ollama.*, ai_tool_* (stays) |
| `src/app/ui/terminal/ai_chat/mod.rs` | 19 | chat.*, inspector_open |
| `src/app/ui/terminal/ai_chat/logic.rs` | 18 | chat.*, settings.*, cancellation_token |
| `src/app/mod.rs` | 18 | chat.loading, chat.response, chat.in_tokens |
| `src/app/ui/ai_panel.rs` | 9 | ai_tool_* (stays), settings.* |
| `src/app/ui/workspace/state/actions.rs` | 8 | settings.*, chat.*, ollama.* (ws_to_panel_state) |
| `src/app/ui/terminal/ai_chat/approval.rs` | 7 | chat.conversation, chat.monologue |
| `src/app/ui/terminal/right/ai_bar.rs` | 5 | ollama.status, settings.selected_provider |
| `src/app/ui/panels.rs` | 5 | chat.focus_requested, settings.* |
| `src/app/ui/terminal/right/mod.rs` | 4 | settings.font_scale, settings.show_settings |
| `src/app/ui/terminal/ai_chat/inspector.rs` | 3 | chat.last_payload, inspector_open |
| `src/app/ui/workspace/modal_dialogs/plugins.rs` | 2 | settings.selected_provider |
| `src/app/ui/workspace/menubar/mod.rs` | 2 | chat.focus_requested |
| `src/app/ui/workspace/mod.rs` | 7 | chat.focus_requested, chat.response |
| `src/app/ui/terminal/bottom/mod.rs` | 1 | settings.font_scale |

**Total: ~156 occurrences across 16 files + struct definition + init**

### Recommended Migration Order

1. **Commit 1: Create state.rs + Extract ChatState** (largest sub-struct, 12 fields, ~70 occurrences)
   - Create `src/app/ai/state.rs` with all 4 structs defined
   - Move ChatState fields from WorkspaceState, add `pub ai: AiState`
   - Update all `ws.ai_prompt` -> `ws.ai.chat.prompt` etc.
   - Move OllamaConnectionStatus to state.rs

2. **Commit 2: Extract OllamaState** (7 fields, ~25 occurrences)
   - Move ollama_* fields into ws.ai.ollama.*
   - Update background.rs, ai_bar.rs, init.rs

3. **Commit 3: Extract AiSettings + top-level AiState fields** (6+3 fields, ~30 occurrences)
   - Move remaining ai_* fields into ws.ai.settings.* and ws.ai.*
   - Update all remaining references
   - Final cleanup, verify no warnings

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Flat fields in WorkspaceState | Sub-struct grouping (AiState) | Phase 14 | Cleaner separation, prepares for Phase 15 UI wiring |

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + cargo test |
| Config file | Cargo.toml |
| Quick run command | `cargo check` |
| Full suite command | `cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CLEN-01 | AI fields consolidated into AiState sub-struct | compilation | `cargo check 2>&1` | N/A (compiler) |
| CLEN-01 | No regression in AI chat functionality | compilation + existing tests | `cargo test` | Existing tests cover i18n, settings, plugins |
| CLEN-01 | No warnings after rename | compilation | `cargo check 2>&1 \| grep warning` | N/A (compiler) |

### Sampling Rate
- **Per task commit:** `cargo check`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full `cargo test` + zero warnings from `cargo check`

### Wave 0 Gaps
None -- this is a pure refactor verified by the Rust compiler. No new test files needed. Existing tests will catch regressions if any logic accidentally changes.

## Open Questions

1. **Should `ai_conversation` type change from `Vec<(String, String)>` to `Vec<AiMessage>`?**
   - What we know: The current type is `Vec<(String, String)>` (user, response), while `AiMessage` and `AiConversation` structs exist in types.rs
   - What's unclear: Whether Phase 15 will need this migration anyway
   - Recommendation: Do NOT change the type in this phase. Phase 14 is strictly a field move, not a type refactor. Phase 15 can address this.

2. **Should `PersistentState` fields be updated to match new naming?**
   - What we know: PersistentState uses `#[serde]` and changing field names breaks deserialization of saved state
   - Recommendation: Keep PersistentState field names as-is. Add `#[serde(rename)]` only if cosmetic alignment is desired, but it's not required.

## Sources

### Primary (HIGH confidence)
- Direct codebase analysis: `src/app/ui/workspace/state/mod.rs` (WorkspaceState definition, 160 lines)
- Direct codebase analysis: `src/app/ui/workspace/state/init.rs` (init_workspace, 220 lines)
- Direct codebase analysis: `src/app/types.rs` (PersistentState, lines 96-128)
- Direct codebase analysis: `src/app/ai/` module (existing AI types and manager)
- Grep analysis: 156 occurrences of ai_*/ollama_*/markdown_cache across 16 source files

### Secondary (MEDIUM confidence)
- Rust borrow checker behavior with nested struct field access -- well-established Rust semantics

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - pure Rust refactoring, no new dependencies
- Architecture: HIGH - sub-struct pattern already established in codebase (Editor, FileTree, ProjectSearch)
- Pitfalls: HIGH - borrow checker patterns are well-understood, field mapping is exhaustive from grep analysis

**Research date:** 2026-03-06
**Valid until:** 2026-04-06 (stable -- pure refactoring, no external dependencies)
