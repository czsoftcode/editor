---
id: T01
parent: S02
milestone: M002
provides:
  - history split view s diff zvýrazněním, navigací a diff cachováním
  - DiffLine/DiffColors struktury pro opakované použití
  - podmíněný editor rendering v history mode
key_files:
  - src/app/ui/workspace/history/mod.rs
  - src/app/ui/workspace/mod.rs
  - locales/cs/ui.ftl
  - locales/en/ui.ftl
  - locales/sk/ui.ftl
  - locales/de/ui.ftl
  - locales/ru/ui.ftl
key_decisions:
  - Diff se počítá přes similar::TextDiff::from_lines(historical, current) — historical je "old", current je "new"
  - Levý panel zobrazuje Equal+Insert (aktuální verze), pravý Equal+Delete (historická verze)
  - Diff je cachovaný per selected_index, přepočítá se jen při změně indexu
  - Resize handle používá identický pattern jako markdown.rs split_axis()
patterns_established:
  - DiffLine struct s owned String — vhodný pro cachování přes framy
  - DiffColors s dark/light větvením — znovupoužitelný vzor pro budoucí diff rendering
observability_surfaces:
  - diff_for_index == selected_index indikuje cache hit/miss v HistoryViewState
  - I/O chyby při čtení snapshot obsahu se zobrazí inline v pravém panelu
duration: ~25 min
verification_result: passed
completed_at: 2026-03-13
blocker_discovered: false
---

# T01: Implementovat split view s diff zvýrazněním, navigací a cachováním

**Nahrazen jednoduchý history panel plnohodnotným split view s diff zvýrazněním, navigačními šipkami, resize handle a per-index diff cachováním.**

## What Happened

1. Kompletně přepsán `HistoryViewState` — odstraněny `preview_content` a `scroll_to_selected`, přidány `current_content`, `cached_diff`, `diff_for_index`, `split_ratio`.
2. Implementovány `DiffLine` a `DiffColors` struktury, `compute_diff()` a `diff_colors()` funkce.
3. Nová `render_history_split_view()` nahradila `render_history_panel()`:
   - Toolbar s heading, version info (timestamp), navigačními šipkami (←/→) s disabled stavem na hranicích, zavíracím tlačítkem.
   - Horizontální split view s resize handle (vzor z markdown.rs).
   - Levý panel: aktuální verze (Equal + Insert řádky se zeleným zvýrazněním).
   - Pravý panel: historická verze (Equal + Delete řádky s červeným zvýrazněním).
   - Diff cache: přepočet jen při změně `selected_index`.
4. V `workspace/mod.rs`: ShowHistory handler inicializuje `current_content` z aktivního tabu, `selected_index = Some(0)`. Editor.ui() se nekreslí v history mode (podmíněné na `history_view.is_none()`).
5. Přidáno 5 i18n klíčů do všech 5 jazyků (cs, en, sk, de, ru).
6. Přidáno 5 unit testů pro compute_diff a diff_colors.

## Verification

- `cargo check` — kompilace bez chyb ✅
- `cargo clippy` — žádné warningy ✅
- `./check.sh` — 133 unit testů prošlo ✅, 1 preexistující selhání v `phase35_delete_foundation` (nesouvisí s S02, hledá chybějící soubor `.planning/phases/35-trash-foundation-async-safety/35-03-PLAN.md`)
- `cargo fmt` — kód je formátovaný ✅
- Manuální ověření: vyžaduje spuštění GUI editoru (desktop app), nelze v headless prostředí

### Slice-level verification status:
- ✅ `cargo check` — kompilace bez chyb
- ✅ `cargo clippy` — žádné nové warningy
- ✅ `./check.sh` — unit testy prochází (S01 testy neporušené)
- ⏳ Manuální ověření v běžícím editoru — vyžaduje GUI

## Diagnostics

- `HistoryViewState.diff_for_index` — pokud se rovná `selected_index`, diff je cachovaný (přeskakuje přepočet)
- I/O chyby při čtení snapshot obsahu se zobrazí jako text "Chyba čtení: ..." v diff panelu
- Nové unit testy: `compute_diff_detects_insertions_and_deletions`, `compute_diff_identical_texts_all_equal`, `diff_colors_dark_mode_has_semitransparent_backgrounds`, `diff_colors_light_mode_has_opaque_backgrounds`, `format_timestamp_produces_correct_format`

## Deviations

- `on_hover_text()` na navigačních šipkách je volán vždy (ne jen při hovered), protože `Response::on_hover_text()` konzumuje self a nelze ho pak použít pro `.clicked()`. Funkčně ekvivalentní — tooltip se zobrazí jen při hoveru.

## Known Issues

- Preexistující selhání testu `phase35_delete_foundation_scope_guard_has_no_restore_foundation_symbols` — nesouvisí s S02.

## Files Created/Modified

- `src/app/ui/workspace/history/mod.rs` — kompletní přepis: nové struktury DiffLine/DiffColors, compute_diff(), diff_colors(), render_history_split_view() s toolbar, navigací, split panely a diff cachováním
- `src/app/ui/workspace/mod.rs` — podmíněný editor rendering v history mode, upravený ShowHistory handler s current_content a selected_index=0, volání render_history_split_view()
- `locales/cs/ui.ftl` — 5 nových i18n klíčů (history-nav-older, history-nav-newer, history-current-label, history-historical-label, history-version-info)
- `locales/en/ui.ftl` — 5 nových i18n klíčů
- `locales/sk/ui.ftl` — 5 nových i18n klíčů
- `locales/de/ui.ftl` — 5 nových i18n klíčů
- `locales/ru/ui.ftl` — 5 nových i18n klíčů
