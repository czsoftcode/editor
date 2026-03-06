---
phase: 17-i18n-wasm-cleanup
plan: 01
subsystem: i18n, ui, ai
tags: [fluent, i18n, ollama, eframe, egui, localization]

# Dependency graph
requires:
  - phase: 15-streaming-chat-ui
    provides: AI chat terminal UI with hardcoded strings
  - phase: 16-tool-use
    provides: Tool approval UI with hardcoded Czech strings
provides:
  - cli.ftl locale files with all CLI/AI chat i18n keys in 5 languages
  - Ollama generation parameters (top_p, top_k, repeat_penalty, seed) in Settings and UI
  - Renamed i18n namespace from ai-chat-*/ai-plugin-bar-* to cli-chat-*/cli-bar-*
affects: [17-02, future-ai-providers]

# Tech tracking
tech-stack:
  added: []
  patterns: [cli-* i18n key namespace for all AI chat strings, build_options helper for Ollama params DRY]

key-files:
  created:
    - locales/en/cli.ftl
    - locales/cs/cli.ftl
    - locales/de/cli.ftl
    - locales/ru/cli.ftl
    - locales/sk/cli.ftl
  modified:
    - src/app/ui/terminal/ai_chat/approval.rs
    - src/app/ui/terminal/ai_chat/render.rs
    - src/app/ui/terminal/ai_chat/inspector.rs
    - src/app/ui/widgets/ai/chat/conversation.rs
    - src/app/ai/provider.rs
    - src/app/ai/ollama.rs
    - src/app/ai/state.rs
    - src/settings.rs
    - src/app/ui/workspace/modal_dialogs/settings.rs
    - src/app/ui/background.rs
    - src/i18n.rs

key-decisions:
  - "Renamed ai-chat-*/ai-plugin-bar-* keys to cli-chat-*/cli-bar-* for consistency with CLI subsystem naming"
  - "Created build_options() helper in ollama.rs to DRY up 4 inline options JSON blocks"
  - "Seed=0 means random (omitted from Ollama request), non-zero seeds are passed through"

patterns-established:
  - "cli-* prefix: All AI chat and CLI-related i18n keys use cli-* namespace"
  - "i18n threading: Approval/inspector functions accept i18n: &I18n parameter for localization"

requirements-completed: [CLEN-03]

# Metrics
duration: 15min
completed: 2026-03-06
---

# Phase 17 Plan 01: i18n + Ollama Params Summary

**CLI i18n keys (cli-chat-*, cli-bar-*, cli-tool-*, cli-settings-*) in 5 languages, hardcoded string elimination in approval/inspector/conversation, Ollama generation parameters (top_p, top_k, repeat_penalty, seed) in Settings UI**

## Performance

- **Duration:** 15 min
- **Started:** 2026-03-06T08:49:12Z
- **Completed:** 2026-03-06T09:04:11Z
- **Tasks:** 2
- **Files modified:** 33

## Accomplishments
- Created cli.ftl locale files in all 5 languages (cs, en, de, ru, sk) with 30+ keys covering chat UI, plugin bar, tool approval, chat buttons, and settings
- Replaced all hardcoded Czech strings in approval.rs and English strings in inspector.rs/conversation.rs/render.rs with i18n calls using tr! macro
- Renamed ai-chat-*/ai-plugin-bar-* keys to cli-chat-*/cli-bar-* across entire codebase
- Deleted unused ai.ftl files (5 languages)
- Added Ollama generation parameters (top_p, top_k, repeat_penalty, seed) to ProviderConfig, Settings, AiSettings, and Ollama API requests
- Added Settings UI controls: sliders for Top-P, Top-K, Repeat Penalty and text input for Seed
- Renamed AI settings section to "PolyCredo CLI" using i18n key

## Task Commits

Each task was committed atomically:

1. **Task 1: i18n key creation, renaming, hardcoded string replacement** - `1682ba1` (feat)
2. **Task 2: Ollama generation params in ProviderConfig, Settings, UI** - `bffc848` (feat)

## Files Created/Modified
- `locales/{cs,en,de,ru,sk}/cli.ftl` - New CLI i18n locale files with all AI chat keys
- `locales/{cs,en,de,ru,sk}/ai.ftl` - Deleted (unused gemini keys)
- `locales/{cs,en,de,ru,sk}/ui.ftl` - Removed migrated ai-chat-* and ai-plugin-bar-* keys
- `src/i18n.rs` - Switched ai.ftl include to cli.ftl
- `src/app/ui/terminal/ai_chat/approval.rs` - Replaced hardcoded Czech with i18n calls
- `src/app/ui/terminal/ai_chat/render.rs` - Renamed i18n keys, added i18n pass-through
- `src/app/ui/terminal/ai_chat/inspector.rs` - Replaced hardcoded English with i18n calls
- `src/app/ui/widgets/ai/chat/conversation.rs` - Replaced hardcoded "Copy" with i18n
- `src/app/ui/widgets/ai/chat/settings.rs` - Renamed ai-chat-* to cli-chat-*
- `src/app/ui/panels.rs` - Renamed ai-plugin-bar-* to cli-bar-*
- `src/app/ai/provider.rs` - Added top_p, top_k, repeat_penalty, seed fields
- `src/app/ai/ollama.rs` - Added build_options() helper, used in all 4 request paths
- `src/app/ai/state.rs` - Added generation param fields to AiSettings
- `src/settings.rs` - Added ollama_top_p/top_k/repeat_penalty/seed with serde(default)
- `src/app/ui/background.rs` - Sync new params from Settings to AiSettings and ProviderConfig
- `src/app/ui/terminal/ai_chat/logic.rs` - Updated ProviderConfig construction
- `src/app/ui/workspace/modal_dialogs/settings.rs` - Added generation param UI controls
- `src/app/ui/workspace/state/init.rs` - Added new AiSettings field initialization

## Decisions Made
- Renamed ai-chat-*/ai-plugin-bar-* keys to cli-chat-*/cli-bar-* for consistent CLI namespace
- Created build_options() helper to DRY up 4 instances of inline Ollama options JSON
- Seed value of 0 means "random" and is omitted from Ollama API request; non-zero values are included
- Temperature and num_ctx already existed in AiSettings; new params follow same pattern

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed missing AiSettings fields in init.rs**
- **Found during:** Task 2 (Ollama params)
- **Issue:** After adding new fields to AiSettings struct, the construction site in init.rs was missing them, causing compilation failure
- **Fix:** Added temperature, num_ctx, top_p, top_k, repeat_penalty, seed initialization from Settings values
- **Files modified:** src/app/ui/workspace/state/init.rs
- **Verification:** cargo check passes
- **Committed in:** bffc848 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Essential fix for compilation. No scope creep.

## Issues Encountered
- ai-plugin-bar-settings-hover replacement was already handled by replace_all on ai-plugin-bar-settings prefix - verified correct, no action needed

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All CLI strings are now i18n-ready across 5 languages
- Ollama generation parameters are configurable via Settings UI
- Ready for plan 17-02 (further cleanup tasks)

---
*Phase: 17-i18n-wasm-cleanup*
*Completed: 2026-03-06*

## Self-Check: PASSED
- All 5 cli.ftl locale files: FOUND
- Commit 1682ba1 (Task 1): FOUND
- Commit bffc848 (Task 2): FOUND
