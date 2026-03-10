---
phase: 29-syntect-theme-mapping
plan: 01
subsystem: ui
tags: [rust, syntect, theme-mapping, egui, testing]
requires:
  - phase: 28-dark-variant-support
    provides: DarkVariant::Default a DarkVariant::Midnight v Settings
provides:
  - Explicitní mapování všech 4 light + 2 dark variant na syntect témata v Settings
  - Validace mapovaného theme name proti ThemeSet::load_defaults()
  - Bezpečný fallback na base16-ocean.dark s warning logem
  - Unit gate testy pro SYNTAX-01 a SYNTAX-02
affects: [syntax-highlighting, settings, highlighter]
tech-stack:
  added: []
  patterns: [explicitní mapovací matice v Settings, deterministický fallback kontrakt]
key-files:
  created: []
  modified: [src/settings.rs]
key-decisions:
  - "Mapování zůstává centralizované v Settings::syntect_theme_name() bez duplikace v UI/highlighteru."
  - "Fallback validace používá OnceLock + ThemeSet::load_defaults() pro neblokující opakované použití."
  - "Protože syntect defaults obsahuje pouze 3 light built-in témata, WarmTan je dočasně mapován na odlišné built-in téma a fáze má blocker na vhodný 4. light kandidát."
patterns-established:
  - "Veškeré variant -> syntect rozhodnutí patří do Settings, ne do render pipeline."
  - "Každá změna mapování musí mít gate testy syntax01_* a syntax02_*."
requirements-completed: [SYNTAX-01, SYNTAX-02]
duration: 9min
completed: 2026-03-10
---

# Phase 29 Plan 01: Syntect Theme Mapping Summary

**Deterministická mapovací matice variant na syntect témata s runtime validací a fallback kontraktem přímo v `Settings`.**

## Performance

- **Duration:** 9 min
- **Started:** 2026-03-10T22:30:00Z
- **Completed:** 2026-03-10T22:39:00Z
- **Tasks:** 3
- **Files modified:** 1

## Accomplishments
- `Settings::syntect_theme_name()` nyní používá explicitní mapování všech 6 variant (4 light + 2 dark).
- Přidána validace názvu tématu proti built-in `ThemeSet` a bezpečný fallback `base16-ocean.dark` s warning logem.
- Doplněny unit gate testy pro `SYNTAX-01` i `SYNTAX-02` (`syntax01_*`, `syntax02_*`, `syntect_theme_fallback_contract`).

## Task Commits

Each task was committed atomically:

1. **Task 1: Zafixovat explicitní mapovací matici všech variant v Settings** - `05ecc40` (feat)
2. **Task 2: Přidat validaci názvu tématu a bezpečný fallback s warning logem** - `359ec92` (fix)
3. **Task 3: Doplnit unit gate testy a uzavřít requirement coverage** - `16bea6d` (test)

## Files Created/Modified
- `src/settings.rs` - explicitní mapování variant, validace syntect tématu, fallback logika a gate testy.

## Decisions Made
- `Settings` zůstává jediným zdrojem pravdy pro mapování variant na syntect téma.
- Pro výkon a stabilitu je validace dostupnosti tématu cachovaná přes `OnceLock<ThemeSet>`.
- Verifikační příkazy z plánu používají `cargo test ... --lib`, ale projekt nemá `lib` target; ověření bylo provedeno ekvivalentně bez `--lib`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Oprava test příkazů pro crate bez `lib` targetu**
- **Found during:** Task 1
- **Issue:** `cargo test ... --lib` selhal (`no library targets found`).
- **Fix:** Ověření běželo jako `cargo test -q <test_name>` bez `--lib`.
- **Files modified:** none
- **Verification:** cílené testy `syntax01_*`, `syntax02_*`, `syntect_theme_fallback_contract` prošly.
- **Committed in:** verification-only deviation (bez změny souborů)

**2. [Rule 3 - Blocking] `sccache` permission error při běhu testů**
- **Found during:** Task 1
- **Issue:** test build padal na `sccache: Operation not permitted`.
- **Fix:** testy a build gate běžely s `RUSTC_WRAPPER=` (disable wrapper pro tuto session).
- **Files modified:** none
- **Verification:** následné `cargo test` a `cargo check` prošly.
- **Committed in:** verification-only deviation (bez změny souborů)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Bez scope creep; odchylky byly nutné pouze pro spuštění verifikace v aktuálním prostředí.

## Issues Encountered
- `./check.sh` selhal na existujícím `cargo fmt` driftu mimo scope plánu (`src/app/ui/git_status.rs`, `src/app/ui/workspace/modal_dialogs/settings.rs`); zaznamenáno v `deferred-items.md`.
- V `ThemeSet::load_defaults()` není k dispozici čtvrtý vhodný light built-in kandidát; WarmTan je mapován odlišně, ale fáze stále vyžaduje produktové rozhodnutí pro ideální 4/4 light charakter.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Implementace je připravená na navazující manuální vizuální UAT kontrolu charakteru variant.
- Otevřený blocker: potvrdit/definovat vhodný 4. light built-in syntect theme kandidát pro WarmTan bez kompromisu vizuálního charakteru.

## Self-Check: PASSED

---
*Phase: 29-syntect-theme-mapping*
*Completed: 2026-03-10*
