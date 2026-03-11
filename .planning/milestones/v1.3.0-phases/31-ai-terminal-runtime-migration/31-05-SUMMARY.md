---
phase: 31-ai-terminal-runtime-migration
plan: 05
subsystem: ui
tags: [ai-terminal, assistant-only, streaming, safety, planning]
requires:
  - phase: 31-02
    provides: terminal chat runtime flow
  - phase: 31-03
    provides: approval and security parity baseline
  - phase: 31-04
    provides: verification baseline and UAT gap diagnosis
provides:
  - source-of-truth boundary aligned to assistant-only runtime
  - ai terminal path without provider-picker coupling in target callsites
  - parity evidence for TERM/SAFE after gap closure
affects: [phase-31, phase-32-stabilization, requirements-traceability]
tech-stack:
  added: []
  patterns: [ai-state-provider-helpers, assistant-only-ai-bar, verification-first-gap-closure]
key-files:
  created:
    - .planning/phases/31-ai-terminal-runtime-migration/31-05-SUMMARY.md
  modified:
    - .planning/REQUIREMENTS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/phases/31-ai-terminal-runtime-migration/31-CONTEXT.md
    - src/app/ui/terminal/right/ai_bar.rs
    - src/app/ui/terminal/ai_chat/logic.rs
    - src/app/ui/background.rs
    - src/app/ui/workspace/state/mod.rs
    - src/app/ai_core/state.rs
    - src/app/ai_core/mod.rs
    - .planning/phases/31-ai-terminal-runtime-migration/31-VERIFICATION.md
key-decisions:
  - "Model/provider picker controls were removed from AI bar to keep assistant-only boundary explicit."
  - "Provider sync/poll and connection access were centralized behind AiState helpers to avoid direct UI/runtime coupling."
  - "SAFE approval/security contract remained unchanged and was re-verified by focused approval/security test suites."
patterns-established:
  - "Assistant-only terminal UX: AI bar exposes assistant selection + start only."
  - "UI modules read provider data through AiState helper methods, not direct nested state fields."
requirements-completed: [ARCH-01, TERM-01, TERM-02, TERM-03, SAFE-01, SAFE-02, SAFE-03]
duration: 7min
completed: 2026-03-11
---

# Phase 31 Plan 05: Gap Closure Summary

**Assistant-only AI terminal flow now runs without provider-picker coupling while preserving prompt/stream/slash behavior and SAFE approval-security parity.**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-11T12:50:39Z
- **Completed:** 2026-03-11T12:57:35Z
- **Tasks:** 3
- **Files modified:** 11

## Accomplishments
- Aligned REQUIREMENTS/ROADMAP/STATE/CONTEXT to assistant-only gap-closure boundary and explicit SAFE parity.
- Removed provider-picker UI/runtime coupling from the target AI terminal path and kept slash/GSD dispatch intact.
- Re-verified approval/security behavior and recorded TERM/SAFE evidence in `31-VERIFICATION.md`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Source-of-truth alignment pro gap closure boundary** - `6f34d59` (chore)
2. **Task 2: Odstranit UI/runtime coupling na ws.ai.ollama.* v AI terminal cestě** - `9c33c8b` (feat)
3. **Task 3: Parity check external assistant + SAFE kontraktu** - `94489ef` (fix)

## Files Created/Modified
- `.planning/REQUIREMENTS.md` - TERM-03 and SAFE boundary wording updated to assistant-only scope.
- `.planning/ROADMAP.md` - Phase 31 success criteria updated for assistant-only runtime flow.
- `.planning/STATE.md` - current focus and decisions aligned with provider-picker decoupling.
- `.planning/phases/31-ai-terminal-runtime-migration/31-CONTEXT.md` - locked boundary clarified for assistant-only scope.
- `src/app/ui/terminal/right/ai_bar.rs` - removed model picker controls from AI bar.
- `src/app/ui/terminal/ai_chat/logic.rs` - runtime provider constructor decoupled from direct provider type naming.
- `src/app/ui/background.rs` - provider sync/poll path switched to AiState helper interface.
- `src/app/ui/workspace/state/mod.rs` - provider accessors now delegate to AiState helper methods.
- `src/app/ai_core/state.rs` - added provider helper API (sync/poll/connection/display/model access).
- `src/app/ai_core/mod.rs` - added `runtime_provider::RuntimeProvider` alias.
- `.planning/phases/31-ai-terminal-runtime-migration/31-VERIFICATION.md` - added 31-05 parity evidence.

## Decisions Made
- Removed model/provider picker UI from AI bar to enforce assistant-only boundary from source-of-truth to runtime.
- Moved provider sync/poll/model-info behavior behind AiState helper methods to reduce direct coupling in UI orchestration code.
- Kept SAFE contract unchanged and used focused approval/security test commands as parity proof.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `cargo check` blocked by `sccache` permission error**
- **Found during:** Task 3 (Parity check external assistant + SAFE kontraktu)
- **Issue:** Environment default `RUSTC_WRAPPER=sccache` failed with `Operation not permitted`.
- **Fix:** Re-ran verification commands with `RUSTC_WRAPPER=` and documented it in verification artifact.
- **Files modified:** `.planning/phases/31-ai-terminal-runtime-migration/31-VERIFICATION.md`
- **Verification:** `RUSTC_WRAPPER= cargo check`, `RUSTC_WRAPPER= ./check.sh`, `RUSTC_WRAPPER= cargo test approval/security`
- **Committed in:** `94489ef`

**2. [Rule 1 - Bug] Borrow-check conflict in stream error branch after decoupling**
- **Found during:** Task 3 (Parity check external assistant + SAFE kontraktu)
- **Issue:** Immutable borrow from provider label blocked mutable updates to `streaming_buffer` and retry state.
- **Fix:** Cloned provider display label to owned `String` before mutating chat state.
- **Files modified:** `src/app/ui/background.rs`
- **Verification:** `RUSTC_WRAPPER= cargo check` PASS
- **Committed in:** `94489ef`

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** Fixes were required for successful verification and did not expand scope beyond 31-05 objectives.

## Issues Encountered
- None beyond the two auto-fixed deviations listed above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 31 gap closure artifacts are aligned and verification gate is green.
- Ready for phase 32 stabilization work with assistant-only boundary locked.

---
*Phase: 31-ai-terminal-runtime-migration*
*Completed: 2026-03-11*

## Self-Check: PASSED

- FOUND: `.planning/phases/31-ai-terminal-runtime-migration/31-05-SUMMARY.md`
- FOUND commit: `6f34d59`
- FOUND commit: `9c33c8b`
- FOUND commit: `94489ef`
