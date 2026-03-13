# M003: Vylepšení UI Historie Souboru

**Vision:** History split view se stane aktivním nástrojem — levý panel editovatelný se syntax highlighting, pravý read-only se syntax highlighting a diff barvami, synchronizovaný scroll, obnovení historické verze s potvrzením (append, ne replace), a inteligentní výchozí stav panelů.

## Success Criteria

- Levý panel v history view je editovatelný (TextEdit) se syntax highlighting a diff zvýrazněním. Změny se propsávají do tab bufferu jako modified (●).
- Pravý panel má syntax highlighting (syntect) kombinovaný s diff zvýrazněním (zelená/červená pozadí).
- Oba panely mají diff zvýraznění oproti protějšímu panelu.
- Rolování jedním panelem synchronizovaně roluje i druhý.
- Tlačítko "Obnovit" v toolbaru → potvrzovací dialog → obsah historické verze se zapíše do editoru, nový snapshot se vytvoří na konec fronty, history view se refreshne. Mezilehlé verze zůstanou zachovány.
- Soubor s 1 verzí (originál) → pravý panel je prázdný. Soubor s >1 verzí → pravý panel zobrazí nejnovější historickou verzi.
- i18n klíče pro nové UI prvky existují ve všech 5 jazycích (cs, en, sk, de, ru).
- `cargo check` + `./check.sh` prochází.

## Key Risks / Unknowns

- **TextEdit + diff overlay + syntax highlighting** — Kombinace editovatelného TextEdit s per-řádkovým diff pozadím a syntect highlighting v jednom layouteru. Normální editor má jen highlighting bez diff. Nový layouter musí znát diff stav každého řádku.
- **Diff recompute při editaci** — V M002 se diff počítal jen při navigaci. Teď se levý panel mění per-keystroke. `similar::TextDiff` je O(n*d) — pro velké soubory potenciálně pomalý. Potřeba debounce nebo per-frame limit.
- **Sync scroll s rozdílným počtem řádků** — Insert/Delete řádky způsobují, že panely mají různý počet řádků. Přímý přenos scroll offset nefunguje — potřeba mapování.

## Proof Strategy

- **TextEdit + diff + highlighting** → retire v S01 postavením reálného editovatelného panelu s diff overlay a syntect layouterem. Proven = levý panel je editovatelný, syntax barvy jsou viditelné, diff pozadí se zobrazuje na správných řádcích.
- **Diff recompute** → retire v S01 implementací debounced diff recompute. Proven = editace nelaguje ani na souboru s 1000+ řádky.
- **Sync scroll** → retire v S01 implementací scroll sync přes proportionální mapování. Proven = scroll jedním panelem pohne druhým na odpovídající pozici.

## Verification Classes

- Contract verification: `cargo check` + `./check.sh` po každé slice. Existující unit testy pro diff logiku.
- Integration verification: end-to-end tok editace → zavření → tab modified; obnovení → snapshot → refresh.
- Operational verification: diff recompute nepůsobí UI freeze na velkých souborech.
- UAT / human verification: syntax highlighting čitelnost, diff barvy + syntax kombinace, sync scroll UX, restore flow — vyžaduje vizuální kontrolu v běžícím editoru.

## Milestone Definition of Done

This milestone is complete only when all are true:

- Obě slice jsou dokončené a verifikované.
- Levý panel je editovatelný se syntax highlighting + diff zvýrazněním.
- Pravý panel je read-only se syntax highlighting + diff zvýrazněním.
- Diff zvýraznění funguje v obou panelech oproti protějšímu panelu.
- Scroll je synchronizovaný mezi panely.
- Editace v levém panelu se průběžně propsávají do tab bufferu (modified ●).
- "Obnovit" → potvrzení → zápis do editoru + nový snapshot (append) + refresh history.
- Výchozí stav: 1 verze → pravý panel prázdný; >1 verze → nejnovější historická vpravo.
- Diff cache se invaliduje při editaci levého panelu.
- i18n klíče ve všech 5 jazycích (cs, en, sk, de, ru).
- `cargo check` + `./check.sh` prochází.

## Requirement Coverage

- Covers: R001, R002, R003, R004, R005, R006, R007, R008, R009
- Partially covers: none
- Leaves for later: none
- Orphan risks: none

## Slices

- [x] **S01: Editovatelný panel se syntax highlighting, diff a sync scrollem** `risk:high` `depends:[]`
  > After this: Levý panel je editovatelný TextEdit se syntax highlighting a diff zvýrazněním. Pravý panel je read-only se syntax highlighting a diff barvami. Scroll je synchronizovaný. Editace se propsávají do tab bufferu. Výchozí stav panelů odpovídá počtu verzí. Diff cache se invaliduje při editaci. Ověřeno `cargo check` + testy + vizuálně v běžícím editoru.

- [x] **S02: Obnovení historické verze s potvrzením a i18n** `risk:medium` `depends:[S01]`
  > After this: Tlačítko "Obnovit" v toolbaru → potvrzovací dialog → obsah historické verze se zapíše do editoru, nový snapshot se vytvoří na konec fronty, history view se refreshne. Mezilehlé verze zůstanou zachovány. i18n klíče kompletní pro všech 5 jazyků. Ověřeno end-to-end: obnovení → snapshot na FS → history list aktualizován.

## Boundary Map

### S01 → S02

Produces:
- Editovatelný levý panel (TextEdit s layouterem) — tab.content se aktualizuje průběžně při editaci.
- Syntax highlighting v obou panelech přes `Highlighter::highlight()` s diff overlay.
- `HistoryViewState` rozšířený o sync scroll state a diff-aware layouter metadata.
- Debounced diff recompute při editaci levého panelu.
- Výchozí stav: `selected_index = None` pokud je jen 1 verze, `Some(0)` pokud je více verzí.
- Nový parametr: `&Highlighter` a `theme_name: &str` předávané do `render_history_split_view()`.

Consumes:
- nothing (first slice)

### S02 (final)

Produces:
- Tlačítko "Obnovit" v toolbaru s enabled/disabled stavem (disabled pokud žádná verze není vybraná).
- Potvrzovací dialog (StandardModal pattern nebo egui::Window).
- Restore logika: zápis obsahu historické verze do tab.content + take_snapshot() + refresh entries.
- Nové i18n klíče pro restore tlačítko, potvrzovací dialog, případný stav prázdného panelu ve všech 5 jazycích.

Consumes from S01:
- Editovatelný levý panel s průběžným tab.content sync.
- `HistoryViewState` s `selected_index`, `entries`, `relative_path`.
- `LocalHistory::take_snapshot()` a `get_history()` pro refresh po restore.
- `background_io_tx` pro odeslání snapshot signálu.
