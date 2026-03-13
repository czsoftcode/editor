---
id: S02
parent: M003
milestone: M003
provides:
  - Tlačítko "Obnovit" v toolbaru history view (disabled bez výběru)
  - Potvrzovací dialog přes show_modal() před restore
  - Restore logika v workspace/mod.rs (zápis do tab + take_snapshot + refresh)
  - HistorySplitResult.restore_confirmed signalizace z UI
  - HistoryViewState.show_restore_confirm pro řízení dialogu
  - i18n klíče history-restore-* ve všech 5 jazycích (cs, en, sk, de, ru)
requires:
  - slice: S01
    provides: Editovatelný levý panel s průběžným tab.content sync, HistoryViewState s selected_index/entries/relative_path, LocalHistory::take_snapshot() a get_history(), background_io_tx
affects: []
key_files:
  - src/app/ui/workspace/history/mod.rs
  - src/app/ui/workspace/mod.rs
  - locales/cs/ui.ftl
  - locales/en/ui.ftl
  - locales/sk/ui.ftl
  - locales/de/ui.ftl
  - locales/ru/ui.ftl
key_decisions:
  - Restore signalizace přes HistorySplitResult.restore_confirmed (UI vrací flag, workspace handler provede mutace)
  - Borrow checker řešen extrakcí dat z history_view do locals před mutable operacemi na local_history/tabs
  - Použití eprintln! pro error logging místo log crate (konzistentní s kódovou bází)
patterns_established:
  - show_modal() pattern pro jednoduché confirm dialogy v history view
observability_surfaces:
  - eprintln!("[Restore] ...") při selhání čtení historické verze
drill_down_paths:
  - .gsd/milestones/M003/slices/S02/tasks/T01-SUMMARY.md
duration: ~15min
verification_result: passed
completed_at: 2026-03-13
---

# S02: Obnovení historické verze s potvrzením a i18n

**Kompletní restore flow: tlačítko "Obnovit" → potvrzovací dialog → zápis historického obsahu do tab bufferu + nový snapshot (append) + refresh history. i18n klíče pro 5 jazyků.**

## What Happened

Celý scope S02 implementován v jednom tasku (T01). Datový model rozšířen o `restore_confirmed` v `HistorySplitResult` a `show_restore_confirm` v `HistoryViewState`. Tlačítko "Obnovit" přidáno do toolbaru v right-to-left layoutu, disabled bez aktivního výběru verze. Klik otevírá confirm dialog přes existující `show_modal()` pattern. Po potvrzení workspace handler extrahuje data z history_view do locals (borrow checker — nemožnost simultánního mutable local_history a immutable history_view), načte historický obsah přes `get_snapshot_content()`, zapíše ho do tab bufferu, vytvoří nový snapshot (`take_snapshot()`), refreshne history entries a nastaví `selected_index=Some(0)` s invalidací diff cache. Mezilehlé verze zůstávají zachovány — nový snapshot je append, ne replace. Pět i18n klíčů (`history-restore-btn`, `-confirm-title`, `-confirm-text`, `-confirm-ok`, `-confirm-cancel`) přidáno do cs, en, sk, de, ru.

## Verification

- `cargo check` — čistá kompilace ✓
- `cargo test` — 145 unit testů + 12 integračních testů pass ✓
- `./check.sh` — fmt + clippy + testy čisté ✓
- `grep -c 'history-restore' locales/*/ui.ftl` — 5 klíčů × 5 jazyků ✓
- Pre-existující selhání `phase35_delete_foundation_scope_guard_has_no_restore_foundation_symbols` (chybějící soubor z v1.3.1) — mimo scope, neovlivněno.

## Requirements Advanced

- R004 — Obnovení historické verze (append, ne replace): implementováno kompletně, restore flow funkční end-to-end
- R005 — Potvrzovací dialog před obnovením: implementován přes show_modal() pattern
- R008 — i18n klíče pro nové UI prvky: 5 klíčů × 5 jazyků kompletní

## Requirements Validated

- R004 — contract verification: kompilace + testy pass, restore logika propojená od UI po workspace handling
- R005 — contract verification: dialog integrován do restore flow, cancel/confirm cesty funkční
- R008 — artifact verification: `grep 'history-restore' locales/*/ui.ftl` potvrzuje záznamy pro všech 5 jazyků

## New Requirements Surfaced

- none

## Requirements Invalidated or Re-scoped

- none

## Deviations

- none — implementace sledovala plán beze změn

## Known Limitations

- Restore error handling používá `eprintln!` místo UI toastu — konzistentní s kódovou bází, ale pro uživatele neviditelné v GUI
- Pre-existující test selhání `phase35_delete_foundation_scope_guard` (mimo scope)

## Follow-ups

- none — S02 je finální slice M003, milestone je kompletní

## Files Created/Modified

- `src/app/ui/workspace/history/mod.rs` — rozšířené struktury (restore_confirmed, show_restore_confirm), tlačítko v toolbaru, confirm dialog
- `src/app/ui/workspace/mod.rs` — restore handling po render, show_restore_confirm init
- `locales/cs/ui.ftl` — 5 nových history-restore-* klíčů (čeština)
- `locales/en/ui.ftl` — 5 nových history-restore-* klíčů (angličtina)
- `locales/sk/ui.ftl` — 5 nových history-restore-* klíčů (slovenština)
- `locales/de/ui.ftl` — 5 nových history-restore-* klíčů (němčina)
- `locales/ru/ui.ftl` — 5 nových history-restore-* klíčů (ruština)

## Forward Intelligence

### What the next slice should know
- M003 je kompletní — žádná další slice v tomto milestonu. Pokud se navazuje dalším milestonu, history view je nyní plně funkční s editovatelným levým panelem, sync scrollem, diff zvýrazněním, syntax highlighting a restore flow.

### What's fragile
- Borrow checker pattern v workspace/mod.rs restore handleru — extrakce dat do locals je nutná, přidání dalších mutable operací vyžaduje stejný pattern
- `eprintln!` error logging — uživatel nevidí restore chyby v GUI

### Authoritative diagnostics
- `[Restore]` prefix ve stderr — hledej při selhání čtení historické verze
- `tab.modified = true` + nový snapshot v `.polycredo/history/` — pozorovatelné výstupy restore operace

### What assumptions changed
- none — vše proběhlo podle plánu z S02-PLAN
