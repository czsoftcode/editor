# PolyCredo Editor

## What This Is

Multiplatformni textovy editor v Rustu (eframe/egui) s terminaly, build workflow a AI terminal panelem. Editor je local-first a ma zustat responzivni i pri delsi praci. Milestone v1.3.0 je cleanup/pivot: odstranit slepou ulicku `src/app/cli/*` a nechat jen AI terminal funkcionalitu.

## Core Value

Editor nesmi zahrivat notebook v klidovem stavu - idle CPU zatez musi byt minimalni.

## Current Milestone: v1.3.0 AI Terminal Cleanup

**Goal:** Odstranit PolyCredo CLI vrstvu (`src/app/cli/*`) a zachovat jen AI terminal se stejnym uzivatelskym chovanim.

**Target features:**
- Odebrani `src/app/cli/*` a presun/nahrazeni pouzitych casti do AI terminal modulu
- Zachovani AI terminal UX (chat, streaming odpovedi, model picker, slash/GSD prikazy)
- Zachovani approval + security guardu pro operace, ktere AI terminal spousti
- Cleanup importu, typu, testu a docs bez ztrat funkcionality

## Requirements

### Validated

- v1.2.2 Additional Themes: WarmTan + Midnight varianty, syntect mapovani, i18n a picker layout
- v1.2.1 Save Modes + Unsaved Changes Guard: manual/auto save, guard dialogy, regression coverage
- v1.2.0 AI Chat Rewrite: streaming chat, provider pipeline, tool execution a security baseline

### Active

- [ ] CLI-01: Kod v `src/app/cli/*` je odstraneny a build/test prochazi bez dead importu.
- [ ] TERM-01: AI terminal zachovava chat, streaming, model picker a slash/GSD cesty bez regresi.
- [ ] SAFE-01: Approval a security pravidla pro AI akce zustavaji funkcni po odstraneni CLI vrstvy.
- [ ] ARCH-01: Stavove struktury a konfigurace nejsou navazane na `app::cli` namespace.

### Out of Scope

- Pridavani novych AI provideru - cilem je cleanup, ne rozsirovani funkci.
- Redesign AI terminal UI - zachovame stavajici UX kontrakt.
- Velke refaktory mimo AI/CLI oblast - mimo rozsah milestone.

## Context

**Shipped:** v1.2.2 Additional Themes (2026-03-11), v1.2.3-dev quality gate cleanup, v1.2.4-dev windows build fix + profiles refresh.

**Known tech debt relevant to this milestone:**
- `src/app/cli/*` drzi cast logiky, ktera je pro produktovy smer zbytecna.
- Importy `app::cli::*` jsou rozlezle v UI/state vrstvach a brani jednoduchemu maintenance.
- Security/approval flow je potreba zachovat i po odstraneni CLI namespace.

## Constraints

- **Tech stack**: Rust + eframe/egui - bez zavadeni noveho runtime/frameworku.
- **Behavioral compatibility**: AI terminal se nesmi funkcne rozbit (chat/send/stream/approval).
- **Quality gate**: Kazda faze musi projit `cargo check` a `./check.sh`.
- **Scope discipline**: Minimalni cilene patche, bez velkych refaktoru mimo potrebu odstraneni CLI vrstvy.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Odstranit `src/app/cli/*` v milestone v1.3.0 | CLI vrstva je slepa ulicka proti smeru produktu | — Pending |
| Zachovat jen AI terminal UX kontrakt | Uzivatelska hodnota je v terminal panelu, ne v internim namespace | — Pending |
| Cleanup po fazich 30+ (ne jednim mega patchem) | Snadnejsi verifikace, mensi riziko regresi | — Pending |

---
*Last updated: 2026-03-11 after starting milestone v1.3.0*
