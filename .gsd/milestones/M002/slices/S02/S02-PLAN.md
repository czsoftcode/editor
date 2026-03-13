# S02: History Split View s Diff a Navigací

**Goal:** "Historie souboru" otevře plný split view — aktuální verze vlevo, historická vpravo s diff zvýrazněním. Šipky přepínají mezi verzemi. Diff je cachovaný. Zavření vrátí normální editor.
**Demo:** Pravý klik na tab → "Historie souboru" → split view se dvěma panely: aktuální vlevo, historická vpravo, řádky přidané zeleně, odebrané červeně. Šipky ← → přepínají verze, diff se aktualizuje. ✕ zavře a editor se vrátí do normálu.

## Must-Haves

- Split view se dvěma read-only ScrollArea panely (aktuální vlevo, historická vpravo) s resize handle.
- Diff zvýraznění: zelená pro řádky jen v aktuální verzi, červená pro řádky jen v historické, beze barvy pro společné.
- Navigační šipky (← starší, → novější) s disabled stavem na hranicích seznamu verzí.
- Diff cachovaný per `selected_index` — ne per-frame.
- Barvy respektují dark/light mode.
- Toolbar nahoře: název souboru, dropdown/info o vybrané verzi, navigační šipky, zavírací tlačítko.
- Editor se nekreslí v history mode (`ws.editor.ui()` podmíněné na `history_view.is_none()`).
- `current_content` se načte jednou při otevření history view (ne per-frame z tab bufferu).
- i18n klíče pro navigaci a split view ve všech 5 jazycích (cs, en, sk, de, ru).
- `cargo check` + `./check.sh` prochází.

## Proof Level

- This slice proves: integration (split view rendering nad reálnými snapshot daty z S01)
- Real runtime required: yes (vizuální ověření split view a diff barev v běžícím editoru)
- Human/UAT required: yes (layout, čitelnost diff, UX navigace)

## Verification

- `cargo check` — kompilace bez chyb
- `cargo clippy` — žádné nové warningy
- `./check.sh` — všechny unit testy prochází (S01 testy neporušené)
- Manuální ověření v běžícím editoru:
  - Uložit soubor 3×, pravý klik → "Historie souboru" → split view se zobrazí
  - Aktuální verze vlevo, historická vpravo, diff barvy viditelné
  - Šipky přepínají verze, diff se aktualizuje (ne per-frame — konzole bez výkonnostních warningů)
  - ✕ zavře split view, editor se vrátí do normálu
  - Přepnout dark/light mode — barvy zůstávají čitelné

## Observability / Diagnostics

- Runtime signals: toast při I/O chybě čtení snapshot obsahu (zděděno z S01)
- Inspection surfaces: `HistoryViewState` pole `diff_for_index` — pokud se rovná `selected_index`, diff je cachovaný
- Failure visibility: chybové hlášky v UI panelu při selhání načtení obsahu verze

## Integration Closure

- Upstream surfaces consumed: `HistoryViewState`, `render_history_panel()`, `LocalHistory::get_snapshot_content()`, `LocalHistory::get_history()`, `TabBarAction::ShowHistory`, diff pattern z `diff_view.rs`, split layout z `render/markdown.rs`
- New wiring introduced: podmíněné skrytí `ws.editor.ui()` v history mode, nový `render_history_split_view()` nahrazující `render_history_panel()`
- What remains before milestone is truly usable: S03 (cleanup, edge cases, finální integrace)

## Tasks

- [x] **T01: Implementovat split view s diff zvýrazněním, navigací a cachováním** `est:90m`
  - Why: Celý scope S02 — nahradit jednoduchý history panel plným split view s diff. Scope je koherentní: stav, rendering, navigace, barvy a i18n patří do jednoho kontextu.
  - Files: `src/app/ui/workspace/history/mod.rs`, `src/app/ui/workspace/mod.rs`, `locales/{cs,en,sk,de,ru}/ui.ftl`
  - Do: (1) Rozšířit `HistoryViewState` o `current_content: String`, `cached_diff: Option<Vec<DiffLine>>`, `diff_for_index: Option<usize>`, `split_ratio: f32`. (2) Vytvořit `DiffLine` struct s `(ChangeTag, String)`. (3) Implementovat `compute_diff()` funkci volající `similar::TextDiff::from_lines()` s výstupem do owned `Vec<DiffLine>`, cachovanou per `selected_index`. (4) Implementovat `diff_colors()` funkci s dark/light větvením (pattern z `diff_view.rs` + light varianta). (5) Nahradit `render_history_panel()` → `render_history_split_view()`: toolbar nahoře (název, info verze, šipky ← →, ✕) + split view dole (aktuální vlevo, historická vpravo) s resize handle (pattern z `render/markdown.rs` `split_axis()`). (6) Diff rendering: oba panely renderují plný soubor jako `LayoutJob` s per-řádkovým barvením. (7) V `workspace/mod.rs` podmínit `ws.editor.ui()` na `ws.history_view.is_none()`, upravit ShowHistory handler aby plnil `current_content` z tab bufferu. (8) Přidat i18n klíče pro navigační UI do všech 5 locale souborů. (9) `cargo check` + `cargo clippy` + `./check.sh`.
  - Verify: `cargo check` + `cargo clippy` + `./check.sh` prochází. Manuální spuštění: split view se zobrazí, diff barvy viditelné, šipky fungují, zavření vrátí editor.
  - Done when: Split view s diff zvýrazněním, navigací šipkami a cachovaným diff se renderuje správně, editor se skrývá v history mode, kompilace a testy prochází.

## Files Likely Touched

- `src/app/ui/workspace/history/mod.rs` — rozšíření HistoryViewState, nový render_history_split_view(), DiffLine, compute_diff(), diff_colors()
- `src/app/ui/workspace/mod.rs` — podmíněné skrytí editor.ui(), úprava ShowHistory handleru (current_content), volání render_history_split_view()
- `locales/cs/ui.ftl` — nové i18n klíče pro navigaci
- `locales/en/ui.ftl` — nové i18n klíče pro navigaci
- `locales/sk/ui.ftl` — nové i18n klíče pro navigaci
- `locales/de/ui.ftl` — nové i18n klíče pro navigaci
- `locales/ru/ui.ftl` — nové i18n klíče pro navigaci
