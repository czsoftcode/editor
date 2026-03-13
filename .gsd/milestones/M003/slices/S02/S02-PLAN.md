# S02: Obnovení historické verze s potvrzením a i18n

**Goal:** Tlačítko "Obnovit" v toolbaru history view → potvrzovací dialog → zápis historické verze do editoru (append snapshot) → refresh history. i18n klíče kompletní.
**Demo:** Uživatel vybere historickou verzi, klikne "Obnovit", potvrdí v dialogu → obsah se zapíše do levého panelu, nový snapshot se objeví na pozici 0 v historii, mezilehlé verze zůstanou zachovány. Vše lokalizované v 5 jazycích.

## Must-Haves

- Tlačítko "Obnovit" v toolbaru (right-to-left blok), disabled když `selected_index.is_none()`.
- `show_modal()` potvrzovací dialog před restore akcí.
- Restore logika v `workspace/mod.rs`: zápis historického obsahu → `take_snapshot()` (append) → `get_history()` refresh → update `HistoryViewState` (entries, selected_index=Some(0), diff cache invalidace).
- `HistorySplitResult` rozšířen o `restore_confirmed: bool`.
- `HistoryViewState` rozšířen o `show_restore_confirm: bool`.
- Mezilehlé verze zachovány (append, ne replace).
- i18n klíče pro restore tlačítko, confirm dialog titulek, confirm dialog text, confirm dialog ok/cancel — ve všech 5 jazycích (cs, en, sk, de, ru).
- `cargo check` + `cargo test` + `./check.sh` prochází.

## Verification

- `cargo check` — kompilace bez chyb
- `cargo test` — všechny testy prochází (včetně existujících history testů)
- `./check.sh` — fmt + clippy + testy
- Manuální inspekce i18n klíčů: `grep 'history-restore' locales/*/ui.ftl` vrací záznamy pro všech 5 jazyků

## Tasks

- [x] **T01: Tlačítko Obnovit, confirm dialog, restore logika a i18n** `est:45m`
  - Why: Celý scope S02 — jedno tlačítko, jeden dialog, restore flow, i18n. Žádný nový pattern, všechno staví na existujících show_modal a HistorySplitResult mechanismech.
  - Files: `src/app/ui/workspace/history/mod.rs`, `src/app/ui/workspace/mod.rs`, `locales/cs/ui.ftl`, `locales/en/ui.ftl`, `locales/sk/ui.ftl`, `locales/de/ui.ftl`, `locales/ru/ui.ftl`
  - Do: (1) Rozšířit datový model — `show_restore_confirm: bool` do HistoryViewState, `restore_confirmed: bool` do HistorySplitResult. (2) Přidat tlačítko "Obnovit" do toolbaru (right-to-left blok, před navigační šipky), enabled jen při selected_index.is_some(). (3) Klik → `show_restore_confirm = true`. Volat `show_modal()` v render funkci když flag aktivní. Confirmed → `restore_confirmed = true`, Cancelled → clear. (4) V workspace/mod.rs: handle `hv_result.restore_confirmed` — extrahovat data z history_view do locals (entries, selected_index, relative_path), načíst `get_snapshot_content()`, zapsat do tab.content, `take_snapshot()`, `get_history()` refresh, update history_view (entries, selected_index=Some(0), content_hash=0, diff_for_index=None, current_content=nový obsah). (5) i18n klíče do 5 locale souborů. (6) Inicializace nových fieldů v workspace/mod.rs.
  - Verify: `cargo check && cargo test && ./check.sh` + `grep 'history-restore' locales/*/ui.ftl` vrací 5 jazyků
  - Done when: Kompilace čistá, testy prochází, i18n klíče existují ve všech 5 jazycích, HistorySplitResult.restore_confirmed je propojený od UI po workspace handling

## Files Likely Touched

- `src/app/ui/workspace/history/mod.rs` — datový model (HistoryViewState, HistorySplitResult), toolbar tlačítko, confirm dialog
- `src/app/ui/workspace/mod.rs` — restore handling po render, inicializace nových fieldů
- `locales/cs/ui.ftl` — i18n klíče (čeština)
- `locales/en/ui.ftl` — i18n klíče (angličtina)
- `locales/sk/ui.ftl` — i18n klíče (slovenština)
- `locales/de/ui.ftl` — i18n klíče (němčina)
- `locales/ru/ui.ftl` — i18n klíče (ruština)
