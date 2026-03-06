---
phase: 16-tool-execution
plan: 02
subsystem: ai
tags: [ollama, tools, streaming, ndjson, context-injection]

# Dependency graph
requires:
  - phase: 15-streaming-chat-ui
    provides: OllamaProvider with stream_chat, AiProvider trait, AiContextPayload
provides:
  - Tools-enabled OllamaProvider (sends tools param, parses tool_calls)
  - Extended AiContextPayload with terminal_output and lsp_diagnostics
  - to_system_message() context formatter
  - build_system_message() for per-message context injection
  - serialize_message() for tool call/result Ollama serialization
  - tools_to_ollama_json() helper
affects: [16-tool-execution, 16-03, 16-04]

# Tech tracking
tech-stack:
  added: []
  patterns: [non-streaming-for-tools, atomic-tool-call-id-counter, context-as-system-message]

key-files:
  created: []
  modified:
    - src/app/ai/types.rs
    - src/app/ai/mod.rs
    - src/app/ai/ollama.rs
    - src/app/ai/provider.rs
    - src/app/ui/terminal/ai_chat/logic.rs
    - src/app/ui/ai_panel.rs

key-decisions:
  - "stream: false when tools present — simpler, more reliable across Ollama models"
  - "AtomicU32 global counter for tool call IDs (tc_{name}_{counter} format)"
  - "terminal_output and lsp_diagnostics passed as params, wired to actual data in Plan 04"

patterns-established:
  - "serialize_message(): centralized message serialization handling tool/assistant/user roles"
  - "to_system_message(): structured context formatting with section omission for empty fields"

requirements-completed: [TOOL-01, TOOL-02]

# Metrics
duration: 5min
completed: 2026-03-06
---

# Phase 16 Plan 02: Ollama Tools API + Editor Context Injection Summary

**Ollama tools API integration with tool_calls NDJSON parsing, extended context payload with terminal/LSP fields, and structured system message builder**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-06T15:01:24Z
- **Completed:** 2026-03-06T15:06:53Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- AiContextPayload extended with terminal_output, lsp_diagnostics, and tool_call_arguments fields
- to_system_message() formats all context into structured markdown with +-50 line active file excerpt
- OllamaProvider sends tools in Ollama format, uses stream:false for tool calls, parses tool_calls responses
- parse_ndjson_line() detects tool_calls in streaming responses with unique atomic counter IDs
- AiProvider trait updated with tools parameter, logic.rs passes get_standard_tools()
- 16 new tests (9 for context payload, 7 for Ollama tools)

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend AiContextPayload + generate_context system message builder** - `503ed1d` (feat)
2. **Task 2: Enable Ollama tools API — send tools param + parse tool_calls** - `a17d623` (feat)

## Files Created/Modified
- `src/app/ai/types.rs` - Added terminal_output, lsp_diagnostics, tool_call_arguments fields + to_system_message() + 9 tests
- `src/app/ai/mod.rs` - Updated generate_context() params, added build_system_message()
- `src/app/ai/ollama.rs` - Tools-enabled stream_chat, tool_calls parsing, serialize_message(), 7 new tests
- `src/app/ai/provider.rs` - AiProvider trait stream_chat() now accepts Vec<AiToolDeclaration>
- `src/app/ui/terminal/ai_chat/logic.rs` - Passes get_standard_tools() to stream_chat
- `src/app/ui/ai_panel.rs` - Updated generate_context() call sites with new params

## Decisions Made
- Used stream:false when tools are present for simpler, more reliable tool_calls parsing across Ollama models
- Global AtomicU32 counter for unique tool call IDs (format: tc_{name}_{counter})
- terminal_output and lsp_diagnostics accepted as params in generate_context(), actual data wiring deferred to Plan 04

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed ai_panel.rs call sites for generate_context()**
- **Found during:** Task 1
- **Issue:** ai_panel.rs called AiManager::generate_context(ws) with only 1 arg, needed update for new 4-param signature
- **Fix:** Updated both call sites to pass (ws, shared, None, Vec::new())
- **Files modified:** src/app/ui/ai_panel.rs
- **Verification:** cargo check passes
- **Committed in:** 503ed1d (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary to maintain compilation. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Tools are declared and sent to Ollama, tool_calls are parsed from responses
- Plan 03 (tool executor) can now receive StreamEvent::ToolCall and execute tools
- Plan 04 can wire terminal_output and lsp_diagnostics to actual data sources

---
*Phase: 16-tool-execution*
*Completed: 2026-03-06*
