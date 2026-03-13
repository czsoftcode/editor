---
id: T02
parent: S01
milestone: M003
provides:
  - Editovatelný levý panel (TextEdit+layouter) se syntax highlighting + diff background
  - Read-only pravý panel (Label+LayoutJob) se syntax highlighting + diff background
  - Proportionální scroll sync mezi panely
  - Tab sync — editace v history view → tab.content + tab.modified
  - Výchozí stav panelů podle počtu verzí (1 verze → None, >1 → Some(0))
  - Nová signatura render_history_split_view s highlighter parametry + HistorySplitResult return
key_files:
  - src/app/ui/workspace/history/mod.rs
  - src/app/ui/workspace/mod.rs
key_decisions:
  - Borrow checker vyřešen extrakcí highlighter ref a tab metadata do lokálních proměnných před mutable borrow na history_view — bez unsafe, bez přesunu highlighteru do WorkspaceState
  - Diff mapy a right_panel_text se klonují před closure capture v layouteru (diff_map clone je levný — Vec<ChangeTag> je Vec<u8-sized enum>)
  - Scroll sync používá epsilon 1.0px pro detekci změny + ScrollSource flag pro prevenci feedback loop
  - content_hash invalidace diff cache integrována přímo do render funkce — diff se přepočítá next frame po editaci
patterns_established:
  - TextEdit+layouter+apply_diff_backgrounds pattern — kombinace syntax highlighting z Highlighter::highlight() s per-řádek diff overlay v editovatelném widgetu
  - HistorySplitResult jako return type pro signalizaci akcí z history view volajícímu (close, content_changed)
observability_surfaces:
  - HistoryViewState.content_hash — změní se po editaci, inspektovatelný přes debugger
  - HistorySplitResult.content_changed — signalizuje potřebu tab sync
  - left_diff_map.len() / right_diff_map.len() — měly by odpovídat počtu řádků v příslušném panelu
  - Pokud diff mapy jsou prázdné ale text není → diff recompute selhal
  - Pokud content_changed je true ale tab.modified je false → tab sync selhal
duration: 25min
verification_result: passed
completed_at: 2026-03-13
blocker_discovered: false
---

# T02: Přepis renderingu, scroll sync a napojení na volajícího

**Přepsána `render_history_split_view()` na editovatelný levý panel (TextEdit+layouter+syntax+diff), read-only pravý panel (Label+LayoutJob+syntax+diff), proportionální scroll sync a tab sync s borrow-checker řešením ve workspace/mod.rs.**

## What Happened

Kompletní přepis `render_history_split_view()`:

1. **Signatura** rozšířena o `&Highlighter`, `theme_name`, `ext`, `fname`, `font_size`. Return type změněn z `bool` na `HistorySplitResult { close, content_changed }`.

2. **Diff cache** invalidace rozšířena — kromě `diff_for_index` (navigace mezi verzemi) nyní invaliduje i při změně `content_hash` (editace). Po invalidaci volá `build_panel_texts()` pro sestavení `right_panel_text` a diff map.

3. **Levý panel** přepsán z `Label`+`LayoutJob` na `TextEdit::multiline(&mut history_view.current_content)` s layouter callbackem. Layouter: `highlighter.highlight()` → clone LayoutJob → `apply_diff_backgrounds()` → `ui.fonts(|f| f.layout_job(job))`. Diff mapy se klonují před capture do closure (borrow checker).

4. **Pravý panel** přepsán z monochrome `LayoutJob` na `Label` s `LayoutJob` se syntax highlighting + diff background. Pokud `selected_index` je `None` (1 verze), zobrazí se informační text místo prázdného panelu.

5. **Scroll sync** implementován přes porovnání aktuálních offsetů s uloženými (epsilon 1.0px tolerance), proportionální přepočet, a `ScrollSource` flag pro prevenci feedback loop.

6. **Tab sync** v `workspace/mod.rs` — po `render_history_split_view()` pokud `result.content_changed`, propsání `current_content` do `tab.content`, nastavení `tab.modified = true`, `tab.last_edit`, `tab.save_status = Modified`.

7. **Borrow checker** vyřešen v `workspace/mod.rs` — extrakce `theme_name`, `ext`, `fname`, `font_size` do lokálních proměnných před mutable borrow na `ws.history_view`. `&ws.editor.highlighter` lze předat přímo, protože Rust rozlišuje disjoint borrows na různých fieldech WorkspaceState.

8. **Výchozí stav** — inicializace `selected_index` změněna na podmíněnou: `if entries.len() > 1 { Some(0) } else { None }`.

## Verification

- `cargo check` — kompilace bez chyb ✅
- `cargo fmt` — formátování OK ✅
- `cargo clippy` — prochází (přidán `#[allow(clippy::too_many_arguments)]`) ✅
- `cargo test` — 145 testů prochází ✅ (1 pre-existující selhání v `phase35_delete_foundation` nesouvisí s touto změnou)
- `cargo test -p polycredo-editor -- history` — 23 history testů prochází ✅
- Vizuální kontrola v běžícím editoru — **pending** (Debian prostředí bez GUI, vyžaduje manuální UAT)

### Slice-level verification status:
- `cargo check` ✅
- `./check.sh` (clippy + testy) ✅ (minus pre-existující phase35 selhání)
- `cargo test -- history` ✅
- UAT vizuální kontrola — **pending** (vyžaduje desktop prostředí)

## Diagnostics

- **Debugger breakpoint** na `render_history_split_view` → inspekce `HistoryViewState` fieldů: `content_hash`, `scroll_source`, `left_diff_map.len()`, `right_diff_map.len()`, `left_scroll_y`, `right_scroll_y`
- **Diff recompute failure** → prázdné diff mapy ale neprázdný text = diff nebyl přepočítán
- **Tab sync failure** → `result.content_changed == true` ale `tab.modified == false` = sync kód v workspace/mod.rs se nevykonal

## Deviations

- `font_size` přidán jako extra parametr (plán ho nezmiňoval explicitně v signaturě, ale T02-PLAN step 1 ho uvádí). Předáván z `Editor::current_editor_font_size(ui)`.
- Syntect theme background (`highlighter.background_color(theme_name)`) použit jako fill pro oba panely (Frame) — nebyl v plánu, ale zlepšuje vizuální konzistenci s hlavním editorem.

## Known Issues

- Vizuální UAT pending — nelze ověřit v headless prostředí. Potenciální rizika:
  - Diff background barvy mohou interferovat se syntax highlighting barvami v TextEdit (dosud nekombinováno)
  - Scroll sync přesnost pro soubory s výrazně odlišným počtem řádků (proportionální mapování "skáče")
- Pre-existující selhání `phase35_delete_foundation_scope_guard_has_no_restore_foundation_symbols` — nesouvisí s touto změnou

## Files Created/Modified

- `src/app/ui/workspace/history/mod.rs` — Přepsaná `render_history_split_view()`: nová signatura, diff cache s content_hash invalidací, TextEdit+layouter levý panel, Label+LayoutJob pravý panel, scroll sync, přidán import `Highlighter`
- `src/app/ui/workspace/mod.rs` — Upravené volání s novými parametry, borrow-checker řešení (extrakce dat do lokálních proměnných), tab sync po content_changed, podmíněná inicializace selected_index
