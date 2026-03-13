---
estimated_steps: 9
estimated_files: 7
---

# T01: Implementovat split view s diff zvýrazněním, navigací a cachováním

**Slice:** S02 — History Split View s Diff a Navigací
**Milestone:** M002

## Description

Nahradit jednoduchý history panel (seznam verzí + monospace náhled) plnohodnotným split view se dvěma read-only panely: aktuální verze vlevo, historická vpravo, s diff zvýrazněním (zelená přidané, červená odebrané). Toolbar nahoře s navigačními šipkami (← starší, → novější), info o vybrané verzi a zavíracím tlačítkem. Diff výsledek cachovaný per selected_index. Barvy respektují dark/light mode. Editor se nekreslí v history mode.

## Steps

1. **Rozšířit `HistoryViewState`** v `workspace/history/mod.rs`:
   - Přidat `current_content: String` (obsah aktuální verze, načtený jednou při otevření).
   - Přidat `cached_diff: Option<Vec<DiffLine>>` kde `DiffLine` je nový struct `{ tag: ChangeTag, text: String }`.
   - Přidat `diff_for_index: Option<usize>` pro invalidaci cache při změně `selected_index`.
   - Přidat `split_ratio: f32` (výchozí 0.5) pro resize handle.
   - Odstranit `preview_content: Option<String>` (nahrazeno diff logikou).
   - Odstranit `scroll_to_selected: bool` (nepotřebné v novém layoutu).

2. **Implementovat `compute_diff()`** v `workspace/history/mod.rs`:
   - Přijímá `current: &str`, `historical: &str`.
   - Volá `similar::TextDiff::from_lines(historical, current)` — historical je "old", current je "new".
   - Iteruje `diff.iter_all_changes()`, pro každou změnu vytvoří `DiffLine { tag, text: change.value().to_string() }`.
   - Vrací `Vec<DiffLine>`.

3. **Implementovat `diff_colors()`** v `workspace/history/mod.rs`:
   - Přijímá `dark_mode: bool`.
   - Dark mode: barvy z `diff_view.rs` — `bg_added(40,100,40,100)`, `bg_removed(120,30,30,100)`, `fg_added(150,255,150)`, `fg_removed(255,150,150)`.
   - Light mode: jasnější barvy — `bg_added(200,240,200,255)`, `bg_removed(255,210,210,255)`, `fg_added(0,100,0)`, `fg_removed(150,0,0)`.
   - Vrací struct `DiffColors { bg_added, bg_removed, fg_added, fg_removed, fg_normal: Color32 }`.

4. **Implementovat `render_history_split_view()`** — hlavní renderovací funkce nahrazující `render_history_panel()`:
   - **Toolbar** (horní řádek): název souboru (heading), info o vybrané verzi (timestamp), navigační šipky (← →) s disabled stavem, zavírací tlačítko (✕). Šipka ← = starší (index+1), šipka → = novější (index-1). Disabled na hranicích.
   - **Split view** (zbytek prostoru): horizontální split s resize handle (pattern z `render/markdown.rs` `split_axis()`).
     - Levý panel: aktuální verze s diff barvami (zelená pro Insert řádky, beze barvy pro Equal).
     - Pravý panel: historická verze s diff barvami (červená pro Delete řádky, beze barvy pro Equal).
   - Diff rendering: pro každý panel vytvořit `LayoutJob` s per-řádkovým `TextFormat` barvením. Levý panel zobrazuje řádky s tagem `Equal` a `Insert` (přeskočí `Delete`). Pravý panel zobrazuje `Equal` a `Delete` (přeskočí `Insert`).
   - Resize handle: `allocate_exact_size` + `Sense::drag` + `drag_delta` + ratio update (vzor z markdown.rs).
   - Oba panely v `ScrollArea::both()` s `auto_shrink([false, false])`.

5. **Diff cache logika** v `render_history_split_view()`:
   - Před renderingem zkontrolovat: pokud `diff_for_index != selected_index`, zavolat `compute_diff()` a uložit výsledek do `cached_diff`, aktualizovat `diff_for_index`.
   - Při navigaci šipkami (změna `selected_index`) se diff přepočítá v dalším frame.

6. **Upravit `ShowHistory` handler** v `workspace/mod.rs`:
   - Při otevření history view načíst `current_content` z `ws.editor.tabs[idx].content.clone()`.
   - Automaticky vybrat první verzi (`selected_index = Some(0)`) a načíst její obsah.
   - Nové pole `HistoryViewState` inicializovat (`split_ratio: 0.5`, `cached_diff: None`, `diff_for_index: None`).

7. **Podmínit editor rendering** v `workspace/mod.rs`:
   - Obalit `ws.editor.ui()` volání do `if ws.history_view.is_none() { ... }` bloku.
   - Zachovat zpracování `editor_res` — přesunout `editor_res` do `Option`, v history mode je `None`.
   - Nahradit `render_history_panel()` voláním `render_history_split_view()`.

8. **Přidat i18n klíče** do `locales/{cs,en,sk,de,ru}/ui.ftl`:
   - `history-nav-older` — tooltip/label pro šipku ← (cs: "Starší verze", en: "Older version", ...)
   - `history-nav-newer` — tooltip/label pro šipku → (cs: "Novější verze", en: "Newer version", ...)
   - `history-current-label` — label pro levý panel (cs: "Aktuální", en: "Current", ...)
   - `history-historical-label` — label pro pravý panel (cs: "Historická", en: "Historical", ...)
   - `history-version-info` — info s parametrem `$date` (cs: "Verze z { $date }", en: "Version from { $date }", ...)

9. **Verifikace**: `cargo check` + `cargo clippy` + `./check.sh`. Manuální spuštění editoru pro vizuální ověření.

## Must-Haves

- [ ] `HistoryViewState` rozšířen o `current_content`, `cached_diff`, `diff_for_index`, `split_ratio`
- [ ] `DiffLine` struct + `compute_diff()` funkce s owned výstupem
- [ ] `DiffColors` struct + `diff_colors(dark_mode)` s dark/light větvením
- [ ] `render_history_split_view()` s toolbar (šipky, info, ✕) a split panely
- [ ] Diff rendering: levý panel (Equal+Insert), pravý panel (Equal+Delete) s barevným LayoutJob
- [ ] Resize handle mezi panely (pattern z markdown.rs)
- [ ] Navigační šipky s disabled stavem na hranicích
- [ ] Diff cache: přepočet jen při změně `selected_index`
- [ ] Editor.ui() podmíněné na `history_view.is_none()`
- [ ] `current_content` a první verze načteny při otevření history view
- [ ] i18n klíče ve všech 5 jazycích
- [ ] `cargo check` + `cargo clippy` + `./check.sh` prochází

## Verification

- `cargo check` — kompilace bez chyb
- `cargo clippy` — žádné nové warningy
- `./check.sh` — všechny existující testy prochází (S01 testy neporušené)
- Manuální: split view se zobrazí po "Historie souboru", dva panely s diff barvami, šipky přepínají verze, ✕ zavře

## Observability Impact

- Signals added/changed: diff cache hit/miss je viditelný přes `diff_for_index == selected_index` stav v `HistoryViewState`
- How a future agent inspects this: `HistoryViewState` pole v debuggeru, visual split view v GUI
- Failure state exposed: I/O chyby při čtení snapshot obsahu se zobrazí inline v panelu (zděděno z S01 pattern)

## Inputs

- `src/app/ui/workspace/history/mod.rs` — stávající `HistoryViewState` a `render_history_panel()` z S01
- `src/app/ui/workspace/mod.rs` — `ShowHistory` handler a overlay rendering z S01
- `src/app/ui/editor/diff_view.rs` — vzor pro diff barvy a LayoutJob rendering
- `src/app/ui/editor/render/markdown.rs` — vzor pro split_axis() a resize handle
- `src/app/local_history.rs` — `get_snapshot_content()` a `get_history()` API

## Expected Output

- `src/app/ui/workspace/history/mod.rs` — rozšířený `HistoryViewState`, nové structy `DiffLine`/`DiffColors`, `compute_diff()`, `diff_colors()`, `render_history_split_view()` nahrazující `render_history_panel()`
- `src/app/ui/workspace/mod.rs` — podmíněný editor rendering, upravený ShowHistory handler, volání `render_history_split_view()`
- `locales/{cs,en,sk,de,ru}/ui.ftl` — 5 nových i18n klíčů v každém souboru
