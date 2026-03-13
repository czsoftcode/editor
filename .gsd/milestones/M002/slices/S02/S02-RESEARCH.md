# S02: History Split View s Diff a Navigací — Research

**Date:** 2026-03-13

## Summary

S02 musí nahradit stávající jednoduchý history panel (seznam verzí + monospace náhled) plnohodnotným split view se dvěma read-only panely: aktuální verze vlevo, historická vpravo, s diff zvýrazněním (zelená/červená) a navigací šipkami mezi verzemi. Diff výsledek musí být cachovaný (ne per-frame).

**Hlavní architektonický problém:** Editor renderuje buď normální kód (`ui_normal`) nebo markdown split (`ui_markdown_split`). History split view je třetí rendering path — musí se vložit do rozhodovacího stromu v `Editor::ui()` (soubor `render/ui.rs`) **nebo** se renderovat na úrovni workspace místo editoru (jako overlay v CentralPanel). S01 zvolil overlay přístup — history panel se renderuje v `workspace/mod.rs` řádek 721 **po** editoru. S02 musí tento overlay nahradit split view renderem.

**Doporučení:** Neměnit `Editor::ui()` — history split view renderovat na úrovni workspace. Když `ws.history_view.is_some()`, editor se buď skryje (a split view zabere celý CentralPanel), nebo se editor renderuje jako levý panel a history jako pravý. Varianta "nahradit editor kompletně" je jednodušší a konzistentní s rozhodnutím v DECISIONS.md: "History split view nahradí normální rendering kompletně."

**Klíčové znovupoužití:**
- **Split layout:** `split_axis()` helper z `render/markdown.rs` — dvě `ScrollArea` s drag handle, ratio clamp. Přesný vzor.
- **Diff barvy:** `diff_view.rs` — `bg_added`, `bg_removed`, `fg_added`, `fg_removed` s `LayoutJob`/`TextFormat`. Barvy jsou hardcoded, ne extrahované. S02 je extrahuje do sdílené funkce nebo zkopíruje — extrakce je čistší.
- **Navigace:** Stávající `HistoryViewState` z S01 už drží `selected_index` a `entries`. Šipky změní index, diff se přepočítá.

## Recommendation

Postupovat ve 3 krocích:

1. **Rozšířit `HistoryViewState`** o diff cache: přidat `cached_diff: Option<CachedDiff>` struct s `Vec<(ChangeTag, String)>` a `diff_for_index: Option<usize>` (invalidace při změně `selected_index`). Přidat pole `current_content: String` pro aktuální verzi souboru (načtená jednou při otevření history view). Přidat `split_ratio: f32` pro resize handle.

2. **Nahradit `render_history_panel()`** novou funkcí `render_history_split_view()` v `workspace/history/mod.rs`. Layout: horizontální split — levý panel (30%) se seznamem verzí + navigačními šipkami, pravý panel (70%) s dvěma vertikálními sub-panely (aktuální nahoře/vlevo, historická dole/vpravo) s diff zvýrazněním. Alternativa: hlavní split je 50/50 (aktuální vlevo, historická vpravo) s verzí list jako toolbar nahoře.

3. **Diff barvy extrahovat** z `diff_view.rs` do sdílené utility (např. `src/app/ui/editor/diff_colors.rs` nebo přímo do `workspace/history/mod.rs` jako privátní funkce). Barvy respektují dark/light mode — `diff_view.rs` má hardcoded dark-mode barvy, S02 musí přidat light-mode větev.

**Layout doporučení:** Třísloupový layout je příliš složitý. Lepší: toolbar nahoře (seznam verzí jako dropdown/šipky + info o vybrané verzi + zavírací tlačítko), pod ním split view (aktuální vlevo, historická vpravo) se sdíleným diff zvýrazněním v obou panelech. Toto je nejblíže konvenčnímu diff UI.

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Text diffing | `similar::TextDiff::from_lines()` (2.7.0, v Cargo.toml) | Stabilní, API ověřené v `diff_view.rs`. |
| Diff barvy + LayoutJob | `diff_view.rs` řádky 51-55, 93-137 | Barvy bg_added/bg_removed, TextFormat pattern. Extrahovat, ne duplikovat. |
| Split layout + drag handle | `render/markdown.rs` `split_axis()` + handle rendering (řádky 90-170) | Ověřený pattern: dvě ScrollArea s resize, cursor icon, dot markers. |
| Timestamp formátování | `workspace/history/mod.rs` `format_timestamp()` + `days_to_date()` | Už implementováno v S01, bez chrono dependency. |
| i18n | Fluent `.ftl` soubory v `locales/{cs,en,sk,de,ru}/` | Standardní pattern: `i18n.get()` / `i18n.get_args()`. |

## Existing Code and Patterns

- `src/app/ui/workspace/history/mod.rs` — S01 výstup. `HistoryViewState` struct s `file_path`, `relative_path`, `entries`, `selected_index`, `preview_content`. `render_history_panel()` renderuje horizontální split (30% seznam, 70% preview). **S02 rozšíří struct a nahradí render funkci.** Formátovací utility (`format_timestamp`, `days_to_date`) se zachovají.
- `src/app/ui/editor/diff_view.rs` — AI diff modal. Klíčový pattern: `TextDiff::from_lines()` → `iter_all_changes()` → `ChangeTag` match → `LayoutJob` s `TextFormat` (font_id, color, background). Side-by-side i inline mód. **Diff logiku znovupoužít, UI kontejner (modal) zahodit.**
- `src/app/ui/editor/render/markdown.rs` — Markdown split view. `split_axis()` closure (řádek 90): `(total - handle_size).max(0.0)` → ratio clamp 50-usable. Handle rendering: `allocate_exact_size` + `Sense::drag` + `drag_delta` + ratio update. **Přesný vzor pro history split handle.**
- `src/app/ui/workspace/mod.rs` řádek 721 — Místo kde se history panel renderuje jako overlay. `ws.history_view.as_mut().unwrap()` → `render_history_panel()`. **S02 změní toto na `render_history_split_view()` s rozšířeným state.** Borrow split pattern (separate `&mut history_view` a `&local_history`) je ověřený.
- `src/app/ui/editor/mod.rs` — `Editor::ui()` rozhoduje markdown vs normal rendering. **S02 nemění tento kód** — history view se renderuje na workspace úrovni.
- `src/app/local_history.rs` — `get_snapshot_content(rel_path, entry) -> io::Result<String>` a `get_history(rel_path) -> Vec<HistoryEntry>`. Obojí S02 přímo používá.
- `src/app/ui/workspace/state/mod.rs` řádek 142 — `history_view: Option<HistoryViewState>`. Místo kde žije stav. S02 rozšíří struct, ne workspace state.

## Constraints

- **`TextDiff` borrowuje vstupní stringy** — `TextDiff::from_lines(&old, &new)` drží reference na `old` a `new`. Pro cachování musím uložit výsledek jako `Vec<(ChangeTag, String)>` (owned), ne držet `TextDiff` struct. Alternativa: držet `old_content` a `new_content` vedle `TextDiff`, ale lifetime komplikace s egui rendering. **Owned vec je bezpečnější.**
- **Diff se musí počítat jen při změně vybrané verze** — ne per-frame. `HistoryViewState` bude mít `diff_for_index: Option<usize>`. Pokud `selected_index == diff_for_index`, diff se nespočítá znovu. Invalidace při navigaci šipkami.
- **Barvy musí respektovat dark/light mode** — `diff_view.rs` má hardcoded RGBA pro dark mode (zelená `rgba(40,100,40,100)`, červená `rgba(120,30,30,100)`). S02 musí přidat light mode větev s jasnějšími/kontrastnějšími barvami (pattern z `git.rs` light variant colors).
- **History panely jsou read-only** — žádný `TextEdit`, jen `Label` s `LayoutJob`. Žádná interakce s LSP, cursor, search.
- **Borrow checker kolem WorkspaceState** — `render_history_split_view()` musí přijímat split reference (pattern z S01: `&mut HistoryViewState` + `&LocalHistory` + `&mut Ui` + `&I18n`). Nesmí borrowovat celé `ws`.
- **`cargo check` + `./check.sh` musí projít** po každé změně.
- **i18n: 5 jazyků** (cs, en, sk, de, ru) — nové klíče pro navigační šipky, split view labels.
- **Nezávislý scroll dvou panelů** — rozhodnutí z DECISIONS.md. Žádný sync scroll.
- **Editor se skryje v history mode** — rozhodnutí z DECISIONS.md: "history split view nahradí normální rendering kompletně". Ale S01 renderuje history panel **po** editoru (overlay). S02 musí buď: (a) podmínit `ws.editor.ui()` na `ws.history_view.is_none()`, nebo (b) skrýt editor jinak. Varianta (a) je čistší — editor se nekreslí vůbec, split view zabere celý CentralPanel.

## Common Pitfalls

- **Diff per-frame rendering** — `TextDiff::from_lines()` je O(n*d). Pro 1000 řádků soubor s 50 změnami to je ~ms, ale per-frame (60fps) je to 60ms/s zbytečné CPU. **Mitigace:** Cachovat diff výsledek jako `Vec<DiffLine>` s `(ChangeTag, String)`. Přepočítat jen když se změní `selected_index`.
- **Velký soubor v ScrollArea** — Celý soubor se renderuje jako `Label` s `LayoutJob`. Pro 10k+ řádků může být pomalý. **Mitigace:** egui `ScrollArea` s `show_rows()` by byl ideální, ale LayoutJob s celým textem to nekomplikuje — egui interně clipuje rendering. Monitorovat výkon.
- **History view stav při zavření tabu** — Pokud uživatel zavře tab, který je zobrazený v history view, stav musí být vyčištěn. **S02 scope:** přidat guard do `ShowHistory` handleru, ale **full edge case handling (close_tab check) je S03 scope**.
- **Split ratio persistence** — `md_split_ratio` v editoru žije v `Editor` struct. History split ratio bude v `HistoryViewState` — ephemeral, nezachová se přes zavření/otevření. To je OK pro S02.
- **Borrow conflict při čtení tab content + history content** — Levý panel potřebuje `ws.editor.tabs[idx].content` (aktuální verze). Pravý panel potřebuje `ws.local_history.get_snapshot_content()`. Oba současně. **Mitigace:** Načíst `current_content` do `HistoryViewState` při otevření (klonování), ne při každém frame. Aktuální verze se změní jen pokud uživatel uloží — ale v history mode editor neběží, takže content se nemění.
- **Navigace šipkami: bounds check** — `selected_index` nesmí jít pod 0 nebo nad `entries.len() - 1`. Šipka vlevo/nahoru = starší verze (vyšší index), šipka vpravo/dolů = novější (nižší index). UI musí disable šipku na hranici.

## Open Risks

- **Vizuální kvalita diff zvýraznění** — Barvy z `diff_view.rs` jsou navržené pro dark mode modal. V inline split view s jiným pozadím (panel_fill) mohou vypadat jinak. Vyžaduje vizuální ověření v běžícím editoru (UAT). Risk: medium, mitigace: parametrizovat barvy s dark/light větvením.
- **Výkon LayoutJob pro velké diff** — Celý soubor (aktuální i historický) se renderuje jako jeden LayoutJob s per-řádkovým barvením. Pro soubory nad ~5000 řádků může být LayoutJob construction pomalý. **Mitigace:** Měřit, a pokud je problém, přejít na `grouped_ops()` s kontextovými řádky (pattern z similar docs). Ale to mění UX — uživatel nevidí celý soubor, jen chunky. Pro S02 začít s plným souborem.
- **Editor skrytí v history mode** — Pokud `ws.editor.ui()` neprobíhá (podmíněno `history_view.is_none()`), editor ztratí per-frame update (autosave timer, LSP debounce). **Mitigace:** Autosave a LSP sync se dějí v `editor.ui()` volání. Pokud editor neběží, autosave se zastaví (akceptovatelné — uživatel prohlíží historii, nedituje). LSP hover/completion stav se stane stale (akceptovatelné — v history mode není potřeba). Ověřit, že po zavření history view editor obnoví normální stav.

## Candidate Requirements (advisory)

- **CR-1:** Split view s diff zvýrazněním: aktuální verze vlevo (zelená pro řádky přítomné jen v aktuální), historická vpravo (červená pro řádky přítomné jen v historické), společné řádky bez barvy.
- **CR-2:** Navigační šipky (← starší, → novější) s disabled stavem na hranicích.
- **CR-3:** Diff cachovaný per selected_index — ne per-frame.
- **CR-4:** Zavírací tlačítko vrátí editor do normálního režimu.
- **CR-5:** i18n klíče pro navigaci a split view ve všech 5 jazycích.
- **CR-6:** Barvy respektují dark/light mode.

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| egui/eframe | — | None found (žádné relevantní egui skills na skills.sh) |
| similar (diff) | — | None found (library docs dostupné přes Context7) |
| Rust desktop | `bobmatnyc/claude-mpm-skills@rust-desktop-applications` | Available — generický, nepotřebný pro tento scope |

## Sources

- `similar` crate API: `TextDiff::from_lines()`, `iter_all_changes()`, `grouped_ops()` (source: [Context7 similar docs](https://context7.com/mitsuhiko/similar))
- egui `ScrollArea`, `LayoutJob`, `TextFormat` patterns (source: codebase `diff_view.rs`, `render/markdown.rs`)
- Codebase exploration: `workspace/mod.rs`, `workspace/history/mod.rs`, `editor/mod.rs`, `render/normal.rs`, `render/markdown.rs`, `diff_view.rs`, `local_history.rs`, `state/mod.rs`
