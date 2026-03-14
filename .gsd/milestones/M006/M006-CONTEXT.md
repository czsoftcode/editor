# M006: Inline Search Panel + Vylepšení In-file Search — Context

**Gathered:** 2026-03-13
**Status:** Queued — pending auto-mode execution.

## Project Description

Přesun project-wide search z modálních dialogů do inline spodního panelu pod editorem (VS Code styl) a sjednocení in-file search (Ctrl+F) s regex/case/whole-word engine z M005. Search panel je persistentní — neblokuje editaci, drží kontext, umožňuje proklikávání výsledků s okamžitým fokusem na řádek.

## Why This Milestone

Tři provázané problémy v aktuálním project search:

1. **Ztráta fokusu po zavření modálu.** Uživatel klikne na výsledek → editor otevře soubor a jumpne na řádek → zavře modal výsledků → editor ztratí fokus na řádku. Uživatel musí kliknout na tab, aby obnovil fokus a viděl zvýrazněný řádek. Příčina: modální dialog přebírá fokus, po zavření se nevrací.

2. **Ztráta kontextu.** Při znovuotevření project search (Ctrl+Shift+F) zůstane query, ale výsledky se ztratí — uživatel musí znovu hledat. Nepamatuje se pozice v seznamu výsledků, stav scrollu, ani naposledy navštívený výsledek.

3. **Modal blokuje editaci.** Modální dialog brání interakci s editorem — uživatel nemůže proklikávat výsledky a zároveň editovat. Musí zavřít výsledky, editovat, znovu otevřít search, znovu hledat. Nemožný workflow pro refactoring (procházení matchů a editace jednoho po druhém).

Všechny tři problémy řeší přesun na inline panel. Zároveň in-file search (Ctrl+F) je stále primitivní substring match bez regex/case/whole-word togglerů — sjednocení s `build_regex()` engine z M005 vyřeší i to.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Stisknout Ctrl+Shift+F → pod editorem se otevře search panel s inputem, togglery a výsledky. Editor zůstane viditelný a editovatelný nad panelem.
- Proklikávat výsledky v panelu → editor otevře soubor a jumpne na řádek s fokusem. Panel zůstane otevřený.
- Přepínat mezi editací a procházením výsledků bez ztráty kontextu — panel si drží výsledky, pozici scrollu, naposledy navštívený výsledek.
- Zavřít panel (Escape nebo ✕) a znovu otevřít → výsledky a stav zůstanou.
- Resize panelu tažením horního okraje (jako split view).
- Stisknout Ctrl+F → in-file search bar má regex/case-sensitive/whole-word togglery (sdílí `build_regex()` engine z M005).
- Replace v panelu funguje jako dříve (preview dialog), ale launch se děje z inline panelu, ne z modálu.

### Entry point / environment

- Entry point: Ctrl+Shift+F (project search panel), Ctrl+F (in-file search s togglery)
- Environment: desktop editor (eframe/egui), local-first, single-process multi-window
- Live dependencies involved: none — vše je lokální filesystem

## Completion Class

- Contract complete means: project search výsledky se renderují v inline `TopBottomPanel` pod editorem; kliknutí na výsledek otevře soubor s fokusem na řádku bez ztráty stavu panelu; in-file search používá `build_regex()` s togglery; unit testy pokrývají in-file regex matching.
- Integration complete means: celý tok funguje end-to-end — Ctrl+Shift+F → panel se otevře → query → výsledky streamují do panelu → klik → editor jumpne a dostane fokus → editace → další klik → další jump. Replace flow funguje z panelu.
- Operational complete means: panel s 1000+ výsledky scrolluje plynule (virtualizovaný seznam nebo lazy rendering); resize panelu neovlivní editor responsiveness.

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- Ctrl+Shift+F → zadání query → výsledky se zobrazí v panelu pod editorem → klik na výsledek → editor otevře soubor s fokusem na řádku → panel zůstane otevřený s výsledky → klik na další výsledek → editor jumpne na nový řádek.
- Zavření panelu (Escape) → Ctrl+Shift+F → panel se otevře se zachovanými výsledky a pozicí.
- Ctrl+F → in-file search bar zobrazí regex/case/whole-word togglery → regex pattern matchuje správně → whole-word toggle rozliší "Result" od "SearchResult".
- Replace z inline panelu: query → replace text → Replace All → preview dialog → potvrzení → soubory modifikovány + snapshot.
- Editor je viditelný a editovatelný zatímco je search panel otevřený.
- Resize panelu tažením horního okraje funguje.
- `cargo check` + `./check.sh` projde čistě.

## Risks and Unknowns

- **egui TopBottomPanel + CentralPanel interakce** — Přidání `TopBottomPanel::bottom` před `CentralPanel` je standardní egui pattern, ale musí se vložit v layoutovém pořadí PŘED CentralPanel rendering v `render_workspace()`. Pokud se CentralPanel renderuje dřív, TopBottomPanel nemá prostor. Pořadí volání je kritické.
- **Fokus management po kliknutí na výsledek** — egui nemá explicitní "focus this widget" API mimo `Response::request_focus()`. Po kliknutí na výsledek a otevření souboru musí editor TextEdit dostat fokus a scroll na řádek. `jump_to_location()` existuje, ale fokus transfer z panelu do editoru musí být ověřen.
- **Virtualizace dlouhých seznamů** — 1000+ výsledků v ScrollArea může být pomalé. egui nemá built-in virtualizaci. Řešení: buď omezení vykreslených výsledků (lazy), nebo custom clip-based rendering.
- **Stav panelu vs. nový search** — Uživatel spustí nový search zatímco výsledky starého jsou zobrazené. Přechod musí být plynulý — vyčistit výsledky a streamovat nové.
- **In-file search bar šířka** — Přidání 3 toggle buttonů do stávajícího search baru může být těsné, zejména s replace inputem. Layout search baru potřebuje redesign.
- **Search input v panelu vs. hotkey dispatch** — Klávesové zkratky (Enter, Escape, togglery) v search inputu nesmí propagovat do centrálního keymap dispatch. Fokusovaný TextEdit by měl konzumovat eventy, ale ověřit s `consume_shortcut` logikou.

## Existing Codebase / Prior Art

- `src/app/ui/search_picker.rs` — Aktuální project search: input dialog (`render_project_search_dialog()`), výsledkový dialog (`poll_and_render_project_search_results()`), replace preview dialog (`render_replace_preview_dialog()`), search engine funkce (`build_regex()`, `search_file_with_context()`, `compute_replace_previews()`, `apply_replacements()`). Vše jako `StandardModal`. Engine funkce jsou znovupoužitelné, UI dialogy budou přepsány.
- `src/app/ui/editor/search.rs` — In-file search bar. Prostý substring match (`update_search()` → `search_matches`). Render přes `search_bar()`. Flagy `show_search`, `show_replace`, `show_goto_line` v `Editor` structu.
- `src/app/ui/workspace/mod.rs` — Layout: status bars → bottom panel (build terminal) → AI panel → left panel → CentralPanel (editor). Search panel se vloží jako `TopBottomPanel::bottom("search_panel")` PŘED `CentralPanel`.
- `src/app/ui/workspace/state/types.rs` — `ProjectSearch` struct s toggle stavy, výsledky, replace daty. `SearchOptions`, `SearchResult`, `SearchBatch`, `ReplacePreview`.
- `src/app/ui/panels.rs` — `render_left_panel()` s `egui::SidePanel::left`. Build terminál je uvnitř levého panelu (ne samostatný bottom panel) — prostor pod editorem je volný.
- `src/app/ui/terminal/bottom/mod.rs` — `render_bottom_panel()` renderuje build terminál jako `StandardTerminalWindow` (floating), ne jako `TopBottomPanel`. Nekoliduje s inline search panelem.
- `src/app/keymap.rs` — `CommandId::ProjectSearch` registrován s `Cmd+Shift+F`, `CommandId::Find` s `Cmd+F`.

> See `.gsd/DECISIONS.md` for all architectural and pattern decisions — it is an append-only register; read it during planning, append to it during execution.

## Relevant Requirements

- Nový scope — zavádí nové requirements pro inline search panel a vylepšený in-file search.
- Navazuje na M005 (R016–R025) — engine funkce (`build_regex()`, `search_file_with_context()`) se znovupoužijí beze změn. UI se kompletně přestaví.
- Částečně relevantní: Active backlog items (V-1 přes V-3, K-1, S-1, S-3, S-4, N-5) nejsou tímto milestone adresovány.

## Scope

### In Scope

- Přesun project search z modálních dialogů do inline `TopBottomPanel::bottom` pod editorem
- Search input s togglery (regex/case/whole-word, file filter) integrovaný v panelu nahoře
- Výsledky zobrazené pod inputem se zachováním zvýraznění matchů a kontextu z M005
- Kliknutí na výsledek → otevření souboru s fokusem na řádku, panel zůstane otevřený
- Persistentní stav panelu (výsledky, pozice scrollu, naposledy navštívený výsledek)
- Panel resize (tažení horního okraje)
- Ctrl+Shift+F toggle: otevře/zavře panel
- Replace flow z inline panelu (Replace All → preview dialog zůstane modální)
- Vylepšení in-file search (Ctrl+F) o regex/case/whole-word togglery sdílející `build_regex()`
- i18n pro nové/změněné UI prvky

### Out of Scope / Non-Goals

- Search history (seznam předchozích queries)
- Sidebar search panel (VS Code sidebar mode) — zůstáváme na bottom panelu
- Multi-line search pattern
- In-file search v panelu (project search a in-file search zůstávají oddělené)
- Virtualizace výsledků (lazy rendering) — odložit pokud se ukáže jako výkonnostní problém
- Přesun replace preview z modálu do panelu — zůstane modální (vhodný pro potvrzovací dialog)

## Technical Constraints

- `cargo check` + `./check.sh` musí projít po každé slice
- Žádné nové runtime závislosti
- Neblokovat UI vlákno — search engine zůstává v background threadu
- `TopBottomPanel::bottom("search_panel")` musí být vložen PŘED `CentralPanel::default().show()` v `render_workspace()` layoutovém pořadí
- Zachovat existující engine funkce beze změny (`build_regex`, `search_file_with_context`, `compute_replace_previews`, `apply_replacements`)
- In-file search bar rozšíření nesmí rozbít stávající Ctrl+F/Ctrl+H/goto line flow
- Zpětná kompatibilita: Ctrl+Shift+F stále otevírá project search, Ctrl+F stále otevírá in-file search

## Integration Points

- `src/app/ui/workspace/mod.rs` — nový `TopBottomPanel::bottom("search_panel")` v layout pořadí. Úprava `render_workspace()` pro vložení panelu mezi status bar a CentralPanel. Fokus management po kliknutí na výsledek.
- `src/app/ui/search_picker.rs` — kompletní přepis UI: z `StandardModal` na inline panel rendering. Engine funkce zůstanou.
- `src/app/ui/workspace/state/types.rs` — rozšíření `ProjectSearch` o panel stav (show_panel, panel_height, last_selected_index, scroll_offset).
- `src/app/ui/editor/search.rs` — přidání regex/case/whole-word togglerů do search baru, napojení na `build_regex()`.
- `src/app/ui/editor/mod.rs` — nové fieldy pro in-file search options (use_regex, case_sensitive, whole_word).
- `locales/*/ui.ftl` — nové/upravené i18n klíče.

## Open Questions

- **Panel default výška** — Kolik prostoru zabrat? Pravděpodobně ~200-250px default, min 100px, max 60% výšky okna. Persistovat do settings.toml?
- **Escape chování** — Escape v search inputu zavře panel? Nebo jen vyčistí query? Pravděpodobně: Escape zavře panel (VS Code chování).
- **F4 / Enter navigace** — VS Code má F4 pro "next result" a Shift+F4 pro "previous result" v search panelu. Implementovat? Nebo jen klik?
- **In-file search toggle state persistence** — Pamatovat regex/case/whole-word stav mezi otevřeními Ctrl+F? Sdílet stav s project search?
- **Highlight matchů v editoru** — Při kliknutí na výsledek, zvýraznit matchující text v editoru? In-file search to dělá (search_matches), ale project search výsledky ne.
