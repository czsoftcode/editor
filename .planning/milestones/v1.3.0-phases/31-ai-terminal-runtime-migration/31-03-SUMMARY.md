---
phase: 31-ai-terminal-runtime-migration
plan: 03
subsystem: ai-runtime-security
tags: [approval, security, audit, tool-executor, egui]
requires:
  - phase: 31-ai-terminal-runtime-migration
    provides: Runtime stream hardening and assistant-only UI gate continuity from plans 31-01 and 31-02
provides:
  - SAFE-01 approval deny/approve output parity with explicit tool context
  - SAFE-02 enforced security guards on approved execution paths
  - SAFE-03 single-line audit evidence and explicit tool error toast visibility
affects: [phase-31-plan-04, ai-chat-runtime, safety-contract]
tech-stack:
  added: []
  patterns: [TDD red-green safety hardening, guard revalidation in approved path]
key-files:
  created: [.planning/phases/31-ai-terminal-runtime-migration/31-03-SUMMARY.md]
  modified:
    - src/app/ai_core/executor.rs
    - src/app/ai_core/audit.rs
    - src/app/ui/background.rs
    - .planning/phases/31-ai-terminal-runtime-migration/31-VALIDATION.md
key-decisions:
  - "execute_approved musí revalidovat stejné SAFE guardy jako pre-approval cesta, aby nešlo obejít policy."
  - "Audit detail payloady se sanitují na jeden řádek kvůli čitelné evidence trail."
patterns-established:
  - "Approved-path parity: blacklist/sandbox/rate limit checks se opakují i po schválení."
  - "Tool failure UX: chat error + toast + retry hint."
requirements-completed: [SAFE-01, SAFE-02, SAFE-03]
duration: 6m
completed: 2026-03-11
---

# Phase 31 Plan 03: SAFE Contract Hardening Summary

**Approval deny/approve parity byla zpevněna a approved execution cesty nově vynucují plné SAFE guardy bez bypassu, s audit stopou a viditelným tool error feedbackem.**

## Performance

- **Duration:** 6m
- **Started:** 2026-03-11T11:41:10Z
- **Completed:** 2026-03-11T11:47:15Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments
- SAFE-01: deny větev vrací kontextový error s názvem nástroje pro konzistentní resume stopu v chatu.
- SAFE-02: `execute_approved` už neobchází sandbox/blacklist/rate-limit/blocked-command guardy.
- SAFE-03: audit log sanituje multiline payloady na jeden řádek a tool chyby mají explicitní toast.

## Task Commits

Each task was committed atomically:

1. **Task 1: SAFE-01 approval parity (approve/deny/resume)**
   - `b410465` (test) add failing deny approval context test
   - `186a68b` (fix) preserve deny tool context for approval resume
2. **Task 2: SAFE-02 security enforcement bez bypassu**
   - `683c52e` (test) add failing security bypass regression tests
   - `dc3cad4` (fix) enforce security guards on approved execution paths
3. **Task 3: SAFE-03 audit + error visibility evidence**
   - `e599bba` (test) add failing multiline audit detail regression
   - `1883555` (fix) harden audit evidence and tool error visibility

## Files Created/Modified
- `.planning/phases/31-ai-terminal-runtime-migration/31-03-SUMMARY.md` - plánový souhrn a traceability
- `src/app/ai_core/executor.rs` - SAFE-01 deny output context + SAFE-02 guard parity v approved cestách + bezpečnostní testy
- `src/app/ai_core/audit.rs` - sanitace multiline detailů pro stabilní audit evidence
- `src/app/ui/background.rs` - explicitní toast feedback při tool error
- `.planning/phases/31-ai-terminal-runtime-migration/31-VALIDATION.md` - důkazní mapa SAFE-01/02/03 verifikací

## Decisions Made
- `execute_approved` není důvěryhodný boundary sám o sobě, proto musí provádět stejné guard checky jako pre-approval fáze.
- Audit musí být jednořádkový per-event, aby byl grepovatelný a forenzně čitelný bez rozpadlých řádků.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Security bypass přes execute_approved cesty**
- **Found during:** Task 2 (SAFE-02 security enforcement bez bypassu)
- **Issue:** `execute_approved` uměl obejít sandbox/blacklist/rate-limit a blocked command policy.
- **Fix:** Doplněna revalidace guardů do `execute_write_approved`, `execute_replace_approved`, `execute_exec_approved`.
- **Files modified:** `src/app/ai_core/executor.rs`
- **Verification:** `RUSTC_WRAPPER= cargo test security -- --nocapture`
- **Committed in:** `dc3cad4`

---

**Total deviations:** 1 auto-fixed (Rule 1)
**Impact on plan:** Nutná bezpečnostní korekce bez scope creep; přímo posiluje SAFE-02.

## Issues Encountered
- Lokální prostředí blokovalo `sccache` (`Operation not permitted`), verifikace běžela s `RUSTC_WRAPPER=` bez změny testovací logiky.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- SAFE kontrakt pro approval/security/audit je pokryt testy i build verifikací.
- Plan 31-04 může navázat bez uvolnění bezpečnostní politiky.

---
*Phase: 31-ai-terminal-runtime-migration*
*Completed: 2026-03-11*

## Self-Check: PASSED

- Found summary file: `.planning/phases/31-ai-terminal-runtime-migration/31-03-SUMMARY.md`
- Found task commits: `b410465`, `186a68b`, `683c52e`, `dc3cad4`, `e599bba`, `1883555`
