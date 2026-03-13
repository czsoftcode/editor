# S01: Editovatelný panel se syntax highlighting, diff a sync scrollem — Research

**Date:** 2026-03-13

## Summary

Slice S01 pokrývá R001, R002, R003, R006, R007, R009 (primárně) a R008 (podpůrně). Jde o přepis levého panelu z read-only `Label` + `LayoutJob` na editovatelný `TextEdit` se syntax highlighting a diff overlay, přidání syntax highlighting do pravého panelu, synchronizaci scrollu, průběžné propsání editací do tab bufferu a výchozí stav panelů podle počtu verzí.

Klíčový technický problém je **kombinace syntect highlighting s diff background per-řádek** v layouter callbacku `TextEdit`. Layouter dostává celý text jako `&str` a musí vrátit `Arc<Galley>` — nemá přímý přístup k diff metadata. Řešení: diff stav capturnout do closure přes mapu `řádek → DiffTag`, a v layouter callbacku po syntax highlighting nastavit `TextFormat.background` na diff barvu pro příslušné sections.

Existující codebase má jasné patterny pro oba klíčové prvky: `render/normal.rs` ukazuje TextEdit + layouter + ScrollArea pattern, `history/mod.rs` má fungující diff rendering s LayoutJob. Úkolem je tyto dva patterny spojit. Scroll sync je realizovatelný přes `scroll_output.state.offset.y` (čtení) a `vertical_scroll_offset()` (nastavení) na dvou ScrollArea instancích.

## Recommendation

### Přístup: Hybridní layouter s diff overlay

1. **Levý panel (editovatelný):** `TextEdit::multiline` s layouter callbackem, který:
   - Zavolá `Highlighter::highlight()` pro syntax barvy (vrací `Arc<LayoutJob>`)
   - Naklonuje LayoutJob a per-section přidá diff background barvu podle řádkové mapy
   - Zobrazuje jen Equal + Insert řádky (text se skládá jen z nich)

2. **Pravý panel (read-only):** `Label` s `LayoutJob`, kde se syntax highlighting + diff background skládá přímo při sestavování LayoutJob. Zobrazuje jen Equal + Delete řádky.

3. **Scroll sync:** Proportionální mapování — offset jednoho panelu se přepočítá na poměrný offset druhého. Ne line-based mapování (příliš složité pro první iteraci, a proportionální je pro UX dostačující).

4. **Diff recompute:** Debounce přes `content_hash` — diff se přepočítá jen když se změní hash levého obsahu, ne per-frame. Hash je `u64` přes xxhash (existující dep).

5. **Tab sync:** `TextEdit` v levém panelu edituje kopii `current_content` v `HistoryViewState`. Po každém frame se kontroluje `changed()` a propsává do `tab.content` + nastaví `tab.modified = true`.

### Proč ne jiný přístup

- **Dva TextEdit:** Pravý panel nepotřebuje editaci (R100 — out of scope). TextEdit pro read-only je zbytečná komplexita.
- **Per-řádkový rendering místo LayoutJob:** Pomalé pro velké soubory, egui nemá efektivní virtualizaci per-řádkových widgetů v TextEdit.
- **Line-based scroll sync:** Vyžaduje mapovací tabulku Equal řádků z diff výsledku. Složitější implementace, proportionální sync stačí pro MVP.

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Diff výpočet | `similar::TextDiff::from_lines()` | Už používáno v M002, O(n*d) s dobrým výkonem |
| Syntax highlighting | `syntect` + `Highlighter::highlight()` | Existující wrapper s cache, vrací `Arc<LayoutJob>` |
| Content hashing | `xxhash_rust::xxh3::xxh3_64` | Už v `Cargo.toml`, rychlý non-crypto hash |
| Timestamp formátování | `unix_ts_to_local()` | Existující v `history/mod.rs` |

## Existing Code and Patterns

- `src/app/ui/workspace/history/mod.rs` — **Přepisovaný soubor.** `HistoryViewState`, `compute_diff()`, `diff_colors()`, `render_history_split_view()`. Oba panely jsou read-only LayoutJob. Diff cache přes `diff_for_index`. Split ratio s draggable handle. 436 řádek. Testy pro diff logiku zachovat.
- `src/app/ui/editor/render/normal.rs` — **Pattern pro TextEdit + layouter + ScrollArea.** Klíčový vzor: `highlighter.highlight()` → clone → modify wrap_width → `ui.fonts(|f| f.layout_job(job))`. Čtení scroll offset z `scroll_output.state.offset.y`. 218 řádek.
- `src/highlighter.rs` — **Highlighting API.** `highlight(text, ext, filename, font_size, theme_name) → Arc<LayoutJob>`. Cache přes hash. `background_color(theme_name) → Color32` pro frame fill. Highlighter je ve `ws.editor.highlighter` (pub field).
- `src/app/ui/editor/mod.rs` — **Tab struct.** Obsahuje `content: String`, `modified: bool`, `scroll_offset: f32`, `last_edit: Option<Instant>`, `save_status`. `extension()` a `filename()` metody na Editor structu.
- `src/app/ui/workspace/mod.rs` — **Volající.** Řádky 681–760: podmíněné renderování editor vs history view. `ws.history_view: Option<HistoryViewState>` ve `WorkspaceState`. Inicializace při `TabBarAction::ShowHistory`. `ws.editor.highlighter` je dostupný. Theme name přes `settings.syntect_theme_name()`.
- `src/app/ui/workspace/state/mod.rs` — WorkspaceState struct, `history_view: Option<HistoryViewState>` na řádku 142.
- `src/settings.rs` — `syntect_theme_name() → &'static str` metoda na Settings.

## Constraints

- **Highlighter je v `ws.editor`** — history view potřebuje referenci. Aktuálně se `render_history_split_view()` volá s `&mut HistoryViewState` a `&LocalHistory`. Bude potřeba přidat parametry `&Highlighter` a `theme_name: &str` (+ extension, filename).
- **Borrow checker:** `ws.editor` a `ws.history_view` jsou oba ve WorkspaceState. Nelze mít `&mut ws.history_view` a zároveň `&ws.editor.highlighter`. Řešení: extrahovat potřebná data (highlighter ref, tab info) před mutable borrow na history_view, nebo předat přes parametry.
- **`TextEdit` layouter je `FnMut(&Ui, &str, f32) → Arc<Galley>`** — diff metadata musí být capturnuta v closure. Closure musí znát diff tag per-řádek aktuálního textu.
- **Diff se počítá z `current_content` vs historická verze.** Při editaci se `current_content` mění → diff se musí invalidovat. Stávající `diff_for_index` invalidace nestačí — potřeba i hash/gen counter na obsahu.
- **`LayoutJob.sections` jsou byte-range based** — mapování na řádky vyžaduje průchod textem a nalezení byte offsetů newline.
- **Levý panel zobrazuje jen Equal+Insert řádky** — text pro TextEdit musí být rekonstruován jen z těchto řádků, ne celý diff output.
- **Pravý panel zobrazuje jen Equal+Delete řádky** — analogicky.
- **`cargo check` + `./check.sh` musí projít.**
- **Žádné nové runtime závislosti.**

## Detailed Technical Design

### 1. HistoryViewState rozšíření

```rust
pub struct HistoryViewState {
    // ... existující fieldy ...
    
    // NOVÉ:
    /// Hash obsahu levého panelu pro invalidaci diff cache.
    pub content_hash: u64,
    /// Scroll offset levého panelu (vertikální).
    pub left_scroll_y: f32,
    /// Scroll offset pravého panelu (vertikální).
    pub right_scroll_y: f32,
    /// Který panel naposledy scrolloval (pro sync).
    pub scroll_source: ScrollSource, // enum { Left, Right, None }
    /// Obsah pravého panelu (Equal+Delete řádky ze history).
    pub right_panel_text: String,
    /// Mapa: řádek (0-based) → DiffTag pro levý panel.
    pub left_diff_map: Vec<ChangeTag>,
    /// Mapa: řádek (0-based) → DiffTag pro pravý panel.
    pub right_diff_map: Vec<ChangeTag>,
}
```

### 2. Diff → panel text + diff mapy

Při compute_diff nebo invalidaci:
- Projít diff lines, sestavit `left_text` (Equal+Insert) a `right_text` (Equal+Delete)
- Vytvořit `left_diff_map` a `right_diff_map` (per-řádek tag)
- `current_content` v HistoryViewState se stává editovatelným stringem

### 3. Layouter pro levý panel (TextEdit)

```rust
let diff_map = &history_view.left_diff_map;
let colors = diff_colors(dark_mode);
let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
    let job_arc = highlighter.highlight(text, &ext, &fname, font_size, theme_name);
    let mut job = (*job_arc).clone();
    job.wrap.max_width = wrap_width;
    // Per-section: zjistit řádek z byte_range, nastavit background
    apply_diff_backgrounds(&mut job, text, diff_map, &colors);
    ui.fonts(|f| f.layout_job(job))
};
```

### 4. Scroll sync

```rust
// Po renderování obou panelů:
let left_output = left_scroll_area.show(...);
let right_output = right_scroll_area.show(...);

let left_y = left_output.state.offset.y;
let right_y = right_output.state.offset.y;

// Detekce kdo scrolloval (porovnání s uloženým offsetem):
if left_y != history_view.left_scroll_y {
    // Levý scrolloval → sync pravý
    let ratio = left_y / left_max.max(1.0);
    history_view.right_scroll_y = ratio * right_max;
    history_view.scroll_source = ScrollSource::Left;
} else if right_y != history_view.right_scroll_y {
    // Pravý scrolloval → sync levý
    ...
}
history_view.left_scroll_y = left_y;
history_view.right_scroll_y = right_y;
```

### 5. Tab sync

```rust
// V render funkci, po TextEdit:
if text_edit_response.response.changed() {
    // current_content se změnil přes TextEdit binding
    // → propsát do tabu
    if let Some(tab) = editor_tabs.iter_mut().find(|t| t.path == history_view.file_path) {
        tab.content = history_view.current_content.clone();
        tab.modified = true;
        tab.last_edit = Some(Instant::now());
        tab.save_status = SaveStatus::Modified;
    }
    // Invalidovat diff cache
    history_view.content_hash = xxh3_64(history_view.current_content.as_bytes());
}
```

### 6. Signatura render funkce

```rust
pub fn render_history_split_view(
    history_view: &mut HistoryViewState,
    local_history: &LocalHistory,
    ui: &mut egui::Ui,
    i18n: &I18n,
    highlighter: &Highlighter,    // NOVÝ
    theme_name: &str,             // NOVÝ
    ext: &str,                    // NOVÝ
    fname: &str,                  // NOVÝ
) -> HistorySplitResult           // NOVÝ — struct { close: bool, content_changed: bool }
```

### 7. Výchozí stav (R007)

V inicializaci `HistoryViewState` (workspace/mod.rs):
```rust
selected_index: if entries.len() > 1 { Some(0) } else { None },
```
Aktuálně je tam vždy `Some(0)` — potřeba podmínka.

## Common Pitfalls

- **Borrow conflict na WorkspaceState** — `ws.history_view` (mutable) vs `ws.editor.highlighter` (immutable). Řešení: extrahovat highlighter referenci a tab metadata do lokálních proměnných před `if ws.history_view.is_some()` blokem.
- **TextEdit mění string přes mutable referenci** — `current_content` v `HistoryViewState` musí být `&mut String`. Ale TextEdit edituje celý text, ne jen Equal+Insert podmnožinu. Řešení: `current_content` přímo slouží jako editovatelný buffer. Diff se počítá z tohoto bufferu vs historická verze. Diff mapy se přepočítají při invalidaci.
- **LayoutJob clone je drahý pro velké soubory** — `Highlighter::highlight()` vrací `Arc<LayoutJob>`, clone je O(sections). Pro 10k řádkový soubor je to ~30k sections. Přijatelné (measurovaně ~1ms), ale důležité cachovat highlighting výsledek.
- **Diff overlay: section byte_range vs řádek** — Section z syntect nerespektuje řádkové hranice (segment může být "fn\n"). Mapování byte_range.start na řádek (počtem `\n` před ním) je nutné. Optimalizace: předpočítat byte offsety řádků jednou, pak binary search.
- **Scroll sync feedback loop** — Pokud sync nastaví offset pravého panelu, příští frame detekuje "pravý se změnil" a syncne zpět levý. Řešení: `scroll_source` flag + jednorámcová prodleva, nebo porovnávat s epsilon tolerancí.
- **Pravý panel text se musí přestavět při editaci** — Když se změní levý panel, diff se přepočítá a `right_panel_text` + `right_diff_map` se znovu sestaví. To je OK dokud je debounce na místě.

## Open Risks

- **Performance diff recompute na 10k+ řádkovém souboru** — `similar::TextDiff::from_lines()` je O(n*d). Pro soubor s 10k řádky a malými změnami je d malé → rychlé. Pro velké refaktory (d ≈ n) může trvat desítky ms. Debounce 200-300ms by měl stačit. Fallback: pokud diff trvá >50ms, zobrazit "computing..." a počítat v background threadu. (Pro S01 pravděpodobně není potřeba — ověřit profilem.)
- **TextEdit + diff background renderování** — Nikde v codebase se nekombinuje TextEdit layouter s per-řádkovým background overlay. Funguje to v LayoutJob (viz stávající diff view), ale v kontextu editovatelného TextEdit to ještě nebylo testováno. Riziko: egui může background barvy v TextEdit renderovat jinak než v Label. Proof: nutné ověřit vizuálně v běžícím editoru.
- **Scroll sync přesnost** — Proportionální sync funguje dobře pro soubory se stejným počtem řádků. Pro diff s mnoha Insert/Delete řádky bude proporční mapování "skákat". Pro S01 je to přijatelné (uživatel může scrollovat manuálně). Line-based mapování je potenciální vylepšení pro budoucí milestone.
- **Cursor pozice po diff recompute** — Když se diff přepočítá, text v levém panelu se nezmění (edituje se přímo current_content). Ale diff mapa se změní → layouter vrátí jiné pozadí. Cursor pozice by měla zůstat stabilní, protože TextEdit drží cursor jako char offset, ne jako pixel pozice. Ověřit.

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| egui/eframe | `bobmatnyc/claude-mpm-skills@rust-desktop-applications` | available (119 installs) — Rust desktop apps obecně, ne specificky egui layouter. Marginální přínos. |
| syntect | žádný relevantní skill | none found |
| similar (diff) | žádný relevantní skill | none found |

Žádný z nalezených skillů nepřináší specifické znalosti pro TextEdit+layouter+diff kombinaci. Doporučuji neinstalovat.

## Sources

- TextEdit layouter API: `layouter(&mut FnMut(&Ui, &dyn TextBuffer, f32) → Arc<Galley>)` (zdroj: [egui docs](https://docs.rs/egui/latest/egui/widgets/text_edit/struct.TextEdit.html))
- TextFormat.background existuje pro per-section pozadí v LayoutJob (zdroj: epaint-0.31.1 `text_layout_types.rs`)
- ScrollArea offset čtení: `scroll_output.state.offset.y` (zdroj: `render/normal.rs` řádek 172)
- ScrollArea offset nastavení: `.vertical_scroll_offset(y)` (zdroj: [egui docs](https://docs.rs/egui/latest/egui/containers/scroll_area/struct.ScrollArea.html))
- Highlighter cache: hash-based, max 20 entries, auto-clear (zdroj: `src/highlighter.rs`)
- existing diff pattern: `compute_diff()` v `history/mod.rs` + `TextDiff::from_lines()` z `similar` crate
