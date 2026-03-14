---
id: T03
parent: S01
milestone: M006
provides:
  - Keymap toggle pro show_panel (Ctrl+Shift+F otevírá/zavírá panel)
  - Menu action vždy otevírá panel (show_panel = true)
  - Escape handling v panelu (zavře panel bez ztráty query/results)
  - Panel heading s i18n klíčem project-search-panel-title
  - Smazání mrtvého kódu (render_project_search_dialog, poll_and_render_project_search_results)
  - Odstranění show_input field z ProjectSearch
key_files:
  - src/app/ui/workspace/mod.rs
  - src/app/ui/workspace/menubar/mod.rs
  - src/app/ui/search_picker.rs
  - src/app/ui/workspace/state/types.rs
  - locales/cs/ui.ftl
  - locales/en/ui.ftl
  - locales/sk/ui.ftl
  - locales/de/ui.ftl
  - locales/ru/ui.ftl
key_decisions:
  - Keymap dispatch pro ProjectSearch řešen přímo v workspace/mod.rs keymap sekci (ne přes generické process_menu_actions) — umožňuje toggle vs. menu always-open bez přidávání rozlišovacího signálu do MenuActions
  - Escape handling jako globální key check před panel renderem (ne uvnitř panel closure) — jednodušší a funguje i když panel nemá focus
patterns_established:
  - Keymap-specific handling mimo generický dispatch pro příkazy vyžadující odlišné chování od menu akcí
observability_surfaces:
  - "ProjectSearch.show_panel: bool — panel viditelnost (toggle via Ctrl+Shift+F, close via Escape/✕)"
  - "grep 'show_input' → 0 = field odstraněn"
  - "grep 'render_project_search_dialog' → 0 = mrtvý kód smazán"
duration: ~15 min
verification_result: passed
completed_at: 2026-03-13
blocker_discovered: false
---

# T03: Keymap dispatch + smazání mrtvých modálů + i18n + cleanup

**Keymap toggle na show_panel, smazání mrtvých modálních dialogů, i18n panel-title ve všech 5 jazycích, Escape handling v panelu.**

## What Happened

1. **Keymap dispatch** — `CommandId::ProjectSearch` v centrální keymap sekci workspace/mod.rs nyní toggle `show_panel` (otevře/zavře). Odchytáváno přímo před generickým dispatch → nevchází do process_menu_actions.

2. **Menu action** — `process_menu_actions()` v menubar/mod.rs nastavuje `show_panel = true` + `focus_requested = true`. Odstraněna reference na `show_input`.

3. **Smazání mrtvých volání** — z workspace/mod.rs odstraněna volání `render_project_search_dialog()` a `poll_and_render_project_search_results()`, včetně importů.

4. **Smazání mrtvých funkcí** — z search_picker.rs kompletně odstraněny `render_project_search_dialog()` (~190 řádků) a `poll_and_render_project_search_results()` (~185 řádků). Ponecháno: `render_replace_preview_dialog()`, `render_search_panel()`, engine funkce + testy.

5. **show_input removal** — field `show_input` odstraněn z `ProjectSearch` struct a Default impl v types.rs. Odstraněna poslední reference v `start_project_search()`.

6. **Escape handling** — přidána detekce `Key::Escape` před panel renderem — zavře panel (`show_panel = false`) bez ztráty query/results. Kontrola: nezavírá pokud je otevřený `show_replace_preview` dialog. Close button (✕) funguje stejně.

7. **i18n** — přidán `project-search-panel-title` klíč do všech 5 jazyků (cs/en/sk/de/ru). Panel zobrazuje heading s tímto klíčem.

8. **cargo fmt + clippy + testy** — vše prošlo čistě.

## Verification

- `./check.sh` → "Quality Gate: All checks passed successfully!" — 192 testů pass, fmt OK, clippy OK
- `grep -c 'render_project_search_dialog' src/app/ui/workspace/mod.rs` → 0 ✓
- `grep -c 'poll_and_render_project_search_results' src/app/ui/workspace/mod.rs` → 0 ✓
- `grep -c 'show_input' src/app/ui/workspace/state/types.rs` → 0 ✓
- `grep -c 'show_panel' src/app/ui/workspace/state/types.rs` → 2 ✓
- `grep -c 'project-search-panel-title' locales/cs/ui.ftl` → 1 ✓
- `grep -c 'project-search-panel-title' locales/en/ui.ftl` → 1 ✓
- `grep -c 'project-search-panel-title' locales/{sk,de,ru}/ui.ftl` → 1 each ✓
- `grep -c 'search_panel' src/app/ui/workspace/mod.rs` → 2 ✓ (import + volání)
- Existující 20 unit testů v search_picker::tests stále pass (engine funkce nezměněny)

### Slice-level verification status (final task)
- [x] `cargo check` — čistá kompilace bez warningů
- [x] `./check.sh` — fmt, clippy, všechny testy pass
- [x] `grep -c 'show_panel' src/app/ui/workspace/state/types.rs` → ≥1 (výsledek: 2)
- [x] `grep -c 'search_panel' src/app/ui/workspace/mod.rs` → ≥1 (výsledek: 2)
- [x] `grep -c 'render_project_search_dialog' src/app/ui/workspace/mod.rs` → 0
- [x] Existující 20 unit testů v search_picker::tests stále pass

## Diagnostics

- Panel viditelnost: `ws.project_search.show_panel` — `true` = zobrazen, `false` = skryt
- Toggle: Ctrl+Shift+F přepíná `show_panel`, menu vždy nastavuje `true`
- Escape: zavře panel pokud `show_replace_preview == false`
- Mrtvý kód: `grep 'render_project_search_dialog\|poll_and_render_project_search_results' src/` → jen funkce v search_picker.rs odstraněny, žádné reference nikde

## Deviations

- Keymap dispatch: plán říkal editovat `keymap.rs` — reálně se dispatch řeší v `workspace/mod.rs` (keymap.rs jen vrací CommandId, nemá přístup k workspace state). Funkčně ekvivalentní.
- Escape handling: implementován jako globální key check před panel renderem, ne uvnitř panel closure. Důvod: jednodušší a funguje konzistentně.

## Known Issues

None.

## Files Created/Modified

- `src/app/ui/workspace/mod.rs` — keymap toggle dispatch pro ProjectSearch, smazání mrtvých volání + importů
- `src/app/ui/workspace/menubar/mod.rs` — smazání show_input reference z process_menu_actions
- `src/app/ui/search_picker.rs` — smazání render_project_search_dialog() + poll_and_render_project_search_results(), přidání Escape handling + panel heading
- `src/app/ui/workspace/state/types.rs` — odstranění show_input field
- `locales/cs/ui.ftl` — přidán project-search-panel-title
- `locales/en/ui.ftl` — přidán project-search-panel-title
- `locales/sk/ui.ftl` — přidán project-search-panel-title
- `locales/de/ui.ftl` — přidán project-search-panel-title
- `locales/ru/ui.ftl` — přidán project-search-panel-title
- `.gsd/milestones/M006/slices/S01/tasks/T03-PLAN.md` — přidána Observability Impact sekce
