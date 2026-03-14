# M006: Inline Search Panel + Vylepšení In-file Search — Research

**Date:** 2026-03-13

## Summary

Přesun project search z modálních dialogů do inline `TopBottomPanel::bottom` je realizovatelný s relativně malým rizikem. egui 0.31.1 nativně podporuje `TopBottomPanel::bottom().resizable(true)` s `default_height()`, `min_height()`, `max_height()` — resize panelu tažením horního okraje je built-in, nevyžaduje custom drag handler. Engine funkce (`build_regex()`, `search_file_with_context()`, `run_project_search()`, `compute_replace_previews()`, `apply_replacements()`) z M005 jsou čisté funkce oddělené od UI a znovupoužitelné beze změn. UI rendering je kompletně v `render_project_search_dialog()` a `poll_and_render_project_search_results()` — obě jako `StandardModal` — a bude přepsán do inline panelu.

Nejvyšší riziko je **layout pořadí v `render_workspace()`**. Aktuální pořadí: MenuBar → dialogy → status bars → floating bottom panel → AI panel → left panel → CentralPanel. Nový search panel (`TopBottomPanel::bottom("search_panel")`) musí být vložen PŘED `CentralPanel::default().show()` — egui vyžaduje, aby všechny side/top/bottom panely byly deklarovány před CentralPanel, protože CentralPanel zabere zbylý prostor. Build terminál je floating (`StandardTerminalWindow`), ne `TopBottomPanel`, takže nekoliduje.

In-file search bar (`editor/search.rs`) používá primitivní `eq_ignore_ascii_case` substring match přes `char_indices()` loop — žádné regex, žádný case toggle, žádný whole-word. Rozšíření o togglery a napojení na `build_regex()` je přímočaré — `update_search()` se přepíše z iterace přes `char_indices` na `regex.find_iter()`, tím se sjednotí matching engine pro oba kontexty.

## Recommendation

**Začít s inline search panelem (highest risk), pak in-file search vylepšení (lower risk).**

Slice 1: Inline search panel — nový `TopBottomPanel::bottom("search_panel")` s inputem, togglery, výsledky, klik → jump, persistentní stav, resize. Nahradí `render_project_search_dialog()` a `poll_and_render_project_search_results()`. Replace flow se spustí z panelu, ale preview dialog zůstane modální. Smazat mrtvé modální dialogy.

Slice 2: In-file search vylepšení — přidání regex/case/whole-word togglerů do `search_bar()`, přepis `update_search()` na `build_regex()`, sdílení SearchOptions mezi in-file a project search.

Toto pořadí je správné protože:
1. Inline panel je architektonicky riskantnější (layout pořadí, fokus management, stav persistence).
2. In-file search je izolovaná změna v jednom souboru (`editor/search.rs`) bez vlivu na layout.
3. S01 bude mít kompletní demoable feature (celý project search workflow v inline panelu).

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Panel resize (drag horního okraje) | `TopBottomPanel::bottom().resizable(true)` (egui built-in) | Nativní egui API, automatická persistence, resize cursor. Custom drag handler by duplikoval existující funkcionalitu. |
| Regex engine pro in-file search | `build_regex()` v `search_picker.rs` | Už pokrývá všech 8 kombinací togglerů, 10 unit testů, error handling. Reimplementace by byla chyba. |
| Search result highlighting | `build_match_layout_job()` / `build_context_layout_job()` v `search_picker.rs` | Multi-section LayoutJob s oranžovým bg pro matche. Přesně to co panel potřebuje — zkopírovat/importovat. |
| Fokus management po kliknutí na výsledek | `open_file_in_ws()` + `jump_to_location()` + `request_editor_focus()` | Existující pipeline pro otevření souboru s fokusem na řádku — stačí zavolat v click handleru. |
| File open + jump | `open_and_jump()` v `state/actions.rs` | Helper který kombinuje open + jump + FocusedPanel::Editor. Přesně to co klik na výsledek potřebuje. |
| Inkrementální streamování výsledků | `SearchBatch` enum + `mpsc` + `try_recv()` loop | Existující per-soubor dávkování z M005. Panel poll loop je identický s tím v `poll_and_render_project_search_results()`. |

## Existing Code and Patterns

- `src/app/ui/search_picker.rs` (1502 řádků) — Vše pro project search: engine funkce (`build_regex`, `search_file_with_context`, `run_project_search`, `compute_replace_previews`, `apply_replacements`) + UI dialogy (`render_project_search_dialog`, `poll_and_render_project_search_results`, `render_replace_preview_dialog`) + unit testy. Engine funkce zůstanou, UI dialogy se přepíší. **Pattern k znovupoužití:** LayoutJob building (`build_match_layout_job`, `build_context_layout_job`), poll loop (`try_recv` → akumulace → `request_repaint`), per-file seskupení výsledků.
- `src/app/ui/editor/search.rs` (242 řádků) — In-file search bar. `update_search()` = primitivní substring match (`eq_ignore_ascii_case`). `search_bar()` = render UI s prev/next/replace/replace_all. `apply_search_highlights()` = zvýraznění matchů v editor LayoutJob. **Přepíše se:** `update_search()` z substring match na `build_regex()` + regex `find_iter()`. **Rozšíří se:** `search_bar()` o toggle buttony pro regex/case/whole-word.
- `src/app/ui/workspace/mod.rs` (1178 řádků) — `render_workspace()` je hlavní layout funkce. Layout pořadí: MenuBar → dialogy → status bars → bottom panel (floating) → AI panel → left panel → CentralPanel. **Integrace:** vložit `TopBottomPanel::bottom("search_panel")` na řádek ~720 (před `CentralPanel::default().show()`). Klik na výsledek handler: `open_and_jump()` z `state/actions.rs`.
- `src/app/ui/workspace/state/types.rs` (157 řádků) — `ProjectSearch` struct, `SearchOptions`, `SearchResult`, `SearchBatch`, `ReplacePreview`. **Rozšíření:** přidat `show_panel: bool`, `panel_height: f32`, `last_selected_index: Option<usize>`.
- `src/app/ui/panels.rs` (189 řádků) — `render_left_panel()` s resize splitter pattern (Sense::drag, ResizeVertical cursor). **Pattern reference** pro vlastní separator, pokud by byl potřeba — ale egui `resizable(true)` na TopBottomPanel to řeší nativně.
- `src/app/ui/workspace/state/actions.rs` — `open_and_jump(ws, path, line)` helper — open + jump + focus. Přesně to co panel click handler potřebuje.
- `src/app/keymap.rs` (515 řádků) — `CommandId::ProjectSearch` mapován na `Cmd+Shift+F`. **Změna:** místo `ws.project_search.show_input = true` nastavit `ws.project_search.show_panel = !ws.project_search.show_panel` (toggle).
- `src/app/ui/workspace/menubar/mod.rs` — `process_menu_actions()` řádky 172-174: `if actions.project_search { ws.project_search.show_input = true; ws.project_search.focus_requested = true; }`. **Změna:** přesměrovat na `show_panel` toggle.
- `src/app/ui/editor/mod.rs` — Editor struct s fieldy `show_search`, `search_query`, `replace_query`, `show_replace`, `search_matches`, `current_match`, `search_focus_requested`. **Rozšíření:** přidat `search_use_regex: bool`, `search_case_sensitive: bool`, `search_whole_word: bool`.
- `locales/{cs,en,sk,de,ru}/ui.ftl` — 32 existujících `project-search-*` klíčů + 5 `search-*` klíčů pro in-file search. **Nové klíče:** panel-specific labely (minimální — většina UI textu se sdílí s existujícími klíči).

## Constraints

- **egui layout pořadí je striktní:** `TopBottomPanel` a `SidePanel` musí být deklarovány PŘED `CentralPanel`. CentralPanel zabere zbylý prostor. Pokud se search panel vloží po CentralPanel, nebude mít prostor.
- **egui 0.31.1** — žádná verze upgrade. Všechny API musí fungovat s touto verzí.
- **Žádné nové runtime závislosti** — `regex`, `globset`, `similar` už jsou v Cargo.toml.
- **`build_regex()` a `search_file_with_context()` nesmí být modifikovány** — jsou pokryty 15 unit testy z M005. Jen se znovupoužijí.
- **Search engine běží v background threadu** — UI nesmí blokovat. `run_project_search()` zůstává v threadu, výsledky streamují přes `mpsc`.
- **`TopBottomPanel::bottom` id musí být unikátní** — existující bottom panely: `"footer_separator"`, `"status_bar"`. Search panel: `"search_panel"`.
- **Repaint throttling** — `has_active_work` v `render_workspace()` nezahrnuje search rx. Poll loop v search panelu musí volat `ctx.request_repaint()` při `TryRecvError::Empty` pokud search stále běží (stávající pattern).
- **Escape handling** — Escape v search inputu nesmí propagovat do centrálního keymap dispatch. egui TextEdit konzumuje Enter/Escape pokud má focus — ověřit s `consume_shortcut` logikou.
- **replace preview dialog zůstane modální** — scope M006 jen přesouvá spuštění z input dialogu do inline panelu. `render_replace_preview_dialog()` se nezmění.

## Common Pitfalls

- **TopBottomPanel vložený po CentralPanel** — egui tiše nedá panelu prostor, panel se nezobrazí nebo překryje editor. **Jak se vyhnout:** vložit PŘED CentralPanel v layoutovém pořadí (řádek ~720), ověřit vizuálně.
- **Fokus zůstane v search inputu po kliknutí na výsledek** — editor nedostane focus a cursor se neobjeví na cílovém řádku. **Jak se vyhnout:** po kliknutí na výsledek zavolat `request_editor_focus()` a `ws.focused_panel = FocusedPanel::Editor`.
- **Search query ztracen při zavření panelu** — pokud se query resetuje při close, uživatel přijde o kontext. **Jak se vyhnout:** `show_panel = false` nezmaže `query` ani `results`. Query a results zůstávají v `ProjectSearch` struct.
- **Duplicitní rendering starých modálních dialogů** — pokud se nezamezí volání `render_project_search_dialog()` a `poll_and_render_project_search_results()`, budou se renderovat dvě verze současně. **Jak se vyhnout:** smazat/podmínit volání v `render_workspace()` — `show_input` se už nenastavuje, ale pro bezpečnost zamezit.
- **In-file search regrese** — přepis `update_search()` na regex může rozbít stávající flow (replace_current, replace_all operují s byte ranges). **Jak se vyhnout:** update `search_matches` přes `regex.find_iter()` který vrací správné byte ranges; existující `replace_range(start..end)` funguje beze změn.
- **Escape v search panelu vs. Escape pro close modálu** — pokud je search panel otevřený a zároveň modální dialog (settings), Escape by měl zavřít dialog, ne panel. **Jak se vyhnout:** Escape handler v search panelu kontroluje, zda nemá přednost modální dialog (dialog_open flag).
- **Panel height persistence** — egui `TopBottomPanel::bottom().resizable(true)` automaticky persistuje výšku v egui memory (per-frame). Ale po restartu aplikace se ztratí. **Jak se vyhnout:** pro MVP nechat egui memory (neperformuje přes restart). Pokud se ukáže jako problém, přidat do settings.toml.

## Open Risks

- **egui TopBottomPanel::bottom s resizable(true) — resize direction** — egui bottom panel resize handle je na horním okraji (separátor). Uživatel táhne směrem nahoru pro zvětšení, dolů pro zmenšení. Toto je standardní chování a odpovídá VS Code, ale ověřit vizuálně.
- **Velký počet výsledků v ScrollArea** — 1000+ výsledků s LayoutJob per-řádek v ScrollArea bez virtualizace může být pomalý. M006 context explicitně uvádí virtualizaci jako out-of-scope ("odložit pokud se ukáže jako výkonnostní problém"). Pokud se ukáže, řešení: omezení renderovaných výsledků na viditelné + lazy loading.
- **Hotkey dispatch priority** — Ctrl+Shift+F dispatch probíhá v centrálním keymapu PŘED widget renderingem. Toggle `show_panel` v dispatch je ok. Ale Escape v search inputu je konzumovaný TextEdit widgetem — musí být renderován PŘED dispatch check, nebo handler Escape v panelu explicitně. Ověřit flow.
- **In-file search toggle state** — Otevření context question: pamatovat regex/case/whole-word stav mezi otevřeními Ctrl+F? Doporučení: ANO, jsou to fieldy v Editor struct, přetrvávají přirozeně. Sdílení stavu s project search: NE — jsou to nezávislé kontexty s odlišnými use cases.

## Candidate Requirements

Na základě researche navrhuji tyto candidate requirements pro M006. Finální rozhodnutí o zařazení je na uživateli.

### Table Stakes (musí být)

| ID | Popis | Zdůvodnění |
|----|-------|------------|
| R026 | Inline search panel pod editorem (`TopBottomPanel::bottom`) s query inputem a togglery | Core deliverable M006. Bez toho není milestone. |
| R027 | Kliknutí na výsledek → otevření souboru s fokusem na řádku, panel zůstane otevřený | Eliminuje problém #1 (ztráta fokusu) a #3 (blokace editace). |
| R028 | Persistentní stav panelu — výsledky a query přežijí close/reopen | Eliminuje problém #2 (ztráta kontextu). |
| R029 | Ctrl+Shift+F toggle: otevře/zavře panel | Konzistence se stávající zkratkou. |
| R030 | In-file search (Ctrl+F) s regex/case/whole-word togglery přes `build_regex()` | Sjednocení engine — explicitní scope M006. |
| R031 | Panel resize tažením horního okraje | Explicitní v context. egui nativní API. |
| R032 | Replace flow spustitelný z inline panelu | Zachování stávající funkcionality v novém UI. |
| R033 | i18n pro nové/změněné UI prvky (5 jazyků) | Konzistence s existujícím i18n systémem. |

### Expected Behaviors (pravděpodobně chtěné)

| ID | Popis | Zdůvodnění |
|----|-------|------------|
| R034 | Spinner + "Hledám..." indikátor v panelu během search | UX — uživatel ví že search běží. Existuje v modálu, přenést do panelu. |
| R035 | Per-file seskupení výsledků s filename hlavičkou | UX standard (VS Code, grep). Existuje v modálu, přenést do panelu. |
| R036 | Kontextové řádky a match highlighting v panelu | Přenesení M005 funkcionality (R017, R018) do nového UI. |

### Volitelné (advisory)

| ID | Popis | Zdůvodnění |
|----|-------|------------|
| R037 | F4 / Shift+F4 navigace mezi výsledky v panelu | VS Code konvence. Nice-to-have, ne blokující. |
| R038 | Highlight matchujícího textu v editoru po kliknutí na výsledek | Vizuální feedback — uživatel vidí kde match je. Příliš složité pro MVP? |
| R039 | Pamatování in-file search toggle stavu mezi otevřeními Ctrl+F | Přirozené — fieldy v Editor struct přetrvávají. Ale explicitně zmínit? |

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| egui | none found | Žádná egui-specifická skill. Generické `bobmatnyc/claude-mpm-skills@rust-desktop-applications` (120 installs) existuje ale je příliš obecné. |
| Rust desktop GUI | `bobmatnyc/claude-mpm-skills@rust-desktop-applications` | available (120 installs) — generické, pravděpodobně nepřinese hodnotu pro egui-specifickou práci |

## Sources

- egui 0.31.1 `TopBottomPanel` API: `~/.cargo/registry/src/*/egui-0.31.1/src/containers/panel.rs` — nativní `resizable(true)`, `default_height()`, `min_height()`, `max_height()`, `height_range()`
- `src/app/ui/search_picker.rs` — engine funkce + UI dialogy, 15 unit testů
- `src/app/ui/editor/search.rs` — in-file search implementace (substring match)
- `src/app/ui/workspace/mod.rs` — layout pořadí v `render_workspace()`
- `src/app/ui/workspace/state/types.rs` — `ProjectSearch`, `SearchOptions`, `SearchResult` structs
- `src/app/ui/panels.rs` — resize splitter pattern reference
- `src/app/ui/workspace/state/actions.rs` — `open_and_jump()` helper
- `src/app/keymap.rs` — `CommandId::ProjectSearch` dispatch
- egui docs (Context7 /emilk/egui) — widget API reference
