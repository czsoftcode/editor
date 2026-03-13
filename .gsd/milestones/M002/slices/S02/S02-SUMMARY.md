---
id: S02
parent: M002
milestone: M002
provides:
  - Split view rendering se dvěma read-only ScrollArea panely (aktuální vlevo, historická vpravo) s resize handle
  - Diff zvýraznění přes similar::TextDiff::from_lines() s cachovaným výsledkem per selected_index
  - Navigační UI (šipky ← starší / → novější) s disabled stavem na hranicích seznamu verzí
  - Zavírací mechanismus history mode → normální editor mode
  - i18n klíče pro split view navigaci ve všech 5 jazycích
requires:
  - slice: S01
    provides: HistoryViewState struct, render_history_panel(), LocalHistory::get_snapshot_content(), LocalHistory::get_history(), TabBarAction::ShowHistory, context menu trigger
affects:
  - S03
key_files:
  - src/app/ui/workspace/history/mod.rs
  - src/app/ui/workspace/mod.rs
  - locales/cs/ui.ftl
  - locales/en/ui.ftl
  - locales/sk/ui.ftl
  - locales/de/ui.ftl
  - locales/ru/ui.ftl
key_decisions:
  - "Split view layout: toolbar nahoře (šipky + info + ✕), pod ním dva panely (aktuální vlevo, historická vpravo)"
  - "Diff rendering: levý panel zobrazuje Equal+Insert řádky, pravý Equal+Delete — side-by-side diff bez duplicity společných řádků"
  - "Light mode diff barvy: bg_added(200,240,200), bg_removed(255,210,210), fg_added(0,100,0), fg_removed(150,0,0) — kontrastnější než dark varianta"
  - "Editor.ui() podmíněné na history_view.is_none() — editor se kompletně nekreslí v history mode"
  - "Diff cache invalidace přes diff_for_index != selected_index — jednoduchý pattern bez generace/verze countera"
patterns_established:
  - DiffLine struct s owned String pro cachování diff výsledků přes framy
  - DiffColors s dark/light větvením — znovupoužitelný vzor pro budoucí diff rendering
  - Podmíněný editor rendering v history/split mode
observability_surfaces:
  - HistoryViewState.diff_for_index == selected_index indikuje diff cache hit/miss
  - I/O chyby při čtení snapshot obsahu se zobrazí inline v pravém diff panelu
drill_down_paths:
  - .gsd/milestones/M002/slices/S02/tasks/T01-SUMMARY.md
duration: ~25 min
verification_result: passed
completed_at: 2026-03-13
---

# S02: History Split View s Diff a Navigací

**Jednoduchý history panel z S01 nahrazen plnohodnotným split view se dvěma read-only panely, diff zvýrazněním (zelená/červená), navigačními šipkami a per-index diff cachováním.**

## What Happened

Kompletně přepsán rendering history view v jednom tasku:

1. **Rozšíření HistoryViewState** — odstraněny S01 pole (`preview_content`, `scroll_to_selected`), přidány `current_content: String` (obsah aktivního tabu při otevření), `cached_diff: Option<Vec<DiffLine>>`, `diff_for_index: Option<usize>`, `split_ratio: f32`.

2. **Diff engine** — `DiffLine` struct (ChangeTag + owned String) a `compute_diff()` volající `similar::TextDiff::from_lines(historical, current)`. Výsledek se cachuje per `selected_index` — přepočet jen při navigaci na jinou verzi, ne per-frame.

3. **DiffColors** — dark mode: semitransparentní pozadí (zelená/červená 0.15 alpha), light mode: opaque RGB hodnoty s vyšším kontrastem. Automatické větvení podle `ui.visuals().dark_mode`.

4. **render_history_split_view()** nahradil `render_history_panel()`:
   - Toolbar nahoře: heading s názvem souboru, info o vybrané verzi (timestamp), navigační šipky (← starší / → novější) s disabled stavem na hranicích, zavírací tlačítko ✕.
   - Horizontální split view pod toolbarem: resize handle (identický pattern jako `render/markdown.rs` `split_axis()`), levý panel (aktuální verze: Equal + Insert řádky), pravý panel (historická verze: Equal + Delete řádky).
   - Oba panely renderují plný soubor jako `LayoutJob` s per-řádkovým barvením.

5. **Podmíněný editor rendering** — v `workspace/mod.rs` `ws.editor.ui()` se nevolá, pokud je `history_view.is_some()`. ShowHistory handler inicializuje `current_content` z aktivního tabu a nastaví `selected_index = Some(0)`.

6. **i18n** — 5 nových klíčů (`history-nav-older`, `history-nav-newer`, `history-current-label`, `history-historical-label`, `history-version-info`) ve všech 5 jazycích (cs, en, sk, de, ru).

7. **Testy** — 5 unit testů: `compute_diff_detects_insertions_and_deletions`, `compute_diff_identical_texts_all_equal`, `diff_colors_dark_mode_has_semitransparent_backgrounds`, `diff_colors_light_mode_has_opaque_backgrounds`, `format_timestamp_produces_correct_format`.

## Verification

- ✅ `cargo check` — kompilace bez chyb
- ✅ `cargo clippy` — žádné nové warningy
- ✅ `./check.sh` — 133 unit testů prošlo, 1 preexistující selhání (`phase35_delete_foundation` hledá odstraněný soubor — mimo scope)
- ⏳ Manuální ověření v běžícím editoru — vyžaduje GUI desktop, popsáno v S02-UAT.md

## Deviations

- `on_hover_text()` na navigačních šipkách je voláno vždy (ne jen při hovered stavu), protože `Response::on_hover_text()` konzumuje self a nelze pak přistupovat k `.clicked()`. Funkčně ekvivalentní — egui tooltip se zobrazí pouze při hoveru.

## Known Limitations

- Manuální UAT vyžaduje GUI prostředí — headless verifikace pokrývá kompilaci, testy a diff logiku, ale ne vizuální layout a UX.
- Synchronized scroll dvou panelů není implementován — panely scrollují nezávisle (záměrné rozhodnutí z plánování).
- Edge cases (zavření tabu v history mode, history mode na posledním tabu) nejsou ošetřeny — scope S03.

## Follow-ups

- S03: Cleanup retence (50 verzí, 30 dní) při startu workspace.
- S03: Edge case handling — zavření tabu v history mode, history mode na posledním tabu.
- S03: Finální i18n audit a vyčištění testovacích dat v `.polycredo/history/`.
- S03: Ověření watcher filtru pro `.polycredo/` adresář.

## Files Created/Modified

- `src/app/ui/workspace/history/mod.rs` — kompletní přepis: DiffLine/DiffColors struktury, compute_diff(), diff_colors(), render_history_split_view() s toolbar, navigací, split panely a diff cachováním, 5 nových unit testů
- `src/app/ui/workspace/mod.rs` — podmíněný editor rendering v history mode, upravený ShowHistory handler s current_content a selected_index=0, volání render_history_split_view()
- `locales/cs/ui.ftl` — 5 nových i18n klíčů pro split view navigaci
- `locales/en/ui.ftl` — 5 nových i18n klíčů pro split view navigaci
- `locales/sk/ui.ftl` — 5 nových i18n klíčů pro split view navigaci
- `locales/de/ui.ftl` — 5 nových i18n klíčů pro split view navigaci
- `locales/ru/ui.ftl` — 5 nových i18n klíčů pro split view navigaci

## Forward Intelligence

### What the next slice should know
- `HistoryViewState` je kompletní — má `current_content`, `cached_diff`, `diff_for_index`, `split_ratio`, plus vše ze S01. S03 nepotřebuje měnit struct, pouze přidat cleanup logiku a edge case handling v workspace/mod.rs.
- `render_history_split_view()` je jediný entry point pro history rendering — volá se z `workspace/mod.rs` místo starého `render_history_panel()`.
- ShowHistory handler v workspace/mod.rs je místo, kde se inicializuje history view stav — edge case handling (zavření tabu) bude pravděpodobně v update cyklu workspace.

### What's fragile
- `current_content` se načte jednou při otevření history view z tab bufferu — pokud se soubor změní externě během prohlížení historie, obsah se neaktualizuje. Pro S03 to může být edge case k ošetření.
- Preexistující test `phase35_delete_foundation` — selhává na chybějící `.planning/phases/35-...` soubor. Nesouvisí s M002, ale zkresluje výstup `./check.sh`.

### Authoritative diagnostics
- `HistoryViewState.diff_for_index` — pokud se rovná `selected_index`, diff cache funguje správně. Pokud ne, přepočítá se.
- Unit testy `compute_diff_*` a `diff_colors_*` — ověřují diff logiku a barevné schéma nezávisle na GUI.

### What assumptions changed
- Původní odhad 90 min → skutečnost ~25 min. Codebase byl dobře připravený z S01, pattern z markdown.rs se přímo použil.
