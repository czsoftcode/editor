---
id: S01
parent: M003
milestone: M003
provides:
  - Editovatelný levý panel (TextEdit+layouter) se syntax highlighting + diff background
  - Read-only pravý panel (Label+LayoutJob) se syntax highlighting + diff background
  - Proportionální scroll sync mezi panely (epsilon 1.0px, ScrollSource flag)
  - Tab sync — editace v history view → tab.content + tab.modified průběžně
  - Výchozí stav panelů (1 verze → None, >1 → Some(0))
  - Diff cache invalidace přes content_hash (xxh3_64) při editaci
  - HistorySplitResult return type pro signalizaci volajícímu
  - Rozšířená signatura render_history_split_view(&Highlighter, theme_name, ext, fname, font_size)
  - PanelTexts + build_panel_texts() + apply_diff_backgrounds() + compute_line_offsets() jako testovatelné stavební bloky
requires:
  - slice: none
    provides: none
affects:
  - S02
key_files:
  - src/app/ui/workspace/history/mod.rs
  - src/app/ui/workspace/mod.rs
key_decisions:
  - "TextEdit+layouter+apply_diff_backgrounds pattern: syntax highlighting z Highlighter::highlight() klonovaný a modifikovaný diff overlay v layouter closure"
  - "Borrow checker vyřešen extrakcí tab metadata do lokálních proměnných; highlighter ref je disjoint borrow — bez unsafe"
  - "Scroll sync přes proportionální mapování s epsilon 1.0px a ScrollSource flag pro prevenci feedback loop"
  - "Diff cache invalidace přes content_hash (xxh3_64) — diff se přepočítá jen při reálné změně obsahu"
  - "apply_diff_backgrounds() používá binary search v předpočítaných line_offsets — O(log n) per section"
  - "Diff mapy (Vec<ChangeTag>) klonované před closure capture — levný clone (Vec<u8-sized enum>)"
  - "Tab sync probíhá ve workspace/mod.rs po render funkci, ne uvnitř ní — render nemá přístup k tab"
patterns_established:
  - PanelTexts jako mezivrstva mezi compute_diff() a renderingem — odděluje diff logiku od UI
  - Per-řádek diff mapa (Vec<ChangeTag>) jako vstup pro layouter overlay
  - TextEdit+layouter+apply_diff_backgrounds — editovatelný widget s per-řádek background barvami a syntax highlighting
  - HistorySplitResult jako return type pro signalizaci akcí z history view volajícímu
observability_surfaces:
  - HistoryViewState.content_hash — změní se po editaci, inspektovatelný přes debugger
  - HistorySplitResult.content_changed — signalizuje potřebu tab sync
  - left_diff_map.len() / right_diff_map.len() — měly by odpovídat počtu řádků v příslušném panelu
  - Pokud diff mapy jsou prázdné ale text není → diff recompute selhal
drill_down_paths:
  - .gsd/milestones/M003/slices/S01/tasks/T01-SUMMARY.md
  - .gsd/milestones/M003/slices/S01/tasks/T02-SUMMARY.md
duration: 40min
verification_result: passed
completed_at: 2026-03-13
---

# S01: Editovatelný panel se syntax highlighting, diff a sync scrollem

**Kompletní přepis history split view — levý panel je editovatelný TextEdit se syntax highlighting a diff barvami, pravý panel je read-only se syntax+diff, scroll synchronizovaný, editace se průběžně propsávají do tab bufferu.**

## What Happened

Slice implementována ve 2 tascích:

**T01 (datový model a helpers):** Přidány všechny stavební bloky — `ScrollSource` enum, rozšířený `HistoryViewState` (content_hash, scroll offsety, diff mapy), `PanelTexts` + `build_panel_texts()` pro konverzi diff→panel texty + diff mapy, `compute_line_offsets()` pro byte→řádek mapování, `apply_diff_backgrounds()` pro diff overlay na LayoutJob sections (binary search), `content_hash()` wrapper nad xxh3_64, `HistorySplitResult` return type. 11 nových unit testů.

**T02 (rendering a integrace):** Přepis `render_history_split_view()` — levý panel z Label+LayoutJob na TextEdit+layouter (highlight + diff overlay), pravý panel z monochrome na Label+LayoutJob se syntax+diff. Scroll sync přes proportionální mapování s epsilon tolerancí a ScrollSource flag. Tab sync v workspace/mod.rs přes HistorySplitResult.content_changed. Borrow checker vyřešen extrakcí metadata do lokálních proměnných, highlighter jako disjoint borrow. Výchozí stav: `selected_index = None` pro 1 verzi, `Some(0)` pro >1.

## Verification

- ✅ `cargo check` — kompilace bez chyb
- ✅ `cargo test` — 145 testů prochází (23 history testů, 8 local_history + 15 UI history)
- ✅ `./check.sh` — fmt + clippy čisté, testy projdou (1 pre-existující selhání `phase35_delete_foundation` mimo scope)
- ⏳ UAT — vizuální kontrola pending (headless prostředí, vyžaduje desktop)

## Requirements Advanced

- R001 — Levý panel je nyní editovatelný TextEdit (ne read-only LayoutJob)
- R002 — Oba panely mají syntax highlighting přes syntect Highlighter::highlight()
- R003 — Scroll synchronizovaný přes proportionální mapování s epsilon tolerancí
- R006 — Editace se průběžně propsávají do tab.content, tab se označí jako modified
- R007 — Výchozí stav: 1 verze → prázdný pravý panel, >1 verze → nejnovější historická
- R009 — Diff zvýraznění v obou panelech kombinované se syntax highlighting

## Requirements Validated

- none (vizuální UAT pending — nelze validovat v headless prostředí)

## New Requirements Surfaced

- none

## Requirements Invalidated or Re-scoped

- none

## Deviations

- `font_size` přidán jako extra parametr render funkce (plán ho nezmiňoval explicitně, ale T02-PLAN step 1 ho uvádí). Předáván z `Editor::current_editor_font_size(ui)`.
- Syntect theme background (`highlighter.background_color(theme_name)`) použit jako fill pro oba panely (Frame) — nebyl v plánu, zlepšuje vizuální konzistenci s hlavním editorem.

## Known Limitations

- Vizuální UAT nelze ověřit v headless prostředí — potenciální rizika:
  - Diff background barvy mohou interferovat se syntax highlighting v TextEdit (dosud nekombinováno v produkci)
  - Scroll sync přesnost pro soubory s výrazně odlišným počtem řádků (proportionální mapování "skáče" místo line-based)
- Scroll sync je proportionální, ne line-based — pro výrazně asymetrické diffy může být UX suboptimální

## Follow-ups

- S02: Tlačítko "Obnovit" + potvrzovací dialog + i18n klíče (závisí na tom, co S01 dodala)
- Po UAT: zvážit line-based scroll sync mapping přes Equal řádky pokud proportionální UX nestačí

## Files Created/Modified

- `src/app/ui/workspace/history/mod.rs` — Kompletní přepis: nové datové struktury (ScrollSource, PanelTexts, HistorySplitResult), helper funkce (build_panel_texts, compute_line_offsets, apply_diff_backgrounds, content_hash), přepsaný render (TextEdit+layouter levý, Label+LayoutJob pravý, scroll sync), 11 nových testů
- `src/app/ui/workspace/mod.rs` — Rozšířené volání s novými parametry, borrow-checker řešení, tab sync po content_changed, podmíněná inicializace selected_index, inicializace nových HistoryViewState fieldů

## Forward Intelligence

### What the next slice should know
- `render_history_split_view()` vrací `HistorySplitResult { close, content_changed }` — S02 potřebuje přidat `restore_requested: bool` nebo podobný signál
- `HistoryViewState` má `selected_index: Option<usize>`, `entries: Vec<HistoryEntry>`, `relative_path: String` — S02 je potřebuje pro restore logiku
- Highlighter, theme_name, ext, fname, font_size se předávají jako parametry — signatura je `#[allow(clippy::too_many_arguments)]`
- `content_hash()` je pub funkce v `history/mod.rs` — S02 ji může použít pokud potřebuje hash kontrolu

### What's fragile
- Scroll sync epsilon (1.0px) — pokud egui změní precision float scrollingu, feedback loop se může vrátit
- Diff mapy se klonují před layouter closure — pokud diff mapy narostou (extrémně velké soubory), clone bude drahý
- Borrow checker řešení v workspace/mod.rs závisí na tom, že highlighter a history_view jsou různé fieldy WorkspaceState — přesun highlighteru by rozbil disjoint borrows

### Authoritative diagnostics
- `HistoryViewState.content_hash` — pokud se nemění po editaci, diff cache invalidace nefunguje
- `left_diff_map.len()` vs počet řádků levého panelu — mismatch = build_panel_texts() bug
- `HistorySplitResult.content_changed` + `tab.modified` — pokud jedno je true a druhé false = tab sync selhal

### What assumptions changed
- Plán předpokládal že font_size není potřeba v signaturě — ukázalo se že ano, pro konzistentní velikost textu v panelech
- Syntect theme background jako panel fill nebyl plánován ale zlepšuje vizuální konzistenci
