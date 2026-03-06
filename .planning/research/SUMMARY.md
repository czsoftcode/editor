# Project Research Summary

**Project:** PolyCredo Editor v1.2.0 — AI Chat Rewrite
**Domain:** Native AI Chat integration for Rust/egui desktop code editor
**Researched:** 2026-03-06
**Confidence:** HIGH

## Executive Summary

PolyCredo Editor v1.2.0 replaces the current WASM-based AI chat system (extism plugins) with a native Rust implementation featuring streaming responses, a provider trait abstraction, and direct tool execution. The project is well-positioned for this rewrite: approximately 60% of the required functionality already exists (context engine, approval UI, conversation history, tool declarations, cancel mechanism, inspector panel). The core new work is an `AiProvider` trait, an `OllamaProvider` implementation with NDJSON streaming, and a `NativeToolExecutor` that extracts logic from the current WASM host functions. The estimated net effect is a smaller, simpler codebase (~1000 LOC new, ~1300 LOC reused, ~2000+ LOC removed).

The recommended approach is blocking HTTP via `ureq` on a background `std::thread`, communicating with the UI thread through the existing `mpsc` + `AppAction` channel pattern. This aligns with the codebase's established philosophy of no async runtime for application logic. **Important conflict note:** STACK.md recommends `reqwest` with tokio async, while ARCHITECTURE.md recommends `ureq` with std::thread. The architecture recommendation is correct -- the project uses `std::thread::spawn` + `mpsc` everywhere, and introducing async HTTP would create a conflicting execution model. tokio exists in the project but is used narrowly (terminal, process I/O), not as the application's threading backbone. Use `ureq` for provider HTTP calls.

The primary risks are: (1) blocking the egui UI thread during streaming, (2) approval state machine race conditions when multiple tool calls queue up, (3) context payload size explosion consuming model context windows, and (4) breaking existing functionality during the WASM-to-native migration. All are preventable with the patterns documented in ARCHITECTURE.md and PITFALLS.md.

## Key Findings

### Recommended Stack

The stack change is minimal -- only 1-2 new dependencies needed. The project already has everything else.

**New dependencies:**
- `ureq`: Blocking HTTP client for Ollama API calls with streaming via `BufReader` -- consistent with std::thread model, no async needed

**Existing dependencies (no changes):**
- `tokio 1` (rt-multi-thread): Already in project for terminal/process I/O -- NOT for AI HTTP
- `egui_commonmark 0.20`: Markdown rendering for chat responses -- already working
- `serde/serde_json 1`: Ollama NDJSON parsing
- `fluent-bundle 0.15`: i18n for new UI strings

**What to remove later:** `extism 1.5` (WASM runtime), potentially `candle-*` / `hf-hub` / `tokenizers` if semantic embeddings move to Ollama `/api/embed`.

**Stack conflict resolution:** STACK.md recommends `reqwest 0.12` with tokio channels. ARCHITECTURE.md recommends `ureq` with std::thread. **Go with ureq + std::thread.** Rationale: the entire codebase uses blocking threads and std::mpsc channels. Adding reqwest async would require bridging async/sync boundaries unnecessarily. ureq streams via `Read` trait on the response body, which works naturally on a background thread.

### Expected Features

**Must have (table stakes):**
- Streaming responses (real-time token display)
- Editor context (open file, git status, build errors) -- already implemented
- File read/write/replace tools with approval UI -- port from WASM
- Command execution tool with approval -- port from WASM
- Conversational history (multi-turn) -- partially exists, needs refactor to `AiConversation`
- Cancel/stop button -- already implemented
- Markdown rendering in responses -- already implemented
- Dark/light mode support -- currently hardcoded dark colors, needs fix

**Should have (differentiators):**
- Expertise role system (Junior/Senior/Master) -- already implemented, port
- Reasoning depth control -- already implemented, port
- Semantic search tool -- already implemented, port
- Multi-provider trait abstraction -- new, key architectural piece
- Ollama auto-detection and model picker -- new, simple HTTP calls
- Inspector panel (debug view) -- already implemented, preserve

**Defer (v2+):**
- Claude/Gemini/OpenAI providers (v1.3+)
- WASM system full removal (after native chat validated)
- Multimodal input
- Inline code suggestions (ghost text)

### Architecture Approach

The architecture is a clean replacement of WASM indirection with native Rust. The `AiProvider` trait replaces `PluginManager.call()`. A `NativeToolExecutor` replaces WASM host functions. The communication pattern (background thread -> `AppAction` via mpsc -> UI thread processes in `update()`) remains identical. A new `AiChatState` sub-struct consolidates ~30 scattered `ai_*` fields in `WorkspaceState`.

**Major components:**
1. `AiProvider` trait + `ProviderRegistry` -- provider abstraction with streaming support
2. `OllamaProvider` -- Ollama `/api/chat` implementation with NDJSON streaming via ureq
3. `NativeToolExecutor` -- extracted tool execution logic (file ops, search, exec, memory)
4. `AiChatState` -- consolidated AI state sub-struct in WorkspaceState
5. `send_query_to_provider()` -- replaces `send_query_to_agent()`, spawns background thread

### Critical Pitfalls

1. **UI thread blocking** -- Never call blocking HTTP inside `update()`. Use `std::thread::spawn` + `mpsc::channel` + `try_recv()` drain loop. Call `request_repaint_after(50ms)` not immediate repaint to avoid CPU storms.
2. **Approval state machine races** -- Queue approvals with `VecDeque` instead of `Option`. Handle sender drop as "Deny". Force AI chat visible when approval is pending. Add 5-minute timeout.
3. **Context payload explosion** -- Truncate active file to +/- 100 lines around cursor. Implement conversation pruning (keep last N messages). Add token budget system.
4. **WASM migration breakage** -- Build new system alongside old. Map new provider output to existing `AppAction` variants initially. Keep extism until final removal phase.
5. **NDJSON parser fragility** -- Buffer incomplete lines across TCP chunks. Handle non-streamed tool call responses. Test with simulated partial JSON.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Provider Foundation
**Rationale:** Everything depends on the provider trait and Ollama implementation. This is the foundational layer with zero UI changes, so it cannot break anything.
**Delivers:** Working `AiProvider` trait, `OllamaProvider` with NDJSON streaming, `ProviderRegistry`, new `AppAction` variants, Ollama connection health check.
**Features:** Provider trait abstraction, Ollama streaming, Ollama auto-detection.
**Avoids:** Over-engineering the trait (Pitfall 6) -- start with 2-3 methods max. NDJSON parser fragility (Pitfall 7) -- buffer partial reads.
**Stack:** `ureq` (new), `serde_json` (existing).

### Phase 2: State Refactor + Data Model
**Rationale:** Mechanical refactoring that must happen before wiring the new provider to UI. No functional changes -- existing WASM chat continues to work.
**Delivers:** `AiChatState` sub-struct, `ws.ai_*` -> `ws.ai.*` rename across codebase, conversation history migrated to `AiConversation` format.
**Features:** Conversational history restructuring.
**Avoids:** History format mismatch (Pitfall 8) -- switch to `AiMessage` format from the start.

### Phase 3: Streaming UI + Provider Bridge
**Rationale:** Connects the provider to the UI. This is where streaming becomes visible to users. Depends on Phase 1 (provider) and Phase 2 (state structure).
**Delivers:** `send_query_to_provider()`, streaming buffer display in chat, provider picker in settings, dark/light mode fix for chat colors.
**Features:** Streaming responses, dark/light mode, provider selector, model picker.
**Avoids:** UI thread blocking (Pitfall 1) -- background thread + try_recv. Repaint storm (Pitfall 5) -- use `request_repaint_after(50ms)`.

### Phase 4: Native Tool Execution
**Rationale:** Tool execution is the complex agentic layer. Depends on working streaming (Phase 3) and requires careful security handling.
**Delivers:** `NativeToolExecutor` with all tools (read/write/replace/exec/search/semantic_search/memory), approval queue, multi-turn tool loop.
**Features:** File read/write tools, command execution, semantic search, agent memory, ask-user, auto-approve.
**Avoids:** Approval race conditions (Pitfall 2) -- VecDeque queue + state machine. Security regression (Pitfall 9) -- port Blacklist, path validation.

### Phase 5: Polish + i18n
**Rationale:** Cleanup and localization after core functionality works.
**Delivers:** i18n keys for all new strings (5 languages), context payload truncation/budgeting, conversation virtualization for long chats, expertise role + reasoning depth ported.
**Features:** i18n, context optimization, markdown performance, expertise roles.
**Avoids:** Hardcoded strings (Pitfall 12). Context explosion (Pitfall 3). Markdown perf (Pitfall 10).

### Phase 6: WASM Removal
**Rationale:** Only after native chat is fully validated. Separate phase to isolate risk.
**Delivers:** Removal of `extism` dependency, deletion of WASM plugin code (~2000 LOC), cleanup of old `AppAction::Plugin*` variants.
**Features:** Codebase simplification.
**Avoids:** Migration breakage (Pitfall 4) -- only remove after full validation.

### Phase Ordering Rationale

- Phases 1-3 follow a strict dependency chain: trait -> state -> UI bridge
- Phase 4 (tools) is the most complex and benefits from a working streaming foundation
- Phase 5 (polish) is intentionally separate from functional phases to avoid scope creep
- Phase 6 (WASM removal) is last because both systems must coexist during the transition
- The entire milestone keeps the old WASM path functional until Phase 6

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 1:** Verify `ureq` streaming API in current version (docs.rs). Resolve the reqwest vs ureq decision definitively by testing both with Ollama locally.
- **Phase 4:** Tool execution security model needs detailed review of existing WASM host function logic (`src/app/registry/plugins/host/`). The approval state machine redesign needs careful planning.

Phases with standard patterns (skip research-phase):
- **Phase 2:** Mechanical refactoring -- no research needed, just careful renaming.
- **Phase 3:** Standard egui patterns (thread + channel + repaint). Well-documented in egui community.
- **Phase 5:** Standard i18n and optimization work.
- **Phase 6:** Deletion of code -- straightforward once Phase 4 is validated.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Minimal additions (1-2 crates). All alternatives evaluated. Only conflict is reqwest vs ureq -- resolved in favor of ureq. |
| Features | HIGH | Based on direct codebase analysis. ~60% already implemented, clear reuse paths documented. |
| Architecture | HIGH | Direct code inspection of all integration points. Data flow patterns verified against existing codebase. |
| Pitfalls | HIGH | Grounded in specific code locations. Real Ollama issues referenced (GitHub #12557). |

**Overall confidence:** HIGH

### Gaps to Address

- **ureq vs reqwest:** STACK.md and ARCHITECTURE.md disagree. Recommendation is ureq, but verify ureq streaming works with Ollama NDJSON in a quick spike before Phase 1 planning.
- **candle/tokenizers removal:** Unclear if semantic embeddings can move to Ollama `/api/embed`. Investigate during Phase 5 or defer to v1.3.
- **Multi-viewport provider isolation:** PITFALLS.md flags potential state conflicts between viewports. Needs architectural decision in Phase 1 -- recommend per-workspace provider instances.
- **Ollama tool calling streaming inconsistencies:** GitHub issue #12557 documents that tool_calls may not stream even with `stream: true`. The NDJSON parser must handle both chunked and single-response tool calls.

## Sources

### Primary (HIGH confidence)
- Project source code: `src/app/ai/`, `src/app/registry/plugins/`, `src/app/ui/terminal/ai_chat/`, `src/app/types.rs`, `src/app/mod.rs`
- [Ollama API Documentation](https://github.com/ollama/ollama/blob/main/docs/api.md)
- [Ollama Streaming Tool Calls](https://ollama.com/blog/streaming-tool)
- [egui threading discussions](https://github.com/emilk/egui/discussions/521)

### Secondary (MEDIUM confidence)
- [reqwest crate docs](https://docs.rs/reqwest/latest/reqwest/)
- [ureq crate docs](https://docs.rs/ureq/latest/ureq/)
- [Ollama tool calling issue #12557](https://github.com/ollama/ollama/issues/12557)
- [genai crate](https://github.com/jeremychone/rust-genai) -- multi-provider abstraction reference
- [Cursor vs Windsurf approval patterns](https://www.builder.io/blog/windsurf-vs-cursor)

### Tertiary (LOW confidence)
- [LLM tool calling in production](https://medium.com/@komalbaparmar007/llm-tool-calling-in-production-rate-limits-retries-and-the-infinite-loop-failure-mode-you-must-2a1e2a1e84c8) -- infinite loop failure mode, needs validation

---
*Research completed: 2026-03-06*
*Ready for roadmap: yes*
