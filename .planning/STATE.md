---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: completed
last_updated: "2026-03-11T00:40:29.091Z"
last_activity: "2026-03-11 - Re-executed Phase 27 Plan 02: WarmTan picker visibility + immediate preview fix"
progress:
  total_phases: 3
  completed_phases: 3
  total_plans: 5
  completed_plans: 5
  percent: 100
---

---

## gsd_state_version: 1.0

## Current Milestone: v1.2.2 Additional Themes

**Goal:** Přidat 4. světlé téma (mezi sepia a hnědou, ne moc tmavé) a volitelně druhé dark téma.

**Target features:**
- 4. světlé téma: barva mezi sepia a hnědou, příjemná pro oči, ne moc tmavá
- (Volitelně) 2. dark téma jako varianta k existujícímu

**Status:** ✅ SHIPPED 2026-03-11

---

## Project Reference

See: .planning/PROJECT.md

**Core value:** Editor nesmí zahřívat notebook v klidovém stavu — idle CPU zátěž musí být minimální.
**Current focus:** Plánování dalšího milníku

## Current Position

Phase: Completed (27-29)
Plan: All plans complete
Status: Shipped
Last activity: 2026-03-11 - Milestone v1.2.2 Additional Themes shipped

Progress: [██████████] 100%

---

## Previous Milestone Context

### v1.2.1 Save Modes + Unsaved Changes Guard (SHIPPED 2026-03-10)

**Phase:** 24-26
**Plans:** 18 completed
**Key accomplishments:**
- Ctrl+S ukládá aktivní tab bez změny fokusu
- Auto/Manual save mode s runtime apply
- Guard dialog při zavírání neuloženého tabu
- Status bar indikace dirty stavu a save režimu

---

## Performance Metrics

**Velocity:**

- v1.0.2: 17 plans completed (5 phases)
- v1.0.6: 1 plan completed (1 phase, covered 3 planned phases)
- v1.1.0: 8 plans completed (4 phases), 15 feat/fix commits, -2,878 net lines
- v1.2.0: 19 plans completed (6 phases), 42 feat/fix commits, +3,212 net lines
- v1.2.1: 18 plans completed (3 phases)
- Total: 63 plans across 19 phases (5 milestones)

---

## Accumulated Context

### Decisions

Key decisions logged in PROJECT.md Key Decisions table.

- [v1.2.2]: 4th light theme bude přidáno jako `LightVariant::WarmTan` (mezi Sepia a Brown)
- [v1.2.2]: Dark variant přidán jako `DarkVariant::Midnight` jako druhá dark varianta
- [Phase 29-syntect-theme-mapping]: Theme mapping zůstává centralizované v Settings::syntect_theme_name().
- [Phase 29-syntect-theme-mapping]: Fallback validace používá OnceLock + ThemeSet::load_defaults().
- [Phase 29-syntect-theme-mapping]: ThemeSet defaults obsahuje jen 3 light built-in témata; WarmTan vyžaduje navazující rozhodnutí.
- [Phase 29-syntect-theme-mapping]: Dark terminal palette je odvozená z aktivního visuals.panel_fill místo statického defaultu.
- [Phase 29-syntect-theme-mapping]: Regresní gate pro dark terminál je definovaná přes distinct variant background + foreground/background contrast >= 4.5.
- [Phase 27-02]: Zavedení `LIGHT_VARIANT_OPTIONS` a lokalizačního testu drží WarmTan viditelný a detekuje chybějící label v Settings pickeru.
- [Phase 27-02 Re-execution]: Light variant picker je viditelný i v dark režimu (disabled) a theme fingerprint zahrnuje dark_theme pro okamžitý preview.

### Known Tech Debt

- Nyquist VALIDATION.md: fáze ve stavu draft
- Warning text kontrast v light mode (Settings modal)
- Syntax highlighting v AI chatu nefunguje (egui_commonmark code blocky černobílé)

### Pending Todos

- Opravit kontrast warning textu v light mode (`modal_dialogs/settings.rs`)
- Upravit dolní lištu: branch oznámení více doprava, UTF více doleva (`src/app/ui/terminal/bottom/git_bar.rs`)
- V `Sestavit > Upravit` přidat online náhled změn zapisovaných do `.polycredo/profiles.toml`
- Zprovoznit `.polycredo/trash` a přesouvat tam smazané soubory

### Blockers/Concerns

- Phase 29-01: chybí 4. vhodný light built-in syntect kandidát pro `WarmTan` při zachování požadovaného vizuálního charakteru.

---

## Quick Tasks Completed

| #  | Description | Date | Commit | Directory |
|----|-------------|------|--------|-----------|
| ... | (pokračování z historie) | | | |

---

*Last updated: 2026-03-11*
| Phase 29-syntect-theme-mapping P01 | 9min | 3 tasks | 1 files |
| Phase 29-syntect-theme-mapping P02 | 6 min | 2 tasks | 1 files |
| Phase 27 P02 | 9 min | 3 tasks | 1 files |
