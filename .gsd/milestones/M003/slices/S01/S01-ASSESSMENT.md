# S01 Assessment — Roadmap Reassessment

## Verdict: Roadmap is fine. No changes needed.

## Risk Retirement

S01 měla retire tři klíčové rizika:
- **TextEdit + diff + highlighting** — ✅ Retired. Levý panel je editovatelný TextEdit s layouterem, syntax highlighting + diff background fungují společně.
- **Diff recompute při editaci** — ✅ Retired. content_hash (xxh3_64) invaliduje diff cache jen při reálné změně obsahu.
- **Sync scroll** — ✅ Retired. Proportionální mapování s epsilon 1.0px a ScrollSource flag funguje bez feedback loop.

## New Risks

Žádné nové rizika nevznikla. S02 scope (restore tlačítko + dialog + i18n) je medium risk a nevyžaduje žádný nový technický průlom.

## Boundary Map Accuracy

S01 summary potvrzuje přesně artefakty, které S02 boundary map consumes:
- `HistorySplitResult { close, content_changed }` — S02 přidá `restore_requested`
- `HistoryViewState` s `selected_index`, `entries`, `relative_path` — k dispozici
- `LocalHistory::take_snapshot()` a `get_history()` — k dispozici
- `background_io_tx` — k dispozici ve WorkspaceState

Boundary map je přesná, žádné úpravy nejsou potřeba.

## Success Criteria Coverage

Všechna kritéria mají ownera:
- Levý panel editovatelný + syntax + diff → ✅ S01 done
- Pravý panel syntax + diff → ✅ S01 done
- Diff zvýraznění v obou panelech → ✅ S01 done
- Synchronizovaný scroll → ✅ S01 done
- Editace → tab buffer modified → ✅ S01 done
- Diff cache invalidace → ✅ S01 done
- Výchozí stav panelů → ✅ S01 done
- Tlačítko "Obnovit" + dialog + snapshot + refresh → S02
- i18n klíče ve všech 5 jazycích → S02
- `cargo check` + `./check.sh` → S02 (finální)

Žádné kritérium není bez ownera.

## Requirement Coverage

9 aktivních requirements, všechny mají ownera:
- R001–R003, R006, R007, R009 — implemented v S01, UAT pending
- R004, R005, R008 — unmapped → S02 scope
- Žádný nový requirement nevznikl, žádný nebyl invalidován.

## Deviations Noted

- `font_size` jako extra parametr (minor, nezasahuje do S02 scope)
- Syntect theme background jako panel fill (vizuální vylepšení, nezasahuje do S02)

Obě odchylky jsou aditivní a neovlivňují S02 plánování.
