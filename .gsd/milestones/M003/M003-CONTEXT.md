# M003: Vylepšení UI Historie Souboru — Context

**Gathered:** 2026-03-13
**Status:** Ready for planning

## Project Description

Vylepšení stávajícího history split view z M002 — levý panel se stane editovatelným se syntax highlighting, pravý panel dostane syntax highlighting s diff barvami, přibude synchronizovaný scroll, obnovení historické verze (append) s potvrzovacím dialogem, a inteligentní výchozí stav panelů.

## Why This Milestone

M002 dodal funkční local history se split view a diff zvýrazněním, ale:
- Oba panely jsou read-only (LayoutJob) — uživatel musí zavřít history view, aby mohl editovat.
- Chybí syntax highlighting — text je monospace bez barevného zvýraznění syntaxe.
- Panely scrollují nezávisle — uživatel musí ručně hledat odpovídající místo.
- Nelze obnovit historickou verzi — view-only bez akce.
- Výchozí stav nezohledňuje počet verzí — vždy se vybere nejnovější historická i když existuje jen jedna verze (originál).

Toto milestone přetváří history view z pasivního prohlížeče na aktivní nástroj pro práci s historií souboru.

## User-Visible Outcome

### When this milestone is complete, the user can:

- V levém panelu přímo editovat aktuální verzi souboru se syntax highlighting a diff zvýrazněním oproti historické verzi vpravo
- V pravém panelu vidět historickou verzi se syntax highlighting a diff zvýrazněním
- Rolovat jedním panelem a druhý se synchronizovaně posune na odpovídající pozici
- Kliknout na "Obnovit" v toolbaru → potvrdit v dialogu → obsah historické verze se zapíše do editoru a vytvoří nový snapshot na konec fronty (žádná verze se neztratí)
- Při otevření historie souboru s jedinou verzí vidět prázdný pravý panel; s více verzemi vidět nejnovější historickou verzi vpravo

### Entry point / environment

- Entry point: pravý klik na editorový tab → "Historie souboru" (existující z M002)
- Environment: desktop editor (eframe/egui), local-first, single-process multi-window
- Live dependencies involved: none — vše je lokální filesystem

## Completion Class

- Contract complete means: `TextEdit` v levém panelu je editovatelný se syntax highlighting; pravý panel má syntax highlighting s diff overlay; scroll je synchronizovaný; obnovení vytváří nový snapshot.
- Integration complete means: celý tok funguje end-to-end — editace v history view → zavření → změny v tab bufferu → uložení → nový snapshot; obnovení → potvrzení → nový snapshot → refresh history.
- Operational complete means: diff cache se invaliduje při editaci levého panelu; obnovení neztratí mezilehlé verze.

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- Otevření historie → editace v levém panelu → scroll dolů (druhý panel se synchronizuje) → zavření → tab ukazuje modified (●) → uložení → nový snapshot existuje.
- Otevření historie se 3+ verzemi → navigace na starší verzi → klik Obnovit → potvrzení → obsah se změní → history list ukazuje nový snapshot na konci → starší verze stále existují.
- Otevření historie souboru s jedinou verzí → pravý panel je prázdný.
- Oba panely mají syntax highlighting + diff zvýraznění.

## Risks and Unknowns

- **TextEdit + diff overlay + syntax highlighting** — Editovatelný `TextEdit` s layouterem (syntect) funguje v normálním editoru. Ale v history view je potřeba kombinovat syntax highlighting s diff pozadím per-řádek. `TextEdit` layout callback nemá přímý přístup k diff metadatám — bude potřeba vlastní layouter, který zná diff stav každého řádku.
- **Sync scroll s rozdílným počtem řádků** — Levý a pravý panel mají různý počet řádků (Insert/Delete). Prostý přenos scroll offset nefunguje — potřeba mapování přes Equal řádky.
- **Diff cache invalidace při editaci** — V M002 se diff počítal jen při navigaci (selected_index). Teď se levý panel mění per-keystroke — diff se musí invalidovat a přepočítat. `similar::TextDiff` je O(n*d), pro velké soubory potenciálně pomalý. Potřeba debounce nebo inkrementální přístup.

## Existing Codebase / Prior Art

- `src/app/ui/workspace/history/mod.rs` — Aktuální history split view z M002. `HistoryViewState`, `compute_diff()`, `diff_colors()`, `render_history_split_view()`. Oba panely jsou read-only `LayoutJob` bez syntax highlighting.
- `src/app/ui/editor/render/normal.rs` — Normální editor rendering. `TextEdit::multiline` s layouterem přes `Highlighter::highlight()`. Pattern pro editovatelný panel se syntax highlighting.
- `src/highlighter.rs` — `Highlighter` struct s `highlight()` → `Arc<LayoutJob>`, cache přes hash. Vrací kompletní LayoutJob pro celý soubor. Používá syntect `HighlightLines`.
- `src/app/ui/editor/diff_view.rs` — Existující AI diff modal s `similar`-based rendering. Nepoužitelné přímo, ale referencí pro diff + LayoutJob kombinaci.
- `src/settings.rs` — `syntect_theme_name()` pro aktuální theme name.
- `src/app/ui/workspace/mod.rs` — Kde se volá `render_history_split_view()`, inicializace `HistoryViewState`, podmíněný editor rendering.
- `src/app/ui/editor/mod.rs` — `Editor` struct, `Tab` struct s `content`, `modified`, `scroll_offset`.

> See `.gsd/DECISIONS.md` for all architectural and pattern decisions — it is an append-only register; read it during planning, append to it during execution.

## Relevant Requirements

- R001 — Editovatelný levý panel (primary S01)
- R002 — Syntax highlighting v obou panelech (primary S01)
- R003 — Synchronizovaný scroll (primary S01)
- R004 — Obnovení historické verze, append (primary S02)
- R005 — Potvrzovací dialog (primary S02)
- R006 — Editace propsání do tab bufferu (primary S01)
- R007 — Výchozí stav panelů (primary S01)
- R008 — i18n klíče (primary S02, supporting S01)
- R009 — Diff zvýraznění v obou panelech se syntax highlighting (primary S01)

## Scope

### In Scope

- Přepis levého panelu na editovatelný TextEdit se syntax highlighting a diff overlay
- Syntax highlighting v pravém panelu (read-only LayoutJob s syntect + diff barvami)
- Synchronizovaný scroll mezi panely
- Tlačítko "Obnovit" v toolbaru s potvrzovacím dialogem
- Obnovení = zápis obsahu do tab bufferu + nový snapshot (append) + refresh
- Výchozí stav: 1 verze → prázdný pravý panel, >1 verze → nejnovější historická
- Diff cache invalidace při editaci levého panelu
- i18n pro nové UI prvky ve všech 5 jazycích

### Out of Scope / Non-Goals

- Editace historické verze v pravém panelu (R100 — zůstává read-only)
- Restore jako samostatný soubor (R101 — přepisuje aktuální obsah)
- Line numbers v history panelech (normální editor je má, ale v diff view by byly matoucí)
- LSP integrace v history view (completion, hover, diagnostics — to by vyžadovalo plnou editor instanci)

## Technical Constraints

- `cargo check` + `./check.sh` musí projít po každé slice
- Žádné nové runtime závislosti
- Neblokovat UI vlákno — diff recompute musí být rychlý nebo debounced
- Zachovat existující architekturu single-process multi-window
- `Highlighter` je ve struct `Editor` — history view potřebuje přístup přes referenci

## Integration Points

- `src/app/ui/workspace/history/mod.rs` — kompletní přepis renderingu, rozšíření HistoryViewState
- `src/app/ui/workspace/mod.rs` — úprava inicializace HistoryViewState, předání highlighteru a theme name
- `src/app/ui/editor/mod.rs` — sync tab.content s editacemi v history view
- `src/app/local_history.rs` — volání take_snapshot() při obnovení
- `locales/*/ui.ftl` — nové i18n klíče

## Open Questions

- **Diff recompute debounce** — Při editaci per-keystroke je O(n*d) diff potenciálně pomalý na velkých souborech. Řešení: debounce 300ms po poslední editaci, nebo invalidace per-frame s limitem na frekvenci. Rozhodnout při implementaci na základě profilu.
- **Sync scroll mapování** — Přímý přenos scroll offset funguje jen pro Equal řádky. Pro Insert/Delete řádky je potřeba mapovací tabulka. Alternativa: sync jen na úrovni "procento scrollu" — jednodušší, ale méně přesné. Rozhodnout při implementaci.
