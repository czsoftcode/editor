---
phase: 31-ai-terminal-runtime-migration
plan: 01
subsystem: ui
tags: [ai-terminal, slash, retry, runtime, safety]
requires:
  - phase: 30-cli-namespace-removal-foundation
    provides: namespace cleanup and ai_core migration baseline
provides:
  - terminal-only runtime branding in ai_core entrypoints
  - slash async generation guard for git/build updates
  - visible runtime error feedback with explicit retry action
affects: [phase-31-plan-02, safe-flow, term-flow]
tech-stack:
  added: []
  patterns: [tdd-red-green, generation-guard, explicit-retry-ux]
key-files:
  created: [.planning/phases/31-ai-terminal-runtime-migration/31-01-SUMMARY.md]
  modified:
    - src/app/ai_core/mod.rs
    - src/app/ai_core/state.rs
    - src/app/ui/terminal/ai_chat/logic.rs
    - src/app/ui/terminal/ai_chat/mod.rs
    - src/app/ui/terminal/ai_chat/render.rs
    - src/app/ui/terminal/ai_chat/slash.rs
    - src/app/ui/background.rs
    - src/app/ui/workspace/state/mod.rs
    - src/app/ui/workspace/state/init.rs
    - src/app/mod.rs
key-decisions:
  - "Retry flow je explicitni UI akce vazana na posledni validni prompt, aktivovana jen po runtime chybe."
  - "Slash async stale-guard je sjednoceny jednim helperem pro /build i /git."
patterns-established:
  - "Runtime chyby se propisuji do chatu i toastu, bez ticheho swallow."
  - "Po /clear nebo /new se async slash vysledky filtruji podle conversation generation."
requirements-completed: [TERM-01, TERM-03, SAFE-03]
duration: 5min
completed: 2026-03-11
---

# Phase 31 Plan 01: Runtime boundary cleanup Summary

**AI terminal runtime byl uzamcen na terminal-only semantiku s guardovaným slash async tokem a explicitním Retry flow pro dočasná selhání asistenta**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-11T11:22:43Z
- **Completed:** 2026-03-11T11:27:37Z
- **Tasks:** 3
- **Files modified:** 10

## Accomplishments
- Odstraněn legacy `CLI` runtime branding z `AiManager::get_logo`, aby entrypoint respektoval terminal-only boundary.
- Slash dispatch drží externí workflow a nově filtruje stale async výsledky `/build` i `/git` přes generation guard.
- Runtime/tool chyby mají viditelný feedback v chatu i toastu a chat nabízí explicitní `Retry` akci nad posledním validním promptem.

## Task Commits

Each task was committed atomically:

1. **Task 1: Runtime boundary audit a cleanup legacy CLI semantik**
2. `b6fbb61` (`test`) RED test pro terminal-only branding
3. `a79527b` (`feat`) odstranění `CLI` markeru z runtime loga
4. **Task 2: Slash dispatch policy alignment (externi slash + GSD)**
5. `b572e2e` (`test`) RED testy generation guardu
6. `23cdbd5` (`feat`) guard helper + `/git` generation tracking + stale filtering
7. **Task 3: SAFE-03 baseline pro viditelny runtime error/audit feedback**
8. `9612ca5` (`test`) RED test retry stavu v chat state
9. `02903f9` (`feat`) explicitní Retry akce a viditelný failure feedback

## Files Created/Modified
- `.planning/phases/31-ai-terminal-runtime-migration/31-01-SUMMARY.md` - Shrnutí provedení plánu 31-01.
- `src/app/ai_core/mod.rs` - Terminal-only branding v runtime logu + TDD test.
- `src/app/ui/terminal/ai_chat/slash.rs` - Helper `should_apply_async_result` a `/git` generation tracking.
- `src/app/ui/workspace/state/mod.rs` - Stav `slash_git_gen` pro stale async guard.
- `src/app/ui/background.rs` - Aplikace generation guardu + runtime error surfacing + retry dostupnost.
- `src/app/ai_core/state.rs` - Retry stav v `ChatState`.
- `src/app/ui/terminal/ai_chat/logic.rs` - `retry_last_prompt` tok a ukládání posledního validního promptu.
- `src/app/ui/terminal/ai_chat/mod.rs` - Nová akce `AiChatAction::Retry`.
- `src/app/ui/terminal/ai_chat/render.rs` - Retry tlačítko v info baru.

## Decisions Made
- Retry je UI-driven akce (`Retry`) místo implicitního auto-resend, aby zůstal zachovaný bezpečný a předvídatelný UX kontrakt.
- Jednotný generation guard helper je zdroj pravdy pro stale async slash update policy.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] sccache blokovalo build/test verifikaci**
- **Found during:** Task 1
- **Issue:** `sccache: Operation not permitted` znemožnil `cargo test`/`cargo check`.
- **Fix:** Verifikační příkazy běžely s `RUSTC_WRAPPER=`.
- **Files modified:** none
- **Verification:** `cargo test` i `cargo check` proběhly úspěšně po přepnutí wrapperu.
- **Committed in:** n/a (environment fix během execution)

**2. [Rule 3 - Blocking] Nové pole `slash_git_gen` chybělo v test inicializátorech WorkspaceState**
- **Found during:** Task 2
- **Issue:** Kompilace selhala (`missing field slash_git_gen`) ve `src/app/mod.rs` test fixtures.
- **Fix:** Doplněna inicializace `slash_git_gen: 0` ve všech affected init blocích.
- **Files modified:** src/app/mod.rs
- **Verification:** `cargo test slash -- --nocapture` PASS.
- **Committed in:** 23cdbd5

---

**Total deviations:** 2 auto-fixed (2x Rule 3 - Blocking)
**Impact on plan:** Obě odchylky byly nutné pro dokončení verifikace a neměnily scope plánu.

## Issues Encountered
- Dočasný build gate přes sccache permissions; vyřešeno neintruzivně přes env override.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Runtime boundary, slash policy a SAFE-03 retry baseline jsou připravené pro navazující TERM/SAFE hardening ve phase 31.
- Žádný aktivní blocker pro pokračování na další plán.

## Self-Check: PASSED
- FOUND: .planning/phases/31-ai-terminal-runtime-migration/31-01-SUMMARY.md
- FOUND: b6fbb61
- FOUND: a79527b
- FOUND: b572e2e
- FOUND: 23cdbd5
- FOUND: 9612ca5
- FOUND: 02903f9

---
*Phase: 31-ai-terminal-runtime-migration*
*Completed: 2026-03-11*
