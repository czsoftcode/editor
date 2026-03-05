---
phase: 12-i18n-cleanup-integrity-verification
plan: 01
subsystem: i18n
tags: [fluent, i18n, locales, sandbox-removal]

# Dependency graph
requires:
  - phase: 11-file-operations-watcher-guard-removal
    provides: "Sandbox code references removed from src/"
provides:
  - "All 10 .ftl locale files free of sandbox references"
  - "i18n key parity verified across all 5 languages"
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns: []

key-files:
  created: []
  modified:
    - locales/en/ui.ftl
    - locales/cs/ui.ftl
    - locales/de/ui.ftl
    - locales/ru/ui.ftl
    - locales/sk/ui.ftl
    - locales/en/ai.ftl
    - locales/cs/ai.ftl
    - locales/de/ai.ftl
    - locales/ru/ai.ftl
    - locales/sk/ai.ftl

key-decisions:
  - "Deleted gemini-default-prompt from all ai.ftl files (code uses ai-chat-default-prompt instead)"
  - "Updated plugins-welcome-text: sandbox -> WASM in all languages"
  - "Updated conflict-* values: removed sandbox promotion references, replaced with generic disk/editor terminology"

patterns-established: []

requirements-completed: [I18N-01, I18N-02]

# Metrics
duration: 10min
completed: 2026-03-06
---

# Phase 12 Plan 01: i18n Cleanup Summary

**Removed all sandbox i18n keys and references from 10 .ftl locale files across 5 languages, verified key parity**

## Performance

- **Duration:** 10 min
- **Started:** 2026-03-05T22:54:44Z
- **Completed:** 2026-03-06T00:04:20Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments
- Deleted all dead sandbox i18n keys from ui.ftl files (panel-files-sandbox, btn-tree-sandbox, btn-build-sandbox-*, hover-build-sandbox, hover-create-deb-disabled, hover-build-menu-disabled, hover-git-disabled-sandbox, ai-sync-*, settings-sandbox-*, sandbox-sync-*, sandbox-delete-*)
- Deleted gemini-default-prompt from all 5 ai.ftl files (unused key, code uses ai-chat-default-prompt)
- Updated 4 keys with sandbox-mentioning values: plugins-welcome-text, conflict-message, conflict-hover-disk, conflict-hover-keep
- Cleaned up duplicate key blocks in de/ui.ftl and ru/ui.ftl
- Verified key parity: test all_lang_keys_match_english passes

## Task Commits

Each task was committed atomically:

1. **Task 1: Delete sandbox i18n keys and update sandbox-mentioning values** - `1568647` (feat)
2. **Task 2: Verify i18n parity** - verification only, no code changes needed

## Files Created/Modified
- `locales/en/ui.ftl` - Removed sandbox keys, updated conflict/plugins values
- `locales/cs/ui.ftl` - Removed sandbox keys, updated conflict/plugins values
- `locales/de/ui.ftl` - Full cleanup: removed sandbox keys, deduplicated, updated values
- `locales/ru/ui.ftl` - Removed sandbox keys and duplicates, updated conflict/plugins values
- `locales/sk/ui.ftl` - Removed sandbox keys, updated conflict/plugins values
- `locales/*/ai.ftl` - Deleted gemini-default-prompt from all 5 languages

## Decisions Made
- Deleted gemini-default-prompt entirely (not just updated) because code uses ai-chat-default-prompt instead
- Updated plugins-welcome-text to reference "WASM" instead of "sandbox" as the isolation mechanism
- Updated conflict dialog values to use generic "changed on disk" / "outside the editor" terminology
- Removed settings-safe-mode* keys (not referenced anywhere in src/)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed duplicate key blocks in de/ui.ftl and ru/ui.ftl**
- **Found during:** Task 1
- **Issue:** German and Russian ui.ftl files had massive duplicate blocks of English-language keys appended at the end (likely from a previous bad merge)
- **Fix:** Rewrote de/ui.ftl cleanly and removed duplicate block from ru/ui.ftl
- **Files modified:** locales/de/ui.ftl, locales/ru/ui.ftl
- **Verification:** cargo test all_lang_keys_match_english passes
- **Committed in:** 1568647

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Essential cleanup of corrupted locale files. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All locale files clean of sandbox references
- Ready for plan 12-02 (integrity verification)

---
*Phase: 12-i18n-cleanup-integrity-verification*
*Completed: 2026-03-06*

## Self-Check: PASSED
