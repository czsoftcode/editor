---
phase: 30-cli-namespace-removal-foundation
plan: 04
subsystem: ui
tags: [rust, ai-terminal, namespace-migration, tdd, cli-cleanup]
requires:
  - phase: 30-01
    provides: namespace bridge `app::ai_core` for staged migration
provides:
  - "AI terminal UI/background import migration from app::cli to app::ai_core"
  - "Assistant-only terminal UI without provider model/status controls"
  - "CLI-02 audit evidence for AI terminal subset"
affects: [phase-31-runtime-migration, cli-removal, ai-terminal]
tech-stack:
  added: []
  patterns: ["TDD source-audit tests for migration gates", "assistant-only terminal UI surface"]
key-files:
  created:
    - "tests/phase30_plan04_ai_terminal_imports.rs"
    - "tests/phase30_plan04_ollama_ui_removal.rs"
    - "tests/phase30_plan04_cli02_audit.rs"
    - ".planning/phases/30-cli-namespace-removal-foundation/30-04-AUDIT.md"
  modified:
    - "src/app/ai_core/mod.rs"
    - "src/app/ui/terminal/ai_chat/mod.rs"
    - "src/app/ui/terminal/ai_chat/logic.rs"
    - "src/app/ui/terminal/ai_chat/render.rs"
    - "src/app/ui/terminal/ai_chat/slash.rs"
    - "src/app/ui/terminal/right/ai_bar.rs"
    - "src/app/ui/widgets/ai/chat/mod.rs"
    - "src/app/ui/widgets/ai/chat/settings.rs"
    - "src/app/ui/background.rs"
    - "src/app/ui/workspace/state/mod.rs"
key-decisions:
  - "AI terminal head/right bar byl uzamcen do assistant-only UX bez provider model/status prvku."
  - "CLI-02 audit scope byl dočištěn i o `ai_chat/slash.rs`, protože je součástí ověřovaného subsetu."
patterns-established:
  - "Pro phase 30 držet `app::cli` reference mimo AI terminal subset a vynucovat to grep/TDD testy."
requirements-completed: [CLI-02]
duration: 8m 6s
completed: 2026-03-11
---

# Phase 30 Plan 04: CLI Namespace Removal Foundation Summary

**AI terminal UI/runtime subset je migrovaný na `app::ai_core` a běží v assistant-only režimu bez provider model/status ovládacích prvků.**

## Performance

- **Duration:** 8m 6s
- **Started:** 2026-03-11T09:55:56Z
- **Completed:** 2026-03-11T10:04:02Z
- **Tasks:** 3
- **Files modified:** 15

## Accomplishments
- Migrovány importy AI terminal subsetu (`ai_chat`, `ai_bar`, `widgets`, `background`) z `app::cli` na `app::ai_core`.
- Odstraněny provider model/status UI prvky a zachován assistant flow (agent combobox + start akce).
- Doplněn audit artefakt `30-04-AUDIT.md` s nulovým nálezem `app::cli` i provider UI tokenů v cílovém subsetu.

## Task Commits

1. **Task 1: Import migration pass pro AI terminal UI + background**
2. `8b7afb3` (test) - RED test pro zákaz `crate::app::cli` importů v cílovém subsetu.
3. `9f9e88c` (feat) - GREEN migrace importů na `app::ai_core`.

4. **Task 2: Locked decision implementation — remove Ollama UI status/model controls**
5. `08fa638` (test) - RED test pro zákaz provider model/status tokenů v UI souborech.
6. `fd594f8` (feat) - GREEN implementace assistant-only UI (bez model/status controls).

7. **Task 3: CLI-02 evidence pro AI terminal UI subset**
8. `d65c685` (test) - RED test vyžadující `30-04-AUDIT.md` s PASS markerem.
9. `f51add5` (feat) - GREEN audit evidence + cleanup zbývajících referencí v audit scope.

## Files Created/Modified
- `src/app/ai_core/mod.rs` - rozšířen bridge o runtime provider re-export pro migraci call-site.
- `src/app/ui/terminal/ai_chat/logic.rs` - přepnutí na `ai_core`, provider access přes `WorkspaceState` helpery.
- `src/app/ui/terminal/ai_chat/render.rs` - odstranění provider model/status hlavičky, ponechán neutrální assistant header.
- `src/app/ui/terminal/right/ai_bar.rs` - odstraněny model/status controls, zachován assistant picker + start.
- `src/app/ui/terminal/ai_chat/slash.rs` - odstraněna `/model` větev a navázané testy v rámci assistant-only locku.
- `src/app/ui/background.rs` - runtime typy/executor přesměrovány na `ai_core`.
- `src/app/ui/workspace/state/mod.rs` - přidány neutrální helpery pro provider connection data.
- `tests/phase30_plan04_ai_terminal_imports.rs` - TDD gate proti `app::cli` importům v subsetu.
- `tests/phase30_plan04_ollama_ui_removal.rs` - TDD gate proti provider model/status tokenům v terminal UI.
- `tests/phase30_plan04_cli02_audit.rs` - TDD gate na audit artefakt.
- `.planning/phases/30-cli-namespace-removal-foundation/30-04-AUDIT.md` - grep evidence CLI-02 pro AI terminal subset.

## Decisions Made
- Assistant-only UX lock byl naplněn i na slash vrstvě (odebrán `/model` command), aby audit scope odpovídal plánovanému verify patternu.
- `app::cli` zůstává interně přítomné mimo tento subset, ale ověřovaný AI terminal UI/runtime scope je čistý.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Verify grep scope odhalil `app::cli` reference v `ai_chat/slash.rs` mimo původní files list**
- **Found during:** Task 3 audit
- **Issue:** `rg -n "crate::app::cli|app::cli" src/app/ui/terminal/ai_chat ...` vracel shody ve `slash.rs`.
- **Fix:** `slash.rs` migrován na `app::ai_core`, odstraněna `/model` větev a související testy pro dosažení assistant-only locku.
- **Files modified:** `src/app/ui/terminal/ai_chat/slash.rs`
- **Verification:** grep audit bez nálezů + `./check.sh` green.
- **Committed in:** `f51add5`

**2. [Rule 3 - Blocking] Clippy/build failure po cleanupu `cli/mod.rs` re-exportů**
- **Found during:** Task 3 verification (`./check.sh`)
- **Issue:** odstranění `OllamaStatus` re-exportu rozbilo `cli/state.rs` import.
- **Fix:** vrácen minimální re-export `pub use ollama::OllamaStatus;` bez návratu legacy importů v cílovém subsetu.
- **Files modified:** `src/app/cli/mod.rs`
- **Verification:** `./check.sh` green.
- **Committed in:** `f51add5`

---

**Total deviations:** 2 auto-fixed (2x Rule 3 - blocking)
**Impact on plan:** Opravy byly nutné pro splnění verify commandů a quality gate; scope plánu zůstal v AI terminal subsetu.

## Issues Encountered
- `./check.sh` nejdřív failnul na format/clippy, opraveno během Task 3 před finálním commitem.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Phase 30 plan 04 je uzavřen s čistým CLI-02 auditem pro AI terminal subset.
- Runtime migrace mimo tento subset může pokračovat v phase 31 bez dependency na `app::cli` v AI terminal UI povrchu.

---
*Phase: 30-cli-namespace-removal-foundation*
*Completed: 2026-03-11*

## Self-Check: PASSED

- SUMMARY file exists.
- All task commits referenced in this summary exist in git history.
