# S01 Assessment — Roadmap Reassessment after S01

## Verdict: Roadmap unchanged

S01 dodal přesně to, co boundary map sliboval. Všechna plánovaná rozhraní pro S02 jsou dostupná:

- `SearchOptions`, `build_regex()`, `ProjectSearch` struct s toggle stavy — přímo znovupoužitelné pro replace matching
- `replace_text: String` a `show_replace: bool` fieldy v `ProjectSearch` připravené pro S02
- Výsledkový dialog (`poll_and_render_project_search_results()`) jako základ pro replace preview UI
- `start_project_search()` helper reusable z replace flow

## Risk Retirement

- **egui klikatelný LayoutJob** — retired. `Label::new(job).sense(Sense::click())` funguje, ověřeno kompilací i pattern matchem z terminal/bottom/mod.rs.
- **Kontextové řádky se sloučením** — retired. 3 unit testy pokrývají simple match, close matches merged, no match.

## Remaining Risk for S02

- **LocalHistory borrow v replace flow** — stále aktivní (medium risk). S01 summary potvrzuje pattern: `take_snapshot()` volaný v workspace handleru na main threadu, ne v background threadu.
- **Replace na 100+ souborech** — akceptovatelné pro MVP (~100ms blokace).

## Success Criteria Coverage

| Criterion | Owner | Status |
|---|---|---|
| Ctrl+Shift+F → regex → zvýrazněné matche → kliknutí otevře soubor | S01 | ✅ validated |
| Case-sensitive toggle (TODO/todo) | S01 | ✅ validated |
| Whole-word toggle (Result/SearchResult) | S01 | ✅ validated |
| File type filtr (*.rs, *.toml) | S01 | ✅ validated |
| Replace → preview diff → checkboxy → potvrzení → soubory + snapshot | S02 | pending |
| Nevalidní regex → inline chyba | S01 | ✅ validated |
| cargo check + ./check.sh | S02 | ongoing gate |

## Requirement Coverage

- R020, R022, R023 → S02 (unchanged)
- R024 → S02 doplní replace-specifické i18n klíče (unchanged)
- Žádné nové, invalidované ani re-scoped requirements.

## Boundary Map

Beze změn — S02 consumes přesně to, co S01 provides.
