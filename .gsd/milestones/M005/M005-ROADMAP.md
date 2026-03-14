# M005: Vylepšení Project Search

**Vision:** Přebudovat project-wide search z minimálního modálního dialogu na plnohodnotný vyhledávací nástroj s regex, togglery, zvýrazněním matchů, kontextovými řádky, file type filtrem a project-wide replace s preview.

## Success Criteria

- Ctrl+Shift+F → zadání `fn\s+\w+` s regex ON → výsledky zobrazí zvýrazněné matchující funkce v kontextu ±2 řádků → kliknutí otevře soubor na daném řádku
- "TODO" s case-sensitive ON najde jen "TODO", ne "todo". S OFF najde všechny varianty.
- "Result" s whole-word ON najde "Result" ale ne "SearchResult". S OFF najde obojí.
- Filtr `*.rs` → výsledky jen z Rust souborů. `*.toml` → jen z TOML.
- Replace: query "old_name" → replace "new_name" → preview ukazuje diff → odškrtnutí jednoho souboru → potvrzení → soubory se změní (kromě odškrtnutého) → local history snapshot existuje
- Nevalidní regex pattern → inline chyba v dialogu, hledání se nespustí
- `cargo check` + `./check.sh` projde čistě

## Key Risks / Unknowns

- **egui klikatelný LayoutJob** — `Label::new(LayoutJob)` s `sense(Sense::click())` nemusí fungovat ve všech egui verzích. Fallback: `Button::new(LayoutJob)` nebo `allocate_response()` + `paint_galley()`.
- **Kontextové řádky se sloučením** — Dva matche blíže než 4 řádky musí sloučit kontextové bloky. Bez sloučení se řádky opakují.
- **LocalHistory borrow v replace flow** — `take_snapshot()` potřebuje `&mut LocalHistory`, ale replace data přicházejí z background threadu. Snapshoty musí proběhnout sekvenčně na main threadu.
- **Replace na 100+ souborech** — 100× `take_snapshot()` + `fs::write()` potenciálně blokuje UI na ~100ms. Akceptovatelné pro MVP.

## Proof Strategy

- egui klikatelný LayoutJob → retire v S01 stavbou reálného výsledkového dialogu se zvýrazněným textem a klikacími výsledky
- LocalHistory borrow → retire v S02 implementací replace flow se snapshot voláním v workspace handleru

## Verification Classes

- Contract verification: `cargo test` pro search engine logiku (regex, case, whole-word, filtr, kontext sloučení), `cargo check` + `./check.sh`
- Integration verification: Ctrl+Shift+F → search → výsledky → kliknutí → soubor otevřen; replace → preview → potvrzení → soubory modifikovány + snapshoty
- Operational verification: none (vše lokální filesystem)
- UAT / human verification: vizuální ověření zvýraznění matchů a replace preview (headless — UAT deferred)

## Milestone Definition of Done

This milestone is complete only when all are true:

- Všechny slicí [x] v roadmapě, všechny summaries existují
- Search engine matchuje regex/plain s case/whole-word togglery korektně (unit testy)
- Výsledky zobrazují zvýrazněné matche v kontextu ±2 řádků se sloučením blízkých bloků
- File type filtr (glob) omezuje scope hledání
- Replace preview zobrazuje diff per-file s checkboxy, potvrzení modifikuje soubory
- Local history snapshot existuje pro každý soubor modifikovaný replacem
- I/O chyby v replace se reportují přes toast (per-file, ne atomic)
- Nevalidní regex zobrazí inline error v dialogu
- i18n klíče pro všechny nové UI prvky ve všech 5 jazycích
- Ctrl+Shift+F dispatch funguje beze změn (M004 keymap)
- `cargo check` + `./check.sh` projde čistě
- Final integrated acceptance scenarios z M005-CONTEXT pass

## Requirement Coverage

- Covers: R016, R017, R018, R019, R020, R021, R022, R023, R024, R025
- Partially covers: none
- Leaves for later: none
- Orphan risks: none

## Slices

- [x] **S01: Vylepšený search dialog s regex engine, zvýrazněnými výsledky a kontextem** `risk:high` `depends:[]`
  > After this: Ctrl+Shift+F otevře nový dialog s regex/case/whole-word togglery a file type filtrem. Výsledky zobrazují zvýrazněné matche v kontextu ±2 řádků. Kliknutí na výsledek otevře soubor na řádku. Inkrementální streamování výsledků. Unit testy pokrývají engine logiku.
- [x] **S02: Project-wide replace s preview a local history** `risk:medium` `depends:[S01]`
  > After this: Uživatel zadá replace text, zobrazí se preview diff per-file s checkboxy, potvrzení modifikuje vybrané soubory s local history snapshotem. I/O chyby se reportují per-file přes toast. i18n kompletní.

## Boundary Map

### S01 → S02

Produces:
- `SearchOptions` struct s `use_regex`, `case_sensitive`, `whole_word`, `file_filter` fieldy
- `SearchResult` rozšířený o `match_ranges: Vec<(usize, usize)>` a `context_before`/`context_after: Vec<String>`
- `ProjectSearch` rozšířený o toggle stavy a `SearchOptions`
- `build_regex(query, options) -> Result<Regex, String>` funkce pro sestavení regex z query + togglerů
- Nový výsledkový dialog se zvýrazněným LayoutJob renderingem a klikacími výsledky
- Streamování výsledků přes `mpsc::Receiver<SearchBatch>` s dávkováním

Consumes:
- nothing (first slice)

### S02 consumes S01

Consumes:
- `SearchOptions` a `build_regex()` pro replace matching
- `ProjectSearch` struct pro toggle stavy a query
- Existující výsledkový dialog jako základ pro replace preview
- `SearchResult` s match_ranges pro replace operace
