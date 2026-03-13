---
id: T01
parent: S02
milestone: M003
provides:
  - HistorySplitResult.restore_confirmed signalizace z UI
  - Tlačítko "Obnovit" v toolbaru history view (disabled bez výběru)
  - Potvrzovací dialog přes show_modal() před restore
  - Restore logika v workspace/mod.rs (zápis do tab + take_snapshot + refresh)
  - i18n klíče history-restore-* ve všech 5 jazycích
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
duration: ~15min
verification_result: passed
completed_at: 2026-03-13
blocker_discovered: false
---

# T01: Tlačítko Obnovit, confirm dialog, restore logika a i18n

**Kompletní restore flow: tlačítko → confirm dialog → zápis historického obsahu do tab + snapshot + refresh history entries ve všech 5 jazycích.**

## What Happened

1. **Datový model rozšířen:** `HistorySplitResult` má nový field `restore_confirmed`, `HistoryViewState` má `show_restore_confirm` pro řízení confirm dialogu.

2. **Tlačítko "Obnovit"** přidáno do toolbaru v right-to-left layoutu — za navigačními šipkami (vizuálně vlevo od šipek). Disabled pokud `selected_index.is_none()`.

3. **Confirm dialog** implementován přes existující `show_modal()` z `widgets::modal`. Klik na OK → `restore_confirmed = true`, Cancel/dismiss → reset flagu. Dialog zobrazuje informaci, že aktuální stav bude uložen jako nová verze.

4. **Restore handling** v `workspace/mod.rs`:
   - Extrakce dat z `history_view` do locals (borrow checker — nelze mutable local_history a immutable history_view současně)
   - Čtení historického obsahu přes `get_snapshot_content()`
   - Zápis do tab bufferu (`content`, `modified`, `last_edit`, `save_status`)
   - `take_snapshot()` pro uložení nového snímku (deduplikace handled interně)
   - `get_history()` refresh + update history_view state (selected_index=0, cache invalidace)

5. **i18n klíče** `history-restore-btn`, `history-restore-confirm-title`, `history-restore-confirm-text`, `history-restore-confirm-ok`, `history-restore-confirm-cancel` přidány do cs, en, sk, de, ru.

## Verification

- `cargo check` — čistá kompilace ✓
- `cargo test` — 145 unit testů pass ✓
- `./check.sh` — fmt + clippy + testy čisté ✓ (1 pre-existující selhání `phase35_delete_foundation_scope_guard` — chybějící soubor z jiného milestonu, nesouvisí)
- `grep -c 'history-restore' locales/*/ui.ftl` — 5 klíčů × 5 jazyků ✓

### Slice-level verification status (this is the only task in S02):
- [x] `cargo check` — kompilace bez chyb
- [x] `cargo test` — všechny testy prochází (pre-existující selhání neovlivněno)
- [x] `./check.sh` — fmt + clippy + testy
- [x] `grep 'history-restore' locales/*/ui.ftl` — záznamy pro všech 5 jazyků

## Diagnostics

- Selhání restore se loguje přes `eprintln!("[Restore] Chyba při čtení historické verze: ...")` — hledej `[Restore]` ve stderr.
- Restore flow je pozorovatelný přes změnu `tab.modified = true` a nový snapshot v `.polycredo/history/`.

## Deviations

- Použití `eprintln!` místo `log::error!` — kódová báze nepoužívá `log` crate, drží se `eprintln!`.

## Known Issues

- Pre-existující selhání testu `phase35_delete_foundation_scope_guard_has_no_restore_foundation_symbols` (chybějící soubor `35-03 plan`).

## Files Created/Modified

- `src/app/ui/workspace/history/mod.rs` — rozšířené struktury (restore_confirmed, show_restore_confirm), tlačítko v toolbaru, confirm dialog
- `src/app/ui/workspace/mod.rs` — restore handling po render, show_restore_confirm init
- `locales/cs/ui.ftl` — 5 nových history-restore-* klíčů (čeština)
- `locales/en/ui.ftl` — 5 nových history-restore-* klíčů (angličtina)
- `locales/sk/ui.ftl` — 5 nových history-restore-* klíčů (slovenština)
- `locales/de/ui.ftl` — 5 nových history-restore-* klíčů (němčina)
- `locales/ru/ui.ftl` — 5 nových history-restore-* klíčů (ruština)
