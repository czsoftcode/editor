---
id: T01
parent: S01
milestone: M003
provides:
  - ScrollSource enum pro scroll sync
  - PanelTexts struct + build_panel_texts() pro diff→panel konverzi
  - compute_line_offsets() pro byte→řádek mapování
  - apply_diff_backgrounds() pro diff barvy na LayoutJob
  - HistorySplitResult return type
  - Rozšířený HistoryViewState s content_hash, scroll offsety, diff mapami
  - content_hash() helper (xxh3)
key_files:
  - src/app/ui/workspace/history/mod.rs
  - src/app/ui/workspace/mod.rs
key_decisions:
  - byte_offset_to_line používá binary search v předpočítaných line_offsets — O(log n) per section
  - apply_diff_backgrounds ponechává Equal řádky beze změny (zachovává syntax background)
  - content_hash je pub funkce — volající (workspace/mod.rs) ji používá při inicializaci HistoryViewState
patterns_established:
  - PanelTexts jako mezivrstva mezi compute_diff() a renderingem — odděluje diff logiku od UI
  - Per-řádek diff mapa (Vec<ChangeTag>) jako vstup pro layouter overlay
observability_surfaces:
  - content_hash field v HistoryViewState inspektovatelný přes debugger
  - left_diff_map.len() / right_diff_map.len() pro kontrolu diff recompute stavu
duration: 15min
verification_result: passed
completed_at: 2026-03-13
blocker_discovered: false
---

# T01: Datový model, diff→panel logika a helper funkce

**Přidány všechny stavební bloky pro history split view rendering: datové struktury, diff→panel konverze, line offset výpočet a diff background overlay funkce. 11 nových unit testů.**

## What Happened

Implementoval jsem kompletní datovou vrstvu pro nadcházející přepis history view renderingu:

1. **ScrollSource enum** — `Left`, `Right`, `None` s derive `Debug, Clone, Copy, PartialEq`.
2. **HistoryViewState rozšířen** o 7 nových fieldů: `content_hash` (u64, xxh3), `left_scroll_y`/`right_scroll_y` (f32), `scroll_source` (ScrollSource), `right_panel_text` (String), `left_diff_map`/`right_diff_map` (Vec<ChangeTag>).
3. **HistorySplitResult** — return type pro render funkci (close, content_changed).
4. **PanelTexts + build_panel_texts()** — z `Vec<DiffLine>` sestaví levý text (Equal+Insert), pravý text (Equal+Delete), a per-řádek diff mapy pro oba panely.
5. **compute_line_offsets()** — vrací byte offsety začátků řádků, potřebné pro mapování LayoutJob section byte_range na řádek.
6. **byte_offset_to_line()** — interní helper s binary search v line_offsets.
7. **apply_diff_backgrounds()** — projde LayoutJob sections, pro každou section zjistí řádek, nastaví background barvu podle diff mapy. Equal řádky ponechává beze změny.
8. **content_hash()** — pub wrapper nad `xxh3_64`.

Inicializace HistoryViewState v `workspace/mod.rs` doplněna o nové fieldy s výchozími hodnotami.

## Verification

- `cargo check` — ✅ kompilace prochází
- `cargo test -p polycredo-editor -- history` — ✅ 23 testů prošlo (12 existujících + 11 nových)
- `./check.sh` — ✅ fmt + clippy čisté, testy prošly (145 passed). Jeden pre-existující test (`phase35_delete_foundation_scope_guard_has_no_restore_foundation_symbols`) selhal i před mými změnami — hledá chybějící soubor z jiného milestonu.

### Slice-level verification status:
- ✅ `cargo check` — kompilace bez chyb
- ✅ `./check.sh` — clippy + testy prochází (minus pre-existující selhání)
- ✅ `cargo test -p polycredo-editor -- history` — unit testy pro diff→panel logiku a apply_diff_backgrounds
- ⏳ UAT: vizuální kontrola — čeká na T02 (rendering ještě nepoužívá nové funkce)

## Diagnostics

- `HistoryViewState.content_hash` — inspektovatelný přes debugger, změní se po editaci
- `left_diff_map.len()` / `right_diff_map.len()` — měly by odpovídat počtu řádků v příslušném panelu
- `build_panel_texts()` je čistá funkce — snadno testovatelná s libovolným diff vstupem

## Deviations

Žádné.

## Known Issues

- Pre-existující test `phase35_delete_foundation_scope_guard_has_no_restore_foundation_symbols` selhává (chybějící soubor) — nesouvisí s M003.

## Files Created/Modified

- `src/app/ui/workspace/history/mod.rs` — přidán import xxhash, ScrollSource enum, HistorySplitResult, PanelTexts struct, build_panel_texts(), compute_line_offsets(), byte_offset_to_line(), apply_diff_backgrounds(), content_hash(), 11 nových unit testů
- `src/app/ui/workspace/mod.rs` — rozšířena inicializace HistoryViewState o nové fieldy (content_hash, scroll offsety, scroll_source, right_panel_text, diff mapy)
