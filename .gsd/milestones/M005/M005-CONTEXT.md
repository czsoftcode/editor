# M005: Vylepšení Project Search — Context

**Gathered:** 2026-03-13
**Status:** Queued — pending auto-mode execution.

## Project Description

Přebudování project-wide search (Ctrl+Shift+F) z minimálního modálního dialogu na plnohodnotný vyhledávací nástroj — regex, case-sensitive/whole-word togglery, zvýraznění matchů ve výsledcích, kontextové řádky kolem nalezených výskytů, filtrování dle typu souboru, a project-wide find & replace s preview a potvrzením.

## Why This Milestone

Aktuální project search má dvě úrovně problémů:

1. **Základní UX chybí.** Input dialog je prostý modal s jedním textovým polem. Výsledky se zobrazují v druhém modalu jako prostý seznam `selectable_label` bez zvýraznění, bez kontextu, bez filtrování. Uživatel nevidí, kde přesně v řádku je match, nevidí okolní řádky, nemůže filtrovat podle typu souboru.

2. **Search engine je primitivní.** `run_project_search()` dělá case-insensitive substring match (`line.to_lowercase().contains(&q)`). Žádný regex, žádný case-sensitive toggle, žádný whole-word mode. Pro programátorský editor je to nedostatečné — uživatel nemůže hledat např. `fn\s+main` nebo rozlišit `Result` od `result`.

3. **Replace v projektu neexistuje.** Editor má in-file replace (Ctrl+H), ale žádný project-wide replace. Regex replace přes celý projekt je běžná operace v programátorských editorech.

`regex` crate je již v Cargo.toml (verze 1). In-file search (`search.rs`) je také prostý substring bez togglerů — project search bude první, který je zavede.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Stisknout Ctrl+Shift+F → otevře se vylepšený search dialog s textovým polem a toggle ikonami pro regex, case-sensitive a whole-word
- Zadat regex pattern (např. `fn\s+\w+`) → výsledky zobrazí zvýrazněné matchující části v kontextu ±2 řádků
- Přepnout case-sensitive toggle → hledání rozliší velká/malá písmena
- Přepnout whole-word toggle → hledá pouze celá slova (ne substrings)
- Filtrovat výsledky podle přípony souboru (např. jen `.rs`, jen `.toml`)
- Kliknout na výsledek → otevře soubor a skočí na řádek
- Zadat replace text → zobrazí se preview všech nahrazení s checkboxy → potvrdí → soubory se modifikují s local history snapshotem

### Entry point / environment

- Entry point: Ctrl+Shift+F (centrální keymap dispatch z M004), nebo menu Edit → Search in Project
- Environment: desktop editor (eframe/egui), local-first, single-process multi-window
- Live dependencies involved: none — vše je lokální filesystem

## Completion Class

- Contract complete means: regex engine matchuje korektně; case-sensitive/whole-word/regex togglery mění chování; výsledky obsahují zvýrazněné matche a kontext; file type filtr omezuje scope; replace preview ukazuje diff; unit testy pokrývají matching logiku, toggle kombinace a replace preview.
- Integration complete means: celý tok funguje end-to-end — Ctrl+Shift+F → query s togglery → výsledky se zvýrazněním → kliknutí otevře soubor → replace → preview → potvrzení → soubory modifikovány → local history snapshot.
- Operational complete means: search na projektu s 1000+ soubory vrátí výsledky bez UI freeze (background thread); replace vytvoří snapshoty pro undo.

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- Ctrl+Shift+F → zadat `fn\s+\w+` s regex toggle ON → výsledky zobrazí zvýrazněné matchující funkce v .rs souborech → kliknutí na výsledek otevře soubor a skočí na řádek.
- Zadat "TODO" s case-sensitive ON → najde jen "TODO", ne "todo" nebo "Todo". S case-sensitive OFF → najde všechny varianty.
- Zadat "Result" s whole-word ON → najde "Result" ale ne "SearchResult". S whole-word OFF → najde obojí.
- Zadat filtr "*.rs" → výsledky jen z Rust souborů. Zadat "*.toml" → jen z TOML.
- Replace: zadat query "old_name" → replace "new_name" → preview ukazuje diff všech souborů → odškrtnout jeden soubor → potvrdit → soubory se změní (kromě odškrtnutého) → local history snapshot existuje pro každý modifikovaný soubor.
- `cargo check` + `./check.sh` projde čistě.

## Risks and Unknowns

- **Regex error handling** — `regex::Regex::new()` může selhat na nevalidním patternu. Dialog musí zobrazit inline error (ne panic), a hledání nespustit. Zvýrazňování matchů musí pracovat s `regex::Match` rozsahy.
- **Výkon regex na velkých souborech** — `regex` crate je rychlá, ale na projektu s tisíci soubory může hledání trvat. Stávající background thread pattern to řeší, ale progresivní streamování výsledků (místo čekání na všechny) by zlepšilo UX.
- **Replace safety** — Project-wide replace je destruktivní. Snapshot přes local history je záchranná síť, ale replace musí proběhnout atomicky (buď všechny vybrané soubory, nebo žádný při chybě) nebo s jasným error reportem.
- **Replace + regex capture groups** — Regex replace s capture groups ($1, $2) je mocný ale složitý. Rozhodnout jestli je to v scope.
- **Kontext řádků a zvýraznění v egui** — egui nemá nativní inline text highlighting (tučný/barevný substring v rámci jednoho label). Řešení: `LayoutJob` s víc `TextSection` pro různé barvy matchujícího a ne-matchujícího textu.
- **File type filtr UX** — Jak specifikovat filtr: volný text (`*.rs`), multi-select checkboxy, glob pattern? Volný text je nejflexibilnější, glob pattern nejpřirozenější pro programátory.

## Existing Codebase / Prior Art

- `src/app/ui/search_picker.rs` — Aktuální project search: `render_project_search_dialog()` (input modal), `poll_and_render_project_search_results()` (výsledkový modal), `run_project_search()` (background thread, case-insensitive substring), `collect_project_files()` (file listing s .git/target/node_modules ignore). 352 řádků.
- `src/app/ui/workspace/state/types.rs:50-79` — `SearchResult` struct (file, line, text) a `ProjectSearch` struct (show_input, query, results, rx, focus_requested, cancel_epoch).
- `src/app/ui/editor/search.rs` — In-file search bar. Prostý substring, `update_search()` → `search_matches`. Render přes `search_bar()`. Nemá regex/case/whole-word togglery.
- `src/app/ui/workspace/index.rs` — `ProjectIndex` s `get_files()` — vrací `Arc<Vec<PathBuf>>` souborů v projektu. Full rescan při startu, inkrementální update z watcheru.
- `src/app/ui/workspace/mod.rs` — Dispatch: `render_project_search_dialog()` a `poll_and_render_project_search_results()` volání, kliknutí na výsledek → `open_file_in_ws()` + `jump_to_location()`.
- `src/app/local_history.rs` — `take_snapshot()` pro local history — replace bude volat před modifikací souborů.
- `locales/*/ui.ftl` — Existující i18n klíče: `project-search-heading`, `-hint`, `-btn`, `-loading`, `-result-label`, `-no-results`, `-max-results`.
- `Cargo.toml` — `regex = "1"` již v dependencies.
- `src/app/keymap.rs` — Centrální dispatch, `CommandId::ProjectSearch` registrován s `Cmd+Shift+F`.

> See `.gsd/DECISIONS.md` for all architectural and pattern decisions — it is an append-only register; read it during planning, append to it during execution.

## Relevant Requirements

- Nový scope — tento milestone zavádí nové requirements pro vylepšený project search. Nenavazuje přímo na existující Active requirements z backlogu.
- Partially relevant: S-3 (I/O error propagace) — replace operace musí reportovat I/O chyby do UI.

## Scope

### In Scope

- Vylepšený search dialog s regex/case-sensitive/whole-word togglery
- Regex engine přes `regex` crate (již v Cargo.toml)
- Case-sensitive/insensitive toggle
- Whole-word matching toggle
- Zvýraznění matchujících částí ve výsledcích (LayoutJob s barevnými TextSection)
- Kontextové řádky (±2 řádky kolem matchujícího řádku)
- File type filtr (glob pattern, např. `*.rs`, `*.toml`)
- Replace v projektu s preview (zobrazení diff všech náhrad)
- Replace preview s checkboxy (potvrzení/odmítnutí per-soubor)
- Local history snapshot před replace (záchranná síť)
- i18n pro nové UI prvky (cs, en, sk, de, ru)
- Inkrementální streamování výsledků (ne čekat na dokončení celého searche)

### Out of Scope / Non-Goals

- Přesunutí project search do sidebar panelu — zůstává modální dialog
- Vim-style search commands nebo search history
- Multi-line search pattern
- Strukturální search (AST-aware, semantic search)
- In-file search vylepšení (Ctrl+F) — to je separátní scope
- Indexování obsahu souborů pro rychlejší vyhledávání (trigram index)

## Technical Constraints

- `cargo check` + `./check.sh` musí projít po každé slice
- Žádné nové runtime závislosti (regex již v Cargo.toml)
- Neblokovat UI vlákno — search a replace v background threadu
- Replace musí vytvořit local history snapshot PŘED modifikací každého souboru
- Replace error handling — pokud zápis jednoho souboru selže, reporting do UI a pokračování s dalšími (ne atomic all-or-nothing)
- Zpětná kompatibilita — Ctrl+Shift+F stále otevírá project search (CommandId::ProjectSearch z M004)
- Regex error musí být zobrazen inline v dialogu (ne panic, ne toast)

## Integration Points

- `src/app/ui/search_picker.rs` — kompletní přestavba: nový input dialog s togglery, vylepšený výsledkový dialog se zvýrazněním a kontextem, replace preview dialog
- `src/app/ui/workspace/state/types.rs` — rozšíření `ProjectSearch` structu o toggle stavy a replace data, rozšíření `SearchResult` o match rozsahy
- `src/app/ui/workspace/mod.rs` — napojení replace flow (snapshot + zápis)
- `src/app/local_history.rs` — volání `take_snapshot()` před replace
- `src/app/ui/workspace/index.rs` — `get_files()` pro file listing s možným filtrováním
- `locales/*/ui.ftl` — nové i18n klíče pro togglery, replace, preview, error messages
- `src/app/keymap.rs` / `src/app/registry/mod.rs` — bez změn (Ctrl+Shift+F dispatch už funguje)

## Open Questions

- **Regex capture groups v replace** — Podpora `$1`, `$2` v replace stringu? `regex` crate to podporuje přes `replace_all()`. Pravděpodobně ano, ale potřeba ošetřit nevalidní backreference.
- **Kontext kolapsy** — Pokud dva matche v souboru jsou blízko u sebe (méně než 2*kontext), mají se kontextové řádky sloučit (jako v `grep -C`)? Pravděpodobně ano — vizuálně čistší.
- **Progresivní streamování** — Výsledky streamovat přes kanál per-soubor, nebo dávkově? Per-soubor je responzivnější ale UI se musí víc přebudovávat. Dávkově (např. po 50 výsledcích) je kompromis.
- **Toggle ikonky** — egui nemá nativní toggle button s ikonou. Řešení: `selectable_label` s textem ".*" / "Aa" / "W" (VS Code styl), nebo custom widget. Rozhodnout při implementaci.
