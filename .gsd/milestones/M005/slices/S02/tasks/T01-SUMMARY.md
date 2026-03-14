---
id: T01
parent: S02
milestone: M005
provides:
  - ReplacePreview struct pro per-file replace data
  - compute_replace_previews() funkce — generuje preview z výsledků hledání
  - apply_replacements() funkce — atomický zápis vybraných souborů
  - 4 unit testy pro replace engine
key_files:
  - src/app/ui/workspace/state/types.rs
  - src/app/ui/search_picker.rs
key_decisions:
  - compute_replace_previews bere root path + relativní cesty z SearchResult, pracuje s absolutními cestami interně
  - apply_replacements vrací Vec<(PathBuf, Result<(), String>)> pro per-file error reporting — caller rozhoduje o zobrazení
  - ReplacePreview obsahuje plný original+new content pro diff preview v UI
patterns_established:
  - Replace engine je standalone (bez UI závislostí), plně testovatelné přes tempdir
observability_surfaces:
  - apply_replacements() výsledkový vektor — per-file Ok/Err s popisnou zprávou
  - compute_replace_previews() propaguje io::Error při nečitelném souboru
duration: 15m
verification_result: passed
completed_at: 2026-03-13
blocker_discovered: false
---

# T01: Replace engine a preview data model

**Implementace replace engine — `ReplacePreview` struct, `compute_replace_previews()`, `apply_replacements()` a 4 unit testy.**

## What Happened

1. Přidán `ReplacePreview` struct do `types.rs` — `file`, `original_content`, `new_content`, `match_count`, `selected` (default true).
2. `ProjectSearch` rozšířen o `replace_previews: Vec<ReplacePreview>` a `show_replace_preview: bool` s odpovídajícími default hodnotami.
3. `compute_replace_previews(root, results, regex, replace_text)` — deduplikuje cesty z výsledků, pro každý soubor načte obsah z disku, provede `regex.replace_all()`, porovná originál vs nový obsah. Capture groups ($1, $2) fungují automaticky.
4. `apply_replacements(previews)` — zapíše `new_content` do souborů kde `selected == true`. Selhání jednoho souboru neblokuje ostatní. Vrací per-file výsledek.
5. 4 unit testy: basic replace, regex capture groups, apply success, partial skip (selected=false).

## Verification

- `cargo test --bin polycredo-editor app::ui::search_picker` — 19 testů pass (15 existujících + 4 nové replace testy)
- `cargo check` — čistá kompilace
- `./check.sh` — fmt, clippy, 191+ testů pass, quality gate passed

### Slice-level verification (partial — T01 z T02):
- ✅ `cargo test --bin polycredo-editor app::ui::search_picker` — replace unit testy pass
- ✅ `cargo check` — kompilace čistá
- ✅ `./check.sh` — fmt, clippy, všechny testy pass
- ⬜ `apply_replacements()` s neexistující cestou — bude testováno v T02 kontextu (UI error handling)

## Diagnostics

- `apply_replacements()` výsledkový vektor inspektovatelný per-file — `Ok(())` pro úspěch, `Err(String)` s cestou a I/O chybou
- `compute_replace_previews()` propaguje `io::Error` — caller zobrazí přes toast (T02)
- `ReplacePreview.selected` flag — kontrolovatelný per-file před zápisem

## Deviations

- `compute_replace_previews()` bere navíc `root: &Path` parametr (plán uváděl jen `results, regex, replace_text`) — nutné pro překlad relativních cest na absolutní. Logické rozšíření, konzistentní s `run_project_search()`.

## Known Issues

None.

## Files Created/Modified

- `src/app/ui/workspace/state/types.rs` — přidán `ReplacePreview` struct, `ProjectSearch` rozšířen o replace fieldy
- `src/app/ui/workspace/state/mod.rs` — export `ReplacePreview`
- `src/app/ui/search_picker.rs` — `compute_replace_previews()`, `apply_replacements()`, 4 unit testy
