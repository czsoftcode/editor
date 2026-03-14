# M006: Inline Search Panel + Vylepšení In-file Search

**Vision:** Přesun project search z modálních dialogů do inline spodního panelu pod editorem (VS Code styl) a sjednocení in-file search (Ctrl+F) s regex/case/whole-word engine z M005. Search panel je persistentní — neblokuje editaci, drží kontext, umožňuje proklikávání výsledků s okamžitým fokusem na řádek.

## Success Criteria

- Ctrl+Shift+F → pod editorem se otevře search panel s inputem, togglery a výsledky. Editor zůstane viditelný a editovatelný nad panelem.
- Kliknutí na výsledek v panelu → editor otevře soubor a jumpne na řádek s fokusem. Panel zůstane otevřený s výsledky.
- Zavření panelu (Escape) → Ctrl+Shift+F → panel se otevře se zachovanými výsledky a pozicí.
- Replace z inline panelu: toggle replace → replace text → Replace All → preview dialog → potvrzení → soubory modifikovány.
- Panel resize tažením horního okraje funguje.
- Ctrl+F → in-file search bar zobrazí regex/case/whole-word togglery → regex pattern matchuje správně → whole-word toggle rozliší "Result" od "SearchResult".
- `cargo check` + `./check.sh` projde čistě.

## Key Risks / Unknowns

- **egui layout pořadí** — `TopBottomPanel::bottom("search_panel")` musí být vložen PŘED `CentralPanel::default().show()` v `render_workspace()`. Špatné pořadí = panel nemá prostor nebo překryje editor. Kritické pro S01.
- **Fokus transfer panel → editor** — Po kliknutí na výsledek musí editor TextEdit dostat fokus a scroll na řádek. egui nemá explicitní "focus widget" API. Závislost na `open_and_jump()` + `request_editor_focus()` pipeline.

## Proof Strategy

- egui layout pořadí → retire v S01 — panel se zobrazí pod editorem, editor zůstane viditelný a editovatelný, resize funguje
- Fokus transfer → retire v S01 — klik na výsledek → editor jumpne na řádek s fokusem, cursor viditelný na cílovém řádku

## Verification Classes

- Contract verification: `cargo check` + `./check.sh` (fmt, clippy, testy) po každém slice
- Integration verification: celý tok Ctrl+Shift+F → query → výsledky → klik → jump → editace → další klik (vizuální UAT na desktopu)
- Operational verification: none (lokální desktop aplikace)
- UAT / human verification: vizuální ověření panelu, resize, fokus transfer (headless build ověří kompilaci, ne UI)

## Milestone Definition of Done

This milestone is complete only when all are true:

- Inline search panel renderuje výsledky v `TopBottomPanel::bottom` pod editorem
- Kliknutí na výsledek otevře soubor s fokusem na řádku bez ztráty stavu panelu
- Panel stav (výsledky, query, pozice) přežije close/reopen
- Replace flow funguje z inline panelu (preview dialog zůstává modální)
- In-file search (Ctrl+F) používá `build_regex()` s regex/case/whole-word togglery
- i18n klíče pro nové UI prvky ve všech 5 jazycích
- `cargo check` + `./check.sh` projde čistě
- Mrtvé modální dialogy (render_project_search_dialog, poll_and_render_project_search_results) jsou smazány nebo podmíněně neaktivní

## Requirement Coverage

- Covers: R026, R027, R028, R029, R030, R031, R032, R033, R034, R035, R036
- Partially covers: none
- Leaves for later: none
- Orphan risks: none

## Slices

- [x] **S01: Inline project search panel** `risk:high` `depends:[]`
  > After this: Ctrl+Shift+F otevře inline panel pod editorem s query inputem, togglery, per-file výsledky se zvýrazněním, klik na výsledek jumpne na řádek s fokusem, replace flow funguje z panelu, stav persistuje přes close/reopen, panel je resizable. Modální search dialogy jsou nahrazeny.
- [x] **S02: In-file search s regex/case/whole-word togglery** `risk:low` `depends:[S01]`
  > After this: Ctrl+F zobrazí search bar s regex/case/whole-word toggle buttony, matching používá build_regex() engine místo primitivního substring match. i18n pro nové toggle labely ve všech 5 jazycích.

## Boundary Map

### S01 → S02

Produces:
- `build_regex()` znovupoužití potvrzeno v inline panelu — S02 napojí stejnou funkci v in-file search kontextu
- `SearchOptions` struct s use_regex/case_sensitive/whole_word fieldy — S02 přidá analogické fieldy do Editor structu
- i18n klíče pro toggle buttony (regex/case/whole-word) — S02 sdílí existující project-search-* klíče kde možné, přidá search-specific kde nutné

Consumes:
- nothing (first slice)
