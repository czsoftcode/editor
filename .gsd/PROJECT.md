# PolyCredo Editor

## What This Is

Multiplatformni textovy editor v Rustu (eframe/egui) s terminaly, build workflow a AI terminal panelem. Produkt je local-first a ma zustat responzivni i pri delsi praci.

## Core Value

Editor nesmi zahrivat notebook v klidovem stavu - idle CPU zatez musi byt minimalni.

## Current State

- **Shipped version:** v1.3.1 Safe Trash Delete (2026-03-12)
- **Last completed milestone:** M001: Migration (2026-03-13) — theme systém, sandbox runtime, slash commands, GSD state engine
- **Active milestone:** M002: Local History — S01+S02 dokončeny, S03 next
- **Quality status:** `cargo check` čistý, clippy čistý, testy zelené (1 předexistující selhání mimo scope)
- **Primary artifacts:** `.gsd/milestones/M002/slices/S02/S02-SUMMARY.md`

## Requirements

### Validated

- ✓ v1.3.1: Safe Trash Delete (TRASH-01..04, RESTORE-01..03, TRASHUI-01, RELIAB-01..03)
- ✓ v1.3.0: CLI cleanup + AI terminal-only boundary + traceability closure (R33-A/R33-B/R33-C/R33-D)
- ✓ v1.2.2: Additional Themes (WarmTan + Midnight varianty, syntect mapovani, i18n)
- ✓ v1.2.1: Save Modes + Unsaved Changes Guard
- ✓ v1.2.0: AI Chat Rewrite baseline

### Active

- [ ] V-1: Presunout environment inicializaci (`set_var`) do `main()` pred startem vlaken
- [ ] V-2: Dodat dukladnou validaci nazvu projektu (regex + zakaz nebezpecnych prefixu/znaku)
- [ ] K-1: Validovat IPC cesty (`is_absolute()`, `is_dir()`) pred pouzitim
- [ ] S-3: Neignorovat I/O chyby, propagovat je do UI toastu
- [ ] N-5: Nahradit rucni JSON serializaci za `serde_json`
- [ ] S-1: Upravit `FileWatcher::try_recv()`, aby neztracel eventy
- [ ] S-4: Pridat handling remove eventu ve watcheru
- [ ] V-3: Udrzet file dialog asynchronni (UI nesmi blokovat)

### Out of Scope

- Pridavani novych AI provideru bez jasneho produktoveho cile.
- Velke refaktory mimo konkretni milestone scope.

## Milestone Sequence

- ~~M001: Migration~~ ✓ (completed 2026-03-13)
- 🔄 M002: Local History (active — S01+S02 done, S03 next)

## Next Milestone Goals

- Uzavrit robustnostni a bezpecnostni backlog pred dalsimi vetsimi refaktory.
- Prevest priorizovany backlog na traceable requirements + faze.
- Udrzet quality gate standard: `cargo check` + `./check.sh` pro kazdou fazi.

## Context

**Known tech debt:**
- Test `phase35_delete_foundation_scope_guard_has_no_restore_foundation_symbols` selhává (hledá odstraněný soubor z v1.3.1)
- UAT skipped scenare ve fazich 35 a 36 (non-blocking)
- Manualni anti-storm UX checkpoint ve fazi 38
- Warning text kontrast v light mode (Settings modal)
- Syntax highlighting v AI chatu (egui_commonmark code blocky)
- warm_ivory_bg() threshold na hraně pro budoucí varianty
- FmDocument dot-notation omezena na 2 úrovně zanoření

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| V1.3.1 delete tok zustava fail-closed bez hard-delete fallbacku | Bezpecnost dat ma prioritu pred agresivnim mazanim | ✓ Implemented v1.3.1 |
| Restore konflikt default je nedestruktivni `obnovit jako kopii` | Zabrani tichemu overwrite uz existujiciho souboru | ✓ Implemented v1.3.1 |
| Watcher stabilizace pouziva dedupe+batch + overflow full reload fallback | Ochrana proti event storm a UI lagum po delete/restore sekvenci | ✓ Implemented v1.3.1 |
| Custom YAML-like parser místo serde_yaml | Nulové nové závislosti, plný round-trip, stačí subset | ✓ Implemented M001/S11 |
| Terminal theme per-frame set_theme() místo PTY restartu | Jednodušší, responsivnější, bez race conditions | ✓ Implemented M001/S04 |
| Sandbox runtime apply místo apply-on-reopen | Lepší UX, okamžitá odezva | ✓ Implemented M001/S08 |

<details>
<summary>Archived milestone context (v1.3.1 planning snapshot)</summary>

- Original milestone goal: nahradit hard delete za safe move-to-trash workflow s restore MVP.
- Closed phase range: 35-38.
- Final audit verdict: tech_debt (non-blocking).

</details>

<details>
<summary>Archived milestone context (M001: Migration)</summary>

- 7 slicí: S02 (Základ), S04 (Terminal Git Barvy), S05 (Light Varianty), S07 (Infrastructure), S08 (Sandbox Apply), S10 (Slash Commands), S11 (GSD State Engine)
- Hlavní deliverables: dark/light theme se 3 variantami, sandbox runtime lifecycle, 7 slash commands, YAML-like parser
- Duration: 2026-03-04 až 2026-03-07

</details>

---
*Last updated: 2026-03-13 after completing slice M002/S02: History Split View s Diff a Navigací*
