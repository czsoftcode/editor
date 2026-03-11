---
phase: 33-odstranit-veskerou-zminku-a-funkce-polycredo-cli-ze-systemu
plan: 01
subsystem: ui
tags: [launcher-only, terminal, cleanup, hard-removal]
requires:
  - phase: 32-cleanup-tests-and-stabilization
    provides: AI terminal baseline before hard removal
provides:
  - hard removal src/app/ai_core/*
  - hard removal src/app/ui/terminal/ai_chat/*
  - launcher-only tok ai_bar -> terminal.send_command
affects: [phase-33-plan-02, phase-33-plan-03, phase-33-plan-04]
tech-stack:
  added: []
  patterns: [remove-only cleanup, launcher-only terminal path]
key-files:
  created: [src/app/ai_prefs.rs, tests/phase33_removal_checks.sh]
  modified: [src/app/mod.rs, src/app/ui/workspace/mod.rs, src/app/ui/background.rs, src/app/ui/terminal/right/mod.rs]
key-decisions:
  - "AiExpertiseRole/AiReasoningDepth byly zachovany mimo ai_core v novem src/app/ai_prefs.rs kvuli kompatibilite settings."
  - "Legacy historicke testy zavisle na ai_core/ai_chat byly prepnuty na assert removal stavu, aby quality gate reflektoval phase 33."
patterns-established:
  - "Launcher-only AI tok: jediny povoleny vstup je ai_bar -> terminal.send_command."
requirements-completed: [R33-A, R33-B, R33-C]
duration: 9min
completed: 2026-03-11
---

# Phase 33 Plan 01: hard removal ai_core + ai_chat Summary

**Aplikace byla prepnuta na launcher-only rezim, kde integrovany AI runtime/chat subsystem byl tvrde odstranen a zustal jen ai_bar tok do aktivniho terminal tabu.**

## Performance

- **Duration:** 9 min
- **Started:** 2026-03-11T19:01:42Z
- **Completed:** 2026-03-11T19:11:19Z
- **Tasks:** 3
- **Files modified:** 51

## Accomplishments
- Fyzicky smazany `src/app/ai_core/*` a `src/app/ui/terminal/ai_chat/*` vcetne modulovych deklaraci.
- Odstranene runtime/chat vstupy (`show_ai_chat`, `tool_executor`, `FocusedPanel::AiChat`) bez fallback UX vrstvy.
- Build gates po remove-only upravach jsou zeleny (`cargo check`, `./check.sh`) a grep guardy pro phase 33 prochazi.

## Task Commits

1. **Task 1: Smazat ai_core a ai_chat moduly vcetne mod declarations**
   - `e3b681a` (test): RED guard pro hard-removal modulu
   - `5a2081a` (feat): hard-removal modulovych stromu + compile-fix migrace stavu
2. **Task 2: Odstranit runtime/chat stav a trigger body mimo ai_bar**
   - `849c2c6` (test): RED guard na `ws.ai` runtime reference
   - `8837015` (feat): odstraneni posledni runtime reference pro launcher-only stav
3. **Task 3: Compile gate po hard removal**
   - `874ad36` (fix): compile + quality gate fixy a aktualizace historickych testu na removal kontrakt

## Files Created/Modified
- `src/app/ai_prefs.rs` - lehky modul s enumy/panelem potrebnymi pro settings/UI bez runtime chatu.
- `src/app/ui/background.rs` - odstranene chat streaming/approval/slash flow vetve, ponechane watchery/git/autosave.
- `src/app/ui/workspace/state/mod.rs` - odstraneny chat runtime fields a `show_ai_chat` stav.
- `src/app/ui/workspace/state/init.rs` - init prepnuty na `ai_panel.font_scale` bez `AiState`.
- `src/app/ui/terminal/right/mod.rs` - panel pocita font scale z `ai_panel`.
- `tests/phase33_removal_checks.sh` - phase 33 guard script pro task1/task2 forbidden patterns.

## Decisions Made
- Enumy `AiExpertiseRole`/`AiReasoningDepth` zustaly zachovane kvuli serializaci settings, ale byly oddelene od odstraneneho `ai_core`.
- `background.rs` byl zjednodusen na remove-only variantu bez chat runtime pollingu, approval flow a slash stale guardu.
- Historicke testy phase30/phase32 byly upraveny na assert odstranenych souboru, protoze puvodni kontrakt "ai_core exists" je po phase33 neplatny.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `cargo check` blokoval `sccache` wrapper**
- **Found during:** Task 3
- **Issue:** `cargo check` padal na `sccache: Operation not permitted`.
- **Fix:** Spusteny gates s `RUSTC_WRAPPER=` pro realny compile check.
- **Verification:** `RUSTC_WRAPPER= cargo check` PASS.
- **Committed in:** `874ad36`

**2. [Rule 3 - Blocking] Historicke testy ctily uz smazane soubory**
- **Found during:** Task 3
- **Issue:** `./check.sh` fail na testech, ktere nacitaly `src/app/ai_core/*` a `src/app/ui/terminal/ai_chat/*`.
- **Fix:** Testy prepnuty na phase33 kontrakt (assert ze soubory neexistuji + stale no-legacy namespace guard).
- **Files modified:** `tests/phase30_plan01_namespace_bootstrap.rs`, `tests/phase30_plan02_readiness_gate.rs`, `tests/phase30_plan04_ai_terminal_imports.rs`, `tests/phase30_plan04_ollama_ui_removal.rs`, `tests/phase32_namespace_guard.rs`, `tests/phase32_runtime_stability.rs`
- **Verification:** `RUSTC_WRAPPER= ./check.sh` PASS.
- **Committed in:** `874ad36`

---

**Total deviations:** 2 auto-fixed (2x Rule 3 - Blocking)  
**Impact on plan:** Bez scope creep; fixy byly nutne pro splneni compile/quality gate po hard-removal.

## Issues Encountered
- Lokalne blokovany `sccache` wrapper pro Rust toolchain.
- Historicke testy navazane na pred-phase33 architekturu vyzadovaly aktualizaci ocekavaneho stavu.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Plan 01 je v launcher-only rezimu stabilni a pripraveny pro navazny i18n/planning cleanup.
- Forbidden-pattern guardy a quality gate jsou pripraveny jako baseline pro dalsi plany phase 33.

## Self-Check: PASSED

---
*Phase: 33-odstranit-veskerou-zminku-a-funkce-polycredo-cli-ze-systemu*
*Completed: 2026-03-11*
