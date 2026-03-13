# PolyCredo Editor

## What This Is

Multiplatformni textovy editor v Rustu (eframe/egui) s terminaly, build workflow a AI terminal panelem. Produkt je local-first a ma zustat responzivni i pri delsi praci.

## Core Value

Editor nesmi zahrivat notebook v klidovem stavu - idle CPU zatez musi byt minimalni.

## Current State

- **Shipped version:** v1.3.1 Safe Trash Delete (2026-03-12)
- **Milestone result:** delete workflow je fail-closed move-to-trash (`.polycredo/trash`) s preview + restore MVP a watcher anti-storm stabilizaci
- **Quality status:** milestone audit `tech_debt` (requirements 11/11, phases 4/4, integration 4/4, flows 4/4, bez kritickych mezer)
- **Primary artifacts:** `.planning/milestones/v1.3.1-ROADMAP.md`, `.planning/milestones/v1.3.1-REQUIREMENTS.md`, `.planning/milestones/v1.3.1-MILESTONE-AUDIT.md`

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

- M001: Migration (completing)
- M002: Local History (queued)

## Next Milestone Goals

- Uzavrit robustnostni a bezpecnostni backlog pred dalsimi vetsimi refaktory.
- Prevest priorizovany backlog na traceable requirements + faze.
- Udrzet quality gate standard: `cargo check` + `./check.sh` pro kazdou fazi.

## Context

**Known tech debt:**
- UAT skipped scenare ve fazich 35 a 36 (non-blocking)
- Manualni anti-storm UX checkpoint ve fazi 38
- Warning text kontrast v light mode (Settings modal)
- Syntax highlighting v AI chatu (egui_commonmark code blocky)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| V1.3.1 delete tok zustava fail-closed bez hard-delete fallbacku | Bezpecnost dat ma prioritu pred agresivnim mazanim | ✓ Implemented v1.3.1 |
| Restore konflikt default je nedestruktivni `obnovit jako kopii` | Zabrani tichemu overwrite uz existujiciho souboru | ✓ Implemented v1.3.1 |
| Watcher stabilizace pouziva dedupe+batch + overflow full reload fallback | Ochrana proti event storm a UI lagum po delete/restore sekvenci | ✓ Implemented v1.3.1 |

<details>
<summary>Archived milestone context (v1.3.1 planning snapshot)</summary>

- Original milestone goal: nahradit hard delete za safe move-to-trash workflow s restore MVP.
- Closed phase range: 35-38.
- Final audit verdict: tech_debt (non-blocking).

</details>

---
*Last updated: 2026-03-12 after shipping milestone v1.3.1*
