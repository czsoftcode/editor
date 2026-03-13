---
id: M003
provides:
  - Editovatelný levý panel v history view (TextEdit+layouter) se syntax highlighting a diff zvýrazněním
  - Read-only pravý panel se syntax highlighting (syntect) a diff zvýrazněním
  - Diff zvýraznění v obou panelech oproti protějšímu panelu (zelená/červená pozadí + syntax barvy)
  - Synchronizovaný scroll mezi panely (proportionální mapování, epsilon 1.0px, ScrollSource flag)
  - Průběžný tab sync — editace v history view se propsávají do tab.content + tab.modified
  - Tlačítko "Obnovit" s potvrzovacím dialogem (show_modal pattern)
  - Restore jako append — nový snapshot na konec fronty, mezilehlé verze zachovány
  - Výchozí stav panelů podle počtu verzí (1 verze → prázdný pravý panel, >1 → nejnovější historická)
  - Diff cache invalidace přes content_hash (xxh3_64)
  - i18n klíče history-restore-* ve všech 5 jazycích (cs, en, sk, de, ru)
key_decisions:
  - "TextEdit+layouter+apply_diff_backgrounds pattern pro editovatelný panel s diff overlay a syntax highlighting"
  - "Proportionální sync scroll s epsilon tolerancí (1.0px) a ScrollSource flag pro prevenci feedback loop"
  - "Diff cache invalidace přes content_hash (xxh3_64) — diff se přepočítá jen při reálné změně obsahu"
  - "Tab sync ve workspace/mod.rs po render funkci, ne uvnitř ní — render nemá přístup k tab"
  - "Restore signalizace přes HistorySplitResult.restore_confirmed — UI vrací flag, workspace handler provede mutace"
  - "Restore jako append (nový snapshot na konec fronty), ne replace — zachování kompletní historie"
  - "Borrow checker vyřešen extrakcí metadata do lokálních proměnných; highlighter ref je disjoint borrow — bez unsafe"
patterns_established:
  - PanelTexts jako mezivrstva mezi compute_diff() a renderingem — odděluje diff logiku od UI
  - Per-řádek diff mapa (Vec<ChangeTag>) jako vstup pro layouter overlay
  - TextEdit+layouter+apply_diff_backgrounds — editovatelný widget s per-řádek background barvami a syntax highlighting
  - HistorySplitResult jako return type pro signalizaci akcí z history view volajícímu
  - show_modal() pattern pro jednoduché confirm dialogy v history view
observability_surfaces:
  - HistoryViewState.content_hash — změní se po editaci, inspektovatelný přes debugger
  - HistorySplitResult.content_changed — signalizuje potřebu tab sync
  - HistorySplitResult.restore_confirmed — signalizuje úspěšné potvrzení restore
  - left_diff_map.len() / right_diff_map.len() — měly by odpovídat počtu řádků v příslušném panelu
  - eprintln!("[Restore] ...") při selhání čtení historické verze
requirement_outcomes:
  - id: R001
    from_status: active
    to_status: validated
    proof: "Levý panel je TextEdit::multiline s layouterem (history/mod.rs:526). cargo check + 195 testů pass. Vizuální UAT pending (headless prostředí)."
  - id: R002
    from_status: active
    to_status: validated
    proof: "Oba panely používají Highlighter::highlight() — levý přes TextEdit layouter, pravý přes Label+LayoutJob. cargo check čistý."
  - id: R003
    from_status: active
    to_status: validated
    proof: "ScrollSource enum + proportionální mapování s epsilon 1.0px (history/mod.rs:615-641). Unit testy pro scroll logiku pass."
  - id: R004
    from_status: active
    to_status: validated
    proof: "Restore flow: workspace/mod.rs:813-836 — get_snapshot_content → tab.content = historical → take_snapshot (append) → refresh entries → selected_index=Some(0). Kompilace + testy pass."
  - id: R005
    from_status: active
    to_status: validated
    proof: "show_restore_confirm flag + show_modal() confirm dialog (history/mod.rs:373-391). Cancel i confirm cesta implementována."
  - id: R006
    from_status: active
    to_status: validated
    proof: "workspace/mod.rs:788-795 — content_changed → tab.content = hv_content, tab.modified = true. Průběžný sync každý frame."
  - id: R007
    from_status: active
    to_status: validated
    proof: "workspace/mod.rs — sel_idx = if entries.len() > 1 { Some(0) } else { None }. Podmíněný výchozí stav."
  - id: R008
    from_status: active
    to_status: validated
    proof: "grep -c 'history-restore' locales/*/ui.ftl → 5 klíčů × 5 jazyků (cs, en, sk, de, ru)."
  - id: R009
    from_status: active
    to_status: validated
    proof: "apply_diff_backgrounds() v layouter closure kombinuje diff pozadí se syntax highlighting sections. Oba panely. cargo check + testy pass."
duration: ~1h
verification_result: passed
completed_at: 2026-03-13
---

# M003: Vylepšení UI Historie Souboru

**History split view přetvořen z pasivního prohlížeče na aktivní nástroj — levý panel editovatelný se syntax highlighting a diff barvami, pravý read-only se syntax+diff, synchronizovaný scroll, obnovení historické verze s potvrzením (append, ne replace), inteligentní výchozí stav panelů.**

## What Happened

Milestone realizován ve 2 slicích, celkem 3 tascích.

**S01 (high risk, 2 tasky)** přestavěl history split view od základů. T01 dodal datový model — `ScrollSource` enum, rozšířený `HistoryViewState` (content_hash, scroll offsety, diff mapy), `PanelTexts` + `build_panel_texts()` pro konverzi diff→panel texty, `compute_line_offsets()` pro byte→řádek mapování, `apply_diff_backgrounds()` pro diff overlay na LayoutJob sections (binary search v line_offsets), `content_hash()` přes xxh3_64, a `HistorySplitResult` return type. 11 nových unit testů. T02 přepsal rendering — levý panel z read-only Label+LayoutJob na editovatelný TextEdit+layouter se syntax highlighting + diff overlay, pravý panel dostal syntax highlighting (syntect) kombinovaný s diff barvami. Scroll sync implementován přes proportionální mapování s epsilon tolerancí a ScrollSource flag. Tab sync v workspace/mod.rs přes HistorySplitResult.content_changed. Borrow checker vyřešen extrakcí metadata do lokálních proměnných.

**S02 (medium risk, 1 task)** přidal restore flow. Tlačítko "Obnovit" v toolbaru (disabled bez výběru), potvrzovací dialog přes existující `show_modal()` pattern, restore logika v workspace/mod.rs (načtení historického obsahu → zápis do tab.content → take_snapshot jako append → refresh entries → selected_index=Some(0)). Pět i18n klíčů (`history-restore-btn`, `-confirm-title`, `-confirm-text`, `-confirm-ok`, `-confirm-cancel`) ve všech 5 jazycích.

Klíčová architektonická rozhodnutí: TextEdit+layouter+apply_diff_backgrounds pattern pro kombinaci editace, syntax highlighting a diff overlay; proportionální scroll sync místo line-based (jednodušší, dostatečný); restore signalizace přes HistorySplitResult flag místo přímého volání (render funkce nemá &mut LocalHistory).

## Cross-Slice Verification

Každé success criterion z roadmapy ověřeno:

| Criterion | Evidence | Status |
|-----------|----------|--------|
| Levý panel editovatelný (TextEdit) se syntax highlighting a diff | `TextEdit::multiline` v history/mod.rs:526, layouter s `Highlighter::highlight()` + `apply_diff_backgrounds()` | ✅ |
| Pravý panel syntax highlighting + diff zvýraznění | Label+LayoutJob se syntect highlight + diff barvami | ✅ |
| Diff zvýraznění v obou panelech oproti protějšímu | `left_diff_map` / `right_diff_map` z `build_panel_texts()`, `apply_diff_backgrounds()` na obou stranách | ✅ |
| Synchronizovaný scroll | `ScrollSource` enum + proportionální mapování, epsilon 1.0px (history/mod.rs:615-641) | ✅ |
| Editace → tab buffer modified (●) | workspace/mod.rs:788-795 — `content_changed → tab.content = hv_content, tab.modified = true` | ✅ |
| "Obnovit" → potvrzení → zápis + snapshot (append) + refresh | workspace/mod.rs:813-836, show_modal() confirm dialog | ✅ |
| 1 verze → prázdný pravý panel; >1 → nejnovější historická | `sel_idx = if entries.len() > 1 { Some(0) } else { None }` | ✅ |
| Diff cache invalidace při editaci | `content_hash` (xxh3_64) srovnání per-frame, history/mod.rs:405-429 | ✅ |
| i18n 5 klíčů × 5 jazyků | `grep -c 'history-restore' locales/*/ui.ftl` → 5 ve všech 5 locale souborech | ✅ |
| `cargo check` + `./check.sh` | 195 testů pass, 0 selhání, fmt + clippy čisté | ✅ |

**Vizuální UAT** (syntax highlighting čitelnost, diff barvy kombinace, scroll sync UX, restore flow) **nelze verifikovat v headless prostředí** — vyžaduje desktop s GUI. Contract verification (kompilace, testy, statická analýza) je kompletní.

## Requirement Changes

- **R001** (Editovatelný levý panel): active → validated — TextEdit::multiline s layouterem implementován, kompilace + testy pass
- **R002** (Syntax highlighting v obou panelech): active → validated — Highlighter::highlight() v obou panelech, kompilace čistá
- **R003** (Synchronizovaný scroll): active → validated — ScrollSource + proportionální mapování implementováno, unit testy pass
- **R004** (Obnovení historické verze, append): active → validated — restore flow end-to-end propojený, take_snapshot jako append, kompilace + testy pass
- **R005** (Potvrzovací dialog): active → validated — show_modal() confirm dialog integrován, cancel/confirm cesty funkční
- **R006** (Editace → tab buffer): active → validated — průběžný sync přes content_changed, tab.modified = true
- **R007** (Výchozí stav panelů): active → validated — podmíněný selected_index dle entries.len()
- **R008** (i18n klíče): active → validated — 5 klíčů × 5 jazyků potvrzeno grepem
- **R009** (Diff zvýraznění + syntax highlighting): active → validated — apply_diff_backgrounds() + highlight() v layouter closure

## Forward Intelligence

### What the next milestone should know
- History view je nyní plně funkční — editovatelný levý panel, sync scroll, diff+syntax, restore s potvrzením. Signatura `render_history_split_view()` je `#[allow(clippy::too_many_arguments)]` s parametry: highlighter, theme_name, ext, fname, font_size.
- `HistorySplitResult` je komunikační kanál z history rendering → workspace handler. Budoucí akce (export, compare across files) by přidaly další booleany.
- Restore error handling používá `eprintln!` — uživatel nevidí chyby v GUI. Pokud se bude řešit S-3 (I/O error propagace do UI), restore by měl být součástí.

### What's fragile
- **Scroll sync epsilon (1.0px)** — pokud egui změní floating point precision scrollingu, feedback loop se může vrátit. ScrollSource flag je obranná vrstva.
- **Borrow checker pattern v workspace/mod.rs** — extrakce metadata do locals je nutná kvůli disjoint borrows. Přidání dalších mutable operací v tom bloku vyžaduje stejný pattern.
- **Diff mapy clone před layouter closure** — pro extrémně velké soubory (100k+ řádků) by clone mohl být měřitelný. Zatím je levný (Vec<u8-sized enum>).
- **Proportionální scroll sync** — pro výrazně asymetrické diffy (jeden panel 100 řádků, druhý 1000) "skáče" místo plynulého mapování. Line-based mapování přes Equal řádky by bylo přesnější.

### Authoritative diagnostics
- `HistoryViewState.content_hash` — pokud se nemění po editaci, diff cache invalidace nefunguje
- `left_diff_map.len()` vs počet řádků levého panelu — mismatch = build_panel_texts() bug
- `HistorySplitResult.content_changed` + `tab.modified` — pokud jedno true a druhé false = tab sync selhal
- `eprintln!("[Restore] ...")` ve stderr — hledej při selhání restore operace
- Nový snapshot v `.polycredo/history/` po restore — pozorovatelný output na filesystem

### What assumptions changed
- Plán nezmiňoval `font_size` jako parametr render funkce — ukázalo se že je nutný pro konzistentní velikost textu v panelech
- Syntect theme background jako panel fill nebyl plánován, ale zlepšuje vizuální konzistenci s hlavním editorem
- Původní odhad z M002 (nezávislý scroll) byl superseded — M003 vyžadoval sync scroll

## Files Created/Modified

- `src/app/ui/workspace/history/mod.rs` — Kompletní přepis: nové datové struktury (ScrollSource, PanelTexts, HistorySplitResult), helper funkce (build_panel_texts, compute_line_offsets, apply_diff_backgrounds, content_hash), TextEdit+layouter levý panel, Label+LayoutJob pravý panel, scroll sync, restore tlačítko + confirm dialog, 11+ nových testů
- `src/app/ui/workspace/mod.rs` — Rozšířené volání render funkce, borrow-checker řešení, tab sync po content_changed, restore handling po confirm, podmíněná inicializace selected_index
- `locales/cs/ui.ftl` — 5 nových history-restore-* i18n klíčů (čeština)
- `locales/en/ui.ftl` — 5 nových history-restore-* i18n klíčů (angličtina)
- `locales/sk/ui.ftl` — 5 nových history-restore-* i18n klíčů (slovenština)
- `locales/de/ui.ftl` — 5 nových history-restore-* i18n klíčů (němčina)
- `locales/ru/ui.ftl` — 5 nových history-restore-* i18n klíčů (ruština)
